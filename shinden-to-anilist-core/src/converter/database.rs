use std::{
    fs::File,
    io::{
        self,
        BufRead,
        BufReader,
        Read,
    },
    path::Path,
};

use indexmap::IndexMap;
use itertools::Itertools;
use memmap2::Mmap;
use rayon::prelude::*;
use thiserror::Error;

pub use self::models::*;
use crate::converter::common::AnimeId;

mod json;

pub mod models;

#[cfg(test)]
mod tests;

#[derive(Error, Debug)]
#[error(transparent)]
pub enum DatabaseError {
    Io(#[from] io::Error),
    Json(#[from] serde_json::Error),
    #[error("can not parse empty file")]
    Empty,
}

pub trait AnimeDatabaseLoad {
    fn get_from_mmap(path: impl AsRef<Path>) -> Result<AnimeDatabase, DatabaseError>;
    fn get_from_reader(reader: impl Read) -> Result<AnimeDatabase, DatabaseError>;
    fn get_from_path(reader: impl AsRef<Path>) -> Result<AnimeDatabase, DatabaseError>;
}

impl AnimeDatabaseLoad for AnimeDatabase {
    fn get_from_mmap(path: impl AsRef<Path>) -> Result<AnimeDatabase, DatabaseError> {
        let file = File::open(path)?;
        let mmap = unsafe { Mmap::map(&file)? };

        let (header, after_header) = match mmap.iter().position(|&b| b == b'\n') {
            Some(pos) => mmap.split_at(pos + 1),
            None => return Err(DatabaseError::Empty),
        };

        let mut db_root = serde_json::from_slice::<json::DatabaseRoot>(header)?.into_model();

        let entries = after_header
            .par_split(|&b| b == b'\n')
            .filter_map(|line| match serde_json::from_slice::<json::AnimeEntry>(line) {
                Ok(v) => v.into_model().map(|a| (a.id, a)).map(Ok),
                Err(e) => Some(Err(DatabaseError::from(e))),
            })
            .collect::<Result<IndexMap<AnimeId, AnimeEntry>, DatabaseError>>()?;

        db_root.entries = entries;

        Ok(db_root)
    }
    fn get_from_reader(reader: impl Read) -> Result<AnimeDatabase, DatabaseError> {
        let buf_reader = BufReader::new(reader);

        let mut lines = buf_reader.lines();

        let mut db_root = serde_json::from_str::<json::DatabaseRoot>(
            &lines.next().ok_or(DatabaseError::Empty)??,
        )?
        .into_model();

        db_root.entries.extend(
            lines
                .chunks(512 * 4)
                .into_iter()
                .map(|c| {
                    c.collect::<Vec<_>>()
                        .into_par_iter()
                        .filter_map(|s| match s {
                            Ok(s) => match serde_json::from_str::<json::AnimeEntry>(&s) {
                                Ok(v) => v.into_model().map(|a| (a.id, a)).map(|a| Ok(Ok(a))),
                                Err(e) => Some(Ok(Err(DatabaseError::from(e)))),
                            },
                            Err(e) => Some(Err(DatabaseError::from(e))),
                        })
                        .flatten()
                        .collect::<Result<IndexMap<AnimeId, AnimeEntry>, DatabaseError>>()
                })
                .flatten_ok()
                .collect::<Result<IndexMap<AnimeId, AnimeEntry>, DatabaseError>>()?,
        );

        Ok(db_root)
    }

    fn get_from_path(path: impl AsRef<Path>) -> Result<AnimeDatabase, DatabaseError> {
        let file = File::open(path)?;
        Self::get_from_reader(file)
    }
}
