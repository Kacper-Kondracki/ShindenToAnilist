use shinden_to_anilist_core::{
    Datelike,
    NaiveDate,
    common::{
        AnimeId,
        ExportView,
    },
    database,
    exporter,
    matcher,
    providers::{
        animezone,
        shinden,
    },
};
use tap::prelude::Conv;

use crate::pb::{
    AnimeStatus,
    AnimeType,
    DatabaseEntry,
    DatabaseMetadata,
    DatabaseReleaseInfo,
    Date,
    MatchResult,
    Season,
    ShindenEntry,
    ShindenMatchResult,
    SourceEntry,
    SourceMatchResult,
    SourceProvider,
    WatchStatus,
};

impl From<&shinden::AnimeEntry> for ShindenEntry {
    fn from(value: &shinden::AnimeEntry) -> Self {
        Self {
            id: value.id(),
            cover_id: value.cover_id(),
            title: value.title().to_string(),
            anime_status: value.anime_status().conv::<AnimeStatus>().into(),
            anime_type: value.anime_type().conv::<AnimeType>().into(),
            premiere_date: value.premiere_date().map(Date::from),
            finish_date: value.finish_date().map(Date::from),
            episodes: value.episodes(),
            is_favourite: value.is_favourite(),
            watch_status: value.watch_status().conv::<WatchStatus>().into(),
            watched_episodes: value.watched_episodes(),
            score: value.score(),
        }
    }
}

impl From<&shinden::AnimeEntry> for SourceEntry {
    fn from(value: &shinden::AnimeEntry) -> Self {
        Self {
            id: value.id(),
            provider: SourceProvider::Shinden.into(),
            title: value.title().to_string(),
            anime_status: value.anime_status().conv::<AnimeStatus>().into(),
            anime_type: value.anime_type().conv::<AnimeType>().into(),
            premiere_date: value.premiere_date().map(Date::from),
            year: value.premiere_date().map(|date| date.year()),
            episodes: value.episodes(),
            watch_status: value.watch_status().conv::<WatchStatus>().into(),
            watched_episodes: value.watched_episodes(),
            score: value.score(),
            source_url: format!("https://shinden.pl/series/{}", value.id()),
            mal_id: None,
        }
    }
}

impl From<&animezone::AnimeZoneEntry> for SourceEntry {
    fn from(value: &animezone::AnimeZoneEntry) -> Self {
        Self {
            id: value.id(),
            provider: SourceProvider::AnimeZone.into(),
            title: value.title().to_string(),
            anime_status: value
                .anime_status()
                .unwrap_or(database::AnimeStatus::Unknown)
                .conv::<AnimeStatus>()
                .into(),
            anime_type: value
                .anime_type()
                .unwrap_or(database::AnimeType::Unknown)
                .conv::<AnimeType>()
                .into(),
            premiere_date: None,
            year: value.year(),
            episodes: value.episodes(),
            watch_status: value.watch_status().conv::<WatchStatus>().into(),
            watched_episodes: ExportView::watched_episodes(value),
            score: value.score(),
            source_url: format!("https://www.animezone.pl/anime/{}", value.slug()),
            mal_id: value.mal_id(),
        }
    }
}

impl From<&database::AnimeEntry> for DatabaseEntry {
    fn from(value: &database::AnimeEntry) -> Self {
        Self {
            id: value.id(),
            sources: value.sources().iter().map(|v| v.to_string()).collect(),
            title: value.title().to_string(),
            anime_type: value.anime_type().conv::<AnimeType>().into(),
            episodes: value.episodes(),
            status: value.status().conv::<AnimeStatus>().into(),
            season: value.season().conv::<Season>().into(),
            year: value.year(),
            picture: value.picture().to_string(),
            thumbnail: value.thumbnail().to_string(),
            duration: value.duration(),
            synonyms: value.synonyms().iter().map(|v| v.to_string()).collect(),
        }
    }
}

impl From<(AnimeId, matcher::ScoreBreakdown)> for MatchResult {
    fn from((id, score): (AnimeId, matcher::ScoreBreakdown)) -> Self {
        Self {
            id,
            final_score: score.final_score,
        }
    }
}

impl From<(AnimeId, matcher::MatchResult)> for ShindenMatchResult {
    fn from((shinden_id, result): (AnimeId, matcher::MatchResult)) -> Self {
        Self {
            shinden_id,
            candidates: result.items().iter().copied().map(MatchResult::from).collect(),
            top_candidates: result.top().iter().copied().map(MatchResult::from).collect(),
            winner: result.winner().map(MatchResult::from),
        }
    }
}

impl From<(AnimeId, matcher::MatchResult)> for SourceMatchResult {
    fn from((source_id, result): (AnimeId, matcher::MatchResult)) -> Self {
        Self {
            source_id,
            candidates: result.items().iter().copied().map(MatchResult::from).collect(),
            top_candidates: result.top().iter().copied().map(MatchResult::from).collect(),
            winner: result.winner().map(MatchResult::from),
        }
    }
}

pub(crate) fn direct_source_match_result(source_id: AnimeId, database_id: AnimeId) -> SourceMatchResult {
    let direct_match = MatchResult {
        id: database_id,
        final_score: 1.0,
    };

    SourceMatchResult {
        source_id,
        candidates: vec![direct_match.clone()],
        top_candidates: vec![direct_match.clone()],
        winner: Some(direct_match),
    }
}

impl From<database::AnimeStatus> for AnimeStatus {
    fn from(value: database::AnimeStatus) -> Self {
        match value {
            database::AnimeStatus::Finished => AnimeStatus::Finished,
            database::AnimeStatus::Ongoing => AnimeStatus::Ongoing,
            database::AnimeStatus::Upcoming => AnimeStatus::Upcoming,
            database::AnimeStatus::Unknown => AnimeStatus::Unknown,
        }
    }
}

impl From<database::AnimeType> for AnimeType {
    fn from(value: database::AnimeType) -> Self {
        match value {
            database::AnimeType::Tv => AnimeType::Tv,
            database::AnimeType::Movie => AnimeType::Movie,
            database::AnimeType::Ova => AnimeType::Ova,
            database::AnimeType::Ona => AnimeType::Ona,
            database::AnimeType::Special => AnimeType::Special,
            database::AnimeType::Unknown => AnimeType::Unknown,
        }
    }
}

impl From<exporter::WatchStatus> for WatchStatus {
    fn from(value: exporter::WatchStatus) -> Self {
        match value {
            exporter::WatchStatus::Dropped => WatchStatus::Dropped,
            exporter::WatchStatus::Completed => WatchStatus::Completed,
            exporter::WatchStatus::Watching => WatchStatus::Watching,
            exporter::WatchStatus::OnHold => WatchStatus::OnHold,
            exporter::WatchStatus::PlanToWatch => WatchStatus::PlanToWatch,
        }
    }
}

impl From<database::Season> for Season {
    fn from(value: database::Season) -> Self {
        match value {
            database::Season::Spring => Season::Spring,
            database::Season::Summer => Season::Summer,
            database::Season::Fall => Season::Fall,
            database::Season::Winter => Season::Winter,
            database::Season::Undefined => Season::Unknown,
        }
    }
}

impl From<NaiveDate> for Date {
    fn from(value: NaiveDate) -> Self {
        Self {
            year: value.year(),
            month: value.month(),
            day: value.day(),
        }
    }
}

impl From<database::updater::DatabaseReleaseInfo> for DatabaseReleaseInfo {
    fn from(value: database::updater::DatabaseReleaseInfo) -> Self {
        let database::updater::DatabaseReleaseInfo {
            release,
            sha256,
            compressed_size,
        } = value;
        Self {
            release,
            sha256,
            compressed_size,
        }
    }
}

impl From<database::DatabaseRootMetadata> for DatabaseMetadata {
    fn from(value: database::DatabaseRootMetadata) -> Self {
        Self {
            last_update: Some(value.last_update().into()),
        }
    }
}
