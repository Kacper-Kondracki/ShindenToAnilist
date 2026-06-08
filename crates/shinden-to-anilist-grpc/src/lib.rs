use std::path::Path;

use shinden_to_anilist_core::{
    database::{
        AnimeDatabase,
        AnimeDatabaseLoad,
        DatabaseError,
    },
    searcher::DefaultSearcher,
};

pub mod pb {
    tonic::include_proto!("shinden_to_anilist.v1");
}

pub mod mapper;
pub mod server;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Versioned<T> {
    pub version: u64,
    pub data: T,
}

impl<T> Versioned<T> {
    pub fn new(data: T) -> Self { Self { version: 0, data } }
    pub fn with_version(version: u64, data: T) -> Self { Self { version, data } }
    pub fn new_inc(previous: &Self, data: T) -> Self {
        Versioned {
            version: previous.version.wrapping_add(1),
            data,
        }
    }
}

#[derive(Debug)]
pub struct DatabaseState {
    pub database: AnimeDatabase,
    pub searcher: DefaultSearcher,
}

impl DatabaseState {
    fn load(path: impl AsRef<Path>) -> Result<Self, DatabaseError> {
        let database = AnimeDatabase::get_from_mmap(path)?;
        let searcher = DefaultSearcher::new(&database);

        Ok(Self { database, searcher })
    }
}
