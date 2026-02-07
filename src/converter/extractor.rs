use std::{
    collections::HashSet,
    sync::LazyLock,
};

use regex::Regex;
use serde::{
    Deserialize,
    Serialize,
};
use unicode_segmentation::UnicodeSegmentation;

use crate::utils::normalize_str;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Default)]
pub struct TitleMetadata {
    season: Option<f32>,
    part: Option<f32>,
    episode: Option<f32>,
    tokens: Vec<Token>,
}

impl TitleMetadata {
    pub fn season(&self) -> Option<f32> { self.season }
    pub fn part(&self) -> Option<f32> { self.part }
    pub fn episode(&self) -> Option<f32> { self.episode }
    pub fn tokens(&self) -> &[Token] { &self.tokens }
    pub fn has_season_keyword(&self) -> bool { self.tokens.iter().any(|t| t.is_season()) }
    pub fn has_part_keyword(&self) -> bool { self.tokens.iter().any(|t| t.is_part()) }
    pub fn has_episode_keyword(&self) -> bool { self.tokens.iter().any(|t| t.is_episode()) }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Copy)]
pub enum Token {
    Num(f32),
    Ordinal(f32),
    Keyword(Keyword),
}

impl Token {
    pub fn is_season(&self) -> bool {
        if let Token::Keyword(kw) = self {
            return kw.is_season();
        }
        false
    }
    pub fn is_part(&self) -> bool {
        if let Token::Keyword(kw) = self {
            return kw.is_part();
        }
        false
    }
    pub fn is_episode(&self) -> bool {
        if let Token::Keyword(kw) = self {
            return kw.is_episode();
        }
        false
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Copy)]
pub enum Keyword {
    Season,
    S,
    Series,
    Part,
    Arc,
    Cour,
    Episode,
    Ova,
    Ona,
    Movie,
    Special,
}

impl Keyword {
    pub fn is_season(&self) -> bool {
        matches!(self, Keyword::Season | Keyword::S | Keyword::Series)
    }
    pub fn is_part(&self) -> bool { matches!(self, Keyword::Part | Keyword::Arc | Keyword::Cour) }
    pub fn is_episode(&self) -> bool {
        matches!(
            self,
            Keyword::Episode | Keyword::Ova | Keyword::Ona | Keyword::Movie | Keyword::Special
        )
    }
}

pub struct TitleProcessor {}

impl TitleProcessor {
    fn word_to_num(word: &str) -> Option<Token> {
        match word {
            "i" | "one" => Some(Token::Num(1.0)),
            "ii" | "two" => Some(Token::Num(2.0)),
            "iii" | "three" => Some(Token::Num(3.0)),
            "iv" | "four" => Some(Token::Num(4.0)),
            "v" | "five" => Some(Token::Num(5.0)),
            "vi" | "six" => Some(Token::Num(6.0)),
            "vii" | "seven" => Some(Token::Num(7.0)),
            "viii" | "eight" => Some(Token::Num(8.0)),
            "ix" | "nine" => Some(Token::Num(9.0)),
            "x" | "ten" => Some(Token::Num(10.0)),
            _ => None,
        }
    }

    fn word_to_ordinal(word: &str) -> Option<Token> {
        match word {
            "first" => Some(Token::Ordinal(1.0)),
            "second" => Some(Token::Ordinal(2.0)),
            "third" => Some(Token::Ordinal(3.0)),
            "fourth" => Some(Token::Ordinal(4.0)),
            "fifth" => Some(Token::Ordinal(5.0)),
            "sixth" => Some(Token::Ordinal(6.0)),
            "seventh" => Some(Token::Ordinal(7.0)),
            "eighth" => Some(Token::Ordinal(8.0)),
            "ninth" => Some(Token::Ordinal(9.0)),
            "tenth" => Some(Token::Ordinal(10.0)),
            "final" | "finale" | "last" => Some(Token::Ordinal(99.0)),
            _ => None,
        }
    }

    fn get_keyword(word: &str) -> Option<Keyword> {
        match word {
            "season" => Some(Keyword::Season),
            "s" => Some(Keyword::S),
            "series" => Some(Keyword::Series),
            "part" => Some(Keyword::Part),
            "arc" => Some(Keyword::Arc),
            "cour" => Some(Keyword::Cour),
            "episode" | "episodes" => Some(Keyword::Episode),
            "ova" => Some(Keyword::Ova),
            "ona" => Some(Keyword::Ona),
            "movie" | "film" | "theatre" | "theater" => Some(Keyword::Movie),
            "special" | "specials" => Some(Keyword::Special),
            _ => None,
        }
    }

    pub fn tokenize(title: &str) -> Vec<Token> {
        let title_normalized = normalize_str(title);
        let mut tokens = Vec::<Token>::new();

        static YEAR_RE: LazyLock<Regex> =
            LazyLock::new(|| Regex::new(r"^(?:19|20)\d{2}$").unwrap());
        static END_NUM_RE: LazyLock<Regex> =
            LazyLock::new(|| Regex::new(r"(\d+\.?\d*)\.?$").unwrap());
        static ORDINAL_RE: LazyLock<Regex> =
            LazyLock::new(|| Regex::new(r"^(\d+)(?:st|nd|rd|th)\.?$").unwrap());

        let words = title_normalized.unicode_word_indices().collect::<Vec<_>>();
        for (i, (iw, word)) in words.iter().copied().enumerate().skip(1).rev() {
            if YEAR_RE.is_match(word) {
                continue;
            }
            let t = END_NUM_RE
                .captures(word)
                .and_then(|cap| cap[1].parse::<f32>().ok().map(Token::Num))
                .or_else(|| {
                    ORDINAL_RE
                        .captures(word)
                        .and_then(|cap| cap[1].parse::<f32>().ok().map(Token::Ordinal))
                })
                .or_else(|| Self::word_to_num(word))
                .or_else(|| Self::word_to_ordinal(word))
                .or_else(|| Self::get_keyword(word).map(Token::Keyword));
            if let Some(t) = t {
                match t {
                    Token::Num(_) | Token::Ordinal(_) => {
                        let left = if i == 0 { None } else { Some(words[i - 1]) };
                        let right = if i == words.len() - 1 { None } else { Some(words[i + 1]) };

                        let yw = iw + word.len();

                        static SPECIAL_CHARS: &[char] = &['(', ')', '[', ']', '<', '>', '-', ':'];
                        let left_has_special = left
                            .map(|w| title_normalized[w.0 + w.1.len()..iw].contains(SPECIAL_CHARS));
                        let right_has_special =
                            right.map(|w| title_normalized[yw..w.0].contains(SPECIAL_CHARS));

                        if left.and_then(|w| Self::get_keyword(w.1)).is_some()
                            || right.and_then(|w| Self::get_keyword(w.1)).is_some()
                            || left_has_special.unwrap_or_default()
                            || right_has_special.unwrap_or_default()
                            || i == words.len() - 1
                        {
                            tokens.push(t);
                        }
                    },
                    _ => {
                        tokens.push(t);
                    },
                }
            }
        }
        tokens.reverse();
        tokens
    }

    pub fn process(title: &str) -> TitleMetadata {
        let tokens = Self::tokenize(title);
        if tokens.is_empty() {
            return TitleMetadata::default();
        }
        let mut meta = TitleMetadata { tokens, ..Default::default() };

        let mut used_indices = HashSet::new();
        for (i, token) in meta.tokens.iter().enumerate() {
            let n = Self::find_num(&meta.tokens, i);
            let mut o = Self::find_ordinal(&meta.tokens, i);
            let v = n.or(o);
            if n.is_some() {
                used_indices.insert(i + 1);
            } else if o.is_some() {
                if used_indices.contains(&(i - 1)) {
                    o = None;
                } else {
                    used_indices.insert(i - 1);
                }
            }
            match token {
                Token::Keyword(kw) => {
                    if kw.is_season() && meta.season.is_none() {
                        meta.season = v;
                    } else if kw.is_part() && meta.part.is_none() {
                        meta.part = v;
                    } else if kw.is_episode() && meta.episode.is_none() {
                        meta.episode = v;
                    }
                },
                _ => continue,
            }
        }

        if meta.season.is_none() {
            let unused = meta.tokens.iter().enumerate().find_map(|(i, t)| {
                if let Token::Num(v) | Token::Ordinal(v) = t
                    && !used_indices.contains(&i)
                {
                    return Some(*v);
                }
                None
            });
            meta.season = unused;
        }

        meta
    }

    fn find_ordinal(tokens: &[Token], kw_idx: usize) -> Option<f32> {
        if kw_idx == 0 {
            return None;
        }
        if let Some(Token::Ordinal(v)) = tokens.get(kw_idx - 1) {
            return Some(*v);
        }
        None
    }
    fn find_num(tokens: &[Token], kw_idx: usize) -> Option<f32> {
        if kw_idx == tokens.len() - 1 {
            return None;
        }
        if let Some(Token::Num(v)) = tokens.get(kw_idx + 1) {
            return Some(*v);
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    //noinspection SpellCheckingInspection
    #[test]
    fn test_extraction_cases() {
        let cases = vec![
            ("Shingeki no Kyojin 1", Some(1.0), None, None),
            ("AOT Season 1 Part II", Some(1.0), Some(2.0), None),
            ("AOT Final Part 2", Some(99.0), Some(2.0), None),
            ("AOT Final", Some(99.0), None, None),
            ("AOT The Final", Some(99.0), None, None),
            ("Shingeki no Kyojin Season 2 Movie: Kakusei no Houkou", Some(2.0), None, None),
            ("Shingeki no Kyojin 2: Kakusei no Houkou", Some(2.0), None, None),
            ("Mob Psycho 100 II", Some(2.0), None, None),
            ("Jujutsu Kaisen Movie 0", None, None, Some(0.0)),
            ("Haikyuu!! 4", Some(4.0), None, None),
            ("Overlord IV", Some(4.0), None, None),
            ("Bleach TYBW Part 2", None, Some(2.0), None),
            ("Spy x Family Cour 2", None, Some(2.0), None),
            ("Spy x Family", None, None, None),
            ("First Strike", None, None, None),
            ("Shakugan no Shana III (Final)", Some(3.0), None, None),
            ("Attack on Titan Final Part Two", Some(99.0), Some(2.0), None),
            ("Attack on Titan 3 Part Two", Some(3.0), Some(2.0), None),
            ("Attack on Titan: 3 Part Two", Some(3.0), Some(2.0), None),
            ("Mob Psycho 100 III", Some(3.0), None, None),
            ("Golden Kamuy 4th Season", Some(4.0), None, None),
            ("The Rising of the Shield Hero S3", Some(3.0), None, None),
            ("Made in Abyss Movie 2", None, None, Some(2.0)),
            ("Kaguya-sama: Love is War - Ultra Romantic", None, None, None),
            ("Dr. Stone: New World Part 2", None, Some(2.0), None),
            ("Mushoku Tensei II Cour 2", Some(2.0), Some(2.0), None),
            ("Oshi no Ko Episode 0", None, None, Some(0.0)),
            ("Chainsaw Man Movie: Reze-hen", None, None, None),
            ("Boku no Hero Academia 6th Season", Some(6.0), None, None),
            ("Monogatari Series: Second Season", Some(2.0), None, None),
            ("Re:Zero Season 2 Part 2", Some(2.0), Some(2.0), None),
        ];
        for (input, s, p, e) in cases {
            let res = TitleProcessor::process(input);
            assert_eq!(res.season, s, "Failed Season, {}", input);
            assert_eq!(res.part, p, "Failed Part, {}", input);
            assert_eq!(res.episode, e, "Failed Episode, {}", input);
        }
    }
}
