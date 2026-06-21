/// App-owned HTTP clients for external services.
///
/// Keep provider-specific reqwest settings here so cookies, headers, timeouts,
/// and redirect behavior can be tuned without hunting through shell entrypoints
/// or provider orchestration code.
use std::sync::{
    Arc,
    RwLock,
};

use reqwest::{
    cookie::Jar,
    header::HeaderMap,
};
use tracing::info;

use crate::cloudflare::{
    AppliedShindenCloudflareClearance,
    ClearanceError,
    ShindenCloudflareClearance,
    shinden_client_from_cloudflare_clearance,
};

#[derive(Debug, Clone)]
pub struct AppHttpClients {
    shinden: ShindenHttpClient,
    pub(crate) animezone: reqwest::Client,
    pub(crate) ogladajanime: reqwest::Client,
    pub(crate) database: reqwest::Client,
}

const APP_USER_AGENT: &str = concat!("ShindenToAnilist/", env!("CARGO_PKG_VERSION"));
const BROWSER_USER_AGENT: &str =
    "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/149.0.0.0 Safari/537.36";

pub struct AppHttpClientBuilder {
    inner: reqwest::ClientBuilder,
}

#[derive(Debug, Clone)]
pub struct ShindenHttpClient {
    inner: Arc<RwLock<reqwest::Client>>,
}

impl ShindenHttpClient {
    pub fn new() -> Result<Self, reqwest::Error> {
        Ok(Self {
            inner: Arc::new(RwLock::new(shinden_client()?)),
        })
    }

    pub fn client(&self) -> reqwest::Client {
        self.inner
            .read()
            .expect("shinden HTTP client lock poisoned")
            .clone()
    }

    pub fn apply_cloudflare_clearance(
        &self,
        clearance: ShindenCloudflareClearance,
    ) -> Result<AppliedShindenCloudflareClearance, ClearanceError> {
        let clearance_client = shinden_client_from_cloudflare_clearance(clearance)?;
        *self.inner.write().expect("shinden HTTP client lock poisoned") = clearance_client.client;
        info!(
            accepted = clearance_client.applied.accepted,
            "applied Shinden Cloudflare clearance to HTTP client"
        );

        Ok(clearance_client.applied)
    }
}

impl Default for AppHttpClientBuilder {
    fn default() -> Self { Self::new() }
}

impl AppHttpClientBuilder {
    pub fn new() -> Self {
        Self {
            inner: reqwest::Client::builder(),
        }
    }

    pub fn with_cookie_store(self) -> Self {
        Self {
            inner: self.inner.cookie_store(true),
        }
    }

    pub fn with_app_user_agent(self) -> Self {
        Self {
            inner: self.inner.user_agent(APP_USER_AGENT),
        }
    }

    pub fn with_browser_user_agent(self) -> Self { self.with_user_agent(BROWSER_USER_AGENT) }

    pub fn with_user_agent(self, user_agent: impl Into<String>) -> Self {
        Self {
            inner: self.inner.user_agent(user_agent.into()),
        }
    }

    pub fn with_cookie_provider(self, jar: Arc<Jar>) -> Self {
        Self {
            inner: self.inner.cookie_provider(jar),
        }
    }

    pub fn with_default_headers(self, headers: HeaderMap) -> Self {
        Self {
            inner: self.inner.default_headers(headers),
        }
    }

    pub fn with_response_compression(self) -> Self {
        Self {
            inner: self.inner.gzip(true).brotli(true).deflate(true).zstd(true),
        }
    }

    pub fn build(self) -> Result<reqwest::Client, reqwest::Error> { self.inner.build() }
}

impl AppHttpClients {
    pub fn new() -> Result<Self, reqwest::Error> {
        Ok(Self {
            shinden: ShindenHttpClient::new()?,
            animezone: animezone_client()?,
            ogladajanime: ogladajanime_client()?,
            database: database_client()?,
        })
    }

    pub(crate) fn shinden(&self) -> reqwest::Client { self.shinden.client() }

    pub fn apply_shinden_cloudflare_clearance(
        &self,
        clearance: ShindenCloudflareClearance,
    ) -> Result<AppliedShindenCloudflareClearance, ClearanceError> {
        self.shinden.apply_cloudflare_clearance(clearance)
    }
}

pub fn shinden_client() -> Result<reqwest::Client, reqwest::Error> { browser_provider_client() }

pub fn animezone_client() -> Result<reqwest::Client, reqwest::Error> { browser_provider_client() }

pub fn ogladajanime_client() -> Result<reqwest::Client, reqwest::Error> { browser_provider_client() }

pub fn database_client() -> Result<reqwest::Client, reqwest::Error> {
    AppHttpClientBuilder::new()
        .with_cookie_store()
        .with_app_user_agent()
        .with_response_compression()
        .build()
}

fn browser_provider_client() -> Result<reqwest::Client, reqwest::Error> {
    AppHttpClientBuilder::new()
        .with_cookie_store()
        .with_browser_user_agent()
        .with_response_compression()
        .build()
}
