#![deny(unreachable_pub)]
#![cfg_attr(not(test), deny(clippy::unwrap_used))]

pub mod ngram;

mod converter;
pub use converter::*;
pub mod utils;

#[cfg(test)]
use mimalloc::MiMalloc;

#[cfg(test)]
#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

pub use chrono::{
    Datelike,
    NaiveDate,
    Utc,
};
pub use compact_str::CompactString;
pub use reqwest::{
    Client as HttpClient,
    Error as HttpError,
};
pub use serde_json::Error as JsonError;
pub use serde_xml_rs::Error as XmlError;
pub use tokio::task::JoinError;
