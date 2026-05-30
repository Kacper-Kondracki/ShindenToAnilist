//! Core library for converting anime lists from [Shinden](https://shinden.pl) to
//! [MyAnimeList](https://myanimelist.net)-compatible XML format.
//!
//! # Overview
//!
//! This crate provides a pipeline for:
//! 1. **Loading** a user's anime list from the Shinden API ([`providers::shinden`]).
//! 2. **Loading** an offline anime database in JSONL format ([`database`]).
//! 3. **Searching** the database for candidate matches using n-gram indexing ([`searcher`]).
//! 4. **Matching** entries against candidates with a weighted multifactor scoring algorithm ([`matcher`]).
//! 5. **Exporting** matched entries to MAL-compatible XML ([`exporter`]).
//!
//! # Re-exports
//!
//! Common external types used across the public API are re-exported for convenience,
//! [`Datelike`], [`NaiveDate`], [`Utc`], [`CompactString`], [`iter`], [`HttpClient`], [`HttpError`], [`JsonError`], [`XmlError`], [`JoinError`].

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
pub use quick_xml::se::SeError as XmlError;
pub use rayon::iter;
pub use reqwest::{
    Client as HttpClient,
    Error as HttpError,
};
pub use serde_json::Error as JsonError;
pub use tokio::task::JoinError;
