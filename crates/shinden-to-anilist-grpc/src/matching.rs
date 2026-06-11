use shinden_to_anilist_core::{
    common::MatchView,
    extractor::{
        TitleMetadata,
        title_processor,
    },
    searcher::{
        Search,
        SearchMode,
    },
    utils::normalize_str,
};

use crate::pb::SearchOptions;

#[derive(Debug)]
pub(crate) struct QueryMatchView {
    title: String,
    normalized_title: String,
    metadata: TitleMetadata,
}

impl QueryMatchView {
    pub(crate) fn new(query: String) -> Self {
        Self {
            normalized_title: normalize_str(&query).to_string(),
            metadata: title_processor::process(&query),
            title: query,
        }
    }

    pub(crate) fn normalized_title(&self) -> &str { &self.normalized_title }
}

impl MatchView for QueryMatchView {
    fn title(&self) -> &str { &self.title }
    fn normalized_title(&self) -> &str { &self.normalized_title }
    fn title_metadata(&self) -> Option<&TitleMetadata> { Some(&self.metadata) }
}

pub(crate) fn search_options(options: Option<SearchOptions>, mode: SearchMode) -> Search {
    let mut search = Search {
        mode,
        ..Search::options().build()
    };

    if let Some(options) = options {
        if options.limit > 0 {
            search.limit = options.limit as usize;
        }
        if let Some(threshold) = options.threshold {
            search.threshold = threshold;
        }
    }

    search
}
