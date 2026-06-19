use std::{
    collections::HashMap,
    fmt,
    fs,
    path::Path,
    sync::Mutex,
};

use serde::{
    Deserialize,
    Deserializer,
    Serialize,
    Serializer,
    de::{
        self,
        Visitor,
    },
};
use shinden_to_anilist_grpc::{
    ShindenToAnilist,
    pb,
};
use tauri::{
    AppHandle,
    Manager,
    State,
    ipc::Channel,
};
use tokio_util::sync::CancellationToken;
use tonic::Status;

struct AppState {
    service: ShindenToAnilist,
    source_fetch_cancellations: Mutex<HashMap<u64, CancellationToken>>,
}

const PRODUCT_NAME: &str = "ShindenToAnilist";

#[derive(Debug, Clone, Copy)]
struct WireNumberDto(u64);

impl From<u64> for WireNumberDto {
    fn from(value: u64) -> Self { Self(value) }
}

impl From<WireNumberDto> for u64 {
    fn from(value: WireNumberDto) -> Self { value.0 }
}

impl Serialize for WireNumberDto {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.0.to_string())
    }
}

impl<'de> Deserialize<'de> for WireNumberDto {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(WireNumberVisitor)
    }
}

struct WireNumberVisitor;

impl Visitor<'_> for WireNumberVisitor {
    type Value = WireNumberDto;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("an unsigned 64-bit integer or decimal string")
    }

    fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E> { Ok(WireNumberDto(value)) }

    fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        u64::try_from(value)
            .map(WireNumberDto)
            .map_err(|_| E::custom(format!("value is outside u64 range: {value}")))
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        value
            .parse::<u64>()
            .map(WireNumberDto)
            .map_err(|_| E::custom(format!("invalid u64 decimal string: {value}")))
    }

    fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        self.visit_str(&value)
    }
}

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
    shinden_version: WireNumberDto,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SourceFetchProgressDto {
    provider: i32,
    phase: i32,
    current: WireNumberDto,
    total: WireNumberDto,
    latest_title: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct FetchSourceListResponseDto {
    source_version: WireNumberDto,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct GetShindenIdsResponseDto {
    shinden_version: WireNumberDto,
    ids: Vec<WireNumberDto>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct GetSourceIdsResponseDto {
    source_version: WireNumberDto,
    ids: Vec<WireNumberDto>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ShindenEntryDto {
    id: WireNumberDto,
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
struct SourceEntryDto {
    id: WireNumberDto,
    provider: i32,
    title: String,
    anime_status: i32,
    anime_type: i32,
    premiere_date: Option<DateDto>,
    year: Option<i32>,
    episodes: Option<i32>,
    watch_status: i32,
    watched_episodes: i32,
    score: Option<i32>,
    source_url: String,
    mal_id: Option<WireNumberDto>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct GetShindenEntriesResponseDto {
    shinden_version: WireNumberDto,
    entries: Vec<ShindenEntryDto>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct GetShindenFullResponseDto {
    shinden_version: WireNumberDto,
    entries: Vec<ShindenEntryDto>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct GetSourceFullResponseDto {
    source_version: WireNumberDto,
    entries: Vec<SourceEntryDto>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct DatabaseReleaseInfoDto {
    release: String,
    sha256: String,
    compressed_size: WireNumberDto,
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
    database_version: WireNumberDto,
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
    database_version: WireNumberDto,
    ids: Vec<WireNumberDto>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct DatabaseEntryDto {
    id: WireNumberDto,
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
    database_version: WireNumberDto,
    entries: Vec<DatabaseEntryDto>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct GetDatabaseFullResponseDto {
    database_version: WireNumberDto,
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
    id: WireNumberDto,
    score: f32,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct FuzzySearchResponseDto {
    database_version: WireNumberDto,
    results: Vec<SearchResultDto>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct MatchResultDto {
    id: WireNumberDto,
    final_score: f32,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct FuzzyMatchResponseDto {
    database_version: WireNumberDto,
    results: Vec<MatchResultDto>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ShindenMatchResultDto {
    shinden_id: WireNumberDto,
    candidates: Vec<MatchResultDto>,
    top_candidates: Vec<MatchResultDto>,
    winner: Option<MatchResultDto>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct MatchShindenListResponseDto {
    shinden_version: WireNumberDto,
    database_version: WireNumberDto,
    results: Vec<ShindenMatchResultDto>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SourceMatchResultDto {
    source_id: WireNumberDto,
    candidates: Vec<MatchResultDto>,
    top_candidates: Vec<MatchResultDto>,
    winner: Option<MatchResultDto>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct MatchSourceListResponseDto {
    source_version: WireNumberDto,
    database_version: WireNumberDto,
    results: Vec<SourceMatchResultDto>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AnimeIdPairDto {
    shinden_id: WireNumberDto,
    database_id: WireNumberDto,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SourceIdPairDto {
    source_id: WireNumberDto,
    database_id: WireNumberDto,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ExportXmlResponseDto {
    source_version: WireNumberDto,
    shinden_version: WireNumberDto,
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

impl From<pb::SourceFetchProgress> for SourceFetchProgressDto {
    fn from(value: pb::SourceFetchProgress) -> Self {
        Self {
            provider: value.provider,
            phase: value.phase,
            current: value.current.into(),
            total: value.total.into(),
            latest_title: value.latest_title,
        }
    }
}

impl From<pb::ShindenEntry> for ShindenEntryDto {
    fn from(value: pb::ShindenEntry) -> Self {
        Self {
            id: value.id.into(),
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

impl From<pb::SourceEntry> for SourceEntryDto {
    fn from(value: pb::SourceEntry) -> Self {
        Self {
            id: value.id.into(),
            provider: value.provider,
            title: value.title,
            anime_status: value.anime_status,
            anime_type: value.anime_type,
            premiere_date: value.premiere_date.map(Into::into),
            year: value.year,
            episodes: value.episodes,
            watch_status: value.watch_status,
            watched_episodes: value.watched_episodes,
            score: value.score,
            source_url: value.source_url,
            mal_id: value.mal_id.map(Into::into),
        }
    }
}

impl From<pb::DatabaseReleaseInfo> for DatabaseReleaseInfoDto {
    fn from(value: pb::DatabaseReleaseInfo) -> Self {
        Self {
            release: value.release,
            sha256: value.sha256,
            compressed_size: value.compressed_size.into(),
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
            id: value.id.into(),
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
            id: value.id.into(),
            score: value.score,
        }
    }
}

impl From<pb::MatchResult> for MatchResultDto {
    fn from(value: pb::MatchResult) -> Self {
        Self {
            id: value.id.into(),
            final_score: value.final_score,
        }
    }
}

impl From<pb::ShindenMatchResult> for ShindenMatchResultDto {
    fn from(value: pb::ShindenMatchResult) -> Self {
        Self {
            shinden_id: value.shinden_id.into(),
            candidates: value.candidates.into_iter().map(Into::into).collect(),
            top_candidates: value.top_candidates.into_iter().map(Into::into).collect(),
            winner: value.winner.map(Into::into),
        }
    }
}

impl From<pb::SourceMatchResult> for SourceMatchResultDto {
    fn from(value: pb::SourceMatchResult) -> Self {
        Self {
            source_id: value.source_id.into(),
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
            shinden_id: value.shinden_id.into(),
            database_id: value.database_id.into(),
        }
    }
}

impl From<SourceIdPairDto> for pb::SourceIdPair {
    fn from(value: SourceIdPairDto) -> Self {
        Self {
            source_id: value.source_id.into(),
            database_id: value.database_id.into(),
        }
    }
}

fn command_error(status: Status) -> String { status.message().to_owned() }

fn display_path(path: &Path) -> String { path.to_string_lossy().into_owned() }

fn wire_numbers(values: Vec<u64>) -> Vec<WireNumberDto> { values.into_iter().map(Into::into).collect() }

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
async fn fetch_source_list(
    state: State<'_, AppState>,
    request_id: u64,
    provider: i32,
    user: String,
    on_progress: Channel<SourceFetchProgressDto>,
) -> Result<FetchSourceListResponseDto, String> {
    let cancellation_token = CancellationToken::new();
    {
        let mut cancellations = state
            .source_fetch_cancellations
            .lock()
            .map_err(|err| err.to_string())?;

        if let Some(previous) = cancellations.insert(request_id, cancellation_token.clone()) {
            previous.cancel();
        }
    }

    let result = state
        .service
        .fetch_source_list_with_progress(
            pb::FetchSourceListRequest { provider, user },
            cancellation_token,
            move |progress| {
                on_progress
                    .send(progress.into())
                    .map_err(|err| Status::internal(err.to_string()))
            },
        )
        .await
        .map(|response| FetchSourceListResponseDto {
            source_version: response.source_version.into(),
        })
        .map_err(command_error);

    state
        .source_fetch_cancellations
        .lock()
        .map_err(|err| err.to_string())?
        .remove(&request_id);

    result
}

#[tauri::command(rename_all = "camelCase")]
fn cancel_source_list_fetch(state: State<'_, AppState>, request_id: u64) -> Result<(), String> {
    if let Some(cancellation_token) = state
        .source_fetch_cancellations
        .lock()
        .map_err(|err| err.to_string())?
        .remove(&request_id)
    {
        cancellation_token.cancel();
    }

    Ok(())
}

#[tauri::command(rename_all = "camelCase")]
async fn get_source_ids(
    state: State<'_, AppState>,
    sorted_by: Option<i32>,
) -> Result<GetSourceIdsResponseDto, String> {
    state
        .service
        .get_source_ids(pb::GetSourceIdsRequest {
            sorted_by: sorted_by.unwrap_or_default(),
        })
        .await
        .map(|response| GetSourceIdsResponseDto {
            source_version: response.source_version.into(),
            ids: wire_numbers(response.ids),
        })
        .map_err(command_error)
}

#[tauri::command]
async fn get_source_full(state: State<'_, AppState>) -> Result<GetSourceFullResponseDto, String> {
    state
        .service
        .get_source_full(pb::GetSourceFullRequest {})
        .await
        .map(|response| GetSourceFullResponseDto {
            source_version: response.source_version.into(),
            entries: response.entries.into_iter().map(Into::into).collect(),
        })
        .map_err(command_error)
}

#[tauri::command(rename_all = "camelCase")]
async fn fetch_shinden_list(
    state: State<'_, AppState>,
    id: WireNumberDto,
) -> Result<FetchShindenListResponseDto, String> {
    state
        .service
        .fetch_shinden_list(pb::FetchShindenListRequest { id: id.into() })
        .await
        .map(|response| FetchShindenListResponseDto {
            shinden_version: response.shinden_version.into(),
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
            shinden_version: response.shinden_version.into(),
            ids: wire_numbers(response.ids),
        })
        .map_err(command_error)
}

#[tauri::command(rename_all = "camelCase")]
async fn get_shinden_entries(
    state: State<'_, AppState>,
    ids: Vec<WireNumberDto>,
) -> Result<GetShindenEntriesResponseDto, String> {
    state
        .service
        .get_shinden_entries(pb::GetShindenEntriesRequest {
            ids: ids.into_iter().map(Into::into).collect(),
        })
        .await
        .map(|response| GetShindenEntriesResponseDto {
            shinden_version: response.shinden_version.into(),
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
            shinden_version: response.shinden_version.into(),
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
            database_version: response.database_version.into(),
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
            database_version: response.database_version.into(),
            ids: wire_numbers(response.ids),
        })
        .map_err(command_error)
}

#[tauri::command(rename_all = "camelCase")]
async fn get_database_entries(
    state: State<'_, AppState>,
    ids: Vec<WireNumberDto>,
) -> Result<GetDatabaseEntriesResponseDto, String> {
    state
        .service
        .get_database_entries(pb::GetDatabaseEntriesRequest {
            ids: ids.into_iter().map(Into::into).collect(),
        })
        .await
        .map(|response| GetDatabaseEntriesResponseDto {
            database_version: response.database_version.into(),
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
            database_version: response.database_version.into(),
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
            database_version: response.database_version.into(),
            results: response.results.into_iter().map(Into::into).collect(),
        })
        .map_err(command_error)
}

#[tauri::command(rename_all = "camelCase")]
async fn fuzzy_match(
    state: State<'_, AppState>,
    query: String,
    options: Option<SearchOptionsDto>,
    shinden_id: Option<WireNumberDto>,
    source_id: Option<WireNumberDto>,
) -> Result<FuzzyMatchResponseDto, String> {
    state
        .service
        .fuzzy_match(pb::FuzzyMatchRequest {
            query,
            options: Some(options.unwrap_or_default().into()),
            shinden_id: shinden_id.map(Into::into),
            source_id: source_id.map(Into::into),
        })
        .await
        .map(|response| FuzzyMatchResponseDto {
            database_version: response.database_version.into(),
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
            shinden_version: response.shinden_version.into(),
            database_version: response.database_version.into(),
            results: response.results.into_iter().map(Into::into).collect(),
        })
        .map_err(command_error)
}

#[tauri::command(rename_all = "camelCase")]
async fn match_source_list(
    state: State<'_, AppState>,
    options: Option<SearchOptionsDto>,
) -> Result<MatchSourceListResponseDto, String> {
    state
        .service
        .match_source_list(pb::MatchSourceListRequest {
            options: Some(options.unwrap_or_default().into()),
        })
        .await
        .map(|response| MatchSourceListResponseDto {
            source_version: response.source_version.into(),
            database_version: response.database_version.into(),
            results: response.results.into_iter().map(Into::into).collect(),
        })
        .map_err(command_error)
}

#[tauri::command(rename_all = "camelCase")]
async fn export_xml(
    state: State<'_, AppState>,
    path: String,
    matches: Vec<SourceIdPairDto>,
) -> Result<ExportXmlResponseDto, String> {
    state
        .service
        .export_xml(pb::ExportXmlRequest {
            path,
            matches: matches.into_iter().map(Into::into).collect(),
        })
        .await
        .map(|response| ExportXmlResponseDto {
            source_version: response.source_version.into(),
            shinden_version: response.shinden_version.into(),
            path: response.path,
        })
        .map_err(command_error)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let client = reqwest::Client::builder()
        .cookie_store(true)
        .build()
        .expect("failed to build HTTP client");

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .manage(AppState {
            service: ShindenToAnilist::new(client),
            source_fetch_cancellations: Mutex::new(HashMap::new()),
        })
        .invoke_handler(tauri::generate_handler![
            app_paths,
            fetch_source_list,
            cancel_source_list_fetch,
            get_source_ids,
            get_source_full,
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
            match_source_list,
            match_shinden_list,
            export_xml
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
