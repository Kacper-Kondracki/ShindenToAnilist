use crate::{
    converter::matcher::ExtractedMetadata,
    converter::{database, exporter},
};
use chrono::NaiveDate;
use indexmap::IndexMap;
use std::hash::DefaultHasher;

pub trait MatchView {
    fn title(&self) -> &str;
    fn extracted_metadata(&self) -> Option<ExtractedMetadata> {
        None
    }
    fn year(&self) -> Option<i32> {
        None
    }
    fn date(&self) -> Option<NaiveDate> {
        None
    }
    fn anime_type(&self) -> Option<database::AnimeType> {
        None
    }
    fn status(&self) -> Option<database::AnimeStatus> {
        None
    }
    fn episodes(&self) -> Option<i32> {
        None
    }
}

impl<T: AsRef<str>> MatchView for T {
    fn title(&self) -> &str {
        self.as_ref()
    }
}

pub trait ExportView {
    fn watched_episodes(&self) -> i32 {
        0
    }
    fn start_date(&self) -> Option<NaiveDate> {
        None
    }
    fn finish_date(&self) -> Option<NaiveDate> {
        None
    }
    fn score(&self) -> i32 {
        0
    }
    fn status(&self) -> exporter::StatusXml;
    fn comments(&self) -> Option<&str> {
        None
    }
}

pub type AnimeId = usize;
pub type AnimeList<T> = IndexMap<AnimeId, T>;

pub fn hash_title(title: &str) -> AnimeId {
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    title.hash(&mut hasher);
    hasher.finish() as AnimeId
}

pub trait IntoAnimeList<T> {
    fn into_anime_list(self) -> AnimeList<T>;
}

impl<I, S> IntoAnimeList<S> for I
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    fn into_anime_list(self) -> AnimeList<S> {
        self.into_iter()
            .map(|s| (hash_title(s.as_ref()), s))
            .collect()
    }
}
