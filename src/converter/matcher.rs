use std::cmp::Reverse;

use ahash::AHashMap;
use chrono::Datelike;
use ordered_float::OrderedFloat;
use serde::{
    Deserialize,
    Serialize,
};

use crate::{
    common::{
        AnimeList,
        MatchView,
    },
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

/// Trait for scoring a set of search candidates against a query entry.
///
/// Given an anime entry (implementing [`MatchView`]) and a list of database
/// candidates with their search scores, a `Matcher` produces a [`MatchResult`]
/// containing scored items, a top-tier list, and an optional winner.
pub trait Matcher {
    /// Scores all `candidates` against `entry` and returns a [`MatchResult`].
    ///
    /// `neutral` is the fallback score (typically `0.5`) used for any
    /// scoring dimension where the entry provides no data (returns `None`).
    fn score_candidates(
        &self,
        entry: &impl MatchView,
        candidates: &[(&database::AnimeEntry, f32)],
        neutral: f32,
    ) -> MatchResult;
}

/// Per-candidate breakdown of all individual scoring dimensions.
///
/// Each field is a `0.0..=1.0` score for one aspect of the match.
/// The `final_score` is the weighted combination of all dimensions.
#[derive(Serialize, Deserialize, Debug, Copy, Clone, Default)]
pub struct ScoreBreakdown {
    /// Score from the search phase.
    pub search_score: f32,
    /// Season / part number agreement.
    pub season_score: f32,
    /// Premiere year proximity.
    pub year_score: f32,
    /// Anime type (TV, Movie, …) agreement.
    pub type_score: f32,
    /// Airing status agreement.
    pub status_score: f32,
    /// Airing season agreement.
    pub seasonal_score: f32,
    /// Episode count proximity.
    pub episodes_score: f32,
    /// Weighted combination of all above scores.
    pub final_score: f32,
}

/// The result of scoring a set of candidates for a single query entry.
///
/// Contains:
/// - **items**: All candidates, sorted descending by `final_score`.
/// - **top**: The subset of items whose score meets `match_threshold`.
/// - **winner**: The single best match, if one is decisive (i.e. it exceeds
///   the threshold AND leads the runner-up by at least `delta_threshold`).
pub struct MatchResult {
    items: Vec<(AnimeId, ScoreBreakdown)>,
    winner: Option<(AnimeId, ScoreBreakdown)>,
    top: Vec<(AnimeId, ScoreBreakdown)>,
}

impl MatchResult {
    /// All scored candidates, sorted descending by `final_score`.
    pub fn items(&self) -> &[(AnimeId, ScoreBreakdown)] { &self.items }
    /// Like [`items`](MatchResult::items), but resolves each [`AnimeId`]
    /// to an [`AnimeEntry`](database::AnimeEntry) reference from `database`.
    pub fn items_ref<'a>(
        &self,
        database: &'a impl AnimeList<Entry = database::AnimeEntry>,
    ) -> impl Iterator<Item = (&'a database::AnimeEntry, ScoreBreakdown)> {
        self.items.iter().map(|&(k, v)| (database.get_unwrap(k), v))
    }
    /// The single decisive winner, if any.
    ///
    /// `None` when no candidate passed the threshold, or when the top two
    /// candidates are too close together (within `delta_threshold`).
    pub fn winner(&self) -> Option<(AnimeId, ScoreBreakdown)> { self.winner }
    /// Like [`winner`](MatchResult::winner), but resolves to an entry reference.
    pub fn winner_ref<'a>(
        &self,
        database: &'a impl AnimeList<Entry = database::AnimeEntry>,
    ) -> Option<(&'a database::AnimeEntry, ScoreBreakdown)> {
        self.winner.map(|(k, v)| (database.get_unwrap(k), v))
    }
    /// Candidates that scored at or above `match_threshold`.
    /// Like [`top`](MatchResult::top), but resolves to entry references.
    pub fn top(&self) -> &[(AnimeId, ScoreBreakdown)] { &self.top }
    pub fn top_ref<'a>(
        &self,
        database: &'a impl AnimeList<Entry = database::AnimeEntry>,
    ) -> impl Iterator<Item = (&'a database::AnimeEntry, ScoreBreakdown)> {
        self.top.iter().map(|&(k, v)| (database.get_unwrap(k), v))
    }
}

/// The default weighted-scoring matcher.
///
/// Each scoring dimension has a weight that sums to `≈1.0`.  The
/// `match_threshold` controls the minimum score to be considered a viable
/// match, and `delta_threshold` is the minimum gap between the top two
/// candidates required to declare a decisive winner.
///
/// # Presets
///
/// - [`Default::default()`] — balanced weights.
/// - [`DefaultMatcher::strict_preset()`] — higher thresholds, tuned via
///   Bayesian optimization.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, Default)]
pub struct DefaultMatcher {
    /// Weight for the search score.
    pub search_weight: f32,
    /// Weight for season/part agreement.
    pub season_weight: f32,
    /// Weight for year proximity.
    pub year_weight: f32,
    /// Weight for type agreement.
    pub type_weight: f32,
    /// Weight for status agreement.
    pub status_weight: f32,
    /// Weight for seasonal (month–season) agreement.
    pub seasonal_weight: f32,
    /// Weight for episode count proximity.
    pub episodes_weight: f32,
    /// Minimum `final_score` to be considered a match.
    pub match_threshold: f32,
    /// Minimum score gap between 1st and 2nd place to declare a winner.
    pub delta_threshold: f32,
}

/// Normalizes a slice of priority values into weights that sum to `1.0`.
///
/// Each value is raised to the power of `gamma` to control the contrast
/// between high and low priorities, then the array is normalized.
///
/// # Edge cases
///
/// - Empty slices are a no-op.
/// - If all values are zero after exponentiation, equal weights are assigned.
///
/// # Example
///
/// ```rust,ignore
/// use shinden_to_anilist_core::matcher::generate_weights;
///
/// let mut w = [2.0, 1.0];
/// generate_weights(&mut w, 1.0);
/// assert!((w[0] - 0.6667).abs() < 0.01);
/// assert!((w[1] - 0.3333).abs() < 0.01);
/// ```
pub fn generate_weights(priorities: &mut [f32], gamma: f32) {
    if priorities.is_empty() {
        return;
    }

    for p in priorities.iter_mut() {
        *p = p.powf(gamma);
    }

    let sum: f32 = priorities.iter().sum();

    if sum == 0.0 {
        let equal_weight = 1.0 / priorities.len() as f32;
        for p in priorities.iter_mut() {
            *p = equal_weight;
        }
    } else {
        for p in priorities.iter_mut() {
            *p /= sum;
        }
    }
}

/// Individual scoring functions for each match dimension.
///
/// All functions return a score in `0.0..=1.0`, where `1.0` is a perfect
/// match and lower values indicate increasing disagreement.
pub mod scoring {
    use super::*;

    /// Scores season and part agreement between a query and a candidate.
    ///
    /// Compares the season number (or episode number as fallback) and part
    /// number.  An exact match or both being "final" yields the highest
    /// score.
    pub fn score_season(metadata: &TitleMetadata, consolidated_metadata: ConsolidatedMetadata) -> f32 {
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

    /// Scores year proximity.
    ///
    /// Returns `1.0` for an exact match, `0.6` for ±1 year, `0.25` for ±2,
    /// and `0.2` for larger differences.  Mixed `Some`/`None` yields `0.4`.
    /// When `None`/`None` → `0.7`.
    pub fn score_year(year_a: Option<i32>, year_b: Option<i32>) -> f32 {
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
            (None, None) => 0.7,
        }
    }

    /// Scores anime type similarity.
    ///
    /// Exact match → `1.0`, similar types (e.g. OVA/ONA/Special) → `0.7`,
    /// one unknown → `0.5`, otherwise `0.2`.
    pub fn score_type(type_a: AnimeType, type_b: AnimeType) -> f32 {
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

    /// Scores airing status compatibility.
    ///
    /// Returns `1.0` for identical statuses and lower values for larger
    /// status mismatches (e.g. Finished vs. Upcoming → `0.2`).
    /// one unknown → `0.5`, otherwise `0.7`
    pub fn score_status(status_a: AnimeStatus, status_b: AnimeStatus) -> f32 {
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
                AnimeStatus::Unknown => 0.7,
                _ => 0.5,
            },
        }
    }

    /// Scores the alignment between a premiere month and an airing season.
    ///
    /// Uses circular month distance.  Within 3 months of the season center
    /// → `1.0`; 4 months → `0.5`; 5–6 months → `0.3`; further → `0.2`.
    /// One unknown → `0.4`, otherwise `0.7`
    pub fn score_seasonal(month: Option<i32>, season: Season) -> f32 {
        let season_center = season_center(season);

        match (month, season_center) {
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
            (None, None) => 0.7,
        }
    }

    /// Scores episode count proximity.
    ///
    /// When both counts are positive, uses a ratio-based power curve.
    /// When either count is zero or unknown, returns `0.7`.
    pub fn score_episodes(episode_a: i32, episode_b: i32) -> f32 {
        match (episode_a, episode_b) {
            (x, y) if x > 0 && y > 0 => {
                let ratio = (x.min(y) as f32) / (x.max(y) as f32);
                ratio.powf(1.15).max(0.2).clamp(0.0, 1.0)
            },
            _ => 0.7,
        }
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

impl DefaultMatcher {
    /// Creates a matcher from explicit weight values.
    ///
    /// The 7-element `weights` array maps to:
    /// `[search, season, year, type, status, seasonal, episodes]`.
    pub fn from_weights(weights: [f32; 7], match_threshold: f32, delta_threshold: f32) -> Self {
        Self {
            search_weight: weights[0],
            season_weight: weights[1],
            year_weight: weights[2],
            type_weight: weights[3],
            status_weight: weights[4],
            seasonal_weight: weights[5],
            episodes_weight: weights[6],
            match_threshold,
            delta_threshold,
        }
    }

    /// A stricter preset with weights tuned via Bayesian optimization.
    ///
    /// Uses `match_threshold = 0.70` and `delta_threshold = 0.075`.
    pub fn strict_preset() -> Self {
        let mut weights = [1.00, 0.19, 0.77, 0.70, 0.48, 0.22, 0.32];
        generate_weights(&mut weights, 0.66);
        Self::from_weights(weights, 0.70, 0.075)
    }

    fn score_candidate(
        &self,
        entry: &impl MatchView,
        candidate: (&database::AnimeEntry, f32),
        neutral: f32,
    ) -> (AnimeId, ScoreBreakdown) {
        use scoring::*;

        let (candidate, search_score) = candidate;

        let season_score = entry
            .title_metadata()
            .map(|v| score_season(v, candidate.consolidated_metadata()))
            .unwrap_or(neutral);
        let year_score = entry
            .year()
            .map(|v| score_year(v, candidate.year()))
            .unwrap_or(neutral);
        let type_score = entry
            .anime_type()
            .map(|v| score_type(v, candidate.anime_type()))
            .unwrap_or(neutral);
        let status_score = entry
            .status()
            .map(|v| score_status(v, candidate.status()))
            .unwrap_or(neutral);
        let seasonal_score = entry
            .date()
            .map(|d| d.map(|d| d.month() as i32))
            .map(|v| score_seasonal(v, candidate.season()))
            .unwrap_or(neutral);
        let episodes_score = entry
            .episodes()
            .map(|v| score_episodes(v, candidate.episodes()))
            .unwrap_or(neutral);

        let final_score = (search_score * self.search_weight
            + season_score * self.season_weight
            + year_score * self.year_weight
            + type_score * self.type_weight
            + status_score * self.status_weight
            + seasonal_score * self.seasonal_weight
            + episodes_score * self.episodes_weight)
            .clamp(0.0, 1.0);

        let score_breakdown = ScoreBreakdown {
            search_score,
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

impl Matcher for DefaultMatcher {
    fn score_candidates(
        &self,
        entry: &impl MatchView,
        candidates: &[(&database::AnimeEntry, f32)],
        neutral: f32,
    ) -> MatchResult {
        let mut scored_items: Vec<(AnimeId, ScoreBreakdown)> = candidates
            .iter()
            .map(|&c| self.score_candidate(entry, c, neutral))
            .collect();

        if scored_items.is_empty() {
            return MatchResult {
                items: Vec::new(),
                winner: None,
                top: Vec::new(),
            };
        }

        scored_items.sort_by_key(|(_, k)| Reverse(OrderedFloat(k.final_score)));

        let mut winner: Option<(AnimeId, ScoreBreakdown)> = None;

        let top: Vec<(AnimeId, ScoreBreakdown)> = scored_items
            .iter()
            .copied()
            .filter(|(_, s)| ge_tol(s.final_score, self.match_threshold))
            .collect();

        if top.len() == 1 {
            winner = Some(top[0]);
        } else if top.len() >= 2 {
            let first = top[0];
            let second = top[1];

            if ge_tol(first.1.final_score - second.1.final_score, self.delta_threshold) {
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

/// De-duplicates winners across multiple [`MatchResult`]s.
///
/// When the same database entry is the winner for more than one query,
/// only the query with the highest `final_score` keeps the win;
/// the others have their `winner` cleared to `None`.
pub fn finalize_matches(results: &mut [&mut MatchResult]) {
    let mut winners: AHashMap<AnimeId, (f32, usize)> = AHashMap::new();
    for (i, result) in results.iter().enumerate() {
        if let Some((id, score)) = result.winner {
            winners
                .entry(id)
                .and_modify(|(x, ind)| {
                    if score.final_score > *x {
                        *x = score.final_score;
                        *ind = i;
                    }
                })
                .or_insert((score.final_score, i));
        }
    }

    for (i, result) in results.iter_mut().enumerate() {
        if let Some((id, _)) = result.winner {
            let (_, ind) = winners[&id];
            if i != ind {
                result.winner = None;
            }
        }
    }
}

/// Extension trait that de-duplicates winners across multiple [`MatchResult`]s.
///
/// When the same database entry is the winner for more than one query,
/// only the query with the highest `final_score` keeps the win;
/// the others have their `winner` cleared to `None`.
///
/// # Example
///
/// ```rust,ignore
/// use shinden_to_anilist_core::matcher::MatcherFinalizer;
///
/// // `results` is Vec<MatchResult>
/// results.iter_mut().finalize_matches();
/// ```
pub trait MatcherFinalizer {
    /// Resolves duplicate winners in-place.
    fn finalize_matches(&mut self);
}

impl<'a, T: Iterator<Item = &'a mut MatchResult>> MatcherFinalizer for T {
    fn finalize_matches(&mut self) {
        let mut view: Vec<&mut MatchResult> = self.collect();
        finalize_matches(view.as_mut_slice())
    }
}

#[cfg(test)]
mod tests {
    use std::{
        fs::File,
        io::BufReader,
        time::Instant,
    };

    use egobox_ego::EgorBuilder;
    use itertools::Itertools;
    use ndarray::{
        Array2,
        ArrayView2,
        array,
    };
    use rayon::prelude::*;

    use crate::{
        common::{
            AnimeList,
            MatchView,
        },
        database,
        database::{
            AnimeDatabase,
            AnimeDatabaseLoad,
        },
        extractor::{
            TitleMetadata,
            title_processor,
        },
        matcher::{
            DefaultMatcher,
            MatchResult,
            Matcher,
            MatcherFinalizer,
            generate_weights,
        },
        providers::{
            shinden,
            shinden::ShindenList,
        },
        searcher::{
            DefaultSearcher,
            Search,
            Searcher,
            SearcherAnimeExt,
        },
        utils::normalize_str,
    };

    #[test]
    fn match_shinden_list_test() {
        let database = AnimeDatabase::get_from_mmap("anime-offline-database.jsonl").unwrap();
        let shinden: ShindenList =
            serde_json::from_reader(BufReader::new(File::open("shinden-test.json").unwrap())).unwrap();

        let searcher = DefaultSearcher::new(&database);
        let matcher = DefaultMatcher::strict_preset();
        dbg!(matcher);
        let now = Instant::now();

        let mut results: Vec<(&shinden::AnimeEntry, MatchResult)> = shinden
            .par_values()
            .map(|entry| entry.search_by_title_ref(&database, &searcher, Search::options().strict().build()))
            .map(|(entry, candidates)| (entry, matcher.score_candidates(entry, &candidates, 0.5)))
            .collect();

        results.iter_mut().map(|(_, result)| result).finalize_matches();

        let elapsed = now.elapsed();

        for (entry, result) in results.iter() {
            println!("=== {} ===", entry.title());
            for (db_entry, scores) in result.items_ref(&database) {
                println!(
                    "[{:.2} {:3}] {}",
                    scores.final_score,
                    if Some(db_entry.id()) == result.winner.map(|x| x.0) {
                        "WIN"
                    } else {
                        ""
                    },
                    db_entry.title()
                )
            }
        }

        let winners_count = results.iter().filter(|(_, m)| m.winner().is_some()).count();
        let tops_count = results.iter().filter(|(_, m)| !m.top().is_empty()).count();

        println!("TOOK       : {:.2?}", elapsed);
        println!("HAS TOP    : {}/{}", tops_count, shinden.len());
        println!("HAS WINNER : {}/{}", winners_count, shinden.len());
    }

    struct MockQuery {
        title: String,
        normalized_title: String,
        metadata: TitleMetadata,
    }

    impl MockQuery {
        fn new(query: &str) -> Self {
            let title = query.to_string();
            let normalized_title = normalize_str(query).to_string();
            let metadata = title_processor::process(query);

            Self {
                title,
                normalized_title,
                metadata,
            }
        }
    }

    impl MatchView for MockQuery {
        fn title(&self) -> &str { &self.title }
        fn normalized_title(&self) -> &str { &self.normalized_title }
        fn title_metadata(&self) -> Option<&TitleMetadata> { Some(&self.metadata) }
    }

    #[test]
    fn match_fuzzy_query_test() {
        let database = AnimeDatabase::get_from_mmap("anime-offline-database.jsonl").unwrap();
        let searcher = DefaultSearcher::new(&database);
        let matcher = DefaultMatcher {
            search_weight: 0.8,
            season_weight: 0.2,
            ..Default::default()
        };

        let osk_queries = [
            MockQuery::new("oshi no ko 1"),
            MockQuery::new("oshi no ko 2"),
            MockQuery::new("oshi no ko 3"),
        ];

        let snk_queries = [
            MockQuery::new("shingeki no kyojin 1"),
            MockQuery::new("shingeki no kyojin 2"),
            MockQuery::new("shingeki no kyojin 3"),
        ];

        for query in osk_queries.iter().chain(&snk_queries) {
            println!("=== {} ===", query.title);
            let candidates = matcher.score_candidates(
                query,
                &searcher.search_ref(
                    &database,
                    &query.normalized_title,
                    Search::options().fuzzy().build(),
                ),
                0.0,
            );

            for (entry, score) in candidates.items_ref(&database).take(5) {
                let text = format!("[{:.2}] {}", score.final_score, entry.title());
                println!("{}", text);
            }
        }
    }

    #[test]
    fn optimize() {
        let database = AnimeDatabase::get_from_mmap("anime-offline-database.jsonl").unwrap();
        let shinden: ShindenList =
            serde_json::from_reader(BufReader::new(File::open("shinden-test.json").unwrap())).unwrap();

        let searcher = DefaultSearcher::new(&database);
        let xlimits = array![
            [0.8, 1.0],
            [0.1, 0.5],
            [0.7, 1.0],
            [0.7, 1.0],
            [0.1, 0.5],
            [0.1, 1.0],
            [0.1, 0.5],
            [0.0, 1.0]
        ];

        let egor = EgorBuilder::optimize(|x: &ArrayView2<f64>| {
            let mut results = Array2::zeros((x.nrows(), 1));

            for i in 0..x.nrows() {
                if let Some(params) = x.row(i).as_slice() {
                    results[[i, 0]] = -score_match(params, &shinden, &database, &searcher);
                }
            }

            results
        })
        .configure(|config| config.max_iters(200).trego(true))
        .min_within(&xlimits)
        .unwrap();

        let egor = egor.run().unwrap();

        let best_x = egor.x_opt.as_slice().unwrap();
        let best_y = -egor.y_opt.as_slice().unwrap()[0];

        println!(
            "Best params: [{}]",
            best_x.iter().map(|x| format!("{x:.2}")).join(", ")
        );
        println!("Best score: {:?}/{}", best_y as i64, shinden.len());
    }

    fn score_match(
        params: &[f64],
        shinden: &impl AnimeList<Entry = shinden::AnimeEntry>,
        database: &impl AnimeList<Entry = database::AnimeEntry>,
        searcher: &(impl Searcher + Sync),
    ) -> f64 {
        let gamma = params[7];

        let mut weights: Vec<f32> = params[..7].iter().map(|&x| x as f32).collect();

        generate_weights(&mut weights, gamma as f32);

        let matcher = DefaultMatcher::from_weights(*weights.as_array().unwrap(), 0.75, 0.075);

        let results: Vec<MatchResult> = shinden
            .par_values()
            .map(|x| x.search_by_title_ref(database, searcher, Search::options().strict().build()))
            .map(|(entry, cands)| matcher.score_candidates(entry, &cands, 0.5))
            .collect();

        results.iter().filter(|m| m.winner().is_some()).count() as f64
    }
}
