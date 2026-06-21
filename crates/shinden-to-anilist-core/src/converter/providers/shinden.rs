use std::{
    io,
    io::Read,
};

use reqwest::{
    Client,
    Response as HttpResponse,
    blocking::{
        Client as BlockingClient,
        Response as BlockingHttpResponse,
    },
    header::{
        CONTENT_TYPE,
        HeaderMap,
    },
};
use thiserror::Error;

pub use self::models::*;

mod json;
pub mod models;

/// Trait for fetching a user's anime list from Shinden.
///
/// Provides methods to load data from the Shinden API or from a JSON serialization.
pub trait ShindenListLoad {
    /// Fetches a specific page of a user's anime list.
    ///
    /// - `user`: The Shinden user ID.
    /// - `limit`: Number of entries per page.
    /// - `offset`: Pagination offset.
    fn shinden_request(
        client: Client,
        user: u64,
        limit: u64,
        offset: u64,
    ) -> impl Future<Output = Result<ShindenList, ShindenError>> + Send;
    /// Fetches the *entire* anime list for a user.
    ///
    /// This implementation requests a very large limit (99999) to retrieve
    /// the full list in a single request.
    fn get_from_shinden(
        client: Client,
        user: u64,
    ) -> impl Future<Output = Result<ShindenList, ShindenError>> + Send;

    /// Fetches a specific page of a user's anime list using reqwest's blocking client.
    fn shinden_request_blocking(
        client: BlockingClient,
        user: u64,
        limit: u64,
        offset: u64,
    ) -> Result<ShindenList, ShindenError>;

    /// Fetches the *entire* anime list for a user using reqwest's blocking client.
    fn get_from_shinden_blocking(client: BlockingClient, user: u64) -> Result<ShindenList, ShindenError>;

    /// Deserializes a [`ShindenList`] from a JSON reader.
    fn from_reader(reader: &mut impl Read) -> Result<ShindenList, ShindenError>;
}

/// Errors that can occur when interacting with Shinden.
#[derive(Error, Debug)]
#[error(transparent)]
pub enum ShindenError {
    /// I/O error during reading/writing.
    Io(#[from] io::Error),
    /// JSON serialization/deserialization error.
    Json(#[from] serde_json::Error),
    /// HTTP request error.
    Request(#[from] reqwest::Error),
    /// HTTP response was not a successful JSON API response.
    #[error(
        "shinden api returned HTTP {status}; content-type: {content_type}; cf-mitigated: {cf_mitigated}; body: {body_preview}"
    )]
    Http {
        status: reqwest::StatusCode,
        content_type: String,
        cf_mitigated: String,
        body_preview: String,
    },
    /// API returned an application-level error message.
    #[error("shinden api returned error: {0}")]
    Shinden(String),
}

const SHINDEN_API_URL: &str = "https://lista.shinden.pl/api/userlist";
const BODY_PREVIEW_CHARS: usize = 200;

fn shinden_url(user: u64, limit: u64, offset: u64) -> String {
    format!("{SHINDEN_API_URL}/{user}/anime?limit={limit}&offset={offset}")
}

fn header_value(headers: &HeaderMap, name: &'static str) -> String {
    headers
        .get(name)
        .and_then(|value| value.to_str().ok())
        .unwrap_or("<missing>")
        .to_owned()
}

fn body_preview(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes)
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .chars()
        .take(BODY_PREVIEW_CHARS)
        .collect()
}

async fn read_shinden_response(response: HttpResponse) -> Result<Vec<u8>, ShindenError> {
    let status = response.status();
    let headers = response.headers().clone();
    let bytes = response.bytes().await?.to_vec();

    if status.is_success() {
        return Ok(bytes);
    }

    Err(ShindenError::Http {
        status,
        content_type: header_value(&headers, CONTENT_TYPE.as_str()),
        cf_mitigated: header_value(&headers, "cf-mitigated"),
        body_preview: body_preview(&bytes),
    })
}

fn read_shinden_response_blocking(response: BlockingHttpResponse) -> Result<Vec<u8>, ShindenError> {
    let status = response.status();
    let headers = response.headers().clone();
    let bytes = response.bytes()?.to_vec();

    if status.is_success() {
        return Ok(bytes);
    }

    Err(ShindenError::Http {
        status,
        content_type: header_value(&headers, CONTENT_TYPE.as_str()),
        cf_mitigated: header_value(&headers, "cf-mitigated"),
        body_preview: body_preview(&bytes),
    })
}

impl ShindenListLoad for ShindenList {
    async fn shinden_request(
        client: Client,
        user: u64,
        limit: u64,
        offset: u64,
    ) -> Result<ShindenList, ShindenError> {
        let response = client.get(shinden_url(user, limit, offset)).send().await?;
        let bytes = read_shinden_response(response).await?;

        let data = serde_json::from_slice::<json::Response>(&bytes)?;
        let shinden_list = data.try_par_into_model().map_err(ShindenError::Shinden)?;

        Ok(shinden_list)
    }

    async fn get_from_shinden(client: Client, user: u64) -> Result<ShindenList, ShindenError> {
        Self::shinden_request(client, user, 99999, 0).await
    }

    fn shinden_request_blocking(
        client: BlockingClient,
        user: u64,
        limit: u64,
        offset: u64,
    ) -> Result<ShindenList, ShindenError> {
        let response = client.get(shinden_url(user, limit, offset)).send()?;
        let bytes = read_shinden_response_blocking(response)?;

        let data = serde_json::from_slice::<json::Response>(&bytes)?;
        data.try_par_into_model().map_err(ShindenError::Shinden)
    }

    fn get_from_shinden_blocking(client: BlockingClient, user: u64) -> Result<ShindenList, ShindenError> {
        Self::shinden_request_blocking(client, user, 99999, 0)
    }

    fn from_reader(reader: &mut impl Read) -> Result<ShindenList, ShindenError> {
        let data: json::Response = serde_json::from_reader(reader)?;
        data.try_par_into_model().map_err(ShindenError::Shinden)
    }
}
