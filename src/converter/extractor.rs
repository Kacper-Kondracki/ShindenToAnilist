use indexmap::IndexMap;
use ordered_float::OrderedFloat;
use serde::{
    Deserialize,
    Serialize,
};
use unicode_segmentation::UnicodeSegmentation;

use crate::utils::normalize_str;

/// Metadata extracted from an anime title string.
///
/// Contains parsed season, part, and episode numbers along with the raw
/// token sequence that produced them.  Constructed via
/// [`title_processor::process`].
///
/// # Example
///
/// ```rust,ignore
/// use shinden_to_anilist_core::extractor::title_processor;
///
/// let meta = title_processor::process("Attack on Titan Season 3 Part 2");
/// assert_eq!(meta.season(), Some(3.0));
/// assert_eq!(meta.part(), Some(2.0));
/// assert_eq!(meta.episode(), None);
/// ```
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Default)]
pub struct TitleMetadata {
    season: Option<f32>,
    part: Option<f32>,
    episode: Option<f32>,
    tokens: Vec<Token>,
}

impl TitleMetadata {
    /// The detected season number, if any.
    pub fn season(&self) -> Option<f32> { self.season }
    /// The detected part number, if any.
    pub fn part(&self) -> Option<f32> { self.part }
    /// The detected episode number (for OVA/Movie numbering), if any.
    pub fn episode(&self) -> Option<f32> { self.episode }
    /// The raw sequence of tokens parsed from the title.
    pub fn tokens(&self) -> &[Token] { &self.tokens }
    /// Returns `true` if any token is a season-type keyword (Season, S, Series).
    pub fn has_season_keyword(&self) -> bool { self.tokens.iter().any(|t| t.is_season()) }
    /// Returns `true` if any token is a part-type keyword (Part, Arc, Cour).
    pub fn has_part_keyword(&self) -> bool { self.tokens.iter().any(|t| t.is_part()) }
    /// Returns `true` if any token is an episode-type keyword (Episode, OVA, Movie, …).
    pub fn has_episode_keyword(&self) -> bool { self.tokens.iter().any(|t| t.is_episode()) }
}

/// A single token extracted from a title during parsing.
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Copy)]
pub enum Token {
    /// A plain number (e.g. `3` from "Season 3").
    Num(f32),
    /// An ordinal (e.g. `2.0` from "2nd", or [`FINAL`] from "Final").
    Ordinal(f32),
    /// A recognized keyword such as "Season" or "Movie".
    Keyword(Keyword),
}

/// Sentinel value representing "final" season/part/episode.
///
/// Used when titles contain words like "Final", "Finale", or "Last".
pub const FINAL: f32 = 99.0;

impl Token {
    /// Returns `true` if this token is a season keyword.
    pub fn is_season(&self) -> bool {
        match self {
            Token::Keyword(kw) => kw.is_season(),
            _ => false,
        }
    }
    /// Returns `true` if this token is a part keyword.
    pub fn is_part(&self) -> bool {
        match self {
            Token::Keyword(kw) => kw.is_part(),
            _ => false,
        }
    }
    /// Returns `true` if this token is an episode keyword.
    pub fn is_episode(&self) -> bool {
        match self {
            Token::Keyword(kw) => kw.is_episode(),
            _ => false,
        }
    }
}

/// A recognized keyword that categorizes the adjacent number.
///
/// Keywords are grouped into three families:
/// - **Season**: `Season`, `S`, `Series`
/// - **Part**: `Part`, `Arc`, `Cour`
/// - **Episode**: `Episode`, `Ova`, `Ona`, `Movie`, `Special`
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Copy)]
pub enum Keyword {
    /// "Season"
    Season,
    /// Short form "S" (e.g. "S3").
    S,
    /// "Series"
    Series,
    /// "Part"
    Part,
    /// "Arc"
    Arc,
    /// "Cour"
    Cour,
    /// "Episode" / "Episodes"
    Episode,
    /// "OVA"
    Ova,
    /// "ONA"
    Ona,
    /// "Movie" / "Film" / "Theatre" / "Theater"
    Movie,
    /// "Special" / "Specials"
    Special,
}

impl Keyword {
    /// Returns `true` for season-family keywords (`Season`, `S`, `Series`).
    pub fn is_season(&self) -> bool { matches!(self, Keyword::Season | Keyword::S | Keyword::Series) }
    /// Returns `true` for part-family keywords (`Part`, `Arc`, `Cour`).
    pub fn is_part(&self) -> bool { matches!(self, Keyword::Part | Keyword::Arc | Keyword::Cour) }
    /// Returns `true` for episode-family keywords (`Episode`, `Ova`, `Ona`, `Movie`, `Special`).
    pub fn is_episode(&self) -> bool {
        matches!(
            self,
            Keyword::Episode | Keyword::Ova | Keyword::Ona | Keyword::Movie | Keyword::Special
        )
    }
}

/// Metadata consolidated from multiple title variants (primary + synonyms).
///
/// Produced by [`title_processor::consolidate`], this picks the most-voted
/// season/part/episode values across all synonym [`TitleMetadata`] instances
/// and tracks whether any of them was marked as "final".
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
    /// Consolidated season number.
    pub fn season(&self) -> Option<f32> { self.season }
    /// Consolidated part number.
    pub fn part(&self) -> Option<f32> { self.part }
    /// Consolidated episode number.
    pub fn episode(&self) -> Option<f32> { self.episode }
    /// `true` if any synonym indicated a final season.
    pub fn is_final_season(&self) -> bool { self.is_final_season }
    /// `true` if any synonym indicated a final part.
    pub fn is_final_part(&self) -> bool { self.is_final_part }
    /// `true` if any synonym indicated a final episode.
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

/// Title parsing and metadata consolidation utilities.
pub mod title_processor {
    use ahash::AHashSet;
    use lazy_regex::regex;

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

    /// Aggregates metadata from multiple [`TitleMetadata`] instances (e.g.
    /// one per synonym) into a single [`ConsolidatedMetadata`].
    ///
    /// For each dimension (season, part, episode), the value that appears
    /// most frequently across all metadata entries wins.  The "is_final"
    /// flags are set if *any* entry contained the [`FINAL`] sentinel.
    pub fn consolidate(metadata_list: &[&TitleMetadata]) -> ConsolidatedMetadata {
        if metadata_list.is_empty() {
            return ConsolidatedMetadata::default();
        }

        let seasons: Vec<f32> = metadata_list.iter().filter_map(|m| m.season).collect();
        let parts: Vec<f32> = metadata_list.iter().filter_map(|m| m.part).collect();
        let episodes: Vec<f32> = metadata_list.iter().filter_map(|m| m.episode).collect();

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
        let year_re = regex!(r"^(?:19|20)\d{2}$");
        let end_num_re = regex!(r"(\d+\.?\d*).?$");
        let ordinal_re = regex!(r"^(\d+)(?:st|nd|rd|th)\.?$");

        if year_re.is_match(word) {
            return None;
        }

        end_num_re
            .captures(word)
            .and_then(|cap| cap[1].parse().ok().map(Token::Num))
            .or_else(|| {
                ordinal_re
                    .captures(word)
                    .and_then(|cap| cap[1].parse().ok().map(Token::Ordinal))
            })
            .or_else(|| word_to_num(word))
            .or_else(|| word_to_ordinal(word))
            .or_else(|| get_keyword(word).map(Token::Keyword))
    }

    fn contains_special_chars(gap: &str) -> bool {
        static SPECIAL_CHARS: &[char] = &['(', ')', '[', ']', '<', '>', '-', ':'];
        gap.contains(SPECIAL_CHARS)
    }

    /// Tokenizes a title into a sequence of [`Token`]s.
    ///
    /// Extracts numbers, ordinals, and keywords from the normalized title.
    /// Numbers embedded in the middle of a title are only kept when adjacent
    /// to a keyword or a special separator character (e.g. `:`, `-`).
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
                Token::Num(n) | Token::Ordinal(n) => {
                    if n != FINAL && n > 20.0 {
                        continue; // Skip unrealistic seasons
                    }

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

    /// Parses a title string into full [`TitleMetadata`].
    ///
    /// This is the main entry point for title metadata extraction.
    /// It tokenizes the title and then resolves season, part, and episode
    /// values from the token stream using keyword–number associations.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use shinden_to_anilist_core::extractor::title_processor;
    ///
    /// let meta = title_processor::process("Mob Psycho 100 III");
    /// assert_eq!(meta.season(), Some(3.0));
    /// ```
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
            ("Mob Psycho 100", None, None, None),
            ("Mob Psycho 100 I", Some(1.0), None, None),
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
