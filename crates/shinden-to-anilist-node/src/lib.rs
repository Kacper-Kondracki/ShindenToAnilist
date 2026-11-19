use std::{
    future::Future,
    pin::Pin,
};

use async_stream::stream;
use futures_util::{
    Stream,
    StreamExt,
};
use napi::{
    Error,
    Result,
    Status,
    bindgen_prelude::Buffer,
};
use napi_derive::napi;
use prost::Message;
use shinden_to_anilist_grpc::{
    ShindenToAnilist,
    diagnostics,
    error::ServiceError,
    pb::*,
};
use tokio::sync::{
    Mutex,
    watch,
};
use tokio_util::sync::CancellationToken;

const BATCH_SIZE: usize = 500;

#[napi(object)]
pub struct NativeReply {
    pub data: Buffer,
    pub is_error: bool,
}

impl NativeReply {
    fn message(value: impl Message) -> Self {
        Self {
            data: value.encode_to_vec().into(),
            is_error: false,
        }
    }

    fn error(error: ServiceError) -> Self {
        Self {
            data: error.report().encode_to_vec().into(),
            is_error: true,
        }
    }

    fn from_result(result: std::result::Result<impl Message, ServiceError>) -> Self {
        match result {
            Ok(message) => Self::message(message),
            Err(error) => Self::error(error),
        }
    }
}

fn decode<T: Message + Default>(input: &[u8]) -> Result<T> {
    T::decode(input).map_err(|error| Error::new(Status::InvalidArg, error.to_string()))
}

fn unknown_method(method: &str) -> Error {
    Error::new(
        Status::InvalidArg,
        format!("Unsupported native service method {method}"),
    )
}

#[napi]
pub struct NativeService {
    service: ShindenToAnilist,
}

#[napi]
impl NativeService {
    #[napi(constructor)]
    pub fn new() -> Result<Self> {
        diagnostics::init("native", env!("CARGO_PKG_VERSION"));
        let service = ShindenToAnilist::new().map_err(|error| Error::from_reason(error.to_string()))?;
        Ok(Self { service })
    }

    #[napi]
    pub async fn call(&self, method: String, input: Buffer) -> Result<NativeReply> {
        let service = &self.service;
        let response = match method.as_str() {
            "GetSourceIds" => NativeReply::from_result(service.get_source_ids(decode(&input)?).await),
            "FetchShindenList" => NativeReply::from_result(service.fetch_shinden_list(decode(&input)?).await),
            "SetShindenCloudflareClearance" => {
                NativeReply::from_result(service.set_shinden_cloudflare_clearance(decode(&input)?))
            },
            "GetShindenIds" => NativeReply::from_result(service.get_shinden_ids(decode(&input)?).await),
            "GetShindenEntries" => {
                NativeReply::from_result(service.get_shinden_entries(decode(&input)?).await)
            },
            "CheckDatabaseUpdate" => {
                NativeReply::from_result(service.check_database_update(decode(&input)?).await)
            },
            "DownloadDatabase" => NativeReply::from_result(service.download_database(decode(&input)?).await),
            "LoadDatabase" => NativeReply::from_result(service.load_database(decode(&input)?).await),
            "GetDatabaseMetadata" => {
                NativeReply::from_result(service.get_database_metadata(decode(&input)?).await)
            },
            "GetDatabaseIds" => NativeReply::from_result(service.get_database_ids(decode(&input)?).await),
            "GetDatabaseEntries" => {
                NativeReply::from_result(service.get_database_entries(decode(&input)?).await)
            },
            "FuzzySearch" => NativeReply::from_result(service.fuzzy_search(decode(&input)?).await),
            "FuzzyMatch" => NativeReply::from_result(service.fuzzy_match(decode(&input)?).await),
            "ExportXml" => NativeReply::from_result(service.export_xml(decode(&input)?).await),
            "ExportWorkspaceXml" => {
                let request: ExportXmlRequest = decode(&input)?;
                if request.expected_source_version.is_none() || request.expected_database_version.is_none() {
                    return Err(Error::new(
                        Status::InvalidArg,
                        "workspace export requires source and database versions",
                    ));
                }
                NativeReply::from_result(service.export_xml(request).await)
            },
            _ => return Err(unknown_method(&method)),
        };
        Ok(response)
    }

    #[napi]
    pub fn open_stream(&self, method: String, input: Buffer) -> Result<NativeStream> {
        let service = self.service.clone();
        let cancellation = CancellationToken::new();
        let replies: Replies = match method.as_str() {
            "FetchSourceList" => Box::pin(fetch_source_list(service, decode(&input)?, cancellation.clone())),
            "GetSourceFull" => {
                let request = decode(&input)?;
                Box::pin(batches(
                    async move { service.get_source_full(request).await },
                    |response| {
                        let source_version = response.source_version;
                        (response.entries, move |entries| GetSourceFullResponse {
                            source_version,
                            entries,
                        })
                    },
                ))
            },
            "GetShindenFull" => {
                let request = decode(&input)?;
                Box::pin(batches(
                    async move { service.get_shinden_full(request).await },
                    |response| {
                        let shinden_version = response.shinden_version;
                        (response.entries, move |entries| GetShindenFullResponse {
                            shinden_version,
                            entries,
                        })
                    },
                ))
            },
            "GetDatabaseFull" => {
                let request = decode(&input)?;
                Box::pin(batches(
                    async move { service.get_database_full(request).await },
                    |response| {
                        let database_version = response.database_version;
                        (response.entries, move |entries| GetDatabaseFullResponse {
                            database_version,
                            entries,
                        })
                    },
                ))
            },
            "MatchSourceList" => {
                let request = decode(&input)?;
                Box::pin(batches(
                    async move { service.match_source_list(request).await },
                    |response| {
                        let (source_version, database_version) =
                            (response.source_version, response.database_version);
                        (response.results, move |results| MatchSourceListResponse {
                            source_version,
                            database_version,
                            results,
                        })
                    },
                ))
            },
            "MatchShindenList" => {
                let request = decode(&input)?;
                Box::pin(batches(
                    async move { service.match_shinden_list(request).await },
                    |response| {
                        let (shinden_version, database_version) =
                            (response.shinden_version, response.database_version);
                        (response.results, move |results| MatchShindenListResponse {
                            shinden_version,
                            database_version,
                            results,
                        })
                    },
                ))
            },
            _ => return Err(unknown_method(&method)),
        };

        Ok(NativeStream {
            replies: Mutex::new(Some(replies)),
            cancellation,
        })
    }
}

type Replies = Pin<Box<dyn Stream<Item = NativeReply> + Send>>;

#[napi]
pub struct NativeStream {
    replies: Mutex<Option<Replies>>,
    cancellation: CancellationToken,
}

#[napi]
impl NativeStream {
    #[napi]
    pub async fn next(&self) -> Option<NativeReply> {
        let mut replies = self.replies.lock().await;
        let stream = replies.as_mut()?;
        let reply = tokio::select! {
            biased;
            () = self.cancellation.cancelled() => None,
            reply = stream.next() => reply,
        };

        if reply.is_none() || reply.as_ref().is_some_and(|reply| reply.is_error) {
            replies.take();
        }

        reply
    }

    #[napi]
    pub fn close(&self) {
        self.cancellation.cancel();
        if let Ok(mut replies) = self.replies.try_lock() {
            replies.take();
        }
    }
}

fn fetch_source_list(
    service: ShindenToAnilist,
    request: FetchSourceListRequest,
    cancellation: CancellationToken,
) -> impl Stream<Item = NativeReply> {
    stream! {
        // Progress may arrive faster than the renderer can read. Keep the latest
        // update and deliver the completion or error independently.
        let (progress, mut updates) = watch::channel(None);
        let fetch = service.fetch_source_list_with_progress(request, cancellation.clone(), |update| {
            if cancellation.is_cancelled() {
                return Err(ServiceError::new(ErrorKind::Cancelled, "source import cancelled"));
            }
            progress.send_replace(Some(update));
            Ok(())
        });
        tokio::pin!(fetch);

        loop {
            tokio::select! {
                biased;
                Ok(()) = updates.changed() => {
                    let progress = updates.borrow_and_update().clone();
                    yield NativeReply::message(FetchSourceListResponse {
                        source_version: 0,
                        progress,
                        done: false,
                    });
                },
                result = &mut fetch => {
                    yield NativeReply::from_result(result);
                    break;
                },
            }
        }
    }
}

fn batches<R, T, M, F>(
    response: impl Future<Output = std::result::Result<R, ServiceError>>,
    split: impl FnOnce(R) -> (Vec<T>, F),
) -> impl Stream<Item = NativeReply>
where
    M: Message,
    F: Fn(Vec<T>) -> M,
{
    stream! {
        let response = match response.await {
            Ok(response) => response,
            Err(error) => {
                yield NativeReply::error(error);
                return;
            },
        };
        let (items, make_response) = split(response);
        let mut items = items.into_iter();

        loop {
            let batch = items.by_ref().take(BATCH_SIZE).collect();
            let last = items.len() == 0;
            // Empty lists still have a version.
            yield NativeReply::message(make_response(batch));
            if last {
                break;
            }
        }
    }
}
