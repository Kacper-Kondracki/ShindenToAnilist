#![deny(unreachable_pub)]

use std::sync::LazyLock;

use reqwest::Client;

pub(crate) mod ngram;

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

pub use chrono;
pub use compact_str;
pub use serde_json;
pub use serde_xml_rs;
pub use tokio::task;
