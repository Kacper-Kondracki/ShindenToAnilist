use std::{
    fs::File,
    str,
};

use rayon::prelude::*;
use shinden_to_anilist_core::{
    common::{
        AnimeId,
        AnimeList,
        MatchView,
    },
    exporter::{
        ExportExt,
        xml::XmlExporter,
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
        ScoreBreakdown,
    },
    searcher::{
        Search,
        SearchMode,
        Searcher,
        SearcherAnimeExt,
    },
    utils::normalize_str,
};

use crate::{
    driver::{
        StaDriver,
        StoredMatchResult,
        StoredShindenMatchResult,
    },
    ffi::{
        StaExportResult,
        StaMatchListResult,
        StaMatchOptions,
        StaMatchQueryOptions,
        StaMatchResult,
        StaMatchSelection,
        StaMatchWinner,
        StaScoreBreakdown,
        StaScoredCandidate,
        StaSearchItem,
        StaSearchOptions,
        StaSearchResult,
        StaShindenMatchResult,
        StaStringView,
        empty_match_result,
        into_raw_string,
    },
};

struct MockQuery {
    title: String,
    normalized_title: String,
    metadata: TitleMetadata,
}

impl MockQuery {
    fn new(query: &str) -> Self {
        Self {
            title: query.to_owned(),
            normalized_title: normalize_str(query).to_string(),
            metadata: title_processor::process(query),
        }
    }
}

impl MatchView for MockQuery {
    fn title(&self) -> &str { &self.title }

    fn normalized_title(&self) -> &str { &self.normalized_title }

    fn title_metadata(&self) -> Option<&TitleMetadata> { Some(&self.metadata) }
}

pub fn search_anime(
    driver: &StaDriver,
    query: &str,
    options: StaSearchOptions,
) -> Result<StaSearchResult, String> {
    let database = driver
        .database()
        .lock()
        .map_err(|_| "database lock is poisoned".to_owned())?;
    let database = database
        .as_ref()
        .ok_or_else(|| "anime database is not loaded".to_owned())?;
    let searcher = driver
        .searcher()
        .lock()
        .map_err(|_| "searcher lock is poisoned".to_owned())?;
    let searcher = searcher
        .as_ref()
        .ok_or_else(|| "anime searcher is not loaded".to_owned())?;

    let search_options = search_options(options)?;
    let mut items = searcher
        .search_ref(database, query, search_options)
        .into_iter()
        .map(|(entry, score)| StaSearchItem {
            id: entry.id(),
            score,
        })
        .collect::<Vec<_>>();

    items.shrink_to_fit();
    let len = items.len();
    let items = items.leak().as_mut_ptr();
    Ok(StaSearchResult { items, len })
}

pub fn match_query(
    driver: &StaDriver,
    query: &str,
    options: StaMatchQueryOptions,
) -> Result<StaMatchResult, String> {
    let database = driver
        .database()
        .lock()
        .map_err(|_| "database lock is poisoned".to_owned())?;
    let database = database
        .as_ref()
        .ok_or_else(|| "anime database is not loaded".to_owned())?;
    let searcher = driver
        .searcher()
        .lock()
        .map_err(|_| "searcher lock is poisoned".to_owned())?;
    let searcher = searcher
        .as_ref()
        .ok_or_else(|| "anime searcher is not loaded".to_owned())?;

    let query = MockQuery::new(query);
    let candidates = searcher.search_ref(
        database,
        query.normalized_title(),
        search_options(options.search)?,
    );
    let result = DefaultMatcher::strict_preset().score_candidates(&query, &candidates, 0.5);
    Ok(match_result_to_ffi(
        &result,
        result_limit(options.result_limit, options.has_result_limit),
    ))
}

pub fn match_loaded_shinden_list(
    driver: &StaDriver,
    options: StaMatchOptions,
) -> Result<StaMatchListResult, String> {
    let database = driver
        .database()
        .lock()
        .map_err(|_| "database lock is poisoned".to_owned())?;
    let database = database
        .as_ref()
        .ok_or_else(|| "anime database is not loaded".to_owned())?;
    let searcher = driver
        .searcher()
        .lock()
        .map_err(|_| "searcher lock is poisoned".to_owned())?;
    let searcher = searcher
        .as_ref()
        .ok_or_else(|| "anime searcher is not loaded".to_owned())?;
    let shinden = driver
        .shinden_list()
        .lock()
        .map_err(|_| "shinden list lock is poisoned".to_owned())?;
    let shinden = shinden
        .as_ref()
        .ok_or_else(|| "shinden list is not loaded".to_owned())?;

    let candidate_limit = if options.candidate_limit == 0 {
        50
    } else {
        options.candidate_limit
    };
    let search_threshold = if options.search_threshold <= 0.0 {
        0.65
    } else {
        options.search_threshold
    };
    let search = Search {
        limit: candidate_limit,
        threshold: search_threshold,
        mode: SearchMode::Strict,
    };
    let matcher = DefaultMatcher::strict_preset();

    let mut results: Vec<(AnimeId, MatchResult)> = shinden
        .par_values()
        .map(|entry| entry.search_by_title_ref(database, searcher, search))
        .map(|(entry, candidates)| (entry.id(), matcher.score_candidates(entry, &candidates, 0.5)))
        .collect();

    results.iter_mut().map(|(_, result)| result).finalize_matches();

    let result_limit = result_limit(options.result_limit, options.has_result_limit);
    let stored = results
        .iter()
        .map(|(shinden_id, result)| StoredShindenMatchResult {
            shinden_id: *shinden_id,
            result: stored_match_result(result),
        })
        .collect::<Vec<_>>();

    {
        let mut state = driver
            .match_results()
            .lock()
            .map_err(|_| "match results lock is poisoned".to_owned())?;
        *state = Some(stored.clone());
    }

    Ok(stored_match_list_to_ffi(&stored, result_limit))
}

pub unsafe fn export_matches(
    driver: &StaDriver,
    path: &str,
    selections: *const StaMatchSelection,
    len: usize,
) -> Result<StaExportResult, String> {
    if selections.is_null() && len > 0 {
        return Err("match selections pointer is null".to_owned());
    }

    let selections = if len == 0 {
        &[]
    } else {
        unsafe { std::slice::from_raw_parts(selections, len) }
    };

    let shinden = driver
        .shinden_list()
        .lock()
        .map_err(|_| "shinden list lock is poisoned".to_owned())?;
    let shinden = shinden
        .as_ref()
        .ok_or_else(|| "shinden list is not loaded".to_owned())?;

    let pairs = selections
        .iter()
        .map(|selection| (selection.shinden_id, selection.database_id))
        .collect::<Vec<_>>();
    let file = File::options()
        .create(true)
        .truncate(true)
        .write(true)
        .open(path)
        .map_err(|error| error.to_string())?;

    shinden
        .export(&XmlExporter {}, pairs.iter().copied(), file)
        .map_err(|error| error.to_string())?;

    Ok(StaExportResult {
        path: into_raw_string(path),
        exported_count: pairs.len(),
    })
}

fn search_options(options: StaSearchOptions) -> Result<Search, String> {
    Ok(Search {
        limit: if options.limit == 0 { 50 } else { options.limit },
        threshold: if options.threshold <= 0.0 {
            0.65
        } else {
            options.threshold
        },
        mode: search_mode(options.mode)?,
    })
}

fn search_mode(value: StaStringView) -> Result<SearchMode, String> {
    let value = string_view_to_str(value)?;
    match value.as_str() {
        "" | "fuzzy" => Ok(SearchMode::Fuzzy),
        "strict" => Ok(SearchMode::Strict),
        _ => Err(format!("unknown search mode: {value}")),
    }
}

fn string_view_to_str(value: StaStringView) -> Result<String, String> {
    if value.ptr.is_null() || value.len == 0 {
        return Ok(String::new());
    }

    let bytes = unsafe { std::slice::from_raw_parts(value.ptr.cast::<u8>(), value.len) };
    str::from_utf8(bytes)
        .map(|value| value.to_owned())
        .map_err(|error| format!("string view is not valid UTF-8: {error}"))
}

fn result_limit(value: usize, has_value: bool) -> Option<usize> { if has_value { Some(value) } else { None } }

fn stored_match_result(result: &MatchResult) -> StoredMatchResult {
    StoredMatchResult {
        items: result
            .items()
            .iter()
            .map(|&(id, score)| scored_candidate_to_ffi(id, score))
            .collect(),
        top: result
            .top()
            .iter()
            .map(|&(id, score)| scored_candidate_to_ffi(id, score))
            .collect(),
        winner: result
            .winner()
            .map(|(id, score)| scored_candidate_to_ffi(id, score)),
    }
}

fn stored_match_list_to_ffi(
    values: &[StoredShindenMatchResult],
    result_limit: Option<usize>,
) -> StaMatchListResult {
    let total = values.len();
    let winners = values
        .iter()
        .filter(|entry| entry.result.winner.is_some())
        .count();
    let has_top = values.iter().filter(|entry| !entry.result.top.is_empty()).count();
    let unmatched = total - winners;

    let mut entries = values
        .iter()
        .map(|entry| StaShindenMatchResult {
            shinden_id: entry.shinden_id,
            result: stored_result_to_ffi(&entry.result, result_limit),
        })
        .collect::<Vec<_>>();
    entries.shrink_to_fit();
    let len = entries.len();
    let entries = entries.leak().as_mut_ptr();

    StaMatchListResult {
        entries,
        len,
        total,
        winners,
        has_top,
        unmatched,
    }
}

fn match_result_to_ffi(value: &MatchResult, result_limit: Option<usize>) -> StaMatchResult {
    stored_result_to_ffi(&stored_match_result(value), result_limit)
}

fn stored_result_to_ffi(value: &StoredMatchResult, result_limit: Option<usize>) -> StaMatchResult {
    let mut items = value.items.clone();
    if let Some(limit) = result_limit {
        items.truncate(limit);
    }
    items.shrink_to_fit();
    let items_len = items.len();
    let items = items.leak().as_mut_ptr();

    let mut top = value.top.clone();
    top.shrink_to_fit();
    let top_len = top.len();
    let top = top.leak().as_mut_ptr();

    StaMatchResult {
        items,
        items_len,
        top,
        top_len,
        winner: value.winner.map_or(
            StaMatchWinner {
                item: StaScoredCandidate {
                    id: 0,
                    score: empty_match_result().winner.item.score,
                },
                has_value: false,
            },
            |item| StaMatchWinner {
                item,
                has_value: true,
            },
        ),
    }
}

fn scored_candidate_to_ffi(id: AnimeId, score: ScoreBreakdown) -> StaScoredCandidate {
    StaScoredCandidate {
        id,
        score: score_to_ffi(score),
    }
}

fn score_to_ffi(score: ScoreBreakdown) -> StaScoreBreakdown {
    StaScoreBreakdown {
        search_score: score.search_score,
        season_score: score.season_score,
        year_score: score.year_score,
        type_score: score.type_score,
        status_score: score.status_score,
        seasonal_score: score.seasonal_score,
        episodes_score: score.episodes_score,
        final_score: score.final_score,
    }
}
