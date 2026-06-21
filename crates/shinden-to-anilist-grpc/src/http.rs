/// App-owned HTTP clients for external services.
///
/// Keep provider-specific reqwest settings here so cookies, headers, timeouts,
/// and redirect behavior can be tuned without hunting through shell entrypoints
/// or provider orchestration code.
#[derive(Debug, Clone)]
pub struct AppHttpClients {
    pub(crate) shinden: reqwest::Client,
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

    pub fn with_browser_user_agent(self) -> Self {
        Self {
            inner: self.inner.user_agent(BROWSER_USER_AGENT),
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
            shinden: shinden_client()?,
            animezone: animezone_client()?,
            ogladajanime: ogladajanime_client()?,
            database: database_client()?,
        })
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
