#![deny(unreachable_pub)]

use std::sync::LazyLock;

use reqwest::Client;

pub(crate) mod ngram;

pub mod converter;
pub mod utils;

pub(crate) static HTTP_CLIENT: LazyLock<Client> =
    LazyLock::new(|| Client::builder().http2_prior_knowledge().zstd(true).build().unwrap());
pub(crate) fn http_client() -> Client { HTTP_CLIENT.clone() }
