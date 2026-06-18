use std::io;

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
    fetch_scraped_list,
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
}

#[derive(Debug, Clone)]
pub struct AnimeZoneFetchOptions {
    base_url: String,
    sections: Vec<AnimeZoneSection>,
    scrape: ScrapeOptions,
}

impl Default for AnimeZoneFetchOptions {
    fn default() -> Self {
        Self {
            base_url: DEFAULT_BASE_URL.to_string(),
            sections: AnimeZoneSection::default_sections().to_vec(),
            scrape: ScrapeOptions::default(),
        }
    }
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
        self.scrape.requests_per_second = requests_per_second;
        self
    }

    pub fn burst_size(mut self, burst_size: u32) -> Self {
        self.scrape.burst_size = burst_size;
        self
    }

    pub fn max_concurrent_detail_requests(mut self, max_concurrent_detail_requests: usize) -> Self {
        self.scrape.max_concurrent_detail_requests = max_concurrent_detail_requests.max(1);
        self
    }

    pub fn request_attempts(mut self, request_attempts: usize) -> Self {
        self.scrape.request_attempts = request_attempts.max(1);
        self
    }

    pub fn fail_on_detail_error(mut self, fail_on_detail_error: bool) -> Self {
        self.scrape.fail_on_detail_error = fail_on_detail_error;
        self
    }
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
    type Output = AnimeZoneList;
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

    fn build(items: Vec<(Self::ListItem, Option<Self::Detail>)>) -> Self::Output {
        AnimeZoneList::from_scraped(items)
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
        let scrape = options.scrape;
        let scrape_client = ScrapeClient::new(client, options.base_url, scrape);
        fetch_scraped_list::<AnimeZoneProvider>(scrape_client, username.into(), options.sections, scrape)
            .await
    }
}
