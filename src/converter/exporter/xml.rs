use std::{
    fmt::{
        Display,
        Formatter,
    },
    io::Write,
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
        NaiveDate,
        XmlError,
    },
    exporter::{
        Exporter,
        WatchStatus,
    },
};

#[derive(Error, Debug)]
#[error(transparent)]
pub enum XmlExportError {
    Xml(#[from] XmlError),
    #[error("id {0} for {1} is out of index")]
    OutOfIndex(AnimeId, &'static str),
}

#[derive(Debug, Default, Clone)]
pub struct XmlExporter {}
impl Exporter for XmlExporter {
    type Error = XmlExportError;

    fn export(
        &self,
        anime_list: &impl AnimeList<Entry = impl ExportView>,
        anime_db: &impl AnimeList,
        entries: impl Iterator<Item = (AnimeId, AnimeId)>,
        writer: impl Write,
    ) -> Result<(), Self::Error> {
        let mut list_root = ListRootXml::default();
        for (id, db_id) in entries {
            let entry = anime_list.get(id).ok_or(XmlExportError::OutOfIndex(id, "anime list"))?;
            let _db_entry = anime_db.get(id).ok_or(XmlExportError::OutOfIndex(id, "database"))?;
            list_root.anime.push(AnimeXml::from_export_view(entry, db_id));
        }
        serde_xml_rs::to_writer(writer, &list_root)?;
        Ok(())
    }
}

pub(crate) fn ser_mal_date<S>(date: &Option<NaiveDate>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let date = if let Some(date) = date { date.to_string() } else { "0000-00-00".to_string() };
    serializer.serialize_str(&date)
}

#[derive(Serialize, Debug, Copy, Clone, Eq, PartialEq)]
#[serde(into = "String")]
pub(crate) enum AnimeStatus {
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
pub(crate) struct AnimeXml {
    #[serde(rename = "series_animedb_id")]
    pub(crate) id: AnimeId,
    #[serde(rename = "my_watched_episodes")]
    pub(crate) watched_episodes: i32,
    #[serde(rename = "my_start_date", serialize_with = "ser_mal_date")]
    pub(crate) start_date: Option<NaiveDate>,
    #[serde(rename = "my_finish_date")]
    pub(crate) finish_date: Option<NaiveDate>,
    #[serde(rename = "my_score")]
    pub(crate) score: i32,
    #[serde(rename = "my_status")]
    pub(crate) status: AnimeStatus,
    #[serde(rename = "update_on_import")]
    pub(crate) update: i32,
    #[serde(rename = "my_comments")]
    pub(crate) comments: String,
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
            comments: export.comments().unwrap_or_default().to_string(),
        }
    }
}

#[derive(Serialize, Debug, Clone, Default, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub(crate) struct InfoXml {
    pub(crate) user_export_type: i32,
}

#[derive(Serialize, Debug, Clone, Default)]
#[serde(rename = "myanimelist")]
#[serde(rename_all = "snake_case")]
pub(crate) struct ListRootXml {
    #[serde(rename = "myinfo")]
    pub(crate) info: InfoXml,
    #[serde(rename = "anime")]
    pub(crate) anime: Vec<AnimeXml>,
}
