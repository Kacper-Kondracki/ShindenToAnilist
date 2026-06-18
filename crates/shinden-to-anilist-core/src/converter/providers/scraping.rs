use std::{
    fmt,
    num::NonZeroU32,
    sync::Arc,
};

use compact_str::{
    CompactString,
    ToCompactString,
};
use futures_util::{
    StreamExt,
    stream,
};
use governor::{
    DefaultDirectRateLimiter,
    Quota,
    RateLimiter,
};
use reqwest::{
    Client,
    Response,
    header::{
        CONTENT_TYPE,
        ORIGIN,
        REFERER,
        USER_AGENT,
    },
};
use scraper::{
    ElementRef,
    Html,
    Selector,
};

use crate::converter::common::AnimeId;

const DEFAULT_USER_AGENT: &str =
    "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/149.0.0.0 Safari/537.36";

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) struct ScrapeOptions {
    pub(crate) requests_per_second: u32,
    pub(crate) burst_size: u32,
    pub(crate) max_concurrent_detail_requests: usize,
    pub(crate) request_attempts: usize,
    pub(crate) fail_on_detail_error: bool,
}

impl Default for ScrapeOptions {
    fn default() -> Self {
        Self {
            requests_per_second: 20,
            burst_size: 20,
            max_concurrent_detail_requests: 8,
            request_attempts: 3,
            fail_on_detail_error: true,
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct ScrapeClient {
    client: Client,
    base_url: CompactString,
    limiter: Arc<DefaultDirectRateLimiter>,
    attempts: usize,
}

#[derive(Debug)]
pub(crate) struct RetryExhausted {
    pub(crate) path: CompactString,
    pub(crate) attempts: usize,
    pub(crate) source: reqwest::Error,
}

impl fmt::Display for RetryExhausted {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "request to {} failed after {} attempts",
            self.path, self.attempts
        )
    }
}

impl std::error::Error for RetryExhausted {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> { Some(&self.source) }
}

impl ScrapeClient {
    pub(crate) fn new(client: Client, base_url: impl Into<CompactString>, options: ScrapeOptions) -> Self {
        let requests_per_second = non_zero(options.requests_per_second);
        let burst_size = non_zero(options.burst_size);
        let quota = Quota::per_second(requests_per_second).allow_burst(burst_size);

        Self {
            client,
            base_url: trim_trailing_slash(base_url.into()),
            limiter: Arc::new(RateLimiter::direct(quota)),
            attempts: options.request_attempts.max(1),
        }
    }

    pub(crate) fn url(&self, path: &str) -> String { join_url(&self.base_url, path) }

    pub(crate) async fn get_html(&self, path: &str) -> Result<String, RetryExhausted> {
        self.request_html(path, |client, url| {
            client.get(url).header(USER_AGENT, DEFAULT_USER_AGENT)
        })
        .await
    }

    pub(crate) async fn post_form_html(
        &self,
        path: &str,
        referer_path: &str,
        form_body: String,
    ) -> Result<String, RetryExhausted> {
        let origin = self.base_url.to_string();
        let referer = self.url(referer_path);

        self.request_html(path, |client, url| {
            client
                .post(url)
                .header(USER_AGENT, DEFAULT_USER_AGENT)
                .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
                .header(ORIGIN, origin.clone())
                .header(REFERER, referer.clone())
                .header("sec-fetch-mode", "cors")
                .header("sec-fetch-site", "same-origin")
                .header("x-requested-with", "XMLHttpRequest")
                .body(form_body.clone())
        })
        .await
    }

    async fn request_html(
        &self,
        path: &str,
        request: impl Fn(&Client, String) -> reqwest::RequestBuilder,
    ) -> Result<String, RetryExhausted> {
        let path = path.to_compact_string();
        let mut last_error = None;

        for _ in 0..self.attempts {
            self.limiter.until_ready().await;
            let result = request(&self.client, self.url(&path))
                .send()
                .await
                .and_then(successful_response);

            match result {
                Ok(response) => match response.text().await {
                    Ok(text) => return Ok(text),
                    Err(error) => last_error = Some(error),
                },
                Err(error) => last_error = Some(error),
            }
        }

        Err(RetryExhausted {
            path,
            attempts: self.attempts,
            source: last_error.expect("at least one request attempt is always made"),
        })
    }
}

fn successful_response(response: Response) -> Result<Response, reqwest::Error> {
    if response.status().is_success() {
        Ok(response)
    } else {
        response.error_for_status()
    }
}

pub(crate) fn join_url(base_url: &str, path: &str) -> String {
    if path.starts_with("http://") || path.starts_with("https://") {
        return path.to_string();
    }

    format!(
        "{}/{}",
        base_url.trim_end_matches('/'),
        path.trim_start_matches('/')
    )
}

pub(crate) fn selector(value: &'static str) -> Selector {
    Selector::parse(value).expect("provider selector should be valid")
}

pub(crate) fn element_text(element: ElementRef<'_>) -> CompactString {
    element
        .text()
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
        .to_compact_string()
}

pub(crate) fn select_text(element: ElementRef<'_>, selector: &Selector) -> Option<CompactString> {
    element.select(selector).next().map(element_text)
}

pub(crate) fn attr(element: ElementRef<'_>, name: &str) -> Option<CompactString> {
    element.value().attr(name).map(|value| value.to_compact_string())
}

pub(crate) fn discover_page_count(document: &Html) -> usize {
    let selector = selector("ul.pagination a[href*=\"page=\"]");
    document
        .select(&selector)
        .filter_map(|element| attr(element, "href"))
        .filter_map(|href| query_page(&href))
        .max()
        .unwrap_or(1)
}

pub(crate) fn extract_mal_id(url: &str) -> Option<AnimeId> {
    let marker = "myanimelist.net/anime/";
    let start = url.find(marker)? + marker.len();
    let digits: String = url[start..].chars().take_while(|c| c.is_ascii_digit()).collect();

    (!digits.is_empty()).then(|| digits.parse().ok()).flatten()
}

pub(crate) trait ScrapedListProvider {
    type Section: Copy + Send + Sync;
    type ListItem: Send;
    type Detail: Send;
    type Error;

    fn section_path(username: &str, section: Self::Section, page: usize) -> String;
    fn detail_path(item: &Self::ListItem) -> String;
    fn parse_list(
        path: &str,
        section: Self::Section,
        html: &str,
    ) -> Result<(Vec<Self::ListItem>, usize), Self::Error>;
    fn parse_detail(path: &str, html: &str) -> Result<Self::Detail, Self::Error>;
}

pub(crate) async fn collect_scraped_list_items<P>(
    client: &ScrapeClient,
    username: &str,
    sections: Vec<P::Section>,
) -> Result<Vec<P::ListItem>, P::Error>
where
    P: ScrapedListProvider,
    P::Error: From<RetryExhausted>,
{
    let mut items = Vec::new();

    for section in sections {
        let first_path = P::section_path(username, section, 1);
        let first_html = client.get_html(&first_path).await?;
        let (mut page_items, page_count) = P::parse_list(&first_path, section, &first_html)?;
        items.append(&mut page_items);

        for page in 2..=page_count {
            let path = P::section_path(username, section, page);
            let html = client.get_html(&path).await?;
            let (mut page_items, _) = P::parse_list(&path, section, &html)?;
            items.append(&mut page_items);
        }
    }

    Ok(items)
}

pub(crate) fn fetch_scraped_details<P>(
    client: ScrapeClient,
    items: Vec<P::ListItem>,
    options: ScrapeOptions,
) -> impl futures_util::Stream<Item = Result<(P::ListItem, Option<P::Detail>), P::Error>> + Send
where
    P: ScrapedListProvider,
    P::ListItem: 'static,
    P::Detail: 'static,
    P::Error: From<RetryExhausted> + Send + 'static,
{
    stream::iter(items.into_iter().map(move |item| {
        let client = client.clone();
        async move {
            let path = P::detail_path(&item);
            let detail = match client.get_html(&path).await {
                Ok(html) => match P::parse_detail(&path, &html) {
                    Ok(detail) => Some(detail),
                    Err(error) if options.fail_on_detail_error => return Err(error),
                    Err(_) => None,
                },
                Err(error) if options.fail_on_detail_error => return Err(error.into()),
                Err(_) => None,
            };

            Ok((item, detail))
        }
    }))
    .buffer_unordered(options.max_concurrent_detail_requests)
}

fn query_page(url: &str) -> Option<usize> {
    let query = url.split_once('?')?.1;
    query.split('&').find_map(|part| {
        let (key, value) = part.split_once('=')?;
        (key == "page").then(|| value.parse().ok()).flatten()
    })
}

fn non_zero(value: u32) -> NonZeroU32 {
    NonZeroU32::new(value.max(1)).expect("value is clamped to be non-zero")
}

fn trim_trailing_slash(value: CompactString) -> CompactString {
    value.trim_end_matches('/').to_compact_string()
}
