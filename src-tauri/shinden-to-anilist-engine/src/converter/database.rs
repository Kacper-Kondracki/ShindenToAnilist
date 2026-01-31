use crate::{converter::matcher, converter::matcher::ExtractedMetadata};
use chrono::NaiveDate;
use eyre::{OptionExt, WrapErr, eyre};
use indexmap::IndexMap;
use itertools::Itertools;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use smol_str::SmolStr;
use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::Path,
};

impl DatabaseRoot {
    pub fn from_reader(reader: &mut (impl BufRead + Send)) -> eyre::Result<Self> {
        let mut lines = reader.lines();

        let Some(root_line) = lines.next() else {
            return Err(eyre!("database contains no entries"));
        };

        let mut root: DatabaseRoot = serde_json::from_str(&root_line?)?;

        let mut entries = lines
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

        entries.par_iter_mut().for_each(|x| {
            if x.sources.iter().any(|x| x.contains("myanimelist")) {
                x.metadata = matcher::extract_metadata_db(x);
            }
        });

        for mut entry in entries {
            let Some(id) = entry.sources.iter().find_map(|x| {
                x.contains("myanimelist")
                    .then(|| x.split("/").last().map(|x| x.parse::<u32>()))
            }) else {
                continue;
            };
            let id = id
                .ok_or_eyre("no id in mal source")?
                .wrap_err("invalid mal id")?;

            entry.id = id;
            if root.data.insert(id, entry).is_some() {
                return Err(eyre!("found duplicate mal id"));
            }
        }

        Ok(root)
    }

    pub fn from_path(path: impl AsRef<Path>) -> eyre::Result<Self> {
        let file = File::open(path)?;
        let mut buf_reader = BufReader::new(file);
        Self::from_reader(&mut buf_reader)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DatabaseRoot {
    pub last_update: NaiveDate,
    #[serde(default)]
    pub data: IndexMap<u32, AnimeEntry>,
}

/// Valid for every single line from the *.jsonl file except the first line which contains the meta data.
/// anime-offline-database
#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AnimeEntry {
    #[serde(default)]
    pub metadata: Vec<ExtractedMetadata>,
    #[serde(default)]
    pub id: u32,
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
