use crate::{
    converter::database::DatabaseRoot,
    converter::view::AnimeList,
    converter::view::{AnimeId, ExportView},
    utils::ser_mal_date,
};
use chrono::NaiveDate;
use indexmap::IndexMap;
use serde::Serialize;
use std::fmt::{Display, Formatter};

pub fn export_list(
    list: &AnimeList<impl ExportView>,
    db: &DatabaseRoot,
    match_map: &IndexMap<AnimeId, AnimeId>,
) -> ListRootXml {
    let mut result = ListRootXml {
        info: InfoXml {
            user_export_type: 1,
        },
        anime: vec![],
    };

    for (&entry_id, &db_id) in match_map {
        let entry = &list[&entry_id];
        let _db_entry = &db.data[&db_id];

        let item = AnimeXml {
            id: db_id,
            watched_episodes: entry.watched_episodes(),
            start_date: entry.start_date(),
            finish_date: entry.finish_date(),
            score: entry.score(),
            status: entry.status(),
            update: 1,
            comments: entry.comments().unwrap_or_default().to_string(),
        };

        result.anime.push(item);
    }

    result
}

#[derive(Serialize, Debug, Clone, Default, Eq, PartialEq)]
#[serde(into = "String")]
pub enum StatusXml {
    Dropped,
    Completed,
    Watching,
    #[serde(rename = "On-Hold")]
    OnHold,
    #[default]
    #[serde(rename = "Plan to Watch")]
    PlanToWatch,
}

impl Display for StatusXml {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let x = match self {
            StatusXml::Dropped => "Dropped",
            StatusXml::Completed => "Completed",
            StatusXml::Watching => "Watching",
            StatusXml::OnHold => "On-Hold",
            StatusXml::PlanToWatch => "Plan to Watch",
        };
        write!(f, "{x}")
    }
}

impl From<StatusXml> for String {
    fn from(value: StatusXml) -> Self {
        value.to_string()
    }
}

#[derive(Serialize, Debug, Clone, Default, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct AnimeXml {
    #[serde(rename = "series_animedb_id")]
    pub id: AnimeId,
    #[serde(rename = "my_watched_episodes")]
    pub watched_episodes: i32,
    #[serde(rename = "my_start_date", serialize_with = "ser_mal_date")]
    pub start_date: Option<NaiveDate>,
    #[serde(rename = "my_finish_date")]
    pub finish_date: Option<NaiveDate>,
    #[serde(rename = "my_score")]
    pub score: i32,
    #[serde(rename = "my_status")]
    pub status: StatusXml,
    #[serde(rename = "update_on_import")]
    pub update: i32,
    #[serde(rename = "my_comments")]
    pub comments: String,
}

#[derive(Serialize, Debug, Clone, Default, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct InfoXml {
    pub user_export_type: i32,
}

#[derive(Serialize, Debug, Clone, Default, Eq, PartialEq)]
#[serde(rename = "myanimelist")]
#[serde(rename_all = "snake_case")]
pub struct ListRootXml {
    #[serde(rename = "myinfo")]
    pub info: InfoXml,
    #[serde(rename = "anime")]
    pub anime: Vec<AnimeXml>,
}
