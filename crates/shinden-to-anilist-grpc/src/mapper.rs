use shinden_to_anilist_core::{
    Datelike,
    NaiveDate,
    database::{
        self,
        DatabaseError,
    },
    exporter,
    providers::shinden::{
        self,
        ShindenError,
    },
};
use tap::prelude::Conv;
use tonic::Status;

use crate::pb::{
    AnimeStatus,
    AnimeType,
    DatabaseEntry,
    DatabaseMetadata,
    DatabaseReleaseInfo,
    Date,
    Season,
    ShindenEntry,
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

pub fn database_error_to_status(value: DatabaseError) -> Status {
    match value {
        shinden_to_anilist_core::database::DatabaseError::Io(..) => Status::internal(value.to_string()),
        shinden_to_anilist_core::database::DatabaseError::Json(..) => Status::internal(value.to_string()),
        shinden_to_anilist_core::database::DatabaseError::Request(..) => {
            Status::unavailable(value.to_string())
        },
        shinden_to_anilist_core::database::DatabaseError::Empty => Status::internal(value.to_string()),
        shinden_to_anilist_core::database::DatabaseError::MissingReleaseAsset { .. } => {
            Status::not_found(value.to_string())
        },
        shinden_to_anilist_core::database::DatabaseError::DigestMismatch { .. } => {
            Status::internal(value.to_string())
        },
    }
}

pub fn shinden_error_to_status(value: ShindenError) -> Status {
    match value {
        ShindenError::Io(..) => Status::internal(value.to_string()),
        ShindenError::Json(..) => Status::internal(value.to_string()),
        ShindenError::Request(..) => Status::internal(value.to_string()),
        ShindenError::Shinden(..) => Status::unavailable(value.to_string()),
    }
}
