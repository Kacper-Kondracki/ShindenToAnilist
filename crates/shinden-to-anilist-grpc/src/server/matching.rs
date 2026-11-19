use shinden_to_anilist_core::{
    matcher::{
        DefaultMatcher,
        Matcher,
    },
    searcher::{
        SearchMode,
        Searcher,
    },
    utils::normalize_str,
};
use tracing::{
    info,
    instrument,
};

use super::{
    ShindenToAnilist,
    database::run_blocking,
    providers,
};
use crate::{
    error::{
        ServiceError,
        database_not_loaded,
        shinden_list_not_loaded,
    },
    export::export_xml_to_path,
    matching::{
        FuzzyMatchView,
        search_options,
    },
    pb::*,
};

impl ShindenToAnilist {
    #[instrument(skip_all, name = "app.fuzzy_search")]
    pub async fn fuzzy_search(
        &self,
        request: FuzzySearchRequest,
    ) -> Result<FuzzySearchResponse, ServiceError> {
        let guard = self.database.load();

        let database_version = guard.version();
        let database = guard.get().ok_or_else(database_not_loaded)?;
        let query = normalize_str(&request.query);
        let options = search_options(request.options, SearchMode::Fuzzy);

        let results: Vec<SearchResult> = database
            .searcher
            .search(&query, options)
            .into_iter()
            .map(|(id, score)| SearchResult { id, score })
            .collect();

        Ok(FuzzySearchResponse {
            database_version,
            results,
        })
    }

    #[instrument(skip_all, name = "app.fuzzy_match")]
    pub async fn fuzzy_match(&self, request: FuzzyMatchRequest) -> Result<FuzzyMatchResponse, ServiceError> {
        let database_guard = self.database.load();

        let database_version = database_guard.version();
        let database = database_guard.get().ok_or_else(database_not_loaded)?;
        let source_guard = self.source_list.load();
        let source_entry_id = request.source_id.or(request.shinden_id);
        let source_entry =
            source_entry_id.and_then(|id| source_guard.get().and_then(|source| source.match_view(id)));
        let query = FuzzyMatchView::new(request.query, source_entry);
        let options = search_options(request.options, SearchMode::Fuzzy);
        let matcher = if source_entry.is_some() {
            DefaultMatcher {
                search_weight: 0.8,
                season_weight: 0.1,
                year_weight: 0.03,
                type_weight: 0.03,
                status_weight: 0.015,
                seasonal_weight: 0.015,
                episodes_weight: 0.01,
                ..Default::default()
            }
        } else {
            DefaultMatcher {
                search_weight: 0.8,
                season_weight: 0.2,
                ..Default::default()
            }
        };

        let candidates = database
            .searcher
            .search_ref(&database.database, query.normalized_title(), options);
        let results: Vec<MatchResult> = matcher
            .score_candidates(&query, &candidates, 0.0)
            .items()
            .iter()
            .copied()
            .map(Into::into)
            .collect();

        Ok(FuzzyMatchResponse {
            database_version,
            results,
        })
    }

    #[instrument(skip_all, name = "app.match_shinden_list")]
    pub async fn match_shinden_list(
        &self,
        request: MatchShindenListRequest,
    ) -> Result<MatchShindenListResponse, ServiceError> {
        let shinden_guard = self.shinden_list.load();
        let database_guard = self.database.load();

        let shinden_version = shinden_guard.version();
        let database_version = database_guard.version();
        let shinden = shinden_guard.get_arc().ok_or_else(shinden_list_not_loaded)?;
        let database = database_guard.get_arc().ok_or_else(database_not_loaded)?;
        let options = search_options(request.options, SearchMode::Strict);

        let results = run_blocking("shinden list matching", move || {
            let matcher = DefaultMatcher::strict_preset();
            Ok(providers::shinden::match_legacy_list(
                &shinden, &database, options, &matcher,
            ))
        })
        .await?;

        Ok(MatchShindenListResponse {
            shinden_version,
            database_version,
            results,
        })
    }

    #[instrument(skip_all, name = "app.match_source_list")]
    pub async fn match_source_list(
        &self,
        request: MatchSourceListRequest,
    ) -> Result<MatchSourceListResponse, ServiceError> {
        let source_guard = self.source_list.load();
        let database_guard = self.database.load();

        let source_version = source_guard.version();
        let database_version = database_guard.version();
        let source = source_guard.get_arc().ok_or_else(shinden_list_not_loaded)?;
        let database = database_guard.get_arc().ok_or_else(database_not_loaded)?;
        let options = search_options(request.options, SearchMode::Strict);
        let source_entries = source.len();

        let results = run_blocking("source list matching", move || {
            let matcher = DefaultMatcher::strict_preset();
            Ok(providers::match_source_list(
                source.as_ref(),
                &database,
                options,
                &matcher,
            ))
        })
        .await?;
        info!(
            source_version,
            database_version,
            source_entries,
            results = results.len(),
            "source list matched"
        );

        Ok(MatchSourceListResponse {
            source_version,
            database_version,
            results,
        })
    }

    #[instrument(skip_all, name = "app.export_xml")]
    pub async fn export_xml(&self, request: ExportXmlRequest) -> Result<ExportXmlResponse, ServiceError> {
        let requested_matches = request.matches.len();

        let guard = self.source_list.load();

        let source_version = guard.version();
        if request
            .expected_source_version
            .is_some_and(|expected| expected != source_version)
        {
            return Err(ServiceError::new(
                ErrorKind::SourceListChanged,
                "source version changed before export",
            ));
        }
        if request
            .expected_database_version
            .is_some_and(|expected| expected != self.database.load().version())
        {
            return Err(ServiceError::new(
                ErrorKind::DatabaseChanged,
                "database version changed before export",
            ));
        }
        let source = guard.get_arc().ok_or_else(shinden_list_not_loaded)?;
        let path = request.path;
        let matches = request
            .matches
            .into_iter()
            .map(|pair| (pair.source_id, pair.database_id))
            .collect::<Vec<_>>();

        let path = run_blocking("xml export", move || {
            export_xml_to_path(&source, matches.into_iter(), &path)?;
            Ok(path)
        })
        .await?;
        info!(source_version, requested_matches, "xml exported");

        Ok(ExportXmlResponse {
            source_version,
            shinden_version: source_version,
            path,
        })
    }
}
