use std::{
    path::PathBuf,
    sync::Mutex,
};

use eyre::Context;
use serde::Serialize;
use shinden_to_anilist_core::{
    HttpClient,
    common::{
        AnimeId,
        AnimeList,
        MatchView,
    },
    database::{
        self,
        AnimeDatabase,
        AnimeDatabaseLoad,
    },
    exporter::{
        Exporter,
        xml::XmlExporter,
    },
    matcher::{
        DefaultMatcher,
        Matcher,
        MatcherFinalizer,
        ScoreBreakdown,
    },
    providers::shinden::{
        ShindenList,
        ShindenListLoad,
    },
    searcher::{
        DefaultSearcher,
        Search,
        SearchMode,
        Searcher,
    },
    utils::normalize_str,
};
use tauri::{
    AppHandle,
    Emitter,
    Manager,
    State,
};

const PROGRESS_EVENT: &str = "pipeline://progress";

#[derive(Default)]
struct AppState {
    database: Mutex<Option<LoadedDatabase>>,
    shinden_list: Mutex<Option<ShindenList>>,
}

struct LoadedDatabase {
    path: PathBuf,
    database: AnimeDatabase,
    searcher: DefaultSearcher,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ApiError {
    message: String,
}

impl ApiError {
    fn from_display(error: impl std::fmt::Display) -> Self {
        Self {
            message: error.to_string(),
        }
    }
}

type CommandResult<T> = Result<T, ApiError>;

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct ProgressEvent {
    stage: PipelineStage,
    message: String,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
enum PipelineStage {
    DatabaseUpdateStarted,
    DatabaseUpdateFinished,
    DatabaseLoadStarted,
    DatabaseLoadFinished,
    ShindenFetchStarted,
    ShindenFetchFinished,
    MatchStarted,
    MatchFinished,
    ExportFinished,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AppStatus {
    database_path: String,
    database_exists: bool,
    database_loaded: bool,
    database_entry_count: usize,
    database_last_update: Option<String>,
    shinden_list_loaded: bool,
    shinden_entry_count: usize,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct DatabaseUpdateResponse {
    status: String,
    release: String,
    sha256: String,
    path: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct DatabaseLoadResponse {
    path: String,
    entry_count: usize,
    last_update: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ShindenLoadResponse {
    user_id: AnimeId,
    entry_count: usize,
    entries: Vec<ShindenEntryDto>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SearchResponse {
    query: String,
    normalized_query: String,
    results: Vec<SearchResultDto>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct MatchResponse {
    source_count: usize,
    matched_count: usize,
    results: Vec<MatchResultDto>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ExportResponse {
    xml: String,
    exported_count: usize,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SearchResultDto {
    entry: DatabaseEntryDto,
    search_score: f32,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct MatchResultDto {
    source: ShindenEntryDto,
    winner: Option<MatchedCandidateDto>,
    candidates: Vec<MatchedCandidateDto>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct MatchedCandidateDto {
    entry: DatabaseEntryDto,
    scores: ScoreBreakdown,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct DatabaseEntryDto {
    id: AnimeId,
    title: String,
    anime_type: String,
    episodes: i32,
    status: String,
    season: String,
    year: Option<i32>,
    picture: String,
    thumbnail: String,
    duration: Option<i32>,
    synonyms: Vec<String>,
    studios: Vec<String>,
    producers: Vec<String>,
    tags: Vec<String>,
    sources: Vec<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ShindenEntryDto {
    id: AnimeId,
    title: String,
    normalized_title: String,
    anime_status: String,
    anime_type: String,
    premiere_date: Option<String>,
    finish_date: Option<String>,
    episodes: Option<i32>,
    is_favourite: bool,
    watch_status: String,
    watched_episodes: i32,
    score: Option<i32>,
    note: Option<String>,
    cover_id: Option<i32>,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct SearchOptions {
    limit: Option<usize>,
    threshold: Option<f32>,
    mode: Option<ApiSearchMode>,
}

#[derive(serde::Deserialize, Clone, Copy)]
#[serde(rename_all = "camelCase")]
enum ApiSearchMode {
    Strict,
    Fuzzy,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct MatchOptions {
    search_limit: Option<usize>,
    search_threshold: Option<f32>,
    search_mode: Option<ApiSearchMode>,
    candidate_limit: Option<usize>,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct SelectedMatch {
    source_id: AnimeId,
    database_id: AnimeId,
}

#[tauri::command]
fn app_status(app: AppHandle, state: State<'_, AppState>) -> CommandResult<AppStatus> {
    let default_path = default_database_path(&app)?;
    let database = state.database.lock().map_err(ApiError::from_display)?;
    let shinden_list = state.shinden_list.lock().map_err(ApiError::from_display)?;
    let database_path = database
        .as_ref()
        .map(|loaded| loaded.path.clone())
        .unwrap_or(default_path);

    Ok(AppStatus {
        database_path: database_path.display().to_string(),
        database_exists: database_path.exists(),
        database_loaded: database.is_some(),
        database_entry_count: database
            .as_ref()
            .map(|loaded| loaded.database.len())
            .unwrap_or_default(),
        database_last_update: database
            .as_ref()
            .map(|loaded| loaded.database.last_update().to_string()),
        shinden_list_loaded: shinden_list.is_some(),
        shinden_entry_count: shinden_list.as_ref().map(AnimeList::len).unwrap_or_default(),
    })
}

#[tauri::command]
async fn update_database(
    app: AppHandle,
    database_path: Option<String>,
) -> CommandResult<DatabaseUpdateResponse> {
    let path = resolve_database_path(&app, database_path)?;
    emit_progress(
        &app,
        PipelineStage::DatabaseUpdateStarted,
        "Checking the latest anime-offline-database release",
    );

    let status = database::updater::update_latest_jsonl_from_github(HttpClient::new(), &path)
        .await
        .map_err(ApiError::from_display)?;

    let response = match status {
        database::updater::DatabaseUpdateStatus::UpToDate { release, sha256 } => DatabaseUpdateResponse {
            status: "upToDate".to_string(),
            release,
            sha256,
            path: path.display().to_string(),
        },
        database::updater::DatabaseUpdateStatus::Updated {
            release,
            sha256,
            path,
        } => DatabaseUpdateResponse {
            status: "updated".to_string(),
            release,
            sha256,
            path: path.display().to_string(),
        },
    };

    emit_progress(
        &app,
        PipelineStage::DatabaseUpdateFinished,
        "Database update check finished",
    );

    Ok(response)
}

#[tauri::command]
async fn load_database(
    app: AppHandle,
    state: State<'_, AppState>,
    database_path: Option<String>,
) -> CommandResult<DatabaseLoadResponse> {
    let path = resolve_database_path(&app, database_path)?;
    emit_progress(&app, PipelineStage::DatabaseLoadStarted, "Loading database");

    let path_for_task = path.clone();
    let loaded = tauri::async_runtime::spawn_blocking(move || {
        let database = AnimeDatabase::get_from_mmap(&path_for_task)?;
        let searcher = DefaultSearcher::new(&database);
        Ok::<_, shinden_to_anilist_core::database::DatabaseError>(LoadedDatabase {
            path: path_for_task,
            database,
            searcher,
        })
    })
    .await
    .map_err(ApiError::from_display)?
    .map_err(ApiError::from_display)?;

    let response = DatabaseLoadResponse {
        path: loaded.path.display().to_string(),
        entry_count: loaded.database.len(),
        last_update: loaded.database.last_update().to_string(),
    };

    *state.database.lock().map_err(ApiError::from_display)? = Some(loaded);

    emit_progress(&app, PipelineStage::DatabaseLoadFinished, "Database loaded");

    Ok(response)
}

#[tauri::command]
fn search_database(
    state: State<'_, AppState>,
    query: String,
    options: Option<SearchOptions>,
) -> CommandResult<SearchResponse> {
    let database = state.database.lock().map_err(ApiError::from_display)?;
    let loaded = database
        .as_ref()
        .ok_or_else(|| ApiError::from_display("database is not loaded"))?;
    let normalized_query = normalize_str(&query).to_string();
    let options = search_options(options);

    let results = loaded
        .searcher
        .search_ref(&loaded.database, &normalized_query, options)
        .into_iter()
        .map(|(entry, search_score)| SearchResultDto {
            entry: database_entry_dto(entry),
            search_score,
        })
        .collect();

    Ok(SearchResponse {
        query,
        normalized_query,
        results,
    })
}

#[tauri::command]
async fn fetch_shinden_list(
    app: AppHandle,
    state: State<'_, AppState>,
    user_id: AnimeId,
) -> CommandResult<ShindenLoadResponse> {
    emit_progress(
        &app,
        PipelineStage::ShindenFetchStarted,
        "Fetching Shinden user list",
    );

    let shinden_list = ShindenList::get_from_shinden(HttpClient::new(), user_id)
        .await
        .map_err(ApiError::from_display)?;
    let entries = shinden_list.values().map(shinden_entry_dto).collect();
    let response = ShindenLoadResponse {
        user_id,
        entry_count: shinden_list.len(),
        entries,
    };

    *state.shinden_list.lock().map_err(ApiError::from_display)? = Some(shinden_list);

    emit_progress(
        &app,
        PipelineStage::ShindenFetchFinished,
        "Shinden user list fetched",
    );

    Ok(response)
}

#[tauri::command]
fn match_shinden_list(
    app: AppHandle,
    state: State<'_, AppState>,
    options: Option<MatchOptions>,
) -> CommandResult<MatchResponse> {
    emit_progress(&app, PipelineStage::MatchStarted, "Matching Shinden list");

    let database = state.database.lock().map_err(ApiError::from_display)?;
    let loaded = database
        .as_ref()
        .ok_or_else(|| ApiError::from_display("database is not loaded"))?;
    let shinden_list = state.shinden_list.lock().map_err(ApiError::from_display)?;
    let shinden_list = shinden_list
        .as_ref()
        .ok_or_else(|| ApiError::from_display("Shinden list is not loaded"))?;

    let options = options.unwrap_or_default();
    let search = match_search_options(&options);
    let candidate_limit = options.candidate_limit.unwrap_or(10);
    let matcher = DefaultMatcher::strict_preset();

    let mut raw_results = shinden_list
        .values()
        .map(|source| {
            let candidates = loaded
                .searcher
                .search_ref(&loaded.database, source.normalized_title(), search);
            matcher.score_candidates(source, &candidates, 0.0)
        })
        .collect::<Vec<_>>();

    raw_results.iter_mut().finalize_matches();

    let results = shinden_list
        .values()
        .zip(raw_results.iter())
        .map(|(source, result)| MatchResultDto {
            source: shinden_entry_dto(source),
            winner: result
                .winner_ref(&loaded.database)
                .map(|(entry, scores)| matched_candidate_dto(entry, scores)),
            candidates: result
                .items_ref(&loaded.database)
                .take(candidate_limit)
                .map(|(entry, scores)| matched_candidate_dto(entry, scores))
                .collect(),
        })
        .collect::<Vec<_>>();
    let matched_count = results.iter().filter(|result| result.winner.is_some()).count();

    emit_progress(&app, PipelineStage::MatchFinished, "Matching finished");

    Ok(MatchResponse {
        source_count: shinden_list.len(),
        matched_count,
        results,
    })
}

#[tauri::command]
fn export_matches_xml(
    app: AppHandle,
    state: State<'_, AppState>,
    matches: Vec<SelectedMatch>,
) -> CommandResult<ExportResponse> {
    let shinden_list = state.shinden_list.lock().map_err(ApiError::from_display)?;
    let shinden_list = shinden_list
        .as_ref()
        .ok_or_else(|| ApiError::from_display("Shinden list is not loaded"))?;

    let exported_count = matches.len();
    let mut xml = Vec::new();
    XmlExporter::default()
        .export(
            shinden_list,
            matches
                .into_iter()
                .map(|selected| (selected.source_id, selected.database_id)),
            &mut xml,
        )
        .map_err(ApiError::from_display)?;
    let xml = String::from_utf8(xml).map_err(ApiError::from_display)?;

    emit_progress(&app, PipelineStage::ExportFinished, "XML export finished");

    Ok(ExportResponse { xml, exported_count })
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() -> eyre::Result<()> {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(AppState::default())
        .invoke_handler(tauri::generate_handler![
            app_status,
            update_database,
            load_database,
            search_database,
            fetch_shinden_list,
            match_shinden_list,
            export_matches_xml
        ])
        .run(tauri::generate_context!())
        .wrap_err("error while running tauri application")
}

fn default_database_path(app: &AppHandle) -> CommandResult<PathBuf> {
    Ok(app
        .path()
        .app_data_dir()
        .map_err(ApiError::from_display)?
        .join("anime-offline-database.jsonl"))
}

fn resolve_database_path(app: &AppHandle, path: Option<String>) -> CommandResult<PathBuf> {
    path.map(PathBuf::from)
        .map(Ok)
        .unwrap_or_else(|| default_database_path(app))
}

fn emit_progress(app: &AppHandle, stage: PipelineStage, message: impl Into<String>) {
    let _ = app.emit(
        PROGRESS_EVENT,
        ProgressEvent {
            stage,
            message: message.into(),
        },
    );
}

fn search_options(options: Option<SearchOptions>) -> Search {
    let options = options.unwrap_or_default();
    Search {
        limit: options.limit.unwrap_or(50),
        threshold: options.threshold.unwrap_or(0.65),
        mode: search_mode(options.mode),
    }
}

fn match_search_options(options: &MatchOptions) -> Search {
    Search {
        limit: options.search_limit.unwrap_or(50),
        threshold: options.search_threshold.unwrap_or(0.65),
        mode: search_mode(options.search_mode),
    }
}

fn search_mode(mode: Option<ApiSearchMode>) -> SearchMode {
    match mode.unwrap_or(ApiSearchMode::Fuzzy) {
        ApiSearchMode::Strict => SearchMode::Strict,
        ApiSearchMode::Fuzzy => SearchMode::Fuzzy,
    }
}

fn database_entry_dto(entry: &database::AnimeEntry) -> DatabaseEntryDto {
    DatabaseEntryDto {
        id: entry.id(),
        title: entry.title().to_string(),
        anime_type: format!("{:?}", entry.anime_type()),
        episodes: entry.episodes(),
        status: format!("{:?}", entry.status()),
        season: format!("{:?}", entry.season()),
        year: entry.year(),
        picture: entry.picture().to_string(),
        thumbnail: entry.thumbnail().to_string(),
        duration: entry.duration(),
        synonyms: strings(entry.synonyms()),
        studios: strings(entry.studios()),
        producers: strings(entry.producers()),
        tags: strings(entry.tags()),
        sources: strings(entry.sources()),
    }
}

fn shinden_entry_dto(entry: &shinden_to_anilist_core::providers::shinden::AnimeEntry) -> ShindenEntryDto {
    ShindenEntryDto {
        id: entry.id(),
        title: entry.title().to_string(),
        normalized_title: entry.normalized_title().to_string(),
        anime_status: format!("{:?}", entry.anime_status()),
        anime_type: format!("{:?}", entry.anime_type()),
        premiere_date: entry.premiere_date().map(|date| date.to_string()),
        finish_date: entry.finish_date().map(|date| date.to_string()),
        episodes: entry.episodes(),
        is_favourite: entry.is_favourite(),
        watch_status: format!("{:?}", entry.watch_status()),
        watched_episodes: entry.watched_episodes(),
        score: entry.score(),
        note: entry.note().map(ToString::to_string),
        cover_id: entry.cover_id(),
    }
}

fn matched_candidate_dto(entry: &database::AnimeEntry, scores: ScoreBreakdown) -> MatchedCandidateDto {
    MatchedCandidateDto {
        entry: database_entry_dto(entry),
        scores,
    }
}

fn strings(values: &[shinden_to_anilist_core::CompactString]) -> Vec<String> {
    values.iter().map(ToString::to_string).collect()
}

impl Default for SearchOptions {
    fn default() -> Self {
        Self {
            limit: None,
            threshold: None,
            mode: None,
        }
    }
}

impl Default for MatchOptions {
    fn default() -> Self {
        Self {
            search_limit: None,
            search_threshold: None,
            search_mode: None,
            candidate_limit: None,
        }
    }
}
