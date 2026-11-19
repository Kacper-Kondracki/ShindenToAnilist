use rayon::prelude::*;
use shinden_to_anilist_core::{
    common::AnimeList,
    matcher::DefaultMatcher,
    providers::animezone::{
        AnimeZoneEntry,
        AnimeZoneFetchEvent,
        AnimeZoneList,
        AnimeZoneListLoad,
    },
    searcher::{
        Search,
        SearcherAnimeExt,
    },
};
use tokio_util::sync::CancellationToken;

use super::{
    direct::source_results_with_direct_matches,
    scraped::{
        ScrapedSourceFetchEvent,
        ScrapedSourceFetchEventParts,
        collect_entries,
    },
};
use crate::{
    DatabaseState,
    error::ServiceError,
    pb::{
        ErrorKind,
        SourceFetchProgress,
        SourceMatchResult,
        SourceProvider,
    },
    server::ShindenToAnilist,
    source::SourceList,
};

pub(in crate::server) async fn fetch_source_list(
    service: &ShindenToAnilist,
    user: &str,
    cancellation_token: CancellationToken,
    emit_progress: &mut impl FnMut(SourceFetchProgress) -> Result<(), ServiceError>,
) -> Result<(SourceList, u64), ServiceError> {
    let username = user.trim();
    if username.is_empty() {
        return Err(ServiceError::new(
            ErrorKind::InvalidSourceUser,
            "empty AnimeZone username",
        ));
    }

    let stream =
        AnimeZoneList::stream_from_animezone(service.http_clients.animezone.clone(), username.to_string());
    let (entries, total_entries) = collect_entries(
        SourceProvider::AnimeZone,
        stream,
        cancellation_token,
        emit_progress,
    )
    .await?;
    Ok((
        SourceList::AnimeZone(AnimeZoneList::from_entries(entries)),
        total_entries,
    ))
}

pub(in crate::server) fn match_source_list(
    animezone: &AnimeZoneList,
    database: &DatabaseState,
    options: Search,
    matcher: &DefaultMatcher,
) -> Vec<SourceMatchResult> {
    source_results_with_direct_matches(
        animezone.direct_mal_matches(),
        animezone.keys(),
        animezone.par_values().map(|entry| {
            (
                entry.id(),
                entry.search_by_title_ref(&database.database, &database.searcher, options),
            )
        }),
        &database.database,
        matcher,
    )
}

impl ScrapedSourceFetchEvent for AnimeZoneFetchEvent {
    type Entry = AnimeZoneEntry;

    fn into_parts(self) -> ScrapedSourceFetchEventParts<<Self as ScrapedSourceFetchEvent>::Entry> {
        match self {
            Self::Started { total_entries } => ScrapedSourceFetchEventParts::Started { total_entries },
            Self::Entry {
                current,
                total_entries,
                entry,
            } => ScrapedSourceFetchEventParts::Entry {
                current,
                total_entries,
                entry,
            },
        }
    }
}
