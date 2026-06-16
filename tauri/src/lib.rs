use std::{
    fs,
    path::Path,
};

use serde::{
    Deserialize,
    Serialize,
};
use shinden_to_anilist_grpc::{
    ShindenToAnilist,
    pb,
};
use tauri::{
    AppHandle,
    Manager,
    State,
};
use tonic::Status;

struct AppState {
    service: ShindenToAnilist,
}

const PRODUCT_NAME: &str = "ShindenToAnilist";

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct AppPathsDto {
    base: String,
    database: String,
    export: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct DateDto {
    year: i32,
    month: u32,
    day: u32,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct FetchShindenListResponseDto {
    shinden_version: u64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct GetShindenIdsResponseDto {
    shinden_version: u64,
    ids: Vec<u64>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ShindenEntryDto {
    id: u64,
    cover_id: Option<i32>,
    title: String,
    anime_status: i32,
    anime_type: i32,
    premiere_date: Option<DateDto>,
    finish_date: Option<DateDto>,
    episodes: Option<i32>,
    is_favourite: bool,
    watch_status: i32,
    watched_episodes: i32,
    score: Option<i32>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct GetShindenEntriesResponseDto {
    shinden_version: u64,
    entries: Vec<ShindenEntryDto>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct GetShindenFullResponseDto {
    shinden_version: u64,
    entries: Vec<ShindenEntryDto>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct DatabaseReleaseInfoDto {
    release: String,
    sha256: String,
    compressed_size: u64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct DatabaseUpdateCheckDto {
    local: Option<DatabaseReleaseInfoDto>,
    remote: Option<DatabaseReleaseInfoDto>,
    needs_update: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct CheckDatabaseUpdateResponseDto {
    status: Option<DatabaseUpdateCheckDto>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct DownloadDatabaseResponseDto {
    status: Option<DatabaseReleaseInfoDto>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct LoadDatabaseResponseDto {
    database_version: u64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct DatabaseMetadataDto {
    last_update: Option<DateDto>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct GetDatabaseMetadataResponseDto {
    metadata: Option<DatabaseMetadataDto>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct GetDatabaseIdsResponseDto {
    database_version: u64,
    ids: Vec<u64>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct DatabaseEntryDto {
    id: u64,
    sources: Vec<String>,
    title: String,
    anime_type: i32,
    episodes: i32,
    status: i32,
    season: i32,
    year: Option<i32>,
    picture: String,
    thumbnail: String,
    duration: Option<i32>,
    synonyms: Vec<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct GetDatabaseEntriesResponseDto {
    database_version: u64,
    entries: Vec<DatabaseEntryDto>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct GetDatabaseFullResponseDto {
    database_version: u64,
    entries: Vec<DatabaseEntryDto>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SearchOptionsDto {
    #[serde(default)]
    limit: u32,
    #[serde(default)]
    threshold: Option<f32>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SearchResultDto {
    id: u64,
    score: f32,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct FuzzySearchResponseDto {
    database_version: u64,
    results: Vec<SearchResultDto>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct MatchResultDto {
    id: u64,
    final_score: f32,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct FuzzyMatchResponseDto {
    database_version: u64,
    results: Vec<MatchResultDto>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ShindenMatchResultDto {
    shinden_id: u64,
    candidates: Vec<MatchResultDto>,
    top_candidates: Vec<MatchResultDto>,
    winner: Option<MatchResultDto>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct MatchShindenListResponseDto {
    shinden_version: u64,
    database_version: u64,
    results: Vec<ShindenMatchResultDto>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AnimeIdPairDto {
    shinden_id: u64,
    database_id: u64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ExportXmlResponseDto {
    shinden_version: u64,
    path: String,
}

impl From<pb::Date> for DateDto {
    fn from(value: pb::Date) -> Self {
        Self {
            year: value.year,
            month: value.month,
            day: value.day,
        }
    }
}

impl From<pb::ShindenEntry> for ShindenEntryDto {
    fn from(value: pb::ShindenEntry) -> Self {
        Self {
            id: value.id,
            cover_id: value.cover_id,
            title: value.title,
            anime_status: value.anime_status,
            anime_type: value.anime_type,
            premiere_date: value.premiere_date.map(Into::into),
            finish_date: value.finish_date.map(Into::into),
            episodes: value.episodes,
            is_favourite: value.is_favourite,
            watch_status: value.watch_status,
            watched_episodes: value.watched_episodes,
            score: value.score,
        }
    }
}

impl From<pb::DatabaseReleaseInfo> for DatabaseReleaseInfoDto {
    fn from(value: pb::DatabaseReleaseInfo) -> Self {
        Self {
            release: value.release,
            sha256: value.sha256,
            compressed_size: value.compressed_size,
        }
    }
}

impl From<pb::DatabaseUpdateCheck> for DatabaseUpdateCheckDto {
    fn from(value: pb::DatabaseUpdateCheck) -> Self {
        Self {
            local: value.local.map(Into::into),
            remote: value.remote.map(Into::into),
            needs_update: value.needs_update,
        }
    }
}

impl From<pb::DatabaseMetadata> for DatabaseMetadataDto {
    fn from(value: pb::DatabaseMetadata) -> Self {
        Self {
            last_update: value.last_update.map(Into::into),
        }
    }
}

impl From<pb::DatabaseEntry> for DatabaseEntryDto {
    fn from(value: pb::DatabaseEntry) -> Self {
        Self {
            id: value.id,
            sources: value.sources,
            title: value.title,
            anime_type: value.anime_type,
            episodes: value.episodes,
            status: value.status,
            season: value.season,
            year: value.year,
            picture: value.picture,
            thumbnail: value.thumbnail,
            duration: value.duration,
            synonyms: value.synonyms,
        }
    }
}

impl From<pb::SearchResult> for SearchResultDto {
    fn from(value: pb::SearchResult) -> Self {
        Self {
            id: value.id,
            score: value.score,
        }
    }
}

impl From<pb::MatchResult> for MatchResultDto {
    fn from(value: pb::MatchResult) -> Self {
        Self {
            id: value.id,
            final_score: value.final_score,
        }
    }
}

impl From<pb::ShindenMatchResult> for ShindenMatchResultDto {
    fn from(value: pb::ShindenMatchResult) -> Self {
        Self {
            shinden_id: value.shinden_id,
            candidates: value.candidates.into_iter().map(Into::into).collect(),
            top_candidates: value.top_candidates.into_iter().map(Into::into).collect(),
            winner: value.winner.map(Into::into),
        }
    }
}

impl From<SearchOptionsDto> for pb::SearchOptions {
    fn from(value: SearchOptionsDto) -> Self {
        Self {
            limit: value.limit,
            threshold: value.threshold,
        }
    }
}

impl From<AnimeIdPairDto> for pb::AnimeIdPair {
    fn from(value: AnimeIdPairDto) -> Self {
        Self {
            shinden_id: value.shinden_id,
            database_id: value.database_id,
        }
    }
}

fn command_error(status: Status) -> String { status.message().to_owned() }

fn display_path(path: &Path) -> String { path.to_string_lossy().into_owned() }

#[tauri::command]
fn app_paths(app: AppHandle) -> Result<AppPathsDto, String> {
    let base = app
        .path()
        .data_dir()
        .map_err(|err| err.to_string())?
        .join(PRODUCT_NAME);

    fs::create_dir_all(&base).map_err(|err| err.to_string())?;
    let export_dir = app.path().document_dir().unwrap_or_else(|_| base.clone());

    Ok(AppPathsDto {
        database: display_path(&base.join("database.jsonl")),
        export: display_path(&export_dir.join("export.xml")),
        base: display_path(&base),
    })
}

#[tauri::command(rename_all = "camelCase")]
async fn fetch_shinden_list(
    state: State<'_, AppState>,
    id: u64,
) -> Result<FetchShindenListResponseDto, String> {
    state
        .service
        .fetch_shinden_list(pb::FetchShindenListRequest { id })
        .await
        .map(|response| FetchShindenListResponseDto {
            shinden_version: response.shinden_version,
        })
        .map_err(command_error)
}

#[tauri::command(rename_all = "camelCase")]
async fn get_shinden_ids(
    state: State<'_, AppState>,
    sorted_by: Option<i32>,
) -> Result<GetShindenIdsResponseDto, String> {
    state
        .service
        .get_shinden_ids(pb::GetShindenIdsRequest {
            sorted_by: sorted_by.unwrap_or_default(),
        })
        .await
        .map(|response| GetShindenIdsResponseDto {
            shinden_version: response.shinden_version,
            ids: response.ids,
        })
        .map_err(command_error)
}

#[tauri::command(rename_all = "camelCase")]
async fn get_shinden_entries(
    state: State<'_, AppState>,
    ids: Vec<u64>,
) -> Result<GetShindenEntriesResponseDto, String> {
    state
        .service
        .get_shinden_entries(pb::GetShindenEntriesRequest { ids })
        .await
        .map(|response| GetShindenEntriesResponseDto {
            shinden_version: response.shinden_version,
            entries: response.entries.into_iter().map(Into::into).collect(),
        })
        .map_err(command_error)
}

#[tauri::command]
async fn get_shinden_full(state: State<'_, AppState>) -> Result<GetShindenFullResponseDto, String> {
    state
        .service
        .get_shinden_full(pb::GetShindenFullRequest {})
        .await
        .map(|response| GetShindenFullResponseDto {
            shinden_version: response.shinden_version,
            entries: response.entries.into_iter().map(Into::into).collect(),
        })
        .map_err(command_error)
}

#[tauri::command(rename_all = "camelCase")]
async fn check_database_update(
    state: State<'_, AppState>,
    path: String,
) -> Result<CheckDatabaseUpdateResponseDto, String> {
    state
        .service
        .check_database_update(pb::CheckDatabaseUpdateRequest { path })
        .await
        .map(|response| CheckDatabaseUpdateResponseDto {
            status: response.status.map(Into::into),
        })
        .map_err(command_error)
}

#[tauri::command(rename_all = "camelCase")]
async fn download_database(
    state: State<'_, AppState>,
    path: String,
) -> Result<DownloadDatabaseResponseDto, String> {
    state
        .service
        .download_database(pb::DownloadDatabaseRequest { path })
        .await
        .map(|response| DownloadDatabaseResponseDto {
            status: response.status.map(Into::into),
        })
        .map_err(command_error)
}

#[tauri::command(rename_all = "camelCase")]
async fn load_database(state: State<'_, AppState>, path: String) -> Result<LoadDatabaseResponseDto, String> {
    state
        .service
        .load_database(pb::LoadDatabaseRequest { path })
        .await
        .map(|response| LoadDatabaseResponseDto {
            database_version: response.database_version,
        })
        .map_err(command_error)
}

#[tauri::command(rename_all = "camelCase")]
async fn get_database_metadata(
    state: State<'_, AppState>,
    path: String,
) -> Result<GetDatabaseMetadataResponseDto, String> {
    state
        .service
        .get_database_metadata(pb::GetDatabaseMetadataRequest { path })
        .await
        .map(|response| GetDatabaseMetadataResponseDto {
            metadata: response.metadata.map(Into::into),
        })
        .map_err(command_error)
}

#[tauri::command(rename_all = "camelCase")]
async fn get_database_ids(
    state: State<'_, AppState>,
    sorted_by: Option<i32>,
) -> Result<GetDatabaseIdsResponseDto, String> {
    state
        .service
        .get_database_ids(pb::GetDatabaseIdsRequest {
            sorted_by: sorted_by.unwrap_or_default(),
        })
        .await
        .map(|response| GetDatabaseIdsResponseDto {
            database_version: response.database_version,
            ids: response.ids,
        })
        .map_err(command_error)
}

#[tauri::command(rename_all = "camelCase")]
async fn get_database_entries(
    state: State<'_, AppState>,
    ids: Vec<u64>,
) -> Result<GetDatabaseEntriesResponseDto, String> {
    state
        .service
        .get_database_entries(pb::GetDatabaseEntriesRequest { ids })
        .await
        .map(|response| GetDatabaseEntriesResponseDto {
            database_version: response.database_version,
            entries: response.entries.into_iter().map(Into::into).collect(),
        })
        .map_err(command_error)
}

#[tauri::command]
async fn get_database_full(state: State<'_, AppState>) -> Result<GetDatabaseFullResponseDto, String> {
    state
        .service
        .get_database_full(pb::GetDatabaseFullRequest {})
        .await
        .map(|response| GetDatabaseFullResponseDto {
            database_version: response.database_version,
            entries: response.entries.into_iter().map(Into::into).collect(),
        })
        .map_err(command_error)
}

#[tauri::command(rename_all = "camelCase")]
async fn fuzzy_search(
    state: State<'_, AppState>,
    query: String,
    options: Option<SearchOptionsDto>,
) -> Result<FuzzySearchResponseDto, String> {
    state
        .service
        .fuzzy_search(pb::FuzzySearchRequest {
            query,
            options: Some(options.unwrap_or_default().into()),
        })
        .await
        .map(|response| FuzzySearchResponseDto {
            database_version: response.database_version,
            results: response.results.into_iter().map(Into::into).collect(),
        })
        .map_err(command_error)
}

#[tauri::command(rename_all = "camelCase")]
async fn fuzzy_match(
    state: State<'_, AppState>,
    query: String,
    options: Option<SearchOptionsDto>,
    shinden_id: Option<u64>,
) -> Result<FuzzyMatchResponseDto, String> {
    state
        .service
        .fuzzy_match(pb::FuzzyMatchRequest {
            query,
            options: Some(options.unwrap_or_default().into()),
            shinden_id,
        })
        .await
        .map(|response| FuzzyMatchResponseDto {
            database_version: response.database_version,
            results: response.results.into_iter().map(Into::into).collect(),
        })
        .map_err(command_error)
}

#[tauri::command(rename_all = "camelCase")]
async fn match_shinden_list(
    state: State<'_, AppState>,
    options: Option<SearchOptionsDto>,
) -> Result<MatchShindenListResponseDto, String> {
    state
        .service
        .match_shinden_list(pb::MatchShindenListRequest {
            options: Some(options.unwrap_or_default().into()),
        })
        .await
        .map(|response| MatchShindenListResponseDto {
            shinden_version: response.shinden_version,
            database_version: response.database_version,
            results: response.results.into_iter().map(Into::into).collect(),
        })
        .map_err(command_error)
}

#[tauri::command(rename_all = "camelCase")]
async fn export_xml(
    state: State<'_, AppState>,
    path: String,
    matches: Vec<AnimeIdPairDto>,
) -> Result<ExportXmlResponseDto, String> {
    state
        .service
        .export_xml(pb::ExportXmlRequest {
            path,
            matches: matches.into_iter().map(Into::into).collect(),
        })
        .await
        .map(|response| ExportXmlResponseDto {
            shinden_version: response.shinden_version,
            path: response.path,
        })
        .map_err(command_error)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .manage(AppState {
            service: ShindenToAnilist::new(reqwest::Client::new()),
        })
        .invoke_handler(tauri::generate_handler![
            app_paths,
            fetch_shinden_list,
            get_shinden_ids,
            get_shinden_entries,
            get_shinden_full,
            check_database_update,
            download_database,
            load_database,
            get_database_metadata,
            get_database_ids,
            get_database_entries,
            get_database_full,
            fuzzy_search,
            fuzzy_match,
            match_shinden_list,
            export_xml
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
