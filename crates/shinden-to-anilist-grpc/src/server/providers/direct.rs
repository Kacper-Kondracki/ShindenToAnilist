use std::collections::{
    HashMap,
    HashSet,
};

use rayon::prelude::*;
use shinden_to_anilist_core::{
    common::AnimeList,
    database::AnimeEntry,
    matcher::{
        DefaultMatcher,
        Matcher,
        MatcherFinalizer,
    },
    searcher::SearcherAnimeExt,
};

use crate::{
    mapper::direct_source_match_result,
    pb::SourceMatchResult,
};

pub(super) fn source_results_with_direct_matches<
    'entry,
    'database,
    Entry,
    DirectMatches,
    OrderIds,
    SearchResults,
>(
    direct_matches: DirectMatches,
    order_ids: OrderIds,
    search_results: SearchResults,
    database: &impl AnimeList<Entry = AnimeEntry>,
    matcher: &DefaultMatcher,
) -> Vec<SourceMatchResult>
where
    Entry: SearcherAnimeExt + Sync + 'entry,
    'database: 'entry,
    DirectMatches: Iterator<Item = (u64, u64)>,
    OrderIds: Iterator<Item = u64>,
    SearchResults: ParallelIterator<Item = (u64, (&'entry Entry, Vec<(&'database AnimeEntry, f32)>))>,
{
    let direct_matches = direct_matches
        .filter(|(_, mal_id)| database.get(*mal_id).is_some())
        .collect::<Vec<_>>();
    let direct_entry_ids = direct_matches
        .iter()
        .map(|(source_id, _)| *source_id)
        .collect::<HashSet<_>>();

    let mut fallback_results = search_results
        .filter(|(source_id, _)| !direct_entry_ids.contains(source_id))
        .map(|(source_id, (entry, candidates))| {
            (source_id, matcher.score_candidates(entry, &candidates, 0.5))
        })
        .collect::<Vec<_>>();

    fallback_results
        .iter_mut()
        .map(|(_, result)| result)
        .finalize_matches();

    let mut results = direct_matches
        .into_iter()
        .map(|(source_id, database_id)| direct_source_match_result(source_id, database_id))
        .chain(fallback_results.into_iter().map(SourceMatchResult::from))
        .collect::<Vec<_>>();
    let order = order_ids
        .enumerate()
        .map(|(index, id)| (id, index))
        .collect::<HashMap<_, _>>();
    results.sort_by_key(|result| order.get(&result.source_id).copied().unwrap_or(usize::MAX));
    results
}
