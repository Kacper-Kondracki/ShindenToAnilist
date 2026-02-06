use std::hash::DefaultHasher;

use ambassador::delegatable_trait;
use chrono::{
    Datelike,
    NaiveDate,
};
use rayon::prelude::*;

use crate::converter::{
    database,
    exporter,
    extractor::TitleMetadata,
};

pub type AnimeId = usize;

pub trait MatchView {
    fn title(&self) -> &str;
    fn title_metadata(&self) -> Option<&TitleMetadata> { None }
    fn year(&self) -> Option<i32> { self.date().map(|d| d.year()) }
    fn date(&self) -> Option<NaiveDate> { None }
    fn anime_type(&self) -> Option<database::AnimeType> { None }
    fn status(&self) -> Option<database::AnimeStatus> { None }
    fn episodes(&self) -> Option<i32> { None }
}

pub trait ExportView {
    fn watched_episodes(&self) -> i32 { 0 }
    fn start_date(&self) -> Option<NaiveDate> { None }
    fn finish_date(&self) -> Option<NaiveDate> { None }
    fn score(&self) -> i32 { 0 }
    fn status(&self) -> exporter::WatchStatus;
    fn comments(&self) -> Option<&str> { None }
}

pub fn hash_title(title: &str) -> AnimeId {
    use std::hash::{
        Hash,
        Hasher,
    };
    let mut hasher = DefaultHasher::new();
    title.hash(&mut hasher);
    hasher.finish() as AnimeId
}

#[delegatable_trait]
pub trait AnimeList: Send + Sync {
    type Entry: Send + Sync;
    fn keys(&self) -> impl Iterator<Item = AnimeId> + '_;
    fn par_keys(&self) -> impl ParallelIterator<Item = AnimeId> + '_;
    fn values(&self) -> impl Iterator<Item = &Self::Entry> + '_;
    fn into_values(self) -> impl Iterator<Item = Self::Entry>;
    fn par_values(&self) -> impl ParallelIterator<Item = &Self::Entry> + '_;
    fn into_par_values(self) -> impl IntoParallelIterator<Item = Self::Entry>;
    fn iter(&self) -> impl Iterator<Item = (AnimeId, &Self::Entry)> + '_;
    fn par_iter(&self) -> impl ParallelIterator<Item = (AnimeId, &Self::Entry)> + '_;
    fn into_iter(self) -> impl IntoIterator<Item = (AnimeId, Self::Entry)>;
    fn into_par_iter(self) -> impl IntoParallelIterator<Item = (AnimeId, Self::Entry)>;
    fn get(&self, key: AnimeId) -> Option<&Self::Entry>;
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool { self.len() == 0 }
}

pub mod impls {
    use indexmap::IndexMap;

    use super::*;

    impl<E> AnimeList for IndexMap<AnimeId, E>
    where
        E: Sync + Send,
    {
        type Entry = E;
        fn keys(&self) -> impl Iterator<Item = AnimeId> { self.keys().copied() }
        fn par_keys(&self) -> impl ParallelIterator<Item = AnimeId> { self.par_keys().copied() }
        fn values(&self) -> impl Iterator<Item = &Self::Entry> { self.values() }
        fn into_values(self) -> impl Iterator<Item = Self::Entry> { self.into_values() }
        fn par_values(&self) -> impl ParallelIterator<Item = &Self::Entry> { self.par_values() }
        fn into_par_values(self) -> impl IntoParallelIterator<Item = Self::Entry> {
            IntoParallelIterator::into_par_iter(self).map(|(_, v)| v)
        }
        fn iter(&self) -> impl Iterator<Item = (AnimeId, &Self::Entry)> {
            self.iter().map(|(&k, v)| (k, v))
        }
        fn par_iter(&self) -> impl ParallelIterator<Item = (AnimeId, &Self::Entry)> {
            IntoParallelRefIterator::par_iter(self).map(|(&k, v)| (k, v))
        }
        fn into_iter(self) -> impl IntoIterator<Item = (AnimeId, Self::Entry)> {
            IntoIterator::into_iter(self)
        }
        fn into_par_iter(self) -> impl IntoParallelIterator<Item = (AnimeId, Self::Entry)> {
            IntoParallelIterator::into_par_iter(self)
        }
        fn get(&self, key: AnimeId) -> Option<&Self::Entry> { IndexMap::get(self, &key) }
        fn len(&self) -> usize { IndexMap::len(self) }
    }
}
