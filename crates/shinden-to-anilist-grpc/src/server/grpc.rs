use tokio::sync::mpsc;
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

use super::{
    ShindenToAnilist,
    streaming,
};
use crate::{
    error::ServiceError,
    pb::{
        shinden_to_anilist_service_server::ShindenToAnilistService,
        *,
    },
};

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
                    .map_err(|_| {
                        ServiceError::new(ErrorKind::Cancelled, "source progress receiver disconnected")
                    })
            });
            tokio::pin!(fetch);

            tokio::select! {
                result = &mut fetch => {
                    match result {
                        Ok(response) => {
                            let _ = tx.send(Ok(response));
                        },
                        Err(status) => {
                            let _ = tx.send(Err(status.into()));
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
            .map_err(Into::into)
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
            .map_err(Into::into)
    }

    async fn set_shinden_cloudflare_clearance(
        &self,
        request: Request<SetShindenCloudflareClearanceRequest>,
    ) -> Result<Response<SetShindenCloudflareClearanceResponse>, Status> {
        ShindenToAnilist::set_shinden_cloudflare_clearance(self, request.into_inner())
            .map(Response::new)
            .map_err(Into::into)
    }

    async fn get_shinden_ids(
        &self,
        request: Request<GetShindenIdsRequest>,
    ) -> Result<Response<GetShindenIdsResponse>, Status> {
        ShindenToAnilist::get_shinden_ids(self, request.into_inner())
            .await
            .map(Response::new)
            .map_err(Into::into)
    }

    async fn get_shinden_entries(
        &self,
        request: Request<GetShindenEntriesRequest>,
    ) -> Result<Response<GetShindenEntriesResponse>, Status> {
        ShindenToAnilist::get_shinden_entries(self, request.into_inner())
            .await
            .map(Response::new)
            .map_err(Into::into)
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
            .map_err(Into::into)
    }

    async fn download_database(
        &self,
        request: Request<DownloadDatabaseRequest>,
    ) -> Result<Response<DownloadDatabaseResponse>, Status> {
        ShindenToAnilist::download_database(self, request.into_inner())
            .await
            .map(Response::new)
            .map_err(Into::into)
    }

    async fn load_database(
        &self,
        request: Request<LoadDatabaseRequest>,
    ) -> Result<Response<LoadDatabaseResponse>, Status> {
        ShindenToAnilist::load_database(self, request.into_inner())
            .await
            .map(Response::new)
            .map_err(Into::into)
    }

    async fn get_database_metadata(
        &self,
        request: Request<GetDatabaseMetadataRequest>,
    ) -> Result<Response<GetDatabaseMetadataResponse>, Status> {
        ShindenToAnilist::get_database_metadata(self, request.into_inner())
            .await
            .map(Response::new)
            .map_err(Into::into)
    }

    async fn get_database_ids(
        &self,
        request: Request<GetDatabaseIdsRequest>,
    ) -> Result<Response<GetDatabaseIdsResponse>, Status> {
        ShindenToAnilist::get_database_ids(self, request.into_inner())
            .await
            .map(Response::new)
            .map_err(Into::into)
    }

    async fn get_database_entries(
        &self,
        request: Request<GetDatabaseEntriesRequest>,
    ) -> Result<Response<GetDatabaseEntriesResponse>, Status> {
        ShindenToAnilist::get_database_entries(self, request.into_inner())
            .await
            .map(Response::new)
            .map_err(Into::into)
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
            .map_err(Into::into)
    }

    async fn fuzzy_match(
        &self,
        request: Request<FuzzyMatchRequest>,
    ) -> Result<Response<FuzzyMatchResponse>, Status> {
        ShindenToAnilist::fuzzy_match(self, request.into_inner())
            .await
            .map(Response::new)
            .map_err(Into::into)
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
            .map_err(Into::into)
    }

    async fn export_workspace_xml(
        &self,
        request: Request<ExportXmlRequest>,
    ) -> Result<Response<ExportXmlResponse>, Status> {
        let request = request.into_inner();
        if request.expected_source_version.is_none() || request.expected_database_version.is_none() {
            return Err(Status::invalid_argument(
                "workspace export requires source and database versions",
            ));
        }
        ShindenToAnilist::export_xml(self, request)
            .await
            .map(Response::new)
            .map_err(Into::into)
    }
}
