use std::{
    cmp::Ordering,
    fs::File,
    io::{
        self,
        BufReader,
        BufWriter,
    },
    path::{
        Path,
        PathBuf,
    },
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
    Pipe,
    Tap,
};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{
    Request,
    Response,
    Status,
    async_trait,
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
    export::export_xml_to_path,
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
const STREAM_CHUNK_SIZE: usize = 500;

#[derive(Debug, Default)]
pub struct ShindenToAnilist {
    http_client: reqwest::Client,
    shinden_list: VersionedArcOption<ShindenList>,
    database: VersionedArcOption<DatabaseState>,
}

impl ShindenToAnilist {
    pub fn new(http_client: reqwest::Client) -> Self {
        Self {
            http_client,
            shinden_list: VersionedArcOption::empty(),
            database: VersionedArcOption::empty(),
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
        return Ok(None);
    }

    let sidecar_path = database_sidecar_path(database_path);
    let sidecar = match File::open(&sidecar_path) {
        Ok(sidecar) => sidecar,
        Err(err) if err.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(err) => {
            return Err(database_sidecar_io_error(err, &sidecar_path, "open").into_status());
        },
    };

    serde_json::from_reader(BufReader::new(sidecar))
        .map(Some)
        .map_err(|err| database_sidecar_json_error(err, &sidecar_path).into_status())
}

fn write_database_sidecar(
    database_path: impl AsRef<Path>,
    status: &CoreDatabaseReleaseInfo,
) -> Result<(), Status> {
    let sidecar_path = database_sidecar_path(database_path);
    let writer = File::options()
        .create(true)
        .write(true)
        .truncate(true)
        .open(&sidecar_path)
        .map_err(|err| database_sidecar_io_error(err, &sidecar_path, "write").into_status())?
        .pipe(BufWriter::new);

    serde_json::to_writer_pretty(writer, status)
        .map_err(|err| database_sidecar_json_error(err, &sidecar_path).into_status())
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
    let (tx, rx) = mpsc::channel(64);

    tokio::spawn(async move {
        let mut iter = entries.into_iter();
        loop {
            let batch = iter.by_ref().take(STREAM_CHUNK_SIZE).collect::<Vec<_>>();
            if batch.is_empty() {
                break;
            }

            if tx.send(Ok(into_response(version, batch))).await.is_err() {
                break;
            }
        }
    });

    ReceiverStream::new(rx)
}

#[async_trait]
impl ShindenToAnilistService for ShindenToAnilist {
    // Shinden
    async fn fetch_shinden_list(
        &self,
        request: Request<FetchShindenListRequest>,
    ) -> Result<Response<FetchShindenListResponse>, Status> {
        let request = request.into_inner();

        let shinden = ShindenList::get_from_shinden(self.http_client.clone(), request.id)
            .await
            .map_err(IntoStatus::into_status)?;

        let shinden_version = self.shinden_list.store(shinden);

        Ok(FetchShindenListResponse { shinden_version }.into())
    }

    async fn get_shinden_ids(
        &self,
        request: Request<GetShindenIdsRequest>,
    ) -> Result<Response<GetShindenIdsResponse>, Status> {
        let request = request.into_inner();

        let guard = self.shinden_list.load();

        let shinden = guard
            .get()
            .ok_or_else(|| shinden_list_not_loaded().into_status())?;

        let shinden_version = guard.version();
        let ids = shinden
            .iter()
            .map(|(id, entry)| (id, entry.premiere_date()))
            .collect::<Vec<_>>()
            .tap_mut(|v| {
                if request.sorted_by() == AnimeListSortedBy::Urgency {
                    v.sort_by(|(_, date_a), (_, date_b)| match (date_a, date_b) {
                        (None, None) => Ordering::Equal,
                        (None, Some(_)) => Ordering::Less,
                        (Some(_), None) => Ordering::Greater,
                        (Some(date_a), Some(date_b)) => date_a.cmp(date_b),
                    })
                }
            })
            .into_iter()
            .map(|(id, _)| id)
            .collect();

        Ok(GetShindenIdsResponse { shinden_version, ids }.into())
    }

    async fn get_shinden_entries(
        &self,
        request: Request<GetShindenEntriesRequest>,
    ) -> Result<Response<GetShindenEntriesResponse>, Status> {
        let request = request.into_inner();

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
    async fn check_database_update(
        &self,
        request: Request<CheckDatabaseUpdateRequest>,
    ) -> Result<Response<CheckDatabaseUpdateResponse>, Status> {
        let request = request.into_inner();

        let remote = latest_database_archive_asset(self.http_client.clone())
            .await
            .map_err(IntoStatus::into_status)?
            .conv::<CoreDatabaseReleaseInfo>();
        let local = read_database_sidecar(&request.path)?;

        Ok(CheckDatabaseUpdateResponse {
            status: Some(database_update_check(local, remote)),
        }
        .into())
    }

    async fn download_database(
        &self,
        request: Request<DownloadDatabaseRequest>,
    ) -> Result<Response<DownloadDatabaseResponse>, Status> {
        let request = request.into_inner();

        let status = update_latest_jsonl_from_github(self.http_client.clone(), &request.path)
            .await
            .map_err(IntoStatus::into_status)?
            .conv::<CoreDatabaseReleaseInfo>();

        write_database_sidecar(&request.path, &status)?;

        Ok(DownloadDatabaseResponse {
            status: Some(status.into()),
        }
        .into())
    }

    async fn load_database(
        &self,
        request: Request<LoadDatabaseRequest>,
    ) -> Result<Response<LoadDatabaseResponse>, Status> {
        let request = request.into_inner();

        let database = DatabaseState::load(request.path).map_err(IntoStatus::into_status)?;

        let database_version = self.database.store(database);

        Ok(LoadDatabaseResponse { database_version }.into())
    }

    async fn get_database_metadata(
        &self,
        request: Request<GetDatabaseMetadataRequest>,
    ) -> Result<Response<GetDatabaseMetadataResponse>, Status> {
        let request = request.into_inner();

        let metadata = root_metadata_from_path(request.path).map_err(IntoStatus::into_status)?;

        Ok(GetDatabaseMetadataResponse {
            metadata: Some(metadata.into()),
        }
        .into())
    }

    async fn get_database_entries(
        &self,
        request: Request<GetDatabaseEntriesRequest>,
    ) -> Result<Response<GetDatabaseEntriesResponse>, Status> {
        let request = request.into_inner();

        let guard = self.database.load();

        let database_version = guard.version();
        let database = guard.get().ok_or_else(|| database_not_loaded().into_status())?;

        let entries = request
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
    async fn fuzzy_search(
        &self,
        request: Request<FuzzySearchRequest>,
    ) -> Result<Response<FuzzySearchResponse>, Status> {
        let request = request.into_inner();
        let guard = self.database.load();

        let database_version = guard.version();
        let database = guard.get().ok_or_else(|| database_not_loaded().into_status())?;
        let query = normalize_str(&request.query);
        let options = search_options(request.options, SearchMode::Fuzzy);

        let results = database
            .searcher
            .search(&query, options)
            .into_iter()
            .map(|(id, score)| SearchResult { id, score })
            .collect();

        Ok(FuzzySearchResponse {
            database_version,
            results,
        }
        .into())
    }

    async fn fuzzy_match(
        &self,
        request: Request<FuzzyMatchRequest>,
    ) -> Result<Response<FuzzyMatchResponse>, Status> {
        let request = request.into_inner();
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
        let results = matcher
            .score_candidates(&query, &candidates, 0.0)
            .items()
            .iter()
            .copied()
            .map(Into::into)
            .collect();

        Ok(FuzzyMatchResponse {
            database_version,
            results,
        }
        .into())
    }

    type MatchShindenListStream = ReceiverStream<Result<MatchShindenListResponse, Status>>;

    async fn match_shinden_list(
        &self,
        request: Request<MatchShindenListRequest>,
    ) -> Result<Response<Self::MatchShindenListStream>, Status> {
        let request = request.into_inner();
        let shinden_guard = self.shinden_list.load();
        let database_guard = self.database.load();

        let shinden_version = shinden_guard.version();
        let database_version = database_guard.version();
        let shinden = shinden_guard
            .get()
            .ok_or_else(|| shinden_list_not_loaded().into_status())?;
        let database = database_guard
            .get()
            .ok_or_else(|| database_not_loaded().into_status())?;
        let options = search_options(request.options, SearchMode::Strict);
        let matcher = DefaultMatcher::strict_preset();

        let mut results = shinden
            .par_values()
            .map(|entry| entry.search_by_title_ref(&database.database, &database.searcher, options))
            .map(|(entry, candidates)| (entry.id(), matcher.score_candidates(entry, &candidates, 0.5)))
            .collect::<Vec<_>>();

        results.iter_mut().map(|(_, result)| result).finalize_matches();

        let results = results
            .into_iter()
            .map(ShindenMatchResult::from)
            .collect::<Vec<_>>();

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

    async fn export_xml(
        &self,
        request: Request<ExportXmlRequest>,
    ) -> Result<Response<ExportXmlResponse>, Status> {
        let request = request.into_inner();
        let guard = self.shinden_list.load();

        let shinden_version = guard.version();
        let shinden = guard
            .get()
            .ok_or_else(|| shinden_list_not_loaded().into_status())?;
        let path = request.path;
        let matches = request
            .matches
            .into_iter()
            .map(|pair| (pair.shinden_id, pair.database_id));

        export_xml_to_path(shinden, matches, &path)?;

        Ok(ExportXmlResponse {
            shinden_version,
            path,
        }
        .into())
    }
}
