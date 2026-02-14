use std::{
    fmt::{
        Display,
        Formatter,
    },
    io::Write,
};

use chrono::NaiveDate;
use compact_str::{
    CompactString,
    ToCompactString,
};
use serde::{
    Serialize,
    Serializer,
};
use thiserror::Error;

use crate::converter::{
    common::{
        AnimeId,
        AnimeList,
        ExportView,
    },
    exporter::{
        Exporter,
        WatchStatus,
    },
};

/// Errors that can occur during XML export.
#[derive(Error, Debug)]
#[error(transparent)]
pub enum XmlExportError {
    /// XML serialization failed.
    Xml(#[from] serde_xml_rs::Error),
    /// A matched anime ID was not found in the source list.
    #[error("id {0} for {1} is out of index")]
    OutOfIndex(AnimeId, &'static str),
}

/// Exports matched anime entries to MAL-compatible XML.
///
/// The output conforms to the MyAnimeList import format, suitable for
/// uploading via MAL's list import tool.
#[derive(Debug, Default, Clone)]
pub struct XmlExporter {}
impl Exporter for XmlExporter {
    type Error = XmlExportError;

    fn export(
        &self,
        anime_list: &impl AnimeList<Entry = impl ExportView>,
        entries: impl Iterator<Item = (AnimeId, AnimeId)>,
        writer: impl Write,
    ) -> Result<(), Self::Error> {
        let mut list_root = ListRootXml {
            info: InfoXml { user_export_type: 1 },
            ..Default::default()
        };
        for (id, db_id) in entries {
            let entry = anime_list
                .get(id)
                .ok_or(XmlExportError::OutOfIndex(id, "anime list"))?;
            list_root.anime.push(AnimeXml::from_export_view(entry, db_id));
        }
        serde_xml_rs::to_writer(writer, &list_root)?;
        Ok(())
    }
}

fn ser_mal_date<S>(date: &Option<NaiveDate>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    if let Some(date) = date {
        serializer.serialize_str(&date.to_compact_string())
    } else {
        serializer.serialize_str("0000-00-00")
    }
}

#[derive(Serialize, Debug, Copy, Clone, Eq, PartialEq)]
#[serde(into = "String")]
enum AnimeStatus {
    Dropped,
    Completed,
    Watching,
    #[serde(rename = "On-Hold")]
    OnHold,
    #[serde(rename = "Plan to Watch")]
    PlanToWatch,
}

impl From<WatchStatus> for AnimeStatus {
    fn from(value: WatchStatus) -> Self {
        match value {
            WatchStatus::Dropped => AnimeStatus::Dropped,
            WatchStatus::Completed => AnimeStatus::Completed,
            WatchStatus::Watching => AnimeStatus::Watching,
            WatchStatus::OnHold => AnimeStatus::OnHold,
            WatchStatus::PlanToWatch => AnimeStatus::PlanToWatch,
        }
    }
}

impl Display for AnimeStatus {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let x = match self {
            AnimeStatus::Dropped => "Dropped",
            AnimeStatus::Completed => "Completed",
            AnimeStatus::Watching => "Watching",
            AnimeStatus::OnHold => "On-Hold",
            AnimeStatus::PlanToWatch => "Plan to Watch",
        };
        write!(f, "{x}")
    }
}

impl From<AnimeStatus> for String {
    fn from(value: AnimeStatus) -> Self { value.to_string() }
}

#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
struct AnimeXml {
    #[serde(rename = "series_animedb_id")]
    id: AnimeId,
    #[serde(rename = "my_watched_episodes")]
    watched_episodes: i32,
    #[serde(rename = "my_start_date", serialize_with = "ser_mal_date")]
    start_date: Option<NaiveDate>,
    #[serde(rename = "my_finish_date", serialize_with = "ser_mal_date")]
    finish_date: Option<NaiveDate>,
    #[serde(rename = "my_score")]
    score: i32,
    #[serde(rename = "my_status")]
    status: AnimeStatus,
    #[serde(rename = "update_on_import")]
    update: i32,
    #[serde(rename = "my_comments")]
    comments: CompactString,
}

impl AnimeXml {
    fn from_export_view(export: &impl ExportView, db_id: AnimeId) -> Self {
        Self {
            id: db_id,
            watched_episodes: export.watched_episodes(),
            start_date: export.start_date(),
            finish_date: export.finish_date(),
            score: export.score(),
            status: export.status().into(),
            update: 1,
            comments: export.comments().unwrap_or_default().to_compact_string(),
        }
    }
}

#[derive(Serialize, Debug, Clone, Default, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
struct InfoXml {
    user_export_type: i32,
}

#[derive(Serialize, Debug, Clone, Default)]
#[serde(rename = "myanimelist")]
#[serde(rename_all = "snake_case")]
struct ListRootXml {
    #[serde(rename = "myinfo")]
    info: InfoXml,
    #[serde(rename = "anime")]
    anime: Vec<AnimeXml>,
}
