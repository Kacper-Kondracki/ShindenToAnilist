use crate::{
    converter::database::DatabaseRoot,
    converter::shinden,
    converter::shinden::{ShindenList, WatchStatus},
};
use chrono::NaiveDate;
use indexmap::IndexMap;
use serde::{Serialize, Serializer};
use std::fmt::{Display, Formatter};

pub fn export_shinden(
    shinden: &ShindenList,
    db: &DatabaseRoot,
    match_map: &IndexMap<u32, u32>,
) -> ListRootXaml {
    let mut list = ListRootXaml {
        info: InfoXaml {
            user_export_type: 1,
        },
        anime: vec![],
    };

    for (&shinden_id, &db_id) in match_map {
        let shinden_entry = &shinden.items[&shinden_id];
        let _db_entry = &db.data[&db_id];

        let item = AnimeXaml {
            id: db_id,
            watched_episodes: shinden_entry.watched_episodes_cnt,
            start_date: shinden_entry.premiere_date,
            finish_date: shinden_entry.finish_date,
            score: shinden_entry.rate_total.unwrap_or_default(),
            status: shinden_entry.watch_status.into(),
            update: 1,
            comments: shinden_entry.user_note.clone().unwrap_or_default(),
        };

        list.anime.push(item);
    }

    list
}

#[derive(Serialize, Debug, Clone, Default, Eq, PartialEq)]
#[serde(into = "String")]
pub enum StatusXaml {
    Dropped,
    Completed,
    Watching,
    #[serde(rename = "On-Hold")]
    OnHold,
    #[default]
    #[serde(rename = "Plan to Watch")]
    PlanToWatch,
}

impl Display for StatusXaml {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let x = match self {
            StatusXaml::Dropped => "Dropped",
            StatusXaml::Completed => "Completed",
            StatusXaml::Watching => "Watching",
            StatusXaml::OnHold => "On-Hold",
            StatusXaml::PlanToWatch => "Plan to Watch",
        };
        write!(f, "{x}")
    }
}

impl From<StatusXaml> for String {
    fn from(value: StatusXaml) -> Self {
        value.to_string()
    }
}

impl From<shinden::WatchStatus> for StatusXaml {
    fn from(value: WatchStatus) -> Self {
        match value {
            WatchStatus::Completed => StatusXaml::Completed,
            WatchStatus::Plan => StatusXaml::PlanToWatch,
            WatchStatus::InProgress => StatusXaml::Watching,
            WatchStatus::Skip => StatusXaml::Dropped,
            WatchStatus::Hold => StatusXaml::OnHold,
            WatchStatus::Dropped => StatusXaml::Dropped,
        }
    }
}

#[derive(Serialize, Debug, Clone, Default, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct AnimeXaml {
    #[serde(rename = "series_animedb_id")]
    pub id: u32,
    #[serde(rename = "my_watched_episodes")]
    pub watched_episodes: i32,
    #[serde(rename = "my_start_date", serialize_with = "ser_mal_date")]
    pub start_date: Option<NaiveDate>,
    #[serde(rename = "my_finish_date")]
    pub finish_date: Option<NaiveDate>,
    #[serde(rename = "my_score")]
    pub score: i32,
    #[serde(rename = "my_status")]
    pub status: StatusXaml,
    #[serde(rename = "update_on_import")]
    pub update: i32,
    #[serde(rename = "my_comments")]
    pub comments: String,
}

fn ser_mal_date<S>(date: &Option<NaiveDate>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let date = if let Some(date) = date {
        date.to_string()
    } else {
        "0000-00-00".to_string()
    };

    serializer.serialize_str(&date)
}

#[derive(Serialize, Debug, Clone, Default, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct InfoXaml {
    pub user_export_type: i32,
}

#[derive(Serialize, Debug, Clone, Default, Eq, PartialEq)]
#[serde(rename = "myanimelist")]
#[serde(rename_all = "snake_case")]
pub struct ListRootXaml {
    #[serde(rename = "myinfo")]
    pub info: InfoXaml,
    #[serde(rename = "anime")]
    pub anime: Vec<AnimeXaml>,
}
