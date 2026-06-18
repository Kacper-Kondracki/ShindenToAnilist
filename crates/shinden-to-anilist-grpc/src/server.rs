use std::{
    cmp::Ordering,
    fs::File,
    io::{
        self,
        BufReader,
        BufWriter,
        Write,
    },
    path::{
        Path,
        PathBuf,
    },
    sync::Arc,
    time::Duration,
};

use rayon::prelude::*;
use shinden_to_anilist_core::{
    common::AnimeList,
    database::{
        self,
        root_metadata_from_path,
        updater::{
            latest_database_archive_asset,
            update_latest_jsonl_from_github,
        },
    },
    matcher::{
        DefaultMatcher,
        Matcher,
        MatcherFinalizer,
    },
    providers::{
        animezone::{
            AnimeZoneFetchEvent,
            AnimeZoneList,
            AnimeZoneListLoad,
        },
        ogladajanime::{
            OgladajAnimeFetchEvent,
            OgladajAnimeList,
            OgladajAnimeListLoad,
        },
        shinden::{
            ShindenList,
            ShindenListLoad,
        },
    },
    searcher::{
        SearchMode,
        Searcher,
        SearcherAnimeExt,
    },
    utils::normalize_str,
};
use tap::prelude::{
    Conv,
    Tap,
};
use tokio::{
    sync::{
        Mutex,
        mpsc,
    },
    time::timeout,
};
use tokio_stream::{
    StreamExt,
    wrappers::{
        ReceiverStream,
        UnboundedReceiverStream,
    },
};
use tokio_util::sync::CancellationToken;
use tonic::{
    Request,
    Response,
    Status,
    async_trait,
};
use tracing::{
    info,
    instrument,
    warn,
};

use crate::{
    DatabaseState,
    VersionedArcOption,
    error::{
        IntoStatus,
        database_not_loaded,
        database_sidecar_io_error,
        database_sidecar_json_error,
        shinden_list_not_loaded,
    },
    export::{
        create_unique_temp_file,
        export_xml_to_path,
        sync_parent_dir,
    },
    mapper::direct_source_match_result,
    matching::{
        FuzzyMatchView,
        search_options,
    },
    pb::{
        shinden_to_anilist_service_server::ShindenToAnilistService,
        *,
    },
    source::SourceList,
};

type CoreDatabaseReleaseInfo = database::updater::DatabaseReleaseInfo;

const DATABASE_SIDECAR_EXTENSION: &str = "info.json";
const DATABASE_DOWNLOAD_LOCK_TIMEOUT: Duration = Duration::from_secs(10);
const SHINDEN_FETCH_TIMEOUT: Duration = Duration::from_secs(10);
const STREAM_CHUNK_SIZE: usize = 500;

#[derive(Debug, Default, Clone)]
pub struct ShindenToAnilist {
    http_client: reqwest::Client,
    shinden_list: VersionedArcOption<ShindenList>,
    source_list: VersionedArcOption<SourceList>,
    database: VersionedArcOption<DatabaseState>,
    database_download_lock: Arc<Mutex<()>>,
}

impl ShindenToAnilist {
    pub fn new(http_client: reqwest::Client) -> Self {
        Self {
            http_client,
            shinden_list: VersionedArcOption::empty(),
            source_list: VersionedArcOption::empty(),
            database: VersionedArcOption::empty(),
            database_download_lock: Arc::new(Mutex::new(())),
        }
    }

    // Source lists
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
            SourceProvider::Shinden => {
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
            },
            SourceProvider::AnimeZone => {
                let username = request.user.trim();
                if username.is_empty() {
                    return Err(Status::invalid_argument(
                        "animezone source user must not be empty",
                    ));
                }

                emit_progress(source_progress(
                    SourceProvider::AnimeZone,
                    SourceFetchPhase::FetchingList,
                    0,
                    0,
                    "",
                ))?;

                let mut entries = Vec::new();
                let mut stream =
                    AnimeZoneList::stream_from_animezone(self.http_client.clone(), username.to_string());

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

                    match event.map_err(IntoStatus::into_status)? {
                        AnimeZoneFetchEvent::Started { total_entries: total } => {
                            total_entries = total as u64;
                            emit_progress(source_progress(
                                SourceProvider::AnimeZone,
                                SourceFetchPhase::FetchingDetails,
                                0,
                                total_entries,
                                "",
                            ))?;
                        },
                        AnimeZoneFetchEvent::Entry {
                            current,
                            total_entries: total,
                            entry,
                        } => {
                            total_entries = total as u64;
                            let latest_title = entry.title().to_string();
                            entries.push(entry);
                            emit_progress(source_progress(
                                SourceProvider::AnimeZone,
                                SourceFetchPhase::FetchingDetails,
                                current as u64,
                                total_entries,
                                latest_title,
                            ))?;
                        },
                    }
                }

                emit_progress(source_progress(
                    SourceProvider::AnimeZone,
                    SourceFetchPhase::Storing,
                    total_entries,
                    total_entries,
                    "",
                ))?;

                let animezone = AnimeZoneList::from_entries(entries);
                let total_entries = animezone.len() as u64;
                let source_version = self.source_list.store(SourceList::AnimeZone(animezone));

                emit_progress(source_progress(
                    SourceProvider::AnimeZone,
                    SourceFetchPhase::Done,
                    total_entries,
                    total_entries,
                    "",
                ))?;

                info!(source_version, total_entries, "source list fetched");
                Ok(FetchSourceListResponse {
                    source_version,
                    progress: Some(source_progress(
                        SourceProvider::AnimeZone,
                        SourceFetchPhase::Done,
                        total_entries,
                        total_entries,
                        "",
                    )),
                    done: true,
                })
            },
            SourceProvider::OgladajAnime => {
                let user_id = request.user.trim().parse::<u64>().map_err(|_| {
                    Status::invalid_argument("ogladajanime source user must be a numeric user id")
                })?;

                emit_progress(source_progress(
                    SourceProvider::OgladajAnime,
                    SourceFetchPhase::FetchingList,
                    0,
                    0,
                    "",
                ))?;

                let mut entries = Vec::new();
                let mut stream =
                    OgladajAnimeList::stream_from_ogladajanime(self.http_client.clone(), user_id.to_string());

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

                    match event.map_err(IntoStatus::into_status)? {
                        OgladajAnimeFetchEvent::Started { total_entries: total } => {
                            total_entries = total as u64;
                            emit_progress(source_progress(
                                SourceProvider::OgladajAnime,
                                SourceFetchPhase::FetchingDetails,
                                0,
                                total_entries,
                                "",
                            ))?;
                        },
                        OgladajAnimeFetchEvent::Entry {
                            current,
                            total_entries: total,
                            entry,
                        } => {
                            total_entries = total as u64;
                            let latest_title = entry.title().to_string();
                            entries.push(entry);
                            emit_progress(source_progress(
                                SourceProvider::OgladajAnime,
                                SourceFetchPhase::FetchingDetails,
                                current as u64,
                                total_entries,
                                latest_title,
                            ))?;
                        },
                    }
                }

                emit_progress(source_progress(
                    SourceProvider::OgladajAnime,
                    SourceFetchPhase::Storing,
                    total_entries,
                    total_entries,
                    "",
                ))?;

                let ogladajanime = OgladajAnimeList::from_entries(entries);
                let total_entries = ogladajanime.len() as u64;
                let source_version = self.source_list.store(SourceList::OgladajAnime(ogladajanime));

                emit_progress(source_progress(
                    SourceProvider::OgladajAnime,
                    SourceFetchPhase::Done,
                    total_entries,
                    total_entries,
                    "",
                ))?;

                info!(source_version, total_entries, "source list fetched");
                Ok(FetchSourceListResponse {
                    source_version,
                    progress: Some(source_progress(
                        SourceProvider::OgladajAnime,
                        SourceFetchPhase::Done,
                        total_entries,
                        total_entries,
                        "",
                    )),
                    done: true,
                })
            },
            SourceProvider::Unspecified => Err(Status::invalid_argument("source provider is not supported")),
        }
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

    // Shinden
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

    // Database
    #[instrument(skip_all, name = "app.check_database_update")]
    pub async fn check_database_update(
        &self,
        request: CheckDatabaseUpdateRequest,
    ) -> Result<CheckDatabaseUpdateResponse, Status> {
        info!(path = %request.path, "checking database update");

        let remote = latest_database_archive_asset(self.http_client.clone())
            .await
            .map_err(IntoStatus::into_status)?
            .conv::<CoreDatabaseReleaseInfo>();
        let local = read_database_sidecar_blocking(&request.path).await?;
        let needs_update = local.as_ref() != Some(&remote);
        info!(path = %request.path, needs_update, "database update check finished");

        Ok(CheckDatabaseUpdateResponse {
            status: Some(database_update_check(local, remote)),
        })
    }

    #[instrument(skip_all, name = "app.download_database")]
    pub async fn download_database(
        &self,
        request: DownloadDatabaseRequest,
    ) -> Result<DownloadDatabaseResponse, Status> {
        info!(path = %request.path, "waiting for database download lock");
        let _download_guard = timeout(DATABASE_DOWNLOAD_LOCK_TIMEOUT, self.database_download_lock.lock())
            .await
            .map_err(|_| {
                Status::deadline_exceeded(format!(
                    "database download lock could not be acquired within {} seconds",
                    DATABASE_DOWNLOAD_LOCK_TIMEOUT.as_secs()
                ))
            })?;
        info!(path = %request.path, "database download lock acquired");

        let status = update_latest_jsonl_from_github(self.http_client.clone(), &request.path)
            .await
            .map_err(IntoStatus::into_status)?
            .conv::<CoreDatabaseReleaseInfo>();

        let status = write_database_sidecar_blocking(&request.path, status).await?;
        info!(path = %request.path, "database downloaded");

        Ok(DownloadDatabaseResponse {
            status: Some(status.into()),
        })
    }

    #[instrument(skip_all, name = "app.load_database")]
    pub async fn load_database(&self, request: LoadDatabaseRequest) -> Result<LoadDatabaseResponse, Status> {
        info!(path = %request.path, "loading database");

        let path = request.path;
        let database = spawn_blocking_status("database loading", move || {
            DatabaseState::load(path).map_err(IntoStatus::into_status)
        })
        .await?;
        let entries = database.database.len();

        let database_version = self.database.store(database);
        info!(database_version, entries, "database loaded");

        Ok(LoadDatabaseResponse { database_version })
    }

    #[instrument(skip_all, name = "app.get_database_metadata")]
    pub async fn get_database_metadata(
        &self,
        request: GetDatabaseMetadataRequest,
    ) -> Result<GetDatabaseMetadataResponse, Status> {
        info!(path = %request.path, "loading database metadata");

        let path = request.path;
        let metadata = spawn_blocking_status("database metadata loading", move || {
            root_metadata_from_path(path).map_err(IntoStatus::into_status)
        })
        .await?;
        info!("database metadata loaded");

        Ok(GetDatabaseMetadataResponse {
            metadata: Some(metadata.into()),
        })
    }

    #[instrument(skip_all, name = "app.get_database_ids")]
    pub async fn get_database_ids(
        &self,
        _request: GetDatabaseIdsRequest,
    ) -> Result<GetDatabaseIdsResponse, Status> {
        info!("loading database ids");

        let guard = self.database.load();

        let database_version = guard.version();
        let database = guard.get().ok_or_else(|| database_not_loaded().into_status())?;

        let ids: Vec<u64> = database.database.values().map(|entry| entry.id()).collect();
        info!(database_version, ids = ids.len(), "database ids loaded");

        Ok(GetDatabaseIdsResponse {
            database_version,
            ids,
        })
    }

    #[instrument(skip_all, name = "app.get_database_entries")]
    pub async fn get_database_entries(
        &self,
        request: GetDatabaseEntriesRequest,
    ) -> Result<GetDatabaseEntriesResponse, Status> {
        let requested_ids = request.ids.len();
        info!(requested_ids, ?request.ids, "loading database entries");

        let guard = self.database.load();

        let database_version = guard.version();
        let database = guard.get().ok_or_else(|| database_not_loaded().into_status())?;

        let entries: Vec<DatabaseEntry> = request
            .ids
            .into_iter()
            .filter_map(|id| database.database.get(id).map(DatabaseEntry::from))
            .collect();

        Ok(GetDatabaseEntriesResponse {
            database_version,
            entries,
        })
    }

    #[instrument(skip_all, name = "app.get_database_full")]
    pub async fn get_database_full(
        &self,
        _request: GetDatabaseFullRequest,
    ) -> Result<GetDatabaseFullResponse, Status> {
        let guard = self.database.load();

        let database_version = guard.version();
        let database = guard.get().ok_or_else(|| database_not_loaded().into_status())?;

        let entries = database
            .database
            .values()
            .map(DatabaseEntry::from)
            .collect::<Vec<_>>();
        info!(database_version, entries = entries.len(), "loading full database");

        Ok(GetDatabaseFullResponse {
            database_version,
            entries,
        })
    }

    // Matching / Export
    #[instrument(skip_all, name = "app.fuzzy_search")]
    pub async fn fuzzy_search(&self, request: FuzzySearchRequest) -> Result<FuzzySearchResponse, Status> {
        let query_len = request.query.len();
        info!(query_len, "running fuzzy search");
        let guard = self.database.load();

        let database_version = guard.version();
        let database = guard.get().ok_or_else(|| database_not_loaded().into_status())?;
        let query = normalize_str(&request.query);
        let options = search_options(request.options, SearchMode::Fuzzy);

        let results: Vec<SearchResult> = database
            .searcher
            .search(&query, options)
            .into_iter()
            .map(|(id, score)| SearchResult { id, score })
            .collect();
        info!(database_version, results = results.len(), "fuzzy search finished");

        Ok(FuzzySearchResponse {
            database_version,
            results,
        })
    }

    #[instrument(skip_all, name = "app.fuzzy_match")]
    pub async fn fuzzy_match(&self, request: FuzzyMatchRequest) -> Result<FuzzyMatchResponse, Status> {
        let query_len = request.query.len();
        info!(
            query_len,
            shinden_id = request.shinden_id,
            source_id = request.source_id,
            "running fuzzy match"
        );
        let database_guard = self.database.load();

        let database_version = database_guard.version();
        let database = database_guard
            .get()
            .ok_or_else(|| database_not_loaded().into_status())?;
        let source_guard = self.source_list.load();
        let source_entry = request
            .source_id
            .or(request.shinden_id)
            .and_then(|id| source_guard.get().and_then(|source| source.match_view(id)));
        let query = FuzzyMatchView::new(request.query, source_entry);
        let options = search_options(request.options, SearchMode::Fuzzy);
        let matcher = if source_entry.is_some() {
            DefaultMatcher {
                search_weight: 0.8,
                season_weight: 0.1,
                year_weight: 0.03,
                type_weight: 0.03,
                status_weight: 0.015,
                seasonal_weight: 0.015,
                episodes_weight: 0.01,
                ..Default::default()
            }
        } else {
            DefaultMatcher {
                search_weight: 0.8,
                season_weight: 0.2,
                ..Default::default()
            }
        };

        let candidates = database
            .searcher
            .search_ref(&database.database, query.normalized_title(), options);
        let results: Vec<MatchResult> = matcher
            .score_candidates(&query, &candidates, 0.0)
            .items()
            .iter()
            .copied()
            .map(Into::into)
            .collect();
        info!(
            database_version,
            candidates = candidates.len(),
            results = results.len(),
            "fuzzy match finished"
        );

        Ok(FuzzyMatchResponse {
            database_version,
            results,
        })
    }

    #[instrument(skip_all, name = "app.match_shinden_list")]
    pub async fn match_shinden_list(
        &self,
        request: MatchShindenListRequest,
    ) -> Result<MatchShindenListResponse, Status> {
        info!("matching shinden list");
        let shinden_guard = self.shinden_list.load();
        let database_guard = self.database.load();

        let shinden_version = shinden_guard.version();
        let database_version = database_guard.version();
        let shinden = shinden_guard
            .get_arc()
            .ok_or_else(|| shinden_list_not_loaded().into_status())?;
        let database = database_guard
            .get_arc()
            .ok_or_else(|| database_not_loaded().into_status())?;
        let options = search_options(request.options, SearchMode::Strict);
        let shinden_entries = shinden.len();
        let database_entries = database.database.len();

        let results = spawn_blocking_status("shinden list matching", move || {
            let matcher = DefaultMatcher::strict_preset();
            let mut results = shinden
                .par_values()
                .map(|entry| entry.search_by_title_ref(&database.database, &database.searcher, options))
                .map(|(entry, candidates)| (entry.id(), matcher.score_candidates(entry, &candidates, 0.5)))
                .collect::<Vec<_>>();

            results.iter_mut().map(|(_, result)| result).finalize_matches();

            Ok(results
                .into_iter()
                .map(ShindenMatchResult::from)
                .collect::<Vec<_>>())
        })
        .await?;
        info!(
            shinden_version,
            database_version,
            shinden_entries,
            database_entries,
            results = results.len(),
            "shinden list matched"
        );

        Ok(MatchShindenListResponse {
            shinden_version,
            database_version,
            results,
        })
    }

    #[instrument(skip_all, name = "app.match_source_list")]
    pub async fn match_source_list(
        &self,
        request: MatchSourceListRequest,
    ) -> Result<MatchSourceListResponse, Status> {
        info!("matching source list");
        let source_guard = self.source_list.load();
        let database_guard = self.database.load();

        let source_version = source_guard.version();
        let database_version = database_guard.version();
        let source = source_guard
            .get_arc()
            .ok_or_else(|| shinden_list_not_loaded().into_status())?;
        let database = database_guard
            .get_arc()
            .ok_or_else(|| database_not_loaded().into_status())?;
        let options = search_options(request.options, SearchMode::Strict);
        let source_entries = source.len();
        let database_entries = database.database.len();

        let results = spawn_blocking_status("source list matching", move || {
            let matcher = DefaultMatcher::strict_preset();

            let results = match source.as_ref() {
                SourceList::Shinden(shinden) => {
                    let mut results = shinden
                        .par_values()
                        .map(|entry| {
                            entry.search_by_title_ref(&database.database, &database.searcher, options)
                        })
                        .map(|(entry, candidates)| {
                            (entry.id(), matcher.score_candidates(entry, &candidates, 0.5))
                        })
                        .collect::<Vec<_>>();

                    results.iter_mut().map(|(_, result)| result).finalize_matches();
                    results
                        .into_iter()
                        .map(SourceMatchResult::from)
                        .collect::<Vec<_>>()
                },
                SourceList::AnimeZone(animezone) => {
                    let direct_matches = animezone
                        .direct_mal_matches()
                        .filter(|(_, mal_id)| database.database.get(*mal_id).is_some())
                        .collect::<Vec<_>>();
                    let direct_entry_ids = direct_matches
                        .iter()
                        .map(|(source_id, _)| *source_id)
                        .collect::<std::collections::HashSet<_>>();

                    let mut fallback_results = animezone
                        .par_values()
                        .filter(|entry| !direct_entry_ids.contains(&entry.id()))
                        .map(|entry| {
                            entry.search_by_title_ref(&database.database, &database.searcher, options)
                        })
                        .map(|(entry, candidates)| {
                            (entry.id(), matcher.score_candidates(entry, &candidates, 0.5))
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
                    let order = animezone
                        .keys()
                        .enumerate()
                        .map(|(index, id)| (id, index))
                        .collect::<std::collections::HashMap<_, _>>();
                    results.sort_by_key(|result| order.get(&result.source_id).copied().unwrap_or(usize::MAX));
                    results
                },
                SourceList::OgladajAnime(ogladajanime) => {
                    let direct_matches = ogladajanime
                        .direct_mal_matches()
                        .filter(|(_, mal_id)| database.database.get(*mal_id).is_some())
                        .collect::<Vec<_>>();
                    let direct_entry_ids = direct_matches
                        .iter()
                        .map(|(source_id, _)| *source_id)
                        .collect::<std::collections::HashSet<_>>();

                    let mut fallback_results = ogladajanime
                        .par_values()
                        .filter(|entry| !direct_entry_ids.contains(&entry.id()))
                        .map(|entry| {
                            entry.search_by_title_ref(&database.database, &database.searcher, options)
                        })
                        .map(|(entry, candidates)| {
                            (entry.id(), matcher.score_candidates(entry, &candidates, 0.5))
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
                    let order = ogladajanime
                        .keys()
                        .enumerate()
                        .map(|(index, id)| (id, index))
                        .collect::<std::collections::HashMap<_, _>>();
                    results.sort_by_key(|result| order.get(&result.source_id).copied().unwrap_or(usize::MAX));
                    results
                },
            };

            Ok(results)
        })
        .await?;
        info!(
            source_version,
            database_version,
            source_entries,
            database_entries,
            results = results.len(),
            "source list matched"
        );

        Ok(MatchSourceListResponse {
            source_version,
            database_version,
            results,
        })
    }

    #[instrument(skip_all, name = "app.export_xml")]
    pub async fn export_xml(&self, request: ExportXmlRequest) -> Result<ExportXmlResponse, Status> {
        let requested_matches = request.matches.len();
        info!(path = %request.path, requested_matches, "exporting xml");
        let guard = self.source_list.load();

        let source_version = guard.version();
        let source = guard
            .get_arc()
            .ok_or_else(|| shinden_list_not_loaded().into_status())?;
        let path = request.path;
        let matches = request
            .matches
            .into_iter()
            .map(|pair| (pair.source_id, pair.database_id))
            .collect::<Vec<_>>();

        let path = spawn_blocking_status("xml export", move || {
            export_xml_to_path(&source, matches.into_iter(), &path)?;
            Ok(path)
        })
        .await?;
        info!(source_version, path = %path, requested_matches, "xml exported");

        Ok(ExportXmlResponse {
            source_version,
            shinden_version: source_version,
            path,
        })
    }
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

fn database_sidecar_path(path: impl AsRef<Path>) -> PathBuf {
    let mut sidecar_path = PathBuf::from(path.as_ref());
    sidecar_path.set_extension(DATABASE_SIDECAR_EXTENSION);
    sidecar_path
}

fn read_database_sidecar(database_path: impl AsRef<Path>) -> Result<Option<CoreDatabaseReleaseInfo>, Status> {
    let database_path = database_path.as_ref();
    if !database_path.exists() {
        info!(path = %database_path.display(), "database file does not exist; no sidecar to read");
        return Ok(None);
    }

    let sidecar_path = database_sidecar_path(database_path);
    let sidecar = match File::open(&sidecar_path) {
        Ok(sidecar) => sidecar,
        Err(err) if err.kind() == io::ErrorKind::NotFound => {
            info!(path = %sidecar_path.display(), "database sidecar not found");
            return Ok(None);
        },
        Err(err) => {
            return Err(database_sidecar_io_error(err, &sidecar_path, "open").into_status());
        },
    };

    let status = serde_json::from_reader(BufReader::new(sidecar))
        .map(Some)
        .map_err(|err| database_sidecar_json_error(err, &sidecar_path).into_status())?;

    info!(path = %sidecar_path.display(), "database sidecar loaded");

    Ok(status)
}

fn write_database_sidecar(
    database_path: impl AsRef<Path>,
    status: &CoreDatabaseReleaseInfo,
) -> Result<(), Status> {
    let sidecar_path = database_sidecar_path(database_path);
    let (mut temp_file, temp_path) = create_unique_temp_file(&sidecar_path)
        .map_err(|err| database_sidecar_io_error(err, &sidecar_path, "create").into_status())?;

    let result = (|| {
        {
            let mut writer = BufWriter::new(&mut temp_file);
            serde_json::to_writer_pretty(&mut writer, status)
                .map_err(|err| database_sidecar_json_error(err, &sidecar_path).into_status())?;
            writer
                .flush()
                .map_err(|err| database_sidecar_io_error(err, &temp_path, "write").into_status())?;
        }

        temp_file
            .sync_all()
            .map_err(|err| database_sidecar_io_error(err, &temp_path, "write").into_status())?;
        drop(temp_file);

        std::fs::rename(&temp_path, &sidecar_path)
            .and_then(|_| sync_parent_dir(&sidecar_path))
            .map_err(|err| database_sidecar_io_error(err, &sidecar_path, "rename").into_status())
    })();

    if result.is_err() {
        let _ = std::fs::remove_file(&temp_path);
    }

    if result.is_ok() {
        info!(path = %sidecar_path.display(), "database sidecar written");
    }

    result
}

async fn spawn_blocking_status<T>(
    operation: &'static str,
    f: impl FnOnce() -> Result<T, Status> + Send + 'static,
) -> Result<T, Status>
where
    T: Send + 'static,
{
    tokio::task::spawn_blocking(f)
        .await
        .map_err(|err| Status::internal(format!("{operation} blocking task failed: {err}")))?
}

async fn read_database_sidecar_blocking(
    database_path: impl AsRef<Path>,
) -> Result<Option<CoreDatabaseReleaseInfo>, Status> {
    let database_path = PathBuf::from(database_path.as_ref());
    spawn_blocking_status("database sidecar read", move || {
        read_database_sidecar(database_path)
    })
    .await
}

async fn write_database_sidecar_blocking(
    database_path: impl AsRef<Path>,
    status: CoreDatabaseReleaseInfo,
) -> Result<CoreDatabaseReleaseInfo, Status> {
    let database_path = PathBuf::from(database_path.as_ref());
    spawn_blocking_status("database sidecar write", move || {
        write_database_sidecar(database_path, &status)?;
        Ok(status)
    })
    .await
}

fn database_update_check(
    local: Option<CoreDatabaseReleaseInfo>,
    remote: CoreDatabaseReleaseInfo,
) -> DatabaseUpdateCheck {
    let needs_update = local.as_ref() != Some(&remote);

    DatabaseUpdateCheck {
        local: local.map(Into::into),
        remote: Some(remote.into()),
        needs_update,
    }
}

fn stream_batches<T, R>(
    version: u64,
    entries: Vec<T>,
    into_response: impl Fn(u64, Vec<T>) -> R + Send + 'static,
) -> ReceiverStream<Result<R, Status>>
where
    T: Send + 'static,
    R: Send + 'static,
{
    let total_entries = entries.len();
    let (tx, rx) = mpsc::channel(64);

    tokio::spawn(async move {
        let mut iter = entries.into_iter();
        let mut batches = 0usize;
        loop {
            let batch = iter.by_ref().take(STREAM_CHUNK_SIZE).collect::<Vec<_>>();
            if batch.is_empty() {
                break;
            }

            batches += 1;
            if tx.send(Ok(into_response(version, batch))).await.is_err() {
                warn!(
                    version,
                    total_entries,
                    batches_sent = batches,
                    "response stream receiver dropped"
                );
                break;
            }
        }

        info!(
            version,
            total_entries,
            batches_sent = batches,
            "response stream finished"
        );
    });

    ReceiverStream::new(rx)
}

#[async_trait]
impl ShindenToAnilistService for ShindenToAnilist {
    type FetchSourceListStream = UnboundedReceiverStream<Result<FetchSourceListResponse, Status>>;

    async fn fetch_source_list(
        &self,
        request: Request<FetchSourceListRequest>,
    ) -> Result<Response<Self::FetchSourceListStream>, Status> {
        let service = self.clone();
        let request = request.into_inner();
        let (tx, rx) = mpsc::unbounded_channel();
        let cancellation_token = CancellationToken::new();

        tokio::spawn(async move {
            let progress_tx = tx.clone();
            let fetch_token = cancellation_token.clone();
            let fetch = service.fetch_source_list_with_progress(request, fetch_token, move |progress| {
                progress_tx
                    .send(Ok(FetchSourceListResponse {
                        source_version: 0,
                        progress: Some(progress),
                        done: false,
                    }))
                    .map_err(|_| Status::cancelled("source fetch stream receiver dropped"))
            });
            tokio::pin!(fetch);

            tokio::select! {
                result = &mut fetch => {
                    match result {
                        Ok(response) => {
                            let _ = tx.send(Ok(response));
                        },
                        Err(status) => {
                            let _ = tx.send(Err(status));
                        },
                    }
                },
                () = tx.closed() => {
                    cancellation_token.cancel();
                },
            }
        });

        Ok(Response::new(UnboundedReceiverStream::new(rx)))
    }

    async fn get_source_ids(
        &self,
        request: Request<GetSourceIdsRequest>,
    ) -> Result<Response<GetSourceIdsResponse>, Status> {
        ShindenToAnilist::get_source_ids(self, request.into_inner())
            .await
            .map(Response::new)
    }

    type GetSourceFullStream = ReceiverStream<Result<GetSourceFullResponse, Status>>;

    async fn get_source_full(
        &self,
        request: Request<GetSourceFullRequest>,
    ) -> Result<Response<Self::GetSourceFullStream>, Status> {
        let response = ShindenToAnilist::get_source_full(self, request.into_inner()).await?;
        Ok(Response::new(stream_batches(
            response.source_version,
            response.entries,
            |source_version, entries| GetSourceFullResponse {
                source_version,
                entries,
            },
        )))
    }

    async fn fetch_shinden_list(
        &self,
        request: Request<FetchShindenListRequest>,
    ) -> Result<Response<FetchShindenListResponse>, Status> {
        ShindenToAnilist::fetch_shinden_list(self, request.into_inner())
            .await
            .map(Response::new)
    }

    async fn get_shinden_ids(
        &self,
        request: Request<GetShindenIdsRequest>,
    ) -> Result<Response<GetShindenIdsResponse>, Status> {
        ShindenToAnilist::get_shinden_ids(self, request.into_inner())
            .await
            .map(Response::new)
    }

    async fn get_shinden_entries(
        &self,
        request: Request<GetShindenEntriesRequest>,
    ) -> Result<Response<GetShindenEntriesResponse>, Status> {
        ShindenToAnilist::get_shinden_entries(self, request.into_inner())
            .await
            .map(Response::new)
    }

    type GetShindenFullStream = ReceiverStream<Result<GetShindenFullResponse, Status>>;

    async fn get_shinden_full(
        &self,
        request: Request<GetShindenFullRequest>,
    ) -> Result<Response<Self::GetShindenFullStream>, Status> {
        let response = ShindenToAnilist::get_shinden_full(self, request.into_inner()).await?;
        Ok(Response::new(stream_batches(
            response.shinden_version,
            response.entries,
            |shinden_version, entries| GetShindenFullResponse {
                shinden_version,
                entries,
            },
        )))
    }

    async fn check_database_update(
        &self,
        request: Request<CheckDatabaseUpdateRequest>,
    ) -> Result<Response<CheckDatabaseUpdateResponse>, Status> {
        ShindenToAnilist::check_database_update(self, request.into_inner())
            .await
            .map(Response::new)
    }

    async fn download_database(
        &self,
        request: Request<DownloadDatabaseRequest>,
    ) -> Result<Response<DownloadDatabaseResponse>, Status> {
        ShindenToAnilist::download_database(self, request.into_inner())
            .await
            .map(Response::new)
    }

    async fn load_database(
        &self,
        request: Request<LoadDatabaseRequest>,
    ) -> Result<Response<LoadDatabaseResponse>, Status> {
        ShindenToAnilist::load_database(self, request.into_inner())
            .await
            .map(Response::new)
    }

    async fn get_database_metadata(
        &self,
        request: Request<GetDatabaseMetadataRequest>,
    ) -> Result<Response<GetDatabaseMetadataResponse>, Status> {
        ShindenToAnilist::get_database_metadata(self, request.into_inner())
            .await
            .map(Response::new)
    }

    async fn get_database_ids(
        &self,
        request: Request<GetDatabaseIdsRequest>,
    ) -> Result<Response<GetDatabaseIdsResponse>, Status> {
        ShindenToAnilist::get_database_ids(self, request.into_inner())
            .await
            .map(Response::new)
    }

    async fn get_database_entries(
        &self,
        request: Request<GetDatabaseEntriesRequest>,
    ) -> Result<Response<GetDatabaseEntriesResponse>, Status> {
        ShindenToAnilist::get_database_entries(self, request.into_inner())
            .await
            .map(Response::new)
    }

    type GetDatabaseFullStream = ReceiverStream<Result<GetDatabaseFullResponse, Status>>;

    async fn get_database_full(
        &self,
        request: Request<GetDatabaseFullRequest>,
    ) -> Result<Response<Self::GetDatabaseFullStream>, Status> {
        let response = ShindenToAnilist::get_database_full(self, request.into_inner()).await?;
        Ok(Response::new(stream_batches(
            response.database_version,
            response.entries,
            |database_version, entries| GetDatabaseFullResponse {
                database_version,
                entries,
            },
        )))
    }

    async fn fuzzy_search(
        &self,
        request: Request<FuzzySearchRequest>,
    ) -> Result<Response<FuzzySearchResponse>, Status> {
        ShindenToAnilist::fuzzy_search(self, request.into_inner())
            .await
            .map(Response::new)
    }

    async fn fuzzy_match(
        &self,
        request: Request<FuzzyMatchRequest>,
    ) -> Result<Response<FuzzyMatchResponse>, Status> {
        ShindenToAnilist::fuzzy_match(self, request.into_inner())
            .await
            .map(Response::new)
    }

    type MatchSourceListStream = ReceiverStream<Result<MatchSourceListResponse, Status>>;

    async fn match_source_list(
        &self,
        request: Request<MatchSourceListRequest>,
    ) -> Result<Response<Self::MatchSourceListStream>, Status> {
        let response = ShindenToAnilist::match_source_list(self, request.into_inner()).await?;
        let database_version = response.database_version;
        Ok(Response::new(stream_batches(
            response.source_version,
            response.results,
            move |source_version, results| MatchSourceListResponse {
                source_version,
                database_version,
                results,
            },
        )))
    }

    type MatchShindenListStream = ReceiverStream<Result<MatchShindenListResponse, Status>>;

    async fn match_shinden_list(
        &self,
        request: Request<MatchShindenListRequest>,
    ) -> Result<Response<Self::MatchShindenListStream>, Status> {
        let response = ShindenToAnilist::match_shinden_list(self, request.into_inner()).await?;
        let database_version = response.database_version;
        Ok(Response::new(stream_batches(
            response.shinden_version,
            response.results,
            move |shinden_version, results| MatchShindenListResponse {
                shinden_version,
                database_version,
                results,
            },
        )))
    }

    async fn export_xml(
        &self,
        request: Request<ExportXmlRequest>,
    ) -> Result<Response<ExportXmlResponse>, Status> {
        ShindenToAnilist::export_xml(self, request.into_inner())
            .await
            .map(Response::new)
    }
}
