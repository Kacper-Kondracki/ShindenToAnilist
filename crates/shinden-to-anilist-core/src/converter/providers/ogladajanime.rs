use std::io;

use bon::Builder;
use futures_util::{
    Stream,
    StreamExt,
    stream::{
        self,
        BoxStream,
    },
};
use reqwest::Client;
use thiserror::Error;
use tracing::{
    debug,
    warn,
};

pub use self::models::*;
use self::{
    detail_page::{
        parse_detail_page,
        parse_tooltip_page,
    },
    list_page::parse_list_page,
};
use super::scraping::{
    RetryExhausted,
    ScrapeClient,
    ScrapeOptions,
    ScrapedListProvider,
    collect_scraped_list_items,
};
use crate::converter::common::AnimeId;

mod detail_page;
mod list_page;
pub mod models;

const DEFAULT_BASE_URL: &str = "https://ogladajanime.pl";
const TOOLTIP_PATH: &str = "/manager.php?action=get_anime_tooltip";

pub trait OgladajAnimeListLoad {
    fn get_from_ogladajanime(
        client: Client,
        user_id: impl Into<String> + Send,
    ) -> impl Future<Output = Result<OgladajAnimeList, OgladajAnimeError>> + Send;

    fn get_from_ogladajanime_with_options(
        client: Client,
        user_id: impl Into<String> + Send,
        options: OgladajAnimeFetchOptions,
    ) -> impl Future<Output = Result<OgladajAnimeList, OgladajAnimeError>> + Send;

    fn stream_from_ogladajanime(client: Client, user_id: impl Into<String> + Send)
    -> OgladajAnimeFetchStream;

    fn stream_from_ogladajanime_with_options(
        client: Client,
        user_id: impl Into<String> + Send,
        options: OgladajAnimeFetchOptions,
    ) -> OgladajAnimeFetchStream;
}

#[derive(Builder, Debug, Clone)]
#[builder(start_fn = options)]
pub struct OgladajAnimeFetchOptions {
    #[builder(default = DEFAULT_BASE_URL.to_string())]
    base_url: String,
    #[builder(default = 3)]
    requests_per_second: u32,
    #[builder(default = 1)]
    burst_size: u32,
    #[builder(default = 1)]
    max_concurrent_detail_requests: usize,
    #[builder(default = 2)]
    request_attempts: usize,
    #[builder(default = true)]
    fail_on_detail_error: bool,
}

impl Default for OgladajAnimeFetchOptions {
    fn default() -> Self { Self::options().build() }
}

impl OgladajAnimeFetchOptions {
    pub fn new() -> Self { Self::default() }

    pub fn base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = base_url.into();
        self
    }

    pub fn requests_per_second(mut self, requests_per_second: u32) -> Self {
        self.requests_per_second = requests_per_second;
        self
    }

    pub fn burst_size(mut self, burst_size: u32) -> Self {
        self.burst_size = burst_size;
        self
    }

    pub fn max_concurrent_detail_requests(mut self, max_concurrent_detail_requests: usize) -> Self {
        self.max_concurrent_detail_requests = max_concurrent_detail_requests.max(1);
        self
    }

    pub fn request_attempts(mut self, request_attempts: usize) -> Self {
        self.request_attempts = request_attempts.max(1);
        self
    }

    pub fn fail_on_detail_error(mut self, fail_on_detail_error: bool) -> Self {
        self.fail_on_detail_error = fail_on_detail_error;
        self
    }

    fn scrape_options(&self) -> ScrapeOptions {
        ScrapeOptions {
            requests_per_second: self.requests_per_second,
            burst_size: self.burst_size,
            max_concurrent_detail_requests: self.max_concurrent_detail_requests.max(1),
            request_attempts: self.request_attempts.max(1),
            fail_on_detail_error: self.fail_on_detail_error,
        }
    }
}

pub type OgladajAnimeFetchStream = BoxStream<'static, Result<OgladajAnimeFetchEvent, OgladajAnimeError>>;

#[derive(Debug, Clone, PartialEq)]
pub enum OgladajAnimeFetchEvent {
    Started {
        total_entries: usize,
    },
    Entry {
        current: usize,
        total_entries: usize,
        entry: OgladajAnimeEntry,
    },
}

impl OgladajAnimeFetchEvent {
    pub fn total_entries(&self) -> usize {
        match self {
            Self::Started { total_entries } | Self::Entry { total_entries, .. } => *total_entries,
        }
    }

    pub fn current(&self) -> usize {
        match self {
            Self::Started { .. } => 0,
            Self::Entry { current, .. } => *current,
        }
    }

    pub fn entry(&self) -> Option<&OgladajAnimeEntry> {
        match self {
            Self::Started { .. } => None,
            Self::Entry { entry, .. } => Some(entry),
        }
    }
}

pub trait OgladajAnimeFetchStreamExt:
    Stream<Item = Result<OgladajAnimeFetchEvent, OgladajAnimeError>> + Sized
{
    fn collect_ogladajanime_list(
        self,
    ) -> impl Future<Output = Result<OgladajAnimeList, OgladajAnimeError>> + Send
    where
        Self: Send,
    {
        async move {
            let stream = self;
            futures_util::pin_mut!(stream);

            let mut entries = Vec::new();
            while let Some(event) = stream.next().await {
                if let OgladajAnimeFetchEvent::Entry { entry, .. } = event? {
                    entries.push(entry);
                }
            }

            Ok(OgladajAnimeList::from_entries(entries))
        }
    }
}

impl<T> OgladajAnimeFetchStreamExt for T where
    T: Stream<Item = Result<OgladajAnimeFetchEvent, OgladajAnimeError>> + Sized
{
}

#[derive(Error, Debug)]
#[error(transparent)]
pub enum OgladajAnimeError {
    Io(#[from] io::Error),
    Request(#[from] reqwest::Error),
    #[error("ogladajanime request to {path} failed after {attempts} attempts")]
    RetryExhausted {
        path: String,
        attempts: usize,
        #[source]
        source: reqwest::Error,
    },
    #[error("ogladajanime parse error at {path}: {message}")]
    Parse {
        path: String,
        message: String,
    },
}

impl From<RetryExhausted> for OgladajAnimeError {
    fn from(error: RetryExhausted) -> Self {
        Self::RetryExhausted {
            path: error.path.into(),
            attempts: error.attempts,
            source: error.source,
        }
    }
}

fn fetch_ogladajanime_details(
    client: ScrapeClient,
    user_id: String,
    items: Vec<OgladajAnimeListItem>,
    options: ScrapeOptions,
) -> impl Stream<Item = Result<(OgladajAnimeListItem, Option<OgladajAnimeDetail>), OgladajAnimeError>> + Send
{
    stream::iter(items.into_iter().map(move |item| {
        let client = client.clone();
        let user_id = user_id.clone();

        async move { fetch_ogladajanime_detail(client, user_id, item, options).await }
    }))
    .buffer_unordered(options.max_concurrent_detail_requests)
}

async fn fetch_ogladajanime_detail(
    client: ScrapeClient,
    user_id: String,
    item: OgladajAnimeListItem,
    options: ScrapeOptions,
) -> Result<(OgladajAnimeListItem, Option<OgladajAnimeDetail>), OgladajAnimeError> {
    let path = OgladajAnimeProvider::detail_path(&item);

    match client.get_html(&path).await {
        Ok(html) => match OgladajAnimeProvider::parse_detail(&path, &html) {
            Ok(detail) => Ok((item, Some(detail))),
            Err(error) => match fetch_tooltip_detail(&client, &user_id, item.id).await {
                Ok(detail) => {
                    warn!(
                        %path,
                        anime_id = item.id,
                        error = %error,
                        "ogladajanime detail page parse failed; tooltip fallback succeeded"
                    );
                    Ok((item, Some(detail)))
                },
                Err(tooltip_error) if options.fail_on_detail_error => {
                    warn!(
                        %path,
                        anime_id = item.id,
                        error = %error,
                        tooltip_error = %tooltip_error,
                        "ogladajanime detail page parse failed and tooltip fallback failed"
                    );
                    Err(error)
                },
                Err(tooltip_error) => {
                    warn!(
                        %path,
                        anime_id = item.id,
                        error = %error,
                        tooltip_error = %tooltip_error,
                        "ogladajanime detail page parse failed and tooltip fallback failed; continuing without detail"
                    );
                    Ok((item, None))
                },
            },
        },
        Err(error) => match fetch_tooltip_detail(&client, &user_id, item.id).await {
            Ok(detail) => {
                debug!(
                    %path,
                    anime_id = item.id,
                    error = %error,
                    "ogladajanime detail page request failed; tooltip fallback succeeded"
                );
                Ok((item, Some(detail)))
            },
            Err(tooltip_error) if options.fail_on_detail_error => {
                warn!(
                    %path,
                    anime_id = item.id,
                    error = %error,
                    tooltip_error = %tooltip_error,
                    "ogladajanime detail page request failed and tooltip fallback failed"
                );
                Err(error.into())
            },
            Err(tooltip_error) => {
                warn!(
                    %path,
                    anime_id = item.id,
                    error = %error,
                    tooltip_error = %tooltip_error,
                    "ogladajanime detail page request failed and tooltip fallback failed; continuing without detail"
                );
                Ok((item, None))
            },
        },
    }
}

async fn fetch_tooltip_detail(
    client: &ScrapeClient,
    user_id: &str,
    anime_id: AnimeId,
) -> Result<OgladajAnimeDetail, OgladajAnimeError> {
    let referer = format!("/anime_list/{user_id}");
    let form = format!("id={anime_id}");
    let html = client.post_form_html(TOOLTIP_PATH, &referer, form).await?;

    parse_tooltip_page(TOOLTIP_PATH, &html)
        .inspect_err(|error| warn!(anime_id, error = %error, "ogladajanime tooltip parse failed"))
}

struct OgladajAnimeProvider;

impl ScrapedListProvider for OgladajAnimeProvider {
    type Section = ();
    type ListItem = OgladajAnimeListItem;
    type Detail = OgladajAnimeDetail;
    type Error = OgladajAnimeError;

    fn section_path(user_id: &str, _section: Self::Section, _page: usize) -> String {
        format!("/anime_list/{user_id}")
    }

    fn detail_path(item: &Self::ListItem) -> String { format!("/anime/{}", item.slug) }

    fn parse_list(
        path: &str,
        _section: Self::Section,
        html: &str,
    ) -> Result<(Vec<Self::ListItem>, usize), Self::Error> {
        parse_list_page(path, html)
    }

    fn parse_detail(path: &str, html: &str) -> Result<Self::Detail, Self::Error> {
        parse_detail_page(path, html)
    }
}

impl OgladajAnimeListLoad for OgladajAnimeList {
    async fn get_from_ogladajanime(
        client: Client,
        user_id: impl Into<String> + Send,
    ) -> Result<OgladajAnimeList, OgladajAnimeError> {
        Self::get_from_ogladajanime_with_options(client, user_id, OgladajAnimeFetchOptions::default()).await
    }

    async fn get_from_ogladajanime_with_options(
        client: Client,
        user_id: impl Into<String> + Send,
        options: OgladajAnimeFetchOptions,
    ) -> Result<OgladajAnimeList, OgladajAnimeError> {
        Self::stream_from_ogladajanime_with_options(client, user_id, options)
            .collect_ogladajanime_list()
            .await
    }

    fn stream_from_ogladajanime(
        client: Client,
        user_id: impl Into<String> + Send,
    ) -> OgladajAnimeFetchStream {
        Self::stream_from_ogladajanime_with_options(client, user_id, OgladajAnimeFetchOptions::default())
    }

    fn stream_from_ogladajanime_with_options(
        client: Client,
        user_id: impl Into<String> + Send,
        options: OgladajAnimeFetchOptions,
    ) -> OgladajAnimeFetchStream {
        let user_id = user_id.into();
        let scrape = options.scrape_options();
        let scrape_client = ScrapeClient::new(client, options.base_url, scrape);
        let list_user_id = user_id.clone();

        stream::once(async move {
            let items =
                collect_scraped_list_items::<OgladajAnimeProvider>(&scrape_client, &list_user_id, vec![()])
                    .await?;
            Ok((scrape_client, items, scrape))
        })
        .flat_map(
            move |result: Result<_, OgladajAnimeError>| -> OgladajAnimeFetchStream {
                match result {
                    Ok((scrape_client, items, scrape)) => {
                        let total_entries = items.len();
                        let started =
                            stream::once(
                                async move { Ok(OgladajAnimeFetchEvent::Started { total_entries }) },
                            );
                        let entries =
                            fetch_ogladajanime_details(scrape_client, user_id.clone(), items, scrape)
                                .enumerate()
                                .map(move |(index, result)| {
                                    result.map(|(item, detail)| OgladajAnimeFetchEvent::Entry {
                                        current: index + 1,
                                        total_entries,
                                        entry: OgladajAnimeEntry::from_scraped(item, detail),
                                    })
                                });

                        started.chain(entries).boxed()
                    },
                    Err(error) => stream::once(async move { Err(error) }).boxed(),
                }
            },
        )
        .boxed()
    }
}
