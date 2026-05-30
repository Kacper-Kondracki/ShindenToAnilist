use std::{
    io,
    io::Read,
};

use reqwest::Client;
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
    /// Error in a background tokio task.
    TaskError(#[from] tokio::task::JoinError),
    /// API returned an application-level error message.
    #[error("shinden api returned error: {0}")]
    Shinden(String),
}

impl ShindenListLoad for ShindenList {
    async fn shinden_request(
        client: Client,
        user: u64,
        limit: u64,
        offset: u64,
    ) -> Result<ShindenList, ShindenError> {
        let bytes = client
            .get(format!(
                "https://lista.shinden.pl/api/userlist/{}/anime?limit={}&offset={}",
                user, limit, offset
            ))
            .send()
            .await?
            .bytes()
            .await?;

        let data = tokio::task::spawn_blocking(move || {
            serde_json::from_slice::<json::Response>(&bytes).map(|r| r.try_par_into_model())
        });

        let shinden_list = data.await??.map_err(ShindenError::Shinden)?;

        Ok(shinden_list)
    }

    async fn get_from_shinden(client: Client, user: u64) -> Result<ShindenList, ShindenError> {
        Self::shinden_request(client, user, 99999, 0).await
    }

    fn from_reader(reader: &mut impl Read) -> Result<ShindenList, ShindenError> {
        let data: json::Response = serde_json::from_reader(reader)?;
        data.try_par_into_model().map_err(ShindenError::Shinden)
    }
}
