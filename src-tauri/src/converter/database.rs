use chrono::NaiveDate;
use eyre::eyre;
use itertools::Itertools;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use smol_str::SmolStr;
use std::io::BufRead;

impl DatabaseRoot {
    pub fn from_reader(reader: &mut (impl BufRead + Send)) -> eyre::Result<DatabaseRoot> {
        let mut lines = reader.lines();

        let Some(root_line) = lines.next() else {
            return Err(eyre!("database contains no entries"));
        };

        let mut root: DatabaseRoot = serde_json::from_str(&root_line?)?;

        let entries = lines
            .chunks(500)
            .into_iter()
            .map(|chunk| {
                chunk
                    .collect::<Vec<_>>()
                    .into_par_iter()
                    .map(|x| Ok(serde_json::from_str::<AnimeEntry>(&x?)?))
                    .collect::<eyre::Result<Vec<AnimeEntry>>>()
            })
            .flatten_ok()
            .collect::<eyre::Result<Vec<AnimeEntry>>>()?;

        root.data = entries;

        Ok(root)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DatabaseRoot {
    pub last_update: NaiveDate,
    #[serde(skip)]
    pub data: Vec<AnimeEntry>,
}

/// Valid for every single line from the *.jsonl file except the first line which contains the meta data.
/// anime-offline-database
#[derive(Serialize, Deserialize, Debug, Eq, Hash, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AnimeEntry {
    /// URLs to the pages of the meta data providers for this anime.
    pub sources: Vec<String>,
    /// Main title.
    pub title: SmolStr,
    /// Distribution type.
    #[serde(rename = "type")]
    pub anime_type: AnimeType,
    /// Number of episodes, movies or parts.
    pub episodes: i32,
    /// Status of distribution.
    pub status: AnimeStatus,
    /// Data on when the anime was first distributed.
    pub anime_season: AnimeSeason,
    /// URL of a picture which represents the anime.
    pub picture: String,
    /// URL of a smaller version of the picture.
    pub thumbnail: String,
    /// Duration per episode.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration: Option<Duration>,
    /// Alternative titles and spellings under which the anime is also known.
    pub synonyms: Vec<SmolStr>,
    /// Lower case studio names. In general a duplicate free list, but might contain duplicates for different writings.
    pub studios: Vec<SmolStr>,
    /// Lower case producers names. Companys only. In general a duplicate free list, but might contain duplicates for different writings.
    pub producers: Vec<SmolStr>,
    /// URLs to the meta data providers for anime that are somehow related to this anime.
    pub related_anime: Vec<String>,
    /// A non-curated list of tags and genres which describe the anime.
    pub tags: Vec<SmolStr>,
}

/// Data on when the anime was first distributed.
#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Hash, Clone)]
pub struct AnimeSeason {
    /// Season.
    pub season: Season,
    /// Year.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub year: Option<i32>,
}
#[derive(Serialize, Deserialize, Copy, Clone, Debug, Eq, PartialEq, Hash)]
#[serde(rename_all = "UPPERCASE")]
pub enum Season {
    Spring,
    Summer,
    Fall,
    Winter,
    Undefined,
}

#[derive(Serialize, Deserialize, Copy, Clone, Debug, Eq, PartialEq, Hash)]
#[serde(rename_all = "UPPERCASE")]
pub enum AnimeType {
    Tv,
    Movie,
    Ova,
    Ona,
    Special,
    Unknown,
}

#[derive(Serialize, Deserialize, Copy, Clone, Debug, Eq, PartialEq, Hash)]
#[serde(rename_all = "UPPERCASE")]
pub enum AnimeStatus {
    Finished,
    Ongoing,
    Upcoming,
    Unknown,
}

#[derive(Serialize, Deserialize, Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct Duration {
    pub value: i32,
}
