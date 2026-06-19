use std::cmp::Ordering;

use shinden_to_anilist_core::{
    common::{
        AnimeList,
        MatchView,
    },
    providers::{
        animezone::{
            AnimeZoneEntry,
            AnimeZoneFetchEvent,
            AnimeZoneList,
            AnimeZoneListLoad,
        },
        ogladajanime::{
            OgladajAnimeEntry,
            OgladajAnimeFetchEvent,
            OgladajAnimeList,
            OgladajAnimeListLoad,
        },
        shinden::{
            ShindenList,
            ShindenListLoad,
        },
    },
};
use tap::Tap;
use tokio::time::timeout;
use tokio_stream::{
    Stream,
    StreamExt,
};
use tokio_util::sync::CancellationToken;
use tonic::Status;
use tracing::{
    info,
    instrument,
};

use super::{
    SHINDEN_FETCH_TIMEOUT,
    ShindenToAnilist,
};
use crate::{
    error::{
        IntoStatus,
        shinden_list_not_loaded,
    },
    pb::*,
    source::SourceList,
};

impl ShindenToAnilist {
    #[instrument(skip_all, name = "app.fetch_source_list")]
    pub async fn fetch_source_list_with_progress(
        &self,
        request: FetchSourceListRequest,
        cancellation_token: CancellationToken,
        mut emit_progress: impl FnMut(SourceFetchProgress) -> Result<(), Status>,
    ) -> Result<FetchSourceListResponse, Status> {
        let provider = request.provider();
        info!(?provider, user = %request.user, "fetching source list");

        match provider {
            SourceProvider::Shinden => self.fetch_shinden_source_list(request, emit_progress).await,
            SourceProvider::AnimeZone => {
                let username = request.user.trim();
                if username.is_empty() {
                    return Err(Status::invalid_argument(
                        "animezone source user must not be empty",
                    ));
                }

                let stream =
                    AnimeZoneList::stream_from_animezone(self.http_client.clone(), username.to_string());
                let (entries, total_entries) = collect_scraped_source_entries(
                    SourceProvider::AnimeZone,
                    stream,
                    cancellation_token,
                    &mut emit_progress,
                )
                .await?;
                self.store_source_list(
                    SourceList::AnimeZone(AnimeZoneList::from_entries(entries)),
                    total_entries,
                    emit_progress,
                )
            },
            SourceProvider::OgladajAnime => {
                let user_id = request.user.trim().parse::<u64>().map_err(|_| {
                    Status::invalid_argument("ogladajanime source user must be a numeric user id")
                })?;

                let stream =
                    OgladajAnimeList::stream_from_ogladajanime(self.http_client.clone(), user_id.to_string());
                let (entries, total_entries) = collect_scraped_source_entries(
                    SourceProvider::OgladajAnime,
                    stream,
                    cancellation_token,
                    &mut emit_progress,
                )
                .await?;
                self.store_source_list(
                    SourceList::OgladajAnime(OgladajAnimeList::from_entries(entries)),
                    total_entries,
                    emit_progress,
                )
            },
            SourceProvider::Unspecified => Err(Status::invalid_argument("source provider is not supported")),
        }
    }

    async fn fetch_shinden_source_list(
        &self,
        request: FetchSourceListRequest,
        mut emit_progress: impl FnMut(SourceFetchProgress) -> Result<(), Status>,
    ) -> Result<FetchSourceListResponse, Status> {
        let user_id = request
            .user
            .parse::<u64>()
            .map_err(|_| Status::invalid_argument("shinden source user must be a numeric user id"))?;

        emit_progress(source_progress(
            SourceProvider::Shinden,
            SourceFetchPhase::FetchingList,
            0,
            0,
            "",
        ))?;

        let shinden = ShindenList::get_from_shinden(self.http_client.clone(), user_id);
        let shinden = timeout(SHINDEN_FETCH_TIMEOUT, shinden)
            .await
            .map_err(|_| {
                Status::deadline_exceeded(format!(
                    "shinden list could not be fetched within {} seconds",
                    SHINDEN_FETCH_TIMEOUT.as_secs()
                ))
            })?
            .map_err(IntoStatus::into_status)?;
        let total_entries = shinden.len() as u64;
        let source_version = self.source_list.store(SourceList::Shinden(shinden.clone()));
        let shinden_version = self.shinden_list.store(shinden);
        debug_assert_eq!(source_version, shinden_version);

        emit_progress(source_progress(
            SourceProvider::Shinden,
            SourceFetchPhase::Done,
            total_entries,
            total_entries,
            "",
        ))?;

        info!(source_version, total_entries, "source list fetched");
        Ok(FetchSourceListResponse {
            source_version,
            progress: Some(source_progress(
                SourceProvider::Shinden,
                SourceFetchPhase::Done,
                total_entries,
                total_entries,
                "",
            )),
            done: true,
        })
    }

    fn store_source_list(
        &self,
        source: SourceList,
        total_entries: u64,
        mut emit_progress: impl FnMut(SourceFetchProgress) -> Result<(), Status>,
    ) -> Result<FetchSourceListResponse, Status> {
        let provider = source.provider();
        emit_progress(source_progress(
            provider,
            SourceFetchPhase::Storing,
            total_entries,
            total_entries,
            "",
        ))?;

        let total_entries = source.len() as u64;
        let source_version = self.source_list.store(source);

        emit_progress(source_progress(
            provider,
            SourceFetchPhase::Done,
            total_entries,
            total_entries,
            "",
        ))?;

        info!(source_version, total_entries, "source list fetched");
        Ok(FetchSourceListResponse {
            source_version,
            progress: Some(source_progress(
                provider,
                SourceFetchPhase::Done,
                total_entries,
                total_entries,
                "",
            )),
            done: true,
        })
    }

    #[instrument(skip_all, name = "app.get_source_ids")]
    pub async fn get_source_ids(&self, request: GetSourceIdsRequest) -> Result<GetSourceIdsResponse, Status> {
        let sorted_by = request.sorted_by();
        let guard = self.source_list.load();
        let source = guard
            .get()
            .ok_or_else(|| shinden_list_not_loaded().into_status())?;

        let source_version = guard.version();
        let mut ids = source.ids();
        if sorted_by == AnimeListSortedBy::Urgency {
            ids = source_ids_by_urgency(source);
        }

        info!(
            source_version,
            provider = ?source.provider(),
            ids = ids.len(),
            "source ids loaded"
        );
        Ok(GetSourceIdsResponse { source_version, ids })
    }

    #[instrument(skip_all, name = "app.get_source_full")]
    pub async fn get_source_full(
        &self,
        _request: GetSourceFullRequest,
    ) -> Result<GetSourceFullResponse, Status> {
        let guard = self.source_list.load();
        let source = guard
            .get()
            .ok_or_else(|| shinden_list_not_loaded().into_status())?;

        let source_version = guard.version();
        let entries = source_entries(source);
        info!(
            source_version,
            provider = ?source.provider(),
            entries = entries.len(),
            "loading full source list"
        );

        Ok(GetSourceFullResponse {
            source_version,
            entries,
        })
    }

    #[instrument(skip_all, name = "app.fetch_shinden_list")]
    pub async fn fetch_shinden_list(
        &self,
        request: FetchShindenListRequest,
    ) -> Result<FetchShindenListResponse, Status> {
        info!(shinden_id = request.id, "fetching shinden list");

        let shinden = ShindenList::get_from_shinden(self.http_client.clone(), request.id);
        let shinden = timeout(SHINDEN_FETCH_TIMEOUT, shinden)
            .await
            .map_err(|_| {
                Status::deadline_exceeded(format!(
                    "shinden list could not be fetched within {} seconds",
                    SHINDEN_FETCH_TIMEOUT.as_secs()
                ))
            })?
            .map_err(IntoStatus::into_status)?;
        let entries = shinden.len();

        let source_version = self.source_list.store(SourceList::Shinden(shinden.clone()));
        let shinden_version = self.shinden_list.store(shinden);
        info!(shinden_version, entries, "shinden list fetched");
        debug_assert_eq!(source_version, shinden_version);

        Ok(FetchShindenListResponse { shinden_version })
    }

    #[instrument(skip_all, name = "app.get_shinden_ids")]
    pub async fn get_shinden_ids(
        &self,
        request: GetShindenIdsRequest,
    ) -> Result<GetShindenIdsResponse, Status> {
        let sorted_by = request.sorted_by();
        info!(?sorted_by, "loading shinden ids");

        let guard = self.shinden_list.load();

        let shinden = guard
            .get()
            .ok_or_else(|| shinden_list_not_loaded().into_status())?;

        let shinden_version = guard.version();
        let ids: Vec<u64> = shinden
            .iter()
            .map(|(id, entry)| (id, entry.premiere_date()))
            .collect::<Vec<_>>()
            .tap_mut(|v| {
                if sorted_by == AnimeListSortedBy::Urgency {
                    v.sort_by(|(_, date_a), (_, date_b)| match (date_a, date_b) {
                        (None, None) => Ordering::Equal,
                        (None, Some(_)) => Ordering::Less,
                        (Some(_), None) => Ordering::Greater,
                        (Some(date_a), Some(date_b)) => date_b.cmp(date_a),
                    })
                }
            })
            .into_iter()
            .map(|(id, _)| id)
            .collect();
        info!(shinden_version, ids = ids.len(), "shinden ids loaded");

        Ok(GetShindenIdsResponse { shinden_version, ids })
    }

    #[instrument(skip_all, name = "app.get_shinden_entries")]
    pub async fn get_shinden_entries(
        &self,
        request: GetShindenEntriesRequest,
    ) -> Result<GetShindenEntriesResponse, Status> {
        let requested_ids = request.ids.len();
        info!(requested_ids, ?request.ids, "loading shinden entries");

        let guard = self.shinden_list.load();

        let shinden = guard
            .get()
            .ok_or_else(|| shinden_list_not_loaded().into_status())?;

        let shinden_version = guard.version();
        let entries: Vec<ShindenEntry> = request
            .ids
            .into_iter()
            .filter_map(|id| shinden.get(id).map(Into::into))
            .collect();

        Ok(GetShindenEntriesResponse {
            shinden_version,
            entries,
        })
    }

    #[instrument(skip_all, name = "app.get_shinden_full")]
    pub async fn get_shinden_full(
        &self,
        _request: GetShindenFullRequest,
    ) -> Result<GetShindenFullResponse, Status> {
        let guard = self.shinden_list.load();

        let shinden = guard
            .get()
            .ok_or_else(|| shinden_list_not_loaded().into_status())?;

        let shinden_version = guard.version();
        let entries: Vec<ShindenEntry> = shinden.values().map(ShindenEntry::from).collect();
        info!(
            shinden_version,
            entries = entries.len(),
            "loading full shinden list"
        );

        Ok(GetShindenFullResponse {
            shinden_version,
            entries,
        })
    }
}

trait ScrapedSourceFetchEvent {
    type Entry: MatchView + Send;

    fn into_parts(self) -> ScrapedSourceFetchEventParts<Self::Entry>;
}

enum ScrapedSourceFetchEventParts<Entry> {
    Started {
        total_entries: usize,
    },
    Entry {
        current: usize,
        total_entries: usize,
        entry: Entry,
    },
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

async fn collect_scraped_source_entries<E, S, Err>(
    provider: SourceProvider,
    mut stream: S,
    cancellation_token: CancellationToken,
    emit_progress: &mut impl FnMut(SourceFetchProgress) -> Result<(), Status>,
) -> Result<(Vec<E::Entry>, u64), Status>
where
    E: ScrapedSourceFetchEvent,
    S: Stream<Item = Result<E, Err>> + Unpin,
    Err: IntoStatus,
{
    emit_progress(source_progress(
        provider,
        SourceFetchPhase::FetchingList,
        0,
        0,
        "",
    ))?;

    let mut entries = Vec::new();
    let mut total_entries = 0u64;
    loop {
        let event = tokio::select! {
            () = cancellation_token.cancelled() => {
                return Err(Status::cancelled("source fetch cancelled"));
            },
            event = stream.next() => event,
        };

        let Some(event) = event else {
            break;
        };

        match event.map_err(IntoStatus::into_status)?.into_parts() {
            ScrapedSourceFetchEventParts::Started { total_entries: total } => {
                total_entries = total as u64;
                emit_progress(source_progress(
                    provider,
                    SourceFetchPhase::FetchingDetails,
                    0,
                    total_entries,
                    "",
                ))?;
            },
            ScrapedSourceFetchEventParts::Entry {
                current,
                total_entries: total,
                entry,
            } => {
                total_entries = total as u64;
                let latest_title = entry.title().to_string();
                entries.push(entry);
                emit_progress(source_progress(
                    provider,
                    SourceFetchPhase::FetchingDetails,
                    current as u64,
                    total_entries,
                    latest_title,
                ))?;
            },
        }
    }

    Ok((entries, total_entries))
}

fn source_progress(
    provider: SourceProvider,
    phase: SourceFetchPhase,
    current: u64,
    total: u64,
    latest_title: impl Into<String>,
) -> SourceFetchProgress {
    SourceFetchProgress {
        provider: provider.into(),
        phase: phase.into(),
        current,
        total,
        latest_title: latest_title.into(),
    }
}

fn source_entries(source: &SourceList) -> Vec<SourceEntry> {
    match source {
        SourceList::Shinden(list) => list.values().map(SourceEntry::from).collect(),
        SourceList::AnimeZone(list) => list.values().map(SourceEntry::from).collect(),
        SourceList::OgladajAnime(list) => list.values().map(SourceEntry::from).collect(),
    }
}

fn source_ids_by_urgency(source: &SourceList) -> Vec<u64> {
    match source {
        SourceList::Shinden(list) => list
            .iter()
            .map(|(id, entry)| (id, entry.premiere_date()))
            .collect::<Vec<_>>()
            .tap_mut(|v| {
                v.sort_by(|(_, date_a), (_, date_b)| match (date_a, date_b) {
                    (None, None) => Ordering::Equal,
                    (None, Some(_)) => Ordering::Less,
                    (Some(_), None) => Ordering::Greater,
                    (Some(date_a), Some(date_b)) => date_b.cmp(date_a),
                })
            })
            .into_iter()
            .map(|(id, _)| id)
            .collect(),
        SourceList::AnimeZone(list) => list.keys().collect(),
        SourceList::OgladajAnime(list) => list.keys().collect(),
    }
}
