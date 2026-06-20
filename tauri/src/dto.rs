use std::fmt;

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
use shinden_to_anilist_grpc::pb;
use tonic::Status;
use tracing::warn;

#[derive(Debug, Clone, Copy)]
pub(crate) struct WireNumberDto(pub(crate) u64);

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
pub(crate) struct AppPathsDto {
    pub(crate) base: String,
    pub(crate) database: String,
    pub(crate) export: String,
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
pub(crate) struct FetchShindenListResponseDto {
    pub(crate) shinden_version: WireNumberDto,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SourceFetchProgressDto {
    provider: i32,
    phase: i32,
    current: WireNumberDto,
    total: WireNumberDto,
    latest_title: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct FetchSourceListResponseDto {
    pub(crate) source_version: WireNumberDto,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct GetShindenIdsResponseDto {
    pub(crate) shinden_version: WireNumberDto,
    pub(crate) ids: Vec<WireNumberDto>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct GetSourceIdsResponseDto {
    pub(crate) source_version: WireNumberDto,
    pub(crate) ids: Vec<WireNumberDto>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ShindenEntryDto {
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
pub(crate) struct SourceEntryDto {
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
pub(crate) struct GetShindenEntriesResponseDto {
    pub(crate) shinden_version: WireNumberDto,
    pub(crate) entries: Vec<ShindenEntryDto>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct GetShindenFullResponseDto {
    pub(crate) shinden_version: WireNumberDto,
    pub(crate) entries: Vec<ShindenEntryDto>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct GetSourceFullResponseDto {
    pub(crate) source_version: WireNumberDto,
    pub(crate) entries: Vec<SourceEntryDto>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DatabaseReleaseInfoDto {
    release: String,
    sha256: String,
    compressed_size: WireNumberDto,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DatabaseUpdateCheckDto {
    local: Option<DatabaseReleaseInfoDto>,
    remote: Option<DatabaseReleaseInfoDto>,
    needs_update: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CheckDatabaseUpdateResponseDto {
    pub(crate) status: Option<DatabaseUpdateCheckDto>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DownloadDatabaseResponseDto {
    pub(crate) status: Option<DatabaseReleaseInfoDto>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct LoadDatabaseResponseDto {
    pub(crate) database_version: WireNumberDto,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DatabaseMetadataDto {
    last_update: Option<DateDto>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct GetDatabaseMetadataResponseDto {
    pub(crate) metadata: Option<DatabaseMetadataDto>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct GetDatabaseIdsResponseDto {
    pub(crate) database_version: WireNumberDto,
    pub(crate) ids: Vec<WireNumberDto>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DatabaseEntryDto {
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
pub(crate) struct GetDatabaseEntriesResponseDto {
    pub(crate) database_version: WireNumberDto,
    pub(crate) entries: Vec<DatabaseEntryDto>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct GetDatabaseFullResponseDto {
    pub(crate) database_version: WireNumberDto,
    pub(crate) entries: Vec<DatabaseEntryDto>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SearchOptionsDto {
    #[serde(default)]
    limit: u32,
    #[serde(default)]
    threshold: Option<f32>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SearchResultDto {
    id: WireNumberDto,
    score: f32,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct FuzzySearchResponseDto {
    pub(crate) database_version: WireNumberDto,
    pub(crate) results: Vec<SearchResultDto>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct MatchResultDto {
    id: WireNumberDto,
    final_score: f32,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct FuzzyMatchResponseDto {
    pub(crate) database_version: WireNumberDto,
    pub(crate) results: Vec<MatchResultDto>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ShindenMatchResultDto {
    shinden_id: WireNumberDto,
    candidates: Vec<MatchResultDto>,
    top_candidates: Vec<MatchResultDto>,
    winner: Option<MatchResultDto>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct MatchShindenListResponseDto {
    pub(crate) shinden_version: WireNumberDto,
    pub(crate) database_version: WireNumberDto,
    pub(crate) results: Vec<ShindenMatchResultDto>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SourceMatchResultDto {
    source_id: WireNumberDto,
    candidates: Vec<MatchResultDto>,
    top_candidates: Vec<MatchResultDto>,
    winner: Option<MatchResultDto>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct MatchSourceListResponseDto {
    pub(crate) source_version: WireNumberDto,
    pub(crate) database_version: WireNumberDto,
    pub(crate) results: Vec<SourceMatchResultDto>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AnimeIdPairDto {
    shinden_id: WireNumberDto,
    database_id: WireNumberDto,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SourceIdPairDto {
    source_id: WireNumberDto,
    database_id: WireNumberDto,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ExportXmlResponseDto {
    pub(crate) source_version: WireNumberDto,
    pub(crate) shinden_version: WireNumberDto,
    pub(crate) path: String,
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

pub(crate) fn command_error(status: Status) -> String {
    warn!(
        code = ?status.code(),
        message = %status.message(),
        "tauri command failed"
    );
    status.message().to_owned()
}

pub(crate) fn wire_numbers(values: Vec<u64>) -> Vec<WireNumberDto> {
    values.into_iter().map(Into::into).collect()
}
