use std::{
    fmt,
    sync::Arc,
    time::{
        SystemTime,
        UNIX_EPOCH,
    },
};

use reqwest::{
    Url,
    cookie::{
        CookieStore,
        Jar,
    },
};
use tracing::debug;

use crate::http::AppHttpClientBuilder;

pub const SHINDEN_ORIGIN: &str = "https://lista.shinden.pl";
pub const SHINDEN_COOKIE_URL: &str = "https://lista.shinden.pl/";
const SHINDEN_HOST: &str = "lista.shinden.pl";
const SHINDEN_PARENT_DOMAIN: &str = "shinden.pl";
const CF_CLEARANCE_COOKIE: &str = "cf_clearance";
const MAX_COOKIE_BYTES: usize = 4096;
const MAX_USER_AGENT_BYTES: usize = 1024;

#[derive(Debug, Clone)]
pub struct ShindenCloudflareClearance {
    pub user_agent: String,
    pub cf_clearance: String,
    pub domain: String,
    pub path: String,
    pub expires_unix_seconds: Option<f64>,
    pub captured_at_ms: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AppliedShindenCloudflareClearance {
    pub accepted: bool,
}

#[derive(Debug)]
pub struct ClearanceClient {
    pub client: reqwest::Client,
    pub applied: AppliedShindenCloudflareClearance,
}

#[derive(Debug)]
pub enum ClearanceError {
    EmptyUserAgent,
    UserAgentTooLong,
    EmptyClearance,
    ClearanceTooLong,
    InvalidCookieValue,
    InvalidDomain,
    InvalidPath,
    Expired,
    InvalidCookieUrl,
    Client(reqwest::Error),
}

impl fmt::Display for ClearanceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyUserAgent => formatter.write_str("missing Cloudflare browser user agent"),
            Self::UserAgentTooLong => formatter.write_str("Cloudflare browser user agent is too long"),
            Self::EmptyClearance => formatter.write_str("missing cf_clearance cookie"),
            Self::ClearanceTooLong => formatter.write_str("cf_clearance cookie is too long"),
            Self::InvalidCookieValue => {
                formatter.write_str("cf_clearance cookie contains invalid characters")
            },
            Self::InvalidDomain => formatter.write_str("cf_clearance cookie domain is not a Shinden domain"),
            Self::InvalidPath => formatter.write_str("cf_clearance cookie path is invalid"),
            Self::Expired => formatter.write_str("cf_clearance cookie is expired"),
            Self::InvalidCookieUrl => formatter.write_str("internal Shinden cookie URL is invalid"),
            Self::Client(error) => write!(formatter, "failed to build Shinden HTTP client: {error}"),
        }
    }
}

impl std::error::Error for ClearanceError {}

impl From<reqwest::Error> for ClearanceError {
    fn from(value: reqwest::Error) -> Self { Self::Client(value) }
}

pub fn shinden_client_from_cloudflare_clearance(
    clearance: ShindenCloudflareClearance,
) -> Result<ClearanceClient, ClearanceError> {
    let clearance = validate_clearance(clearance)?;
    let jar = Arc::new(Jar::default());
    let cookie_url = Url::parse(SHINDEN_COOKIE_URL).map_err(|_| ClearanceError::InvalidCookieUrl)?;
    let cookie = set_cookie_header(&clearance);
    jar.add_cookie_str(&cookie, &cookie_url);

    let accepted = jar
        .cookies(&cookie_url)
        .and_then(|header| header.to_str().ok().map(str::to_owned))
        .is_some_and(|header| {
            header
                .split(';')
                .any(|pair| pair.trim_start().starts_with("cf_clearance="))
        });

    debug!(
        accepted,
        user_agent_len = clearance.user_agent.len(),
        cookie_len = clearance.cf_clearance.len(),
        domain = %clearance.domain,
        path = %clearance.path,
        captured_at_ms = clearance.captured_at_ms,
        clearance = %redacted_clearance_hint(&clearance.cf_clearance),
        "validated Shinden Cloudflare clearance"
    );

    let client = AppHttpClientBuilder::new()
        .with_cookie_provider(jar)
        .with_user_agent(clearance.user_agent)
        .with_response_compression()
        .build()?;

    Ok(ClearanceClient {
        client,
        applied: AppliedShindenCloudflareClearance { accepted },
    })
}

pub fn redacted_clearance_hint(value: &str) -> String { format!("<redacted:{} bytes>", value.len()) }

fn validate_clearance(
    mut clearance: ShindenCloudflareClearance,
) -> Result<ShindenCloudflareClearance, ClearanceError> {
    clearance.user_agent = clearance.user_agent.trim().to_owned();
    clearance.cf_clearance = clearance.cf_clearance.trim().to_owned();
    clearance.domain = normalize_domain(&clearance.domain);
    clearance.path = normalize_path(&clearance.path)?;

    if clearance.user_agent.is_empty() {
        return Err(ClearanceError::EmptyUserAgent);
    }
    if clearance.user_agent.len() > MAX_USER_AGENT_BYTES {
        return Err(ClearanceError::UserAgentTooLong);
    }
    if clearance.cf_clearance.is_empty() {
        return Err(ClearanceError::EmptyClearance);
    }
    if clearance.cf_clearance.len() > MAX_COOKIE_BYTES {
        return Err(ClearanceError::ClearanceTooLong);
    }
    if has_cookie_separator(&clearance.cf_clearance) {
        return Err(ClearanceError::InvalidCookieValue);
    }
    if !is_shinden_domain(&clearance.domain) {
        return Err(ClearanceError::InvalidDomain);
    }
    if clearance
        .expires_unix_seconds
        .is_some_and(|expires| expires.is_finite() && expires <= unix_timestamp_seconds())
    {
        return Err(ClearanceError::Expired);
    }

    Ok(clearance)
}

fn set_cookie_header(clearance: &ShindenCloudflareClearance) -> String {
    let mut cookie = format!("{CF_CLEARANCE_COOKIE}={}", clearance.cf_clearance);
    cookie.push_str("; Domain=");
    cookie.push_str(&clearance.domain);
    cookie.push_str("; Path=");
    cookie.push_str(&clearance.path);
    cookie.push_str("; Secure; HttpOnly");

    if let Some(expires) = clearance.expires_unix_seconds
        && expires.is_finite()
        && expires > unix_timestamp_seconds()
    {
        cookie.push_str("; Max-Age=");
        cookie.push_str(&(expires - unix_timestamp_seconds()).floor().to_string());
    }

    cookie
}

fn normalize_domain(domain: &str) -> String {
    let domain = domain.trim().trim_start_matches('.').to_ascii_lowercase();
    if domain.is_empty() {
        SHINDEN_HOST.to_owned()
    } else {
        domain
    }
}

fn normalize_path(path: &str) -> Result<String, ClearanceError> {
    let path = path.trim();
    if path.is_empty() {
        return Ok("/".to_owned());
    }
    if has_cookie_separator(path) {
        return Err(ClearanceError::InvalidPath);
    }
    if path.starts_with('/') {
        Ok(path.to_owned())
    } else {
        Err(ClearanceError::InvalidPath)
    }
}

fn is_shinden_domain(domain: &str) -> bool {
    domain == SHINDEN_HOST || domain == SHINDEN_PARENT_DOMAIN || domain.ends_with(".shinden.pl")
}

fn has_cookie_separator(value: &str) -> bool { value.chars().any(|ch| matches!(ch, '\r' | '\n' | ';')) }

fn unix_timestamp_seconds() -> f64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs_f64())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_clearance() -> ShindenCloudflareClearance {
        ShindenCloudflareClearance {
            user_agent: "Mozilla/5.0 Test".to_owned(),
            cf_clearance: "clearance-value".to_owned(),
            domain: ".lista.shinden.pl".to_owned(),
            path: "/".to_owned(),
            expires_unix_seconds: Some(unix_timestamp_seconds() + 3600.0),
            captured_at_ms: 123,
        }
    }

    #[test]
    fn accepts_valid_clearance_and_builds_cookie_jar() {
        let client = shinden_client_from_cloudflare_clearance(valid_clearance())
            .expect("valid clearance should build client");

        assert!(client.applied.accepted);
    }

    #[test]
    fn rejects_non_shinden_domain() {
        let mut clearance = valid_clearance();
        clearance.domain = "example.com".to_owned();

        assert!(matches!(
            shinden_client_from_cloudflare_clearance(clearance).expect_err("domain should be rejected"),
            ClearanceError::InvalidDomain
        ));
    }

    #[test]
    fn rejects_expired_cookie() {
        let mut clearance = valid_clearance();
        clearance.expires_unix_seconds = Some(unix_timestamp_seconds() - 1.0);

        assert!(matches!(
            shinden_client_from_cloudflare_clearance(clearance).expect_err("cookie should be expired"),
            ClearanceError::Expired
        ));
    }

    #[test]
    fn rejects_cookie_separators() {
        let mut clearance = valid_clearance();
        clearance.cf_clearance = "value; injected=true".to_owned();

        assert!(matches!(
            shinden_client_from_cloudflare_clearance(clearance).expect_err("cookie should be invalid"),
            ClearanceError::InvalidCookieValue
        ));
    }

    #[test]
    fn redacted_hint_contains_only_length() {
        assert_eq!(redacted_clearance_hint("secret"), "<redacted:6 bytes>");
    }
}
