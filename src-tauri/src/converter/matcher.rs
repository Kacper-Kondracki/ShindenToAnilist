use crate::converter::{database, regexes, shinden};
use chrono::Datelike;
use itertools::Itertools;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::iter;
use std::ops::Range;

#[derive(Serialize, Deserialize, Debug, Copy, Clone, Default, Eq, PartialEq)]
pub struct ExtractedMetadata {
    pub season: Option<u8>,
    pub part: Option<u8>,
}

fn roman_to_decimal(roman: &str) -> Option<u8> {
    match roman.to_uppercase().as_str() {
        "I" => Some(1),
        "II" => Some(2),
        "III" => Some(3),
        "IV" => Some(4),
        "V" => Some(5),
        "VI" => Some(6),
        "VII" => Some(7),
        "VIII" => Some(8),
        "IX" => Some(9),
        _ => None,
    }
}

fn word_to_decimal(word: &str) -> Option<u8> {
    match word.to_lowercase().as_str() {
        "one" | "first" => Some(1),
        "two" | "second" => Some(2),
        "three" | "third" => Some(3),
        "four" | "fourth" => Some(4),
        "five" | "fifth" => Some(5),
        "six" | "sixth" => Some(6),
        "seven" | "seventh" => Some(7),
        "eight" | "eighth" => Some(8),
        "nine" | "ninth" => Some(9),
        "final" | "last" => Some(99),
        _ => None,
    }
}

fn capture_to_integer(capture: &str) -> Option<u8> {
    if let Ok(n) = capture.parse::<u8>() {
        return Some(n);
    }

    if let Some(n) = roman_to_decimal(capture) {
        return Some(n);
    }

    word_to_decimal(capture)
}

pub fn extract_metadata(title: &str) -> ExtractedMetadata {
    let title_no_year = regexes::YEAR
        .replace_all(title, "")
        .split_whitespace()
        .join(" ");

    let seasons = extract_season(&title_no_year);
    let parts = extract_part(&title_no_year);

    let mut season = None;
    let mut part = None;

    if !seasons.is_empty() && parts.is_empty() {
        season = Some(seasons.first().unwrap().1);
    } else if seasons.is_empty() && !parts.is_empty() {
        part = Some(parts.first().unwrap().1);
    }

    'outer: for s in &seasons {
        season = Some(s.1);
        part = None;
        for p in &parts {
            if s.0.end <= p.0.start || p.0.end <= s.0.start {
                part = Some(p.1);
                break 'outer;
            }
        }
    }

    ExtractedMetadata { season, part }
}

pub fn extract_metadata_db(entry: &database::AnimeEntry) -> Vec<ExtractedMetadata> {
    let mut metadata_list = Vec::new();
    let mut results = Vec::new();

    let mut has_season_final = false;
    let mut has_part_final = false;
    for title in iter::once(&entry.title).chain(&entry.synonyms) {
        let metadata = extract_metadata(title.as_str());

        if metadata.season == Some(99) {
            has_season_final = true;
        }

        if metadata.part == Some(99) {
            has_part_final = true;
        }

        metadata_list.push(metadata);
    }
    let metadata = metadata_list
        .iter()
        .fold(ExtractedMetadata::default(), |mut acc, m| {
            match (acc.season, m.season) {
                (None, Some(y)) => {
                    acc.season = Some(y);
                }
                (Some(x), Some(y)) if y > x && y != 99 => {
                    acc.season = Some(y);
                }
                _ => {}
            }
            match (acc.part, m.part) {
                (None, Some(y)) => {
                    acc.part = Some(y);
                }
                (Some(x), Some(y)) if y > x && y != 99 => {
                    acc.part = Some(y);
                }
                _ => {}
            }

            acc
        });

    results.push(metadata);

    if has_season_final || has_part_final {
        let mut metadata_final = metadata;
        if has_season_final {
            metadata_final.season = Some(99);
        }
        if has_part_final {
            metadata_final.part = Some(99)
        }

        results.push(metadata_final);
    }

    results
}

fn extract_season(title: &str) -> Vec<(Range<usize>, u8)> {
    let season_regexes: &[&Regex] = &[
        &regexes::SEASON_DECIMAL,
        &regexes::SEASON_ROMAN,
        &regexes::SEASON_NUMERAL,
        &regexes::DECIMAL_SEASON,
        &regexes::ROMAN_SEASON,
        &regexes::NUMERAL_SEASON,
    ];

    let end_regexes: &[&Regex] = &[
        &regexes::SEASON_DECIMAL_END,
        &regexes::SEASON_ROMAN_END,
        &regexes::SEASON_NUMERAL_END,
    ];

    let mut matches = Vec::new();

    if regexes::SEASON.is_match(title) {
        matches.extend(collect_matches(title, season_regexes));
    }

    if !regexes::PART.is_match(title) {
        matches.extend(collect_matches(title, end_regexes));
    }

    matches
}

fn extract_part(title: &str) -> Vec<(Range<usize>, u8)> {
    let part_regexes: &[&Regex] = &[
        &regexes::PART_DECIMAL,
        &regexes::PART_ROMAN,
        &regexes::PART_NUMERAL,
        &regexes::DECIMAL_PART,
        &regexes::ROMAN_PART,
        &regexes::NUMERAL_PART,
    ];

    let mut matches = Vec::new();

    if regexes::PART.is_match(title) {
        matches.extend(collect_matches(title, part_regexes));
    }

    matches
}

fn collect_matches(title: &str, regexes: &[&Regex]) -> Vec<(Range<usize>, u8)> {
    let mut matches = Vec::new();
    for regex in regexes {
        for cap in regex.captures_iter(title) {
            if let Some(m) = cap.get(1)
                && let Some(n) = capture_to_integer(m.as_str())
            {
                matches.push((cap.get_match().range(), n));
            }
        }
    }

    matches
}

impl From<shinden::TitleStatus> for database::AnimeStatus {
    fn from(value: shinden::TitleStatus) -> Self {
        match value {
            shinden::TitleStatus::FinishedAiring => database::AnimeStatus::Finished,
            shinden::TitleStatus::CurrentlyAiring => database::AnimeStatus::Ongoing,
            shinden::TitleStatus::NotYetAired => database::AnimeStatus::Upcoming,
            shinden::TitleStatus::Proposal => database::AnimeStatus::Upcoming,
        }
    }
}

fn score_anime_status(x: impl Into<database::AnimeStatus>, y: database::AnimeStatus) -> f32 {
    let x = x.into();

    match (x, y) {
        (database::AnimeStatus::Finished, y) => match y {
            database::AnimeStatus::Finished => 1.0,
            database::AnimeStatus::Ongoing => 0.7,
            database::AnimeStatus::Upcoming => 0.2,
            database::AnimeStatus::Unknown => 0.5,
        },
        (database::AnimeStatus::Ongoing, y) => match y {
            database::AnimeStatus::Finished => 0.7,
            database::AnimeStatus::Ongoing => 1.0,
            database::AnimeStatus::Upcoming => 0.7,
            database::AnimeStatus::Unknown => 0.5,
        },
        (database::AnimeStatus::Upcoming, y) => match y {
            database::AnimeStatus::Finished => 0.2,
            database::AnimeStatus::Ongoing => 0.7,
            database::AnimeStatus::Upcoming => 1.0,
            database::AnimeStatus::Unknown => 0.5,
        },
        (database::AnimeStatus::Unknown, _) => 0.5,
    }
}

impl From<shinden::AnimeType> for database::AnimeType {
    fn from(value: shinden::AnimeType) -> Self {
        match value {
            shinden::AnimeType::Music => database::AnimeType::Special,
            shinden::AnimeType::Ova => database::AnimeType::Ova,
            shinden::AnimeType::Special => database::AnimeType::Special,
            shinden::AnimeType::Tv => database::AnimeType::Tv,
            shinden::AnimeType::Ona => database::AnimeType::Ona,
            shinden::AnimeType::Movie => database::AnimeType::Movie,
        }
    }
}

fn score_anime_type(x: impl Into<database::AnimeType>, y: database::AnimeType) -> f32 {
    let x = x.into();

    if x == y {
        return 1.0;
    }

    let similar_groups: &[&[database::AnimeType]] = &[
        &[
            database::AnimeType::Ova,
            database::AnimeType::Ona,
            database::AnimeType::Special,
        ],
        &[database::AnimeType::Tv, database::AnimeType::Ona],
    ];

    for group in similar_groups {
        if group.contains(&x) && group.contains(&y) {
            return 0.6;
        }
    }

    if y == database::AnimeType::Unknown {
        return 0.5;
    }

    0.2
}

fn score_year(x: Option<i32>, y: Option<i32>) -> f32 {
    match (x, y) {
        (Some(sy), Some(dy)) => {
            let diff = (sy - dy).abs();
            match diff {
                0 => 1.0,
                1 => 0.8,
                2 => 0.5,
                _ => 0.2,
            }
        }
        _ => 0.5,
    }
}

fn score_season_part(x: ExtractedMetadata, y: ExtractedMetadata) -> f32 {
    let mut score: f32 = 0.5;

    match (x.season, y.season) {
        (Some(x), Some(y)) => {
            if x == y {
                score += 0.5;
            } else {
                score -= 0.4;
            }
        }
        (Some(x), None) => {
            if x == 1 {
                score += 0.4;
            } else {
                score -= 0.4;
            }
        }
        (None, Some(y)) => {
            if y == 1 {
                score += 0.4;
            } else {
                score -= 0.4;
            }
        }
        (None, None) => {
            score += 0.4;
        }
    }

    match (x.part, y.part) {
        (Some(x), Some(y)) => {
            if x == y {
                score += 0.3;
            } else {
                score -= 0.3;
            }
        }
        (Some(x), None) => {
            if x == 1 {
                score += 0.2;
            } else {
                score -= 0.2;
            }
        }
        (None, Some(y)) => {
            if y == 1 {
                score += 0.2;
            } else {
                score -= 0.2;
            }
        }
        _ => {}
    }

    score.clamp(0.0, 1.0)
}

fn score_episodes(x: Option<i32>, y: i32) -> f32 {
    match (x, y) {
        (Some(x), y) if x > 0 && y > 0 => {
            let ratio = (x.min(y) as f32) / (x.max(y) as f32);
            (ratio * ratio).sqrt()
        }
        _ => 0.5,
    }
}

fn season_center(season: database::Season) -> Option<i32> {
    match season {
        database::Season::Spring => Some(3),
        database::Season::Summer => Some(6),
        database::Season::Fall => Some(9),
        database::Season::Winter => Some(12),
        database::Season::Undefined => None,
    }
}

fn circular_month_distance(x: i32, y: i32) -> i32 {
    let diff = (x - y).abs();
    diff.min(12 - diff)
}

fn score_month_season(x: Option<i32>, y: database::Season) -> f32 {
    let season_center = season_center(y);

    match (x, season_center) {
        (Some(x), Some(y)) => {
            let diff = circular_month_distance(x, y);
            match diff {
                0..=3 => 1.0,
                4 => 0.8,
                5..=6 => 0.5,
                _ => 0.2,
            }
        }
        _ => 0.5,
    }
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone, Default)]
pub struct ScoreBreakdown {
    pub ngram_score: f32,
    pub season_part_score: f32,
    pub year_score: f32,
    pub type_score: f32,
    pub status_score: f32,
    pub month_season_score: f32,
    pub episode_score: f32,
    pub final_score: f32,
}

#[derive(Serialize, Debug, Clone)]
pub struct MatchCandidate<'db> {
    pub candidate: &'db database::AnimeEntry,
    pub entry_metadata: ExtractedMetadata,
    pub candidate_metadata: Vec<ExtractedMetadata>,
    pub score_breakdown: ScoreBreakdown,
    pub likely_match: bool,
}

#[derive(Debug, Clone, Copy)]
pub struct MatcherConfig {
    pub ngram_weight: f32,
    pub season_part_weight: f32,
    pub year_weight: f32,
    pub type_weight: f32,
    pub status_weight: f32,
    pub month_season_weight: f32,
    pub episode_weight: f32,
    pub match_threshold: f32,
}

impl Default for MatcherConfig {
    fn default() -> Self {
        Self {
            ngram_weight: 0.45,
            season_part_weight: 0.10,
            year_weight: 0.10,
            type_weight: 0.10,
            status_weight: 0.05,
            month_season_weight: 0.05,
            episode_weight: 0.10,
            match_threshold: 0.85,
        }
    }
}

pub fn score_shinden_candidate<'db>(
    shinden: &shinden::AnimeEntry,
    candidate: &'db database::AnimeEntry,
    ngram_score: f32,
    shinden_metadata: ExtractedMetadata,
    db_metadata: &[ExtractedMetadata],
    config: MatcherConfig,
) -> MatchCandidate<'db> {
    let season_part_score = db_metadata
        .iter()
        .copied()
        .map(|y| score_season_part(shinden_metadata, y))
        .fold(0.0f32, |acc, x| acc.max(x));

    let year_score = score_year(
        shinden.premiere_date.map(|x| x.year()),
        candidate.anime_season.year,
    );
    let type_score = score_anime_type(shinden.anime_type, candidate.anime_type);
    let status_score = score_anime_status(shinden.title_status, candidate.status);
    let month_season_score = score_month_season(
        shinden.premiere_date.map(|x| x.month() as i32),
        candidate.anime_season.season,
    );
    let episode_score = score_episodes(shinden.episodes, candidate.episodes);

    let final_score = ngram_score * config.ngram_weight
        + season_part_score * config.season_part_weight
        + year_score * config.year_weight
        + type_score * config.type_weight
        + status_score * config.status_weight
        + month_season_score * config.month_season_weight
        + episode_score * config.episode_weight;

    let ngram_score = ngram_score.clamp(0.0, 1.0);
    let season_part_score = season_part_score.clamp(0.0, 1.0);
    let year_score = year_score.clamp(0.0, 1.0);
    let type_score = type_score.clamp(0.0, 1.0);
    let status_score = status_score.clamp(0.0, 1.0);
    let month_season_score = month_season_score.clamp(0.0, 1.0);
    let episode_score = episode_score.clamp(0.0, 1.0);
    let final_score = final_score.clamp(0.0, 1.0);

    let score_breakdown = ScoreBreakdown {
        ngram_score,
        season_part_score,
        year_score,
        type_score,
        status_score,
        month_season_score,
        episode_score,
        final_score,
    };

    MatchCandidate {
        candidate,
        entry_metadata: shinden_metadata,
        candidate_metadata: db_metadata.into(),
        score_breakdown,
        likely_match: final_score >= config.match_threshold,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_metadata() {
        assert_eq!(
            extract_metadata("Attack on Titan Season 2"),
            ExtractedMetadata {
                season: Some(2),
                part: None
            }
        );
        assert_eq!(
            extract_metadata("Attack on Titan S2"),
            ExtractedMetadata {
                season: Some(2),
                part: None
            }
        );
        assert_eq!(
            extract_metadata("Attack on Titan: Season 3"),
            ExtractedMetadata {
                season: Some(3),
                part: None
            }
        );
        assert_eq!(
            extract_metadata("Shingeki no Kyojin Season III"),
            ExtractedMetadata {
                season: Some(3),
                part: None
            }
        );
        assert_eq!(
            extract_metadata("Boku no Hero Academia 3"),
            ExtractedMetadata {
                season: Some(3),
                part: None
            }
        );
        assert_eq!(
            extract_metadata("Boku no Hero Academia III"),
            ExtractedMetadata {
                season: Some(3),
                part: None
            }
        );
        assert_eq!(
            extract_metadata("Jojo's Bizarre Adventure Part 5"),
            ExtractedMetadata {
                season: None,
                part: Some(5)
            }
        );
        assert_eq!(
            extract_metadata("Jojo's Bizarre Adventure Part V"),
            ExtractedMetadata {
                season: None,
                part: Some(5)
            }
        );
        assert_eq!(
            extract_metadata("Attack on Titan: The Final Season Part 2"),
            ExtractedMetadata {
                season: Some(99),
                part: Some(2)
            }
        );
    }
}
