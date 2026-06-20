use rayon::prelude::*;
use shinden_to_anilist_core::{
    common::AnimeList,
    matcher::{
        DefaultMatcher,
        Matcher,
        MatcherFinalizer,
    },
    providers::shinden::{
        ShindenList,
        ShindenListLoad,
    },
    searcher::SearcherAnimeExt,
};
use tokio::time::timeout;
use tonic::Status;
use tracing::info;

use super::super::{
    SHINDEN_FETCH_TIMEOUT,
    ShindenToAnilist,
};
use crate::{
    DatabaseState,
    error::IntoStatus,
    pb::{
        FetchShindenListRequest,
        FetchShindenListResponse,
        FetchSourceListRequest,
        FetchSourceListResponse,
        ShindenMatchResult,
        SourceFetchPhase,
        SourceFetchProgress,
        SourceMatchResult,
        SourceProvider,
    },
    source::SourceList,
};

pub(in crate::server) async fn fetch_source_list(
    service: &ShindenToAnilist,
    request: FetchSourceListRequest,
    mut emit_progress: impl FnMut(SourceFetchProgress) -> Result<(), Status>,
) -> Result<FetchSourceListResponse, Status> {
    let user_id = request
        .user
        .parse::<u64>()
        .map_err(|_| Status::invalid_argument("Id użytkownika Shinden musi być liczbą."))?;

    emit_progress(super::super::source::source_progress(
        SourceProvider::Shinden,
        SourceFetchPhase::FetchingList,
        0,
        0,
        "",
    ))?;

    let shinden = fetch_list(service, user_id).await?;
    let total_entries = shinden.len() as u64;
    let source_version = service.source_list.store(SourceList::Shinden(shinden.clone()));
    let shinden_version = service.shinden_list.store(shinden);
    debug_assert_eq!(source_version, shinden_version);

    emit_progress(super::super::source::source_progress(
        SourceProvider::Shinden,
        SourceFetchPhase::Done,
        total_entries,
        total_entries,
        "",
    ))?;

    info!(source_version, total_entries, "source list fetched");
    Ok(FetchSourceListResponse {
        source_version,
        progress: Some(super::super::source::source_progress(
            SourceProvider::Shinden,
            SourceFetchPhase::Done,
            total_entries,
            total_entries,
            "",
        )),
        done: true,
    })
}

pub(in crate::server) async fn fetch_legacy_list(
    service: &ShindenToAnilist,
    request: FetchShindenListRequest,
) -> Result<FetchShindenListResponse, Status> {
    let shinden = fetch_list(service, request.id).await?;
    let entries = shinden.len();

    let source_version = service.source_list.store(SourceList::Shinden(shinden.clone()));
    let shinden_version = service.shinden_list.store(shinden);
    info!(shinden_version, entries, "shinden list fetched");
    debug_assert_eq!(source_version, shinden_version);

    Ok(FetchShindenListResponse { shinden_version })
}

pub(in crate::server) fn match_legacy_list(
    shinden: &ShindenList,
    database: &DatabaseState,
    options: shinden_to_anilist_core::searcher::Search,
    matcher: &DefaultMatcher,
) -> Vec<ShindenMatchResult> {
    let mut results = shinden
        .par_values()
        .map(|entry| entry.search_by_title_ref(&database.database, &database.searcher, options))
        .map(|(entry, candidates)| (entry.id(), matcher.score_candidates(entry, &candidates, 0.5)))
        .collect::<Vec<_>>();

    results.iter_mut().map(|(_, result)| result).finalize_matches();
    results
        .into_iter()
        .map(ShindenMatchResult::from)
        .collect::<Vec<_>>()
}

pub(in crate::server) fn match_source_list(
    shinden: &ShindenList,
    database: &DatabaseState,
    options: shinden_to_anilist_core::searcher::Search,
    matcher: &DefaultMatcher,
) -> Vec<SourceMatchResult> {
    let mut results = shinden
        .par_values()
        .map(|entry| entry.search_by_title_ref(&database.database, &database.searcher, options))
        .map(|(entry, candidates)| (entry.id(), matcher.score_candidates(entry, &candidates, 0.5)))
        .collect::<Vec<_>>();

    results.iter_mut().map(|(_, result)| result).finalize_matches();
    results
        .into_iter()
        .map(SourceMatchResult::from)
        .collect::<Vec<_>>()
}

async fn fetch_list(service: &ShindenToAnilist, user_id: u64) -> Result<ShindenList, Status> {
    let shinden = ShindenList::get_from_shinden(service.http_client.clone(), user_id);
    timeout(SHINDEN_FETCH_TIMEOUT, shinden)
        .await
        .map_err(|_| {
            Status::deadline_exceeded(format!(
                "shinden list could not be fetched within {} seconds",
                SHINDEN_FETCH_TIMEOUT.as_secs()
            ))
        })?
        .map_err(IntoStatus::into_status)
}
