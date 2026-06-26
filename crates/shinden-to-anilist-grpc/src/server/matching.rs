use shinden_to_anilist_core::{
    common::AnimeList,
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
use tonic::Status;
use tracing::{
    info,
    instrument,
};

use super::{
    ShindenToAnilist,
    database::spawn_blocking_status,
    providers,
};
use crate::{
    error::{
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
    pub async fn fuzzy_search(&self, request: FuzzySearchRequest) -> Result<FuzzySearchResponse, Status> {
        let query_len = request.query.len();
        info!(query_len, "running fuzzy search");
        let guard = self.database.load();

        let database_version = guard.version();
        let database = guard.get().ok_or_else(|| database_not_loaded().into_status())?;
        let query = normalize_str(&request.query);
        let options = search_options(request.options, SearchMode::Fuzzy);

        let results: Vec<SearchResult> = database
            .searcher
            .search(&query, options)
            .into_iter()
            .map(|(id, score)| SearchResult { id, score })
            .collect();
        info!(database_version, results = results.len(), "fuzzy search finished");

        Ok(FuzzySearchResponse {
            database_version,
            results,
        })
    }

    #[instrument(skip_all, name = "app.fuzzy_match")]
    pub async fn fuzzy_match(&self, request: FuzzyMatchRequest) -> Result<FuzzyMatchResponse, Status> {
        let query_len = request.query.len();
        info!(
            query_len,
            shinden_id = request.shinden_id,
            source_id = request.source_id,
            "running fuzzy match"
        );
        let database_guard = self.database.load();

        let database_version = database_guard.version();
        let database = database_guard
            .get()
            .ok_or_else(|| database_not_loaded().into_status())?;
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
        info!(
            database_version,
            candidates = candidates.len(),
            results = results.len(),
            "fuzzy match finished"
        );

        Ok(FuzzyMatchResponse {
            database_version,
            results,
        })
    }

    #[instrument(skip_all, name = "app.match_shinden_list")]
    pub async fn match_shinden_list(
        &self,
        request: MatchShindenListRequest,
    ) -> Result<MatchShindenListResponse, Status> {
        info!("matching shinden list");
        let shinden_guard = self.shinden_list.load();
        let database_guard = self.database.load();

        let shinden_version = shinden_guard.version();
        let database_version = database_guard.version();
        let shinden = shinden_guard
            .get_arc()
            .ok_or_else(|| shinden_list_not_loaded().into_status())?;
        let database = database_guard
            .get_arc()
            .ok_or_else(|| database_not_loaded().into_status())?;
        let options = search_options(request.options, SearchMode::Strict);
        let shinden_entries = shinden.len();
        let database_entries = database.database.len();

        let results = spawn_blocking_status("shinden list matching", move || {
            let matcher = DefaultMatcher::strict_preset();
            Ok(providers::shinden::match_legacy_list(
                &shinden, &database, options, &matcher,
            ))
        })
        .await?;
        info!(
            shinden_version,
            database_version,
            shinden_entries,
            database_entries,
            results = results.len(),
            "shinden list matched"
        );

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
    ) -> Result<MatchSourceListResponse, Status> {
        info!("matching source list");
        let source_guard = self.source_list.load();
        let database_guard = self.database.load();

        let source_version = source_guard.version();
        let database_version = database_guard.version();
        let source = source_guard
            .get_arc()
            .ok_or_else(|| shinden_list_not_loaded().into_status())?;
        let database = database_guard
            .get_arc()
            .ok_or_else(|| database_not_loaded().into_status())?;
        let options = search_options(request.options, SearchMode::Strict);
        let source_entries = source.len();
        let database_entries = database.database.len();

        let results = spawn_blocking_status("source list matching", move || {
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
            database_entries,
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
    pub async fn export_xml(&self, request: ExportXmlRequest) -> Result<ExportXmlResponse, Status> {
        let requested_matches = request.matches.len();
        info!(path = %request.path, requested_matches, "exporting xml");
        let guard = self.source_list.load();

        let source_version = guard.version();
        let source = guard
            .get_arc()
            .ok_or_else(|| shinden_list_not_loaded().into_status())?;
        let path = request.path;
        let matches = request
            .matches
            .into_iter()
            .map(|pair| (pair.source_id, pair.database_id))
            .collect::<Vec<_>>();

        let path = spawn_blocking_status("xml export", move || {
            export_xml_to_path(&source, matches.into_iter(), &path)?;
            Ok(path)
        })
        .await?;
        info!(source_version, path = %path, requested_matches, "xml exported");

        Ok(ExportXmlResponse {
            source_version,
            shinden_version: source_version,
            path,
        })
    }
}
