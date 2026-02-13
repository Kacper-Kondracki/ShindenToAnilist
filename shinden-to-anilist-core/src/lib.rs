#![deny(unreachable_pub)]
// #![deny(clippy::indexing_slicing)]
// #![deny(clippy::unwrap_used)]

use std::sync::LazyLock;

use reqwest::Client;

pub mod ngram;

mod converter;
pub use converter::*;
pub mod utils;

#[cfg(test)]
use mimalloc::MiMalloc;

#[cfg(test)]
#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

pub(crate) static HTTP_CLIENT: LazyLock<Client> = LazyLock::new(|| Client::builder().build().unwrap());
pub(crate) fn http_client() -> Client { HTTP_CLIENT.clone() }

pub use chrono::{
    Datelike,
    NaiveDate,
    Utc,
};
pub use compact_str::CompactString;
pub use serde_json::Error as JsonError;
pub use serde_xml_rs::Error as XmlError;
pub use tokio::task::JoinError;
