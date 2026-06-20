use rayon::prelude::*;
use shinden_to_anilist_core::{
    common::AnimeList,
    matcher::DefaultMatcher,
    providers::ogladajanime::{
        OgladajAnimeEntry,
        OgladajAnimeFetchEvent,
        OgladajAnimeList,
        OgladajAnimeListLoad,
    },
    searcher::{
        Search,
        SearcherAnimeExt,
    },
};
use tokio_util::sync::CancellationToken;
use tonic::Status;

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
    pb::{
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
    emit_progress: &mut impl FnMut(SourceFetchProgress) -> Result<(), Status>,
) -> Result<(SourceList, u64), Status> {
    let user_id = user
        .trim()
        .parse::<u64>()
        .map_err(|_| Status::invalid_argument("Id użytkownika Oglądaj Anime musi być liczbą."))?;

    let stream = OgladajAnimeList::stream_from_ogladajanime(service.http_client.clone(), user_id.to_string());
    let (entries, total_entries) = collect_entries(
        SourceProvider::OgladajAnime,
        stream,
        cancellation_token,
        emit_progress,
    )
    .await?;
    Ok((
        SourceList::OgladajAnime(OgladajAnimeList::from_entries(entries)),
        total_entries,
    ))
}

pub(in crate::server) fn match_source_list(
    ogladajanime: &OgladajAnimeList,
    database: &DatabaseState,
    options: Search,
    matcher: &DefaultMatcher,
) -> Vec<SourceMatchResult> {
    source_results_with_direct_matches(
        ogladajanime.direct_mal_matches(),
        ogladajanime.keys(),
        ogladajanime.par_values().map(|entry| {
            (
                entry.id(),
                entry.search_by_title_ref(&database.database, &database.searcher, options),
            )
        }),
        &database.database,
        matcher,
    )
}

impl ScrapedSourceFetchEvent for OgladajAnimeFetchEvent {
    type Entry = OgladajAnimeEntry;

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
