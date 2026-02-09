use std::cmp::Reverse;

use chrono::Datelike;
use ordered_float::OrderedFloat;
use serde::{
    Deserialize,
    Serialize,
};

use crate::{
    common::MatchView,
    converter::{
        common::AnimeId,
        database::{
            AnimeStatus,
            AnimeType,
            Season,
        },
    },
    database,
    extractor::{
        ConsolidatedMetadata,
        FINAL,
        TitleMetadata,
    },
    utils::ge_tol,
};

pub trait Matcher {
    fn score_candidates(
        &self,
        entry: &impl MatchView,
        candidates: Vec<(&database::AnimeEntry, f32)>,
    ) -> MatchResult;
}

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
    winner: Option<(AnimeId, ScoreBreakdown)>,
    top: Vec<(AnimeId, ScoreBreakdown)>,
}

impl MatchResult {
    pub fn items(&self) -> &[(AnimeId, ScoreBreakdown)] { &self.items }
    pub fn winner(&self) -> Option<(AnimeId, ScoreBreakdown)> { self.winner }
    pub fn top(&self) -> &[(AnimeId, ScoreBreakdown)] { &self.top }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct DefaultMatcher {
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

impl Default for DefaultMatcher {
    fn default() -> Self {
        Self {
            search_weight: 0.00,
            similarity_weight: 0.00,
            season_weight: 0.00,
            year_weight: 0.00,
            type_weight: 0.00,
            status_weight: 0.00,
            seasonal_weight: 0.00,
            episodes_weight: 0.00,
            match_threshold: 0.00,
            delta_threshold: 0.00,
            single_threshold: 0.00,
        }
    }
}

impl DefaultMatcher {
    fn score_status(status_a: Option<AnimeStatus>, status_b: AnimeStatus) -> f32 {
        let Some(status_a) = status_a else { return 0.5 };
        match (status_a, status_b) {
            (AnimeStatus::Finished, b) => match b {
                AnimeStatus::Finished => 1.0,
                AnimeStatus::Ongoing => 0.6,
                AnimeStatus::Upcoming => 0.2,
                AnimeStatus::Unknown => 0.2,
            },
            (AnimeStatus::Ongoing, b) => match b {
                AnimeStatus::Finished => 0.2,
                AnimeStatus::Ongoing => 1.0,
                AnimeStatus::Upcoming => 0.6,
                AnimeStatus::Unknown => 0.2,
            },
            (AnimeStatus::Upcoming, b) => match b {
                AnimeStatus::Finished => 0.2,
                AnimeStatus::Ongoing => 0.2,
                AnimeStatus::Upcoming => 1.0,
                AnimeStatus::Unknown => 0.4,
            },
            (AnimeStatus::Unknown, b) => match b {
                AnimeStatus::Unknown => 0.5,
                _ => 0.4,
            },
        }
    }
    fn score_type(type_a: Option<AnimeType>, type_b: AnimeType) -> f32 {
        let Some(type_a) = type_a else {
            return 0.5;
        };

        if type_a == type_b {
            return 1.0;
        }

        let similar_groups: &[&[AnimeType]] = &[
            &[AnimeType::Ova, AnimeType::Ona, AnimeType::Special],
            &[AnimeType::Tv, AnimeType::Ona],
        ];

        for group in similar_groups {
            if group.contains(&type_a) && group.contains(&type_b) {
                return 0.7;
            }
        }

        if type_a == AnimeType::Unknown || type_b == AnimeType::Unknown {
            return 0.5;
        }

        0.2
    }
    fn score_year(year_a: Option<Option<i32>>, year_b: Option<i32>) -> f32 {
        let Some(year_a) = year_a else {
            return 0.5;
        };

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
            (None, Some(_)) | (Some(_), None) => 0.4,
            (None, None) => 0.5,
        }
    }
    fn score_season(metadata: Option<&TitleMetadata>, consolidated_metadata: ConsolidatedMetadata) -> f32 {
        let Some(metadata) = metadata else {
            return 0.5;
        };

        let (s_a, s_fin_a) = metadata
            .season()
            .map(|s| (s, s == FINAL))
            .or(metadata.episode().map(|e| (e, e == FINAL)))
            .unwrap_or((1.0, false));

        let (s_b, s_fin_b) = consolidated_metadata
            .season()
            .map(|s| (s, consolidated_metadata.is_final_season()))
            .or(metadata
                .episode()
                .map(|e| (e, consolidated_metadata.is_final_episode())))
            .unwrap_or((1.0, false));

        let (p_a, p_fin_a) = metadata.part().map(|p| (p, p == FINAL)).unwrap_or((1.0, false));
        let (p_b, p_fin_b) = (
            consolidated_metadata.part().unwrap_or(1.0),
            consolidated_metadata.is_final_part(),
        );

        let s_a = s_a as i32;
        let s_b = s_b as i32;

        let p_a = p_a as i32;
        let p_b = p_b as i32;

        let mut score: f32 = 0.5;

        if s_a == s_b || s_fin_a && s_fin_b {
            score += 0.5;
        } else {
            score -= 0.4;
        }

        if p_a == p_b || p_fin_a && p_fin_b {
            score += 0.3;
        } else {
            score -= 0.2;
        }

        score.clamp(0.0, 1.0)
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
    fn score_seasonal(year: Option<Option<i32>>, season: Season) -> f32 {
        let Some(year) = year else {
            return 0.5;
        };

        let season_center = season_center(season);

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
            (None, Some(_)) | (Some(_), None) => 0.4,
            (None, None) => 0.5,
        }
    }

    fn score_candidate_initial(
        &self,
        entry: &impl MatchView,
        candidate: (&database::AnimeEntry, f32),
    ) -> (AnimeId, ScoreBreakdown) {
        let (candidate, search_score) = candidate;

        let season_score = Self::score_season(entry.title_metadata(), candidate.consolidated_metadata());
        let year_score = Self::score_year(entry.year(), candidate.year());
        let type_score = Self::score_type(entry.anime_type(), candidate.anime_type());
        let status_score = Self::score_status(entry.status(), candidate.status());
        let seasonal_score = Self::score_seasonal(
            entry.date().map(|d| d.map(|d| d.month() as i32)),
            candidate.season(),
        );
        let episodes_score = Self::score_episodes(entry.episodes(), candidate.episodes());

        let final_score = (search_score * self.search_weight
            // No similarity yet
            + season_score * self.season_weight
            + year_score * self.year_weight
            + type_score * self.type_weight
            + status_score * self.status_weight
            + seasonal_score * self.seasonal_weight
            + episodes_score * self.episodes_weight)
            .clamp(0.0, 1.0);

        let score_breakdown = ScoreBreakdown {
            search_score,
            similarity_score: 0.0,
            season_score,
            year_score,
            type_score,
            status_score,
            seasonal_score,
            episodes_score,
            final_score,
        };

        (candidate.id(), score_breakdown)
    }
}

fn season_center(season: Season) -> Option<i32> {
    match season {
        Season::Spring => Some(3),
        Season::Summer => Some(6),
        Season::Fall => Some(9),
        Season::Winter => Some(12),
        Season::Undefined => None,
    }
}
fn circular_month_distance(month_a: i32, month_b: i32) -> i32 {
    let diff = (month_a - month_b).abs();
    diff.min(12 - diff)
}

impl Matcher for DefaultMatcher {
    fn score_candidates(
        &self,
        entry: &impl MatchView,
        candidates: Vec<(&database::AnimeEntry, f32)>,
    ) -> MatchResult {
        let mut scored_items = candidates
            .into_iter()
            .map(|c| self.score_candidate_initial(entry, c))
            .collect::<Vec<_>>();

        if scored_items.is_empty() {
            return MatchResult {
                items: Vec::new(),
                winner: None,
                top: Vec::new(),
            };
        }

        scored_items.sort_by_key(|(_, k)| Reverse(OrderedFloat(k.final_score)));

        let mut winner: Option<(AnimeId, ScoreBreakdown)> = None;
        let first = scored_items[0];
        let mut top = scored_items[1..]
            .iter()
            .copied()
            .filter(|(_, s)| ge_tol(s.final_score, self.match_threshold))
            .collect::<Vec<_>>();

        if top.is_empty() {
            if ge_tol(first.1.final_score, self.single_threshold) {
                winner = Some(first);
                top.push(first);
            }
        } else {
            let second = top[0];
            top.insert(0, first);
            let delta = first.1.final_score - second.1.final_score;
            if ge_tol(delta, self.delta_threshold) && ge_tol(first.1.final_score, self.single_threshold) {
                winner = Some(first);
            }
        }

        MatchResult {
            winner,
            items: scored_items,
            top,
        }
    }
}
#[cfg(test)]
mod tests {
    use std::{
        fs::File,
        io::BufReader,
        time::Instant,
    };

    use rayon::prelude::*;

    use crate::{
        common::AnimeList,
        database::{
            AnimeDatabase,
            AnimeDatabaseLoad,
        },
        matcher::{
            DefaultMatcher,
            Matcher,
        },
        providers::shinden::ShindenList,
        searcher::{
            DefaultSearcher,
            Search,
            SearcherAnimeExt,
        },
    };

    #[tokio::test]
    async fn match_shinden_list_test() {
        let database = AnimeDatabase::get_from_mmap("anime-offline-database.jsonl").unwrap();
        // let shinden = ShindenList::get_from_shinden(196402).await.unwrap();
        let shinden: ShindenList =
            serde_json::from_reader(BufReader::new(File::open("shinden-test.json").unwrap())).unwrap();

        let searcher = DefaultSearcher::new(&database);
        let matcher = DefaultMatcher::default();
        let now = Instant::now();

        let results = shinden
            .par_values()
            .map(|entry| entry.search_by_title_ref(&database, &searcher, Search::options().strict().build()))
            .map(|(entry, candidates)| (entry, matcher.score_candidates(entry, candidates)))
            .collect::<Vec<_>>();

        let elapsed = now.elapsed();

        for (entry, result) in results.iter() {
            println!("=== {} ===", entry.title());
            for v in result.items() {
                println!("[{:.2}] {}", v.1.final_score(), database[v.0].title())
            }
        }

        let winners_count = results.iter().filter(|(_, m)| m.winner().is_some()).count();
        let tops_count = results.iter().filter(|(_, m)| !m.top().is_empty()).count();

        println!("TOOK       : {:.2?}", elapsed);
        println!("HAS TOP    : {}/{}", tops_count, shinden.len());
        println!("HAS WINNER : {}/{}", winners_count, shinden.len());
    }
}
