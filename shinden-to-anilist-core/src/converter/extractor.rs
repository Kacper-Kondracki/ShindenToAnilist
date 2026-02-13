use std::sync::LazyLock;

use indexmap::IndexMap;
use ordered_float::OrderedFloat;
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

pub const FINAL: f32 = 99.0;

impl Token {
    pub fn is_season(&self) -> bool {
        match self {
            Token::Keyword(kw) => kw.is_season(),
            _ => false,
        }
    }
    pub fn is_part(&self) -> bool {
        match self {
            Token::Keyword(kw) => kw.is_part(),
            _ => false,
        }
    }
    pub fn is_episode(&self) -> bool {
        match self {
            Token::Keyword(kw) => kw.is_episode(),
            _ => false,
        }
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
    pub fn is_season(&self) -> bool { matches!(self, Keyword::Season | Keyword::S | Keyword::Series) }
    pub fn is_part(&self) -> bool { matches!(self, Keyword::Part | Keyword::Arc | Keyword::Cour) }
    pub fn is_episode(&self) -> bool {
        matches!(
            self,
            Keyword::Episode | Keyword::Ova | Keyword::Ona | Keyword::Movie | Keyword::Special
        )
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Copy, Clone, Default)]
pub struct ConsolidatedMetadata {
    season: Option<f32>,
    part: Option<f32>,
    episode: Option<f32>,

    is_final_season: bool,
    is_final_part: bool,
    is_final_episode: bool,
}

impl ConsolidatedMetadata {
    pub fn season(&self) -> Option<f32> { self.season }
    pub fn part(&self) -> Option<f32> { self.part }
    pub fn episode(&self) -> Option<f32> { self.episode }
    pub fn is_final_season(&self) -> bool { self.is_final_season }
    pub fn is_final_part(&self) -> bool { self.is_final_part }
    pub fn is_final_episode(&self) -> bool { self.is_final_episode }
}

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
        "final" | "finale" | "last" => Some(Token::Ordinal(FINAL)),
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

pub mod title_processor {
    use ahash::AHashSet;

    use super::*;

    fn resolve_value_with_finality(values: &[f32]) -> (Option<f32>, bool) {
        let mut is_final = false;
        let mut counts: IndexMap<OrderedFloat<f32>, usize> = IndexMap::new();
        for &val in values {
            if val == FINAL {
                is_final = true;
                continue;
            }
            *counts.entry(OrderedFloat(val)).or_default() += 1;
        }

        let best_val = counts.into_iter().max_by_key(|(_, c)| *c).map(|(k, _)| *k);

        (best_val.or(if is_final { Some(FINAL) } else { None }), is_final)
    }

    pub fn consolidate(metadata_list: &[&TitleMetadata]) -> ConsolidatedMetadata {
        if metadata_list.is_empty() {
            return ConsolidatedMetadata::default();
        }

        let seasons = metadata_list.iter().filter_map(|m| m.season).collect::<Vec<_>>();
        let parts = metadata_list.iter().filter_map(|m| m.part).collect::<Vec<_>>();
        let episodes = metadata_list.iter().filter_map(|m| m.episode).collect::<Vec<_>>();

        let (best_season, is_final_season) = resolve_value_with_finality(&seasons);
        let (best_part, is_final_part) = resolve_value_with_finality(&parts);
        let (best_episode, is_final_episode) = resolve_value_with_finality(&episodes);

        ConsolidatedMetadata {
            season: best_season.or(is_final_season.then_some(FINAL)),
            part: best_part.or(is_final_part.then_some(FINAL)),
            episode: best_episode.or(is_final_episode.then_some(FINAL)),
            is_final_season,
            is_final_part,
            is_final_episode,
        }
    }

    fn parse_raw_token(word: &str) -> Option<Token> {
        static YEAR_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^(?:19|20)\d{2}$").unwrap());
        static END_NUM_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(\d+\.?\d*).?$").unwrap());
        static ORDINAL_RE: LazyLock<Regex> =
            LazyLock::new(|| Regex::new(r"^(\d+)(?:st|nd|rd|th)\.?$").unwrap());

        if YEAR_RE.is_match(word) {
            return None;
        }

        END_NUM_RE
            .captures(word)
            .and_then(|cap| cap[1].parse::<f32>().ok().map(Token::Num))
            .or_else(|| {
                ORDINAL_RE
                    .captures(word)
                    .and_then(|cap| cap[1].parse::<f32>().ok().map(Token::Ordinal))
            })
            .or_else(|| word_to_num(word))
            .or_else(|| word_to_ordinal(word))
            .or_else(|| get_keyword(word).map(Token::Keyword))
    }

    fn contains_special_chars(gap: &str) -> bool {
        static SPECIAL_CHARS: &[char] = &['(', ')', '[', ']', '<', '>', '-', ':'];
        gap.contains(SPECIAL_CHARS)
    }

    pub fn tokenize(title: &str) -> Vec<Token> {
        let title_normalized = normalize_str(title);

        let words: Vec<(usize, &str)> = title_normalized.unicode_word_indices().collect();
        let mut tokens = Vec::new();

        for i in (1..words.len()).rev() {
            let (start, word) = words[i];

            let Some(token) = parse_raw_token(word) else {
                continue;
            };
            match token {
                Token::Keyword(_) => tokens.push(token),
                Token::Num(_) | Token::Ordinal(_) => {
                    let left = words.get(i - 1);
                    let right = words.get(i + 1);

                    let special_left = left.is_some_and(|&(l_start, l_word)| {
                        let gap = &title_normalized[l_start + l_word.len()..start];
                        contains_special_chars(gap)
                    });
                    let special_right = right.is_some_and(|&(r_start, _)| {
                        let gap = &title_normalized[start + word.len()..r_start];
                        contains_special_chars(gap)
                    });

                    let kw_left = left.and_then(|(_, w)| get_keyword(w)).is_some();
                    let kw_right = right.and_then(|(_, w)| get_keyword(w)).is_some();

                    let is_last_word = i == words.len() - 1;

                    if kw_left || kw_right || special_left || special_right || is_last_word {
                        tokens.push(token);
                    }
                },
            }
        }
        tokens.reverse();
        tokens
    }

    pub fn process(title: &str) -> TitleMetadata {
        let tokens = tokenize(title);
        if tokens.is_empty() {
            return TitleMetadata::default();
        }
        let mut meta = TitleMetadata {
            tokens,
            ..Default::default()
        };

        let mut used_indices = AHashSet::new();
        let mut last_ordinal: Option<(usize, f32)> = None;
        let mut tokens_iter = meta.tokens.iter().enumerate().peekable();
        while let Some((i, token)) = tokens_iter.next() {
            match token {
                Token::Ordinal(v) => {
                    last_ordinal = Some((i, *v));
                },
                Token::Keyword(kw) => {
                    let next_val = tokens_iter.peek().and_then(|(_, next_t)| {
                        if let Token::Num(v) = next_t {
                            Some(*v)
                        } else {
                            None
                        }
                    });

                    let mut value = None;

                    if let Some(v) = next_val {
                        value = Some(v);
                        used_indices.insert(i + 1);
                        tokens_iter.next();
                    } else if let Some((idx, v)) = last_ordinal
                        && !used_indices.contains(&idx)
                    {
                        value = Some(v);
                        used_indices.insert(idx);
                    }

                    if let Some(v) = value {
                        match () {
                            _ if kw.is_season() && meta.season.is_none() => meta.season = Some(v),
                            _ if kw.is_part() && meta.part.is_none() => meta.part = Some(v),
                            _ if kw.is_episode() && meta.episode.is_none() => meta.episode = Some(v),
                            _ => {},
                        }
                    }
                    last_ordinal = None;
                },
                Token::Num(_) => {
                    last_ordinal = None;
                },
            }
        }

        if meta.season.is_none() {
            meta.season = meta.tokens.iter().enumerate().find_map(|(i, t)| match t {
                Token::Num(v) | Token::Ordinal(v) if !used_indices.contains(&i) => Some(*v),
                _ => None,
            });
        }

        meta
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
            ("snk 1", Some(1.0), None, None),
            ("AOT Season 1 Part II", Some(1.0), Some(2.0), None),
            ("AOT Final Part 2", Some(FINAL), Some(2.0), None),
            ("AOT Final", Some(FINAL), None, None),
            ("AOT The Final", Some(FINAL), None, None),
            (
                "Shingeki no Kyojin Season 2 Movie: Kakusei no Houkou",
                Some(2.0),
                None,
                None,
            ),
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
            ("Attack on Titan Final Part Two", Some(FINAL), Some(2.0), None),
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
            let res = title_processor::process(input);
            assert_eq!(res.season, s, "Failed Season, {}", input);
            assert_eq!(res.part, p, "Failed Part, {}", input);
            assert_eq!(res.episode, e, "Failed Episode, {}", input);
        }
    }
}
