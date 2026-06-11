use std::{
    cmp::Ordering,
    fs::File,
    io::{
        self,
        BufReader,
        BufWriter,
    },
    path::PathBuf,
};

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
    providers::shinden::{
        ShindenList,
        ShindenListLoad,
    },
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
    mapper::{
        database_error_to_status,
        shinden_error_to_status,
    },
    pb::{
        shinden_to_anilist_service_server::ShindenToAnilistService,
        *,
    },
};

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
            .map_err(shinden_error_to_status)?;

        let version = self.shinden_list.store(shinden);

        Ok(FetchShindenListResponse { version }.into())
    }

    async fn get_shinden_ids(
        &self,
        request: Request<GetShindenIdsRequest>,
    ) -> Result<Response<GetShindenIdsResponse>, Status> {
        let request = request.into_inner();

        let guard = self.shinden_list.load();

        let shinden = guard
            .get()
            .ok_or_else(|| Status::failed_precondition("shinden list is not loaded"))?;

        let version = guard.version();
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

        Ok(GetShindenIdsResponse { version, ids }.into())
    }

    async fn get_shinden_entries(
        &self,
        request: Request<GetShindenEntriesRequest>,
    ) -> Result<Response<GetShindenEntriesResponse>, Status> {
        let request = request.into_inner();

        let guard = self.shinden_list.load();

        let shinden = guard
            .get()
            .ok_or_else(|| Status::failed_precondition("shinden list is not loaded"))?;

        let version = guard.version();
        let entries: Vec<ShindenEntry> = request
            .ids
            .into_iter()
            .filter_map(|id| shinden.get(id).map(Into::into))
            .collect();

        Ok(GetShindenEntriesResponse { version, entries }.into())
    }

    async fn get_shinden_full(
        &self,
        _request: Request<GetShindenFullRequest>,
    ) -> Result<Response<GetShindenFullResponse>, Status> {
        let guard = self.shinden_list.load();

        let shinden = guard
            .get()
            .ok_or_else(|| Status::failed_precondition("shinden list is not loaded"))?;

        let version = guard.version();
        let entries: Vec<ShindenEntry> = shinden.values().map(ShindenEntry::from).collect();

        Ok(GetShindenFullResponse { version, entries }.into())
    }

    // Database
    async fn check_database_update(
        &self,
        request: Request<CheckDatabaseUpdateRequest>,
    ) -> Result<Response<CheckDatabaseUpdateResponse>, Status> {
        let request = request.into_inner();

        let remote = latest_database_archive_asset(self.http_client.clone())
            .await
            .map_err(database_error_to_status)?
            .conv::<database::updater::DatabaseReleaseInfo>();

        if !PathBuf::from(&request.path).exists() {
            return Ok(CheckDatabaseUpdateResponse {
                status: Some(DatabaseUpdateCheck {
                    local: None,
                    remote: Some(remote.into()),
                    needs_update: true,
                }),
            }
            .into());
        }

        let mut sidecar_path = PathBuf::from(request.path);
        sidecar_path.set_extension("info.json");

        let sidecar = File::open(sidecar_path);
        if let Err(err) = &sidecar
            && err.kind() == io::ErrorKind::NotFound
        {
            return Ok(CheckDatabaseUpdateResponse {
                status: Some(DatabaseUpdateCheck {
                    local: None,
                    remote: Some(remote.into()),
                    needs_update: true,
                }),
            }
            .into());
        }
        let sidecar = sidecar?.pipe(BufReader::new);
        let local = serde_json::from_reader::<_, database::updater::DatabaseReleaseInfo>(sidecar)
            .map_err(|err| Status::internal(err.to_string()))?;

        let needs_update = local != remote;

        Ok(CheckDatabaseUpdateResponse {
            status: Some(DatabaseUpdateCheck {
                local: Some(local.into()),
                remote: Some(remote.into()),
                needs_update,
            }),
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
            .map_err(database_error_to_status)?
            .conv::<database::updater::DatabaseReleaseInfo>();

        let mut sidecar_path = PathBuf::from(request.path);
        sidecar_path.set_extension("info.json");

        let writer = File::options()
            .create(true)
            .write(true)
            .truncate(true)
            .open(sidecar_path)?
            .pipe(BufWriter::new);

        serde_json::to_writer_pretty(writer, &status).map_err(|err| Status::internal(err.to_string()))?;

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

        let database = DatabaseState::load(request.path).map_err(database_error_to_status)?;

        let version = self.database.store(database);

        Ok(LoadDatabaseResponse { version }.into())
    }

    async fn get_database_metadata(
        &self,
        request: Request<GetDatabaseMetadataRequest>,
    ) -> Result<Response<GetDatabaseMetadataResponse>, Status> {
        let request = request.into_inner();

        let metadata = root_metadata_from_path(request.path).map_err(database_error_to_status)?;

        Ok(GetDatabaseMetadataResponse {
            metadata: Some(metadata.into()),
        }
        .into())
    }

    async fn get_database_entries(
        &self,
        request: Request<GetDatabaseEntriesRequest>,
    ) -> Result<Response<GetDatabaseEntriesResponse>, Status> {
        todo!()
    }

    type GetDatabaseFullStream = ReceiverStream<Result<GetDatabaseFullResponse, Status>>;

    async fn get_database_full(
        &self,
        request: Request<GetDatabaseFullRequest>,
    ) -> Result<Response<Self::GetDatabaseFullStream>, Status> {
        let guard = self.database.load();

        let version = guard.version();
        let database = guard
            .get()
            .ok_or_else(|| Status::failed_precondition("database is not loaded"))?;

        let entries = database
            .database
            .values()
            .map(DatabaseEntry::from)
            .collect::<Vec<_>>();

        let (tx, rx) = mpsc::channel(64);

        tokio::spawn(async move {
            let chunk_size = 500;
            let mut iter = entries.into_iter();
            loop {
                let batch: Vec<DatabaseEntry> = iter.by_ref().take(chunk_size).collect();
                if batch.is_empty() {
                    break;
                }

                let response = GetDatabaseFullResponse {
                    version,
                    entries: batch,
                };
                if tx.send(Ok(response)).await.is_err() {
                    break;
                }
            }
        });
        Ok(Response::new(ReceiverStream::new(rx)))
    }
}
