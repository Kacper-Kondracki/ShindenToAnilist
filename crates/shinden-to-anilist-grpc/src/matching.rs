use shinden_to_anilist_core::{
    NaiveDate,
    common::MatchView,
    database::{
        AnimeStatus,
        AnimeType,
    },
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

pub(crate) struct FuzzyMatchView<'a> {
    query: QueryMatchView,
    source_entry: Option<&'a dyn MatchView>,
}

impl<'a> FuzzyMatchView<'a> {
    pub(crate) fn new(query: String, source_entry: Option<&'a dyn MatchView>) -> Self {
        Self {
            query: QueryMatchView::new(query),
            source_entry,
        }
    }

    pub(crate) fn normalized_title(&self) -> &str { self.query.normalized_title() }
}

impl MatchView for FuzzyMatchView<'_> {
    fn title(&self) -> &str { self.query.title() }
    fn normalized_title(&self) -> &str { self.query.normalized_title() }
    fn title_metadata(&self) -> Option<&TitleMetadata> { self.query.title_metadata() }
    fn date(&self) -> Option<Option<NaiveDate>> { self.source_entry.and_then(MatchView::date) }
    fn year(&self) -> Option<Option<i32>> { self.source_entry.and_then(MatchView::year) }
    fn anime_type(&self) -> Option<AnimeType> { self.source_entry.and_then(MatchView::anime_type) }
    fn status(&self) -> Option<AnimeStatus> { self.source_entry.and_then(MatchView::status) }
    fn episodes(&self) -> Option<i32> { self.source_entry.and_then(MatchView::episodes) }
}

pub(crate) fn search_options(options: Option<SearchOptions>, mode: SearchMode) -> Search {
    let mut search = Search {
        mode,
        ..Search::options().threshold(0.5).build()
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
