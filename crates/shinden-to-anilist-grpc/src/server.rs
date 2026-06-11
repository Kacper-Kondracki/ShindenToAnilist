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
    providers::shinden::{
        ShindenList,
        ShindenListLoad,
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
use tokio_stream::wrappers::ReceiverStream;
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
    matching::{
        QueryMatchView,
        search_options,
    },
    pb::{
        shinden_to_anilist_service_server::ShindenToAnilistService,
        *,
    },
};

type CoreDatabaseReleaseInfo = database::updater::DatabaseReleaseInfo;

const DATABASE_SIDECAR_EXTENSION: &str = "info.json";
const DATABASE_DOWNLOAD_LOCK_TIMEOUT: Duration = Duration::from_secs(10);
const SHINDEN_FETCH_TIMEOUT: Duration = Duration::from_secs(10);
const STREAM_CHUNK_SIZE: usize = 500;

#[derive(Debug, Default)]
pub struct ShindenToAnilist {
    http_client: reqwest::Client,
    shinden_list: VersionedArcOption<ShindenList>,
    database: VersionedArcOption<DatabaseState>,
    database_download_lock: Mutex<()>,
}

impl ShindenToAnilist {
    pub fn new(http_client: reqwest::Client) -> Self {
        Self {
            http_client,
            shinden_list: VersionedArcOption::empty(),
            database: VersionedArcOption::empty(),
            database_download_lock: Mutex::new(()),
        }
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
    // Shinden
    #[instrument(skip_all, name = "grpc.fetch_shinden_list")]
    async fn fetch_shinden_list(
        &self,
        request: Request<FetchShindenListRequest>,
    ) -> Result<Response<FetchShindenListResponse>, Status> {
        let request = request.into_inner();
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

        let shinden_version = self.shinden_list.store(shinden);
        info!(shinden_version, entries, "shinden list fetched");

        Ok(FetchShindenListResponse { shinden_version }.into())
    }

    #[instrument(skip_all, name = "grpc.get_shinden_ids")]
    async fn get_shinden_ids(
        &self,
        request: Request<GetShindenIdsRequest>,
    ) -> Result<Response<GetShindenIdsResponse>, Status> {
        let request = request.into_inner();
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

        Ok(GetShindenIdsResponse { shinden_version, ids }.into())
    }

    #[instrument(skip_all, name = "grpc.get_shinden_entries")]
    async fn get_shinden_entries(
        &self,
        request: Request<GetShindenEntriesRequest>,
    ) -> Result<Response<GetShindenEntriesResponse>, Status> {
        let request = request.into_inner();
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
        }
        .into())
    }

    type GetShindenFullStream = ReceiverStream<Result<GetShindenFullResponse, Status>>;

    #[instrument(skip_all, name = "grpc.get_shinden_full")]
    async fn get_shinden_full(
        &self,
        _request: Request<GetShindenFullRequest>,
    ) -> Result<Response<Self::GetShindenFullStream>, Status> {
        let guard = self.shinden_list.load();

        let shinden = guard
            .get()
            .ok_or_else(|| shinden_list_not_loaded().into_status())?;

        let shinden_version = guard.version();
        let entries: Vec<ShindenEntry> = shinden.values().map(ShindenEntry::from).collect();
        info!(
            shinden_version,
            entries = entries.len(),
            "streaming full shinden list"
        );

        Ok(Response::new(stream_batches(
            shinden_version,
            entries,
            |shinden_version, entries| GetShindenFullResponse {
                shinden_version,
                entries,
            },
        )))
    }

    // Database
    #[instrument(skip_all, name = "grpc.check_database_update")]
    async fn check_database_update(
        &self,
        request: Request<CheckDatabaseUpdateRequest>,
    ) -> Result<Response<CheckDatabaseUpdateResponse>, Status> {
        let request = request.into_inner();
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
        }
        .into())
    }

    #[instrument(skip_all, name = "grpc.download_database")]
    async fn download_database(
        &self,
        request: Request<DownloadDatabaseRequest>,
    ) -> Result<Response<DownloadDatabaseResponse>, Status> {
        let request = request.into_inner();
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
        }
        .into())
    }

    #[instrument(skip_all, name = "grpc.load_database")]
    async fn load_database(
        &self,
        request: Request<LoadDatabaseRequest>,
    ) -> Result<Response<LoadDatabaseResponse>, Status> {
        let request = request.into_inner();
        info!(path = %request.path, "loading database");

        let path = request.path;
        let database = spawn_blocking_status("database loading", move || {
            DatabaseState::load(path).map_err(IntoStatus::into_status)
        })
        .await?;
        let entries = database.database.len();

        let database_version = self.database.store(database);
        info!(database_version, entries, "database loaded");

        Ok(LoadDatabaseResponse { database_version }.into())
    }

    #[instrument(skip_all, name = "grpc.get_database_metadata")]
    async fn get_database_metadata(
        &self,
        request: Request<GetDatabaseMetadataRequest>,
    ) -> Result<Response<GetDatabaseMetadataResponse>, Status> {
        let request = request.into_inner();
        info!(path = %request.path, "loading database metadata");

        let path = request.path;
        let metadata = spawn_blocking_status("database metadata loading", move || {
            root_metadata_from_path(path).map_err(IntoStatus::into_status)
        })
        .await?;
        info!("database metadata loaded");

        Ok(GetDatabaseMetadataResponse {
            metadata: Some(metadata.into()),
        }
        .into())
    }

    #[instrument(skip_all, name = "grpc.get_database_ids")]
    async fn get_database_ids(
        &self,
        _request: Request<GetDatabaseIdsRequest>,
    ) -> Result<Response<GetDatabaseIdsResponse>, Status> {
        info!("loading database ids");

        let guard = self.database.load();

        let database_version = guard.version();
        let database = guard.get().ok_or_else(|| database_not_loaded().into_status())?;

        let ids: Vec<u64> = database.database.values().map(|entry| entry.id()).collect();
        info!(database_version, ids = ids.len(), "database ids loaded");

        Ok(GetDatabaseIdsResponse {
            database_version,
            ids,
        }
        .into())
    }

    #[instrument(skip_all, name = "grpc.get_database_entries")]
    async fn get_database_entries(
        &self,
        request: Request<GetDatabaseEntriesRequest>,
    ) -> Result<Response<GetDatabaseEntriesResponse>, Status> {
        let request = request.into_inner();
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
        }
        .into())
    }

    type GetDatabaseFullStream = ReceiverStream<Result<GetDatabaseFullResponse, Status>>;

    #[instrument(skip_all, name = "grpc.get_database_full")]
    async fn get_database_full(
        &self,
        _request: Request<GetDatabaseFullRequest>,
    ) -> Result<Response<Self::GetDatabaseFullStream>, Status> {
        let guard = self.database.load();

        let database_version = guard.version();
        let database = guard.get().ok_or_else(|| database_not_loaded().into_status())?;

        let entries = database
            .database
            .values()
            .map(DatabaseEntry::from)
            .collect::<Vec<_>>();
        info!(
            database_version,
            entries = entries.len(),
            "streaming full database"
        );

        Ok(Response::new(stream_batches(
            database_version,
            entries,
            |database_version, entries| GetDatabaseFullResponse {
                database_version,
                entries,
            },
        )))
    }

    // Matching / Export
    #[instrument(skip_all, name = "grpc.fuzzy_search")]
    async fn fuzzy_search(
        &self,
        request: Request<FuzzySearchRequest>,
    ) -> Result<Response<FuzzySearchResponse>, Status> {
        let request = request.into_inner();
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
        }
        .into())
    }

    #[instrument(skip_all, name = "grpc.fuzzy_match")]
    async fn fuzzy_match(
        &self,
        request: Request<FuzzyMatchRequest>,
    ) -> Result<Response<FuzzyMatchResponse>, Status> {
        let request = request.into_inner();
        let query_len = request.query.len();
        info!(query_len, "running fuzzy match");
        let guard = self.database.load();

        let database_version = guard.version();
        let database = guard.get().ok_or_else(|| database_not_loaded().into_status())?;
        let query = QueryMatchView::new(request.query);
        let options = search_options(request.options, SearchMode::Fuzzy);
        let matcher = DefaultMatcher {
            search_weight: 0.8,
            season_weight: 0.2,
            ..Default::default()
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
        }
        .into())
    }

    type MatchShindenListStream = ReceiverStream<Result<MatchShindenListResponse, Status>>;

    #[instrument(skip_all, name = "grpc.match_shinden_list")]
    async fn match_shinden_list(
        &self,
        request: Request<MatchShindenListRequest>,
    ) -> Result<Response<Self::MatchShindenListStream>, Status> {
        let request = request.into_inner();
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

        Ok(Response::new(stream_batches(
            shinden_version,
            results,
            move |shinden_version, results| MatchShindenListResponse {
                shinden_version,
                database_version,
                results,
            },
        )))
    }

    #[instrument(skip_all, name = "grpc.export_xml")]
    async fn export_xml(
        &self,
        request: Request<ExportXmlRequest>,
    ) -> Result<Response<ExportXmlResponse>, Status> {
        let request = request.into_inner();
        let requested_matches = request.matches.len();
        info!(path = %request.path, requested_matches, "exporting xml");
        let guard = self.shinden_list.load();

        let shinden_version = guard.version();
        let shinden = guard
            .get_arc()
            .ok_or_else(|| shinden_list_not_loaded().into_status())?;
        let path = request.path;
        let matches = request
            .matches
            .into_iter()
            .map(|pair| (pair.shinden_id, pair.database_id))
            .collect::<Vec<_>>();

        let path = spawn_blocking_status("xml export", move || {
            export_xml_to_path(&shinden, matches.into_iter(), &path)?;
            Ok(path)
        })
        .await?;
        info!(shinden_version, path = %path, requested_matches, "xml exported");

        Ok(ExportXmlResponse {
            shinden_version,
            path,
        }
        .into())
    }
}
