mod database;
mod matching;
mod source;
mod streaming;

use std::{
    sync::Arc,
    time::Duration,
};

use shinden_to_anilist_core::providers::shinden::ShindenList;
use tokio::sync::{
    Mutex,
    mpsc,
};
use tokio_stream::wrappers::{
    ReceiverStream,
    UnboundedReceiverStream,
};
use tokio_util::sync::CancellationToken;
use tonic::{
    Request,
    Response,
    Status,
    async_trait,
};

use crate::{
    DatabaseState,
    VersionedArcOption,
    pb::{
        shinden_to_anilist_service_server::ShindenToAnilistService,
        *,
    },
    source::SourceList,
};

const DATABASE_DOWNLOAD_LOCK_TIMEOUT: Duration = Duration::from_secs(10);
const SHINDEN_FETCH_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Debug, Default, Clone)]
pub struct ShindenToAnilist {
    pub(super) http_client: reqwest::Client,
    pub(super) shinden_list: VersionedArcOption<ShindenList>,
    pub(super) source_list: VersionedArcOption<SourceList>,
    pub(super) database: VersionedArcOption<DatabaseState>,
    pub(super) database_download_lock: Arc<Mutex<()>>,
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
        Ok(Response::new(streaming::stream_batches(
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
        Ok(Response::new(streaming::stream_batches(
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
        Ok(Response::new(streaming::stream_batches(
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
        Ok(Response::new(streaming::stream_batches(
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
        Ok(Response::new(streaming::stream_batches(
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
