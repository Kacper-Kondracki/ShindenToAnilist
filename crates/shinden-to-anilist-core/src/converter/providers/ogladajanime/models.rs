use ambassador::Delegate;
use chrono::{
    Datelike,
    NaiveDate,
};
use compact_str::{
    CompactString,
    ToCompactString,
};
use indexmap::IndexMap;
use rayon::prelude::*;
use serde::{
    Deserialize,
    Serialize,
};

use crate::{
    converter::{
        common::{
            AnimeId,
            AnimeList,
            ExportView,
            MatchView,
            ambassador_impl_AnimeList,
        },
        database::{
            AnimeStatus,
            AnimeType,
        },
        exporter::WatchStatus,
        extractor::{
            TitleMetadata,
            title_processor,
        },
    },
    utils::normalize_str,
};

#[derive(Serialize, Deserialize, Debug, Clone, Delegate, PartialEq)]
#[delegate(AnimeList, target = "entries")]
pub struct OgladajAnimeList {
    pub(super) entries: IndexMap<AnimeId, OgladajAnimeEntry>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct OgladajAnimeEntry {
    pub(super) id: AnimeId,
    pub(super) slug: CompactString,
    pub(super) title: CompactString,
    pub(super) normalized_title: CompactString,
    pub(super) metadata: TitleMetadata,
    pub(super) original_title: Option<CompactString>,
    pub(super) mal_id: Option<AnimeId>,
    pub(super) anime_status: Option<AnimeStatus>,
    pub(super) anime_type: Option<AnimeType>,
    pub(super) premiere_date: Option<NaiveDate>,
    pub(super) finish_date: Option<NaiveDate>,
    pub(super) episodes: Option<i32>,
    pub(super) watch_status: WatchStatus,
    pub(super) watched_episodes: i32,
    pub(super) score: Option<i32>,
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct OgladajAnimeListItem {
    pub(super) id: AnimeId,
    pub(super) slug: CompactString,
    pub(super) title: CompactString,
    pub(super) anime_type: Option<AnimeType>,
    pub(super) watch_status: WatchStatus,
    pub(super) watched_episodes: i32,
    pub(super) total_episodes: Option<i32>,
    pub(super) score: Option<i32>,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub(super) struct OgladajAnimeDetail {
    pub(super) mal_id: Option<AnimeId>,
    pub(super) original_title: Option<CompactString>,
    pub(super) anime_status: Option<AnimeStatus>,
    pub(super) anime_type: Option<AnimeType>,
    pub(super) premiere_date: Option<NaiveDate>,
    pub(super) finish_date: Option<NaiveDate>,
    pub(super) episodes: Option<i32>,
}

impl OgladajAnimeList {
    pub fn from_entries(entries: impl IntoIterator<Item = OgladajAnimeEntry>) -> Self {
        let mut map = IndexMap::new();

        for entry in entries {
            map.entry(entry.id)
                .and_modify(|existing: &mut OgladajAnimeEntry| existing.merge_missing_detail(&entry))
                .or_insert(entry);
        }

        Self { entries: map }
    }

    pub fn direct_mal_matches(&self) -> impl Iterator<Item = (AnimeId, AnimeId)> + '_ {
        self.entries
            .iter()
            .filter_map(|(&id, entry)| entry.mal_id.map(|mal_id| (id, mal_id)))
    }

    pub fn entries_without_mal_id(&self) -> impl Iterator<Item = &OgladajAnimeEntry> {
        self.entries.values().filter(|entry| entry.mal_id.is_none())
    }

    pub fn missing_mal_id_count(&self) -> usize { self.entries_without_mal_id().count() }
}

impl OgladajAnimeEntry {
    pub fn id(&self) -> AnimeId { self.id }
    pub fn slug(&self) -> &str { &self.slug }
    pub fn title(&self) -> &str { &self.title }
    pub fn original_title(&self) -> Option<&str> { self.original_title.as_deref() }
    pub fn mal_id(&self) -> Option<AnimeId> { self.mal_id }
    pub fn anime_status(&self) -> Option<AnimeStatus> { self.anime_status }
    pub fn anime_type(&self) -> Option<AnimeType> { self.anime_type }
    pub fn premiere_date(&self) -> Option<NaiveDate> { self.premiere_date }
    pub fn finish_date(&self) -> Option<NaiveDate> { self.finish_date }
    pub fn year(&self) -> Option<i32> { self.premiere_date.map(|date| date.year()) }
    pub fn episodes(&self) -> Option<i32> { self.episodes }
    pub fn watch_status(&self) -> WatchStatus { self.watch_status }
    pub fn watched_episodes(&self) -> i32 { self.watched_episodes }
    pub fn score(&self) -> Option<i32> { self.score }

    pub(super) fn from_scraped(item: OgladajAnimeListItem, detail: Option<OgladajAnimeDetail>) -> Self {
        let detail = detail.unwrap_or_default();
        let metadata = title_processor::process(&item.title);
        let normalized_title = normalize_str(&item.title);
        let episodes = detail.episodes.or(item.total_episodes);

        Self {
            id: item.id,
            slug: item.slug,
            title: item.title,
            normalized_title,
            metadata,
            original_title: detail.original_title,
            mal_id: detail.mal_id,
            anime_status: detail.anime_status,
            anime_type: detail.anime_type.or(item.anime_type),
            premiere_date: detail.premiere_date,
            finish_date: detail.finish_date,
            episodes,
            watch_status: item.watch_status,
            watched_episodes: clamp_watched_episodes(item.watched_episodes, episodes),
            score: item.score,
        }
    }

    fn merge_missing_detail(&mut self, other: &Self) {
        self.original_title = self
            .original_title
            .clone()
            .or_else(|| other.original_title.clone());
        self.mal_id = self.mal_id.or(other.mal_id);
        self.anime_status = self.anime_status.or(other.anime_status);
        self.anime_type = self.anime_type.or(other.anime_type);
        self.premiere_date = self.premiere_date.or(other.premiere_date);
        self.finish_date = self.finish_date.or(other.finish_date);
        self.episodes = self.episodes.or(other.episodes);
        self.watched_episodes = clamp_watched_episodes(self.watched_episodes, self.episodes);
        self.score = self.score.or(other.score);
    }
}

fn clamp_watched_episodes(watched_episodes: i32, episodes: Option<i32>) -> i32 {
    match episodes {
        Some(episodes) if episodes >= 0 => watched_episodes.clamp(0, episodes),
        _ => watched_episodes,
    }
}

impl MatchView for OgladajAnimeEntry {
    fn title(&self) -> &str { &self.title }
    fn normalized_title(&self) -> &str { &self.normalized_title }
    fn title_metadata(&self) -> Option<&TitleMetadata> { Some(&self.metadata) }
    fn date(&self) -> Option<Option<NaiveDate>> { Some(self.premiere_date) }
    fn anime_type(&self) -> Option<AnimeType> { self.anime_type }
    fn status(&self) -> Option<AnimeStatus> { self.anime_status }
    fn episodes(&self) -> Option<i32> { self.episodes }
}

impl ExportView for OgladajAnimeEntry {
    fn watched_episodes(&self) -> i32 { self.watched_episodes }
    fn start_date(&self) -> Option<NaiveDate> { None }
    fn finish_date(&self) -> Option<NaiveDate> { self.finish_date }
    fn score(&self) -> i32 { self.score.unwrap_or_default() }
    fn status(&self) -> WatchStatus { self.watch_status }
}

pub(super) fn clean_cell_text(value: &str) -> Option<CompactString> {
    let value = value.trim();
    (!value.is_empty() && !value.eq_ignore_ascii_case("brak")).then(|| value.to_compact_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::converter::database::AnimeType;

    #[test]
    fn clamps_watched_episodes_to_detail_episode_count() {
        let entry = OgladajAnimeEntry::from_scraped(
            OgladajAnimeListItem {
                id: 12913,
                slug: "douluo-dalu-2nd-season".into(),
                title: "Douluo Dalu 2nd Season".into(),
                anime_type: Some(AnimeType::Ona),
                watch_status: WatchStatus::Completed,
                watched_episodes: 238,
                total_episodes: Some(238),
                score: Some(8),
            },
            Some(OgladajAnimeDetail {
                episodes: Some(52),
                ..OgladajAnimeDetail::default()
            }),
        );

        assert_eq!(entry.episodes(), Some(52));
        assert_eq!(entry.watched_episodes(), 52);
    }
}
