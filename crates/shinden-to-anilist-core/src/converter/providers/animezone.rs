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

pub use self::models::*;
use self::{
    detail_page::parse_detail_page,
    list_page::parse_list_page,
};
use super::scraping::{
    RetryExhausted,
    ScrapeClient,
    ScrapeOptions,
    ScrapedListProvider,
    collect_scraped_list_items,
    fetch_scraped_details,
};

mod detail_page;
mod list_page;
pub mod models;

const DEFAULT_BASE_URL: &str = "https://www.animezone.pl";

pub trait AnimeZoneListLoad {
    fn get_from_animezone(
        client: Client,
        username: impl Into<String> + Send,
    ) -> impl Future<Output = Result<AnimeZoneList, AnimeZoneError>> + Send;

    fn get_from_animezone_with_options(
        client: Client,
        username: impl Into<String> + Send,
        options: AnimeZoneFetchOptions,
    ) -> impl Future<Output = Result<AnimeZoneList, AnimeZoneError>> + Send;

    fn stream_from_animezone(client: Client, username: impl Into<String> + Send) -> AnimeZoneFetchStream;

    fn stream_from_animezone_with_options(
        client: Client,
        username: impl Into<String> + Send,
        options: AnimeZoneFetchOptions,
    ) -> AnimeZoneFetchStream;
}

#[derive(Builder, Debug, Clone)]
#[builder(start_fn = options)]
pub struct AnimeZoneFetchOptions {
    #[builder(default = DEFAULT_BASE_URL.to_string())]
    base_url: String,
    #[builder(default = AnimeZoneSection::default_sections().to_vec())]
    sections: Vec<AnimeZoneSection>,
    #[builder(default = 160)]
    requests_per_second: u32,
    #[builder(default = 20)]
    burst_size: u32,
    #[builder(default = 20)]
    max_concurrent_detail_requests: usize,
    #[builder(default = 3)]
    request_attempts: usize,
    #[builder(default = true)]
    fail_on_detail_error: bool,
}

impl Default for AnimeZoneFetchOptions {
    fn default() -> Self { Self::options().build() }
}

impl AnimeZoneFetchOptions {
    pub fn new() -> Self { Self::default() }

    pub fn base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = base_url.into();
        self
    }

    pub fn sections(mut self, sections: impl Into<Vec<AnimeZoneSection>>) -> Self {
        self.sections = sections.into();
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

pub type AnimeZoneFetchStream = BoxStream<'static, Result<AnimeZoneFetchEvent, AnimeZoneError>>;

#[derive(Debug, Clone, PartialEq)]
pub enum AnimeZoneFetchEvent {
    Started {
        total_entries: usize,
    },
    Entry {
        current: usize,
        total_entries: usize,
        entry: AnimeZoneEntry,
    },
}

impl AnimeZoneFetchEvent {
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

    pub fn entry(&self) -> Option<&AnimeZoneEntry> {
        match self {
            Self::Started { .. } => None,
            Self::Entry { entry, .. } => Some(entry),
        }
    }
}

pub trait AnimeZoneFetchStreamExt:
    Stream<Item = Result<AnimeZoneFetchEvent, AnimeZoneError>> + Sized
{
    fn collect_animezone_list(self) -> impl Future<Output = Result<AnimeZoneList, AnimeZoneError>> + Send
    where
        Self: Send,
    {
        async move {
            let stream = self;
            futures_util::pin_mut!(stream);

            let mut entries = Vec::new();
            while let Some(event) = stream.next().await {
                if let AnimeZoneFetchEvent::Entry { entry, .. } = event? {
                    entries.push(entry);
                }
            }

            Ok(AnimeZoneList::from_entries(entries))
        }
    }
}

impl<T> AnimeZoneFetchStreamExt for T where
    T: Stream<Item = Result<AnimeZoneFetchEvent, AnimeZoneError>> + Sized
{
}

#[derive(Error, Debug)]
#[error(transparent)]
pub enum AnimeZoneError {
    Io(#[from] io::Error),
    Request(#[from] reqwest::Error),
    #[error("animezone request to {path} failed after {attempts} attempts")]
    RetryExhausted {
        path: String,
        attempts: usize,
        #[source]
        source: reqwest::Error,
    },
    #[error("animezone parse error at {path}: {message}")]
    Parse {
        path: String,
        message: String,
    },
}

impl From<RetryExhausted> for AnimeZoneError {
    fn from(error: RetryExhausted) -> Self {
        Self::RetryExhausted {
            path: error.path.into(),
            attempts: error.attempts,
            source: error.source,
        }
    }
}

struct AnimeZoneProvider;

impl ScrapedListProvider for AnimeZoneProvider {
    type Section = AnimeZoneSection;
    type ListItem = AnimeZoneListItem;
    type Detail = AnimeZoneDetail;
    type Error = AnimeZoneError;

    fn section_path(username: &str, section: Self::Section, page: usize) -> String {
        match page {
            1 => format!("/user/{}/{}", username, section.path_segment()),
            page => format!("/user/{}/{}?page={}", username, section.path_segment(), page),
        }
    }

    fn detail_path(item: &Self::ListItem) -> String { format!("/anime/{}", item.slug) }

    fn parse_list(
        path: &str,
        section: Self::Section,
        html: &str,
    ) -> Result<(Vec<Self::ListItem>, usize), Self::Error> {
        parse_list_page(path, section, html)
    }

    fn parse_detail(path: &str, html: &str) -> Result<Self::Detail, Self::Error> {
        parse_detail_page(path, html)
    }
}

impl AnimeZoneListLoad for AnimeZoneList {
    async fn get_from_animezone(
        client: Client,
        username: impl Into<String> + Send,
    ) -> Result<AnimeZoneList, AnimeZoneError> {
        Self::get_from_animezone_with_options(client, username, AnimeZoneFetchOptions::default()).await
    }

    async fn get_from_animezone_with_options(
        client: Client,
        username: impl Into<String> + Send,
        options: AnimeZoneFetchOptions,
    ) -> Result<AnimeZoneList, AnimeZoneError> {
        Self::stream_from_animezone_with_options(client, username, options)
            .collect_animezone_list()
            .await
    }

    fn stream_from_animezone(client: Client, username: impl Into<String> + Send) -> AnimeZoneFetchStream {
        Self::stream_from_animezone_with_options(client, username, AnimeZoneFetchOptions::default())
    }

    fn stream_from_animezone_with_options(
        client: Client,
        username: impl Into<String> + Send,
        options: AnimeZoneFetchOptions,
    ) -> AnimeZoneFetchStream {
        let username = username.into();
        let scrape = options.scrape_options();
        let scrape_client = ScrapeClient::new(client, options.base_url, scrape);

        stream::once(async move {
            let items =
                collect_scraped_list_items::<AnimeZoneProvider>(&scrape_client, &username, options.sections)
                    .await?;
            Ok((scrape_client, items, scrape))
        })
        .flat_map(|result: Result<_, AnimeZoneError>| -> AnimeZoneFetchStream {
            match result {
                Ok((scrape_client, items, scrape)) => {
                    let total_entries = items.len();
                    let started =
                        stream::once(async move { Ok(AnimeZoneFetchEvent::Started { total_entries }) });
                    let entries = fetch_scraped_details::<AnimeZoneProvider>(scrape_client, items, scrape)
                        .enumerate()
                        .map(move |(index, result)| {
                            result.map(|(item, detail)| AnimeZoneFetchEvent::Entry {
                                current: index + 1,
                                total_entries,
                                entry: AnimeZoneEntry::from_scraped(item, detail),
                            })
                        });

                    started.chain(entries).boxed()
                },
                Err(error) => stream::once(async move { Err(error) }).boxed(),
            }
        })
        .boxed()
    }
}
