use serde::{
    Deserialize,
    Serialize,
};

use crate::converter::{
    common::AnimeId,
    database::{
        AnimeStatus,
        AnimeType,
        Season,
    },
};

#[derive(Serialize, Deserialize, Debug, Copy, Clone, Default)]
pub struct ScoreBreakdown {
    search_score: f32,
    similarity_score: f32,
    season_score: f32,
    year_score: f32,
    type_score: f32,
    status_score: f32,
    seasonal_score: f32,
    episodes_score: f32,
    final_score: f32,
}

impl ScoreBreakdown {
    pub fn search_score(&self) -> f32 { self.search_score }
    pub fn similarity_score(&self) -> f32 { self.similarity_score }
    pub fn season_score(&self) -> f32 { self.season_score }
    pub fn year_score(&self) -> f32 { self.year_score }
    pub fn type_score(&self) -> f32 { self.type_score }
    pub fn status_score(&self) -> f32 { self.status_score }
    pub fn seasonal_score(&self) -> f32 { self.seasonal_score }
    pub fn episodes_score(&self) -> f32 { self.episodes_score }
    pub fn final_score(&self) -> f32 { self.final_score }
}

pub struct MatchResult {
    items: Vec<(AnimeId, ScoreBreakdown)>,
    winner: Vec<(AnimeId, ScoreBreakdown)>,
    top: Vec<(AnimeId, ScoreBreakdown)>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct Matcher {
    pub search_weight: f32,
    pub similarity_weight: f32,
    pub season_weight: f32,
    pub year_weight: f32,
    pub type_weight: f32,
    pub status_weight: f32,
    pub seasonal_weight: f32,
    pub episodes_weight: f32,
    pub match_threshold: f32,
    pub delta_threshold: f32,
    pub single_threshold: f32,
}

impl Matcher {
    pub fn strict_preset() -> Self {
        Self {
            search_weight: 0.18,
            similarity_weight: 0.19,
            season_weight: 0.06,
            year_weight: 0.19,
            type_weight: 0.13,
            status_weight: 0.06,
            seasonal_weight: 0.14,
            episodes_weight: 0.06,
            match_threshold: 0.89,
            delta_threshold: 0.15,
            single_threshold: 0.74,
        }
    }

    pub fn fuzzy_preset() -> Self {
        Self {
            search_weight: 0.30,
            similarity_weight: 0.10,
            season_weight: 0.04,
            year_weight: 0.11,
            type_weight: 0.08,
            status_weight: 0.19,
            seasonal_weight: 0.12,
            episodes_weight: 0.06,
            match_threshold: 0.93,
            delta_threshold: 0.15,
            single_threshold: 0.80,
        }
    }

    fn score_status(status_a: Option<AnimeStatus>, status_b: Option<AnimeStatus>) -> f32 {
        let Some(status_a) = status_a else { return 0.5 };
        match (status_a, status_b) {
            (AnimeStatus::Finished, b) => match b {
                Some(AnimeStatus::Finished) => 1.0,
                Some(AnimeStatus::Ongoing) => 0.6,
                Some(AnimeStatus::Upcoming) => 0.2,
                None => 0.2,
            },
            (AnimeStatus::Ongoing, b) => match b {
                Some(AnimeStatus::Finished) => 0.2,
                Some(AnimeStatus::Ongoing) => 1.0,
                Some(AnimeStatus::Upcoming) => 0.6,
                None => 0.2,
            },
            (AnimeStatus::Upcoming, b) => match b {
                Some(AnimeStatus::Finished) => 0.2,
                Some(AnimeStatus::Ongoing) => 0.2,
                Some(AnimeStatus::Upcoming) => 1.0,
                None => 0.4,
            },
        }
    }
    fn score_type(type_a: Option<AnimeType>, type_b: Option<AnimeType>) -> f32 {
        let (type_a, type_b) = match (type_a, type_b) {
            (None, _) => return 0.5,
            (_, None) => return 0.5,
            (Some(a), Some(b)) => {
                if a == b {
                    return 1.0;
                }
                (a, b)
            },
        };

        let similar_groups: &[&[AnimeType]] = &[
            &[AnimeType::Ova, AnimeType::Ona, AnimeType::Special],
            &[AnimeType::Tv, AnimeType::Ona],
        ];

        for group in similar_groups {
            if group.contains(&type_a) && group.contains(&type_b) {
                return 0.7;
            }
        }

        0.2
    }
    fn score_year(year_a: Option<i32>, year_b: Option<i32>) -> f32 {
        match (year_a, year_b) {
            (Some(sy), Some(dy)) => {
                let diff = (sy - dy).abs();
                match diff {
                    0 => 1.0,
                    1 => 0.6,
                    2 => 0.25,
                    _ => 0.2,
                }
            },
            _ => 0.5,
        }
    }
    fn score_episodes(episode_a: Option<i32>, episode_b: i32) -> f32 {
        match (episode_a, episode_b) {
            (Some(x), y) if x > 0 && y > 0 => {
                let ratio = (x.min(y) as f32) / (x.max(y) as f32);
                ratio.powf(1.15).max(0.2).clamp(0.0, 1.0)
            },
            _ => 0.5,
        }
    }
    fn score_seasonal(year: Option<i32>, season: Option<Season>) -> f32 {
        let season_center = season.map(season_center);

        match (year, season_center) {
            (Some(x), Some(y)) => {
                let diff = circular_month_distance(x, y);
                match diff {
                    0..=3 => 1.0,
                    4 => 0.5,
                    5..=6 => 0.3,
                    _ => 0.2,
                }
            },
            _ => 0.5,
        }
    }

    pub fn score() -> MatchResult { todo!() }
}

fn season_center(season: Season) -> i32 {
    match season {
        Season::Spring => 3,
        Season::Summer => 6,
        Season::Fall => 9,
        Season::Winter => 12,
    }
}
fn circular_month_distance(month_a: i32, month_b: i32) -> i32 {
    let diff = (month_a - month_b).abs();
    diff.min(12 - diff)
}
