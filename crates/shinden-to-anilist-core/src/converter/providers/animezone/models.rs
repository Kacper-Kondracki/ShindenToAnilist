use ambassador::Delegate;
use chrono::NaiveDate;
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
use sha2::{
    Digest,
    Sha256,
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
pub struct AnimeZoneList {
    pub(super) entries: IndexMap<AnimeId, AnimeZoneEntry>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct AnimeZoneEntry {
    pub(super) id: AnimeId,
    pub(super) slug: CompactString,
    pub(super) title: CompactString,
    pub(super) normalized_title: CompactString,
    pub(super) metadata: TitleMetadata,
    pub(super) alternative_title: Option<CompactString>,
    pub(super) mal_id: Option<AnimeId>,
    pub(super) anime_status: Option<AnimeStatus>,
    pub(super) anime_type: Option<AnimeType>,
    pub(super) year: Option<i32>,
    pub(super) episodes: Option<i32>,
    pub(super) watch_status: WatchStatus,
    pub(super) section: AnimeZoneSection,
    pub(super) score: Option<i32>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, Eq, PartialEq)]
pub enum AnimeZoneSection {
    Rated,
    Watching,
    Plans,
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct AnimeZoneListItem {
    pub(super) section: AnimeZoneSection,
    pub(super) slug: CompactString,
    pub(super) title: CompactString,
    pub(super) score: Option<i32>,
    pub(super) site_status: Option<AnimeStatus>,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub(super) struct AnimeZoneDetail {
    pub(super) mal_id: Option<AnimeId>,
    pub(super) alternative_title: Option<CompactString>,
    pub(super) anime_status: Option<AnimeStatus>,
    pub(super) anime_type: Option<AnimeType>,
    pub(super) year: Option<i32>,
    pub(super) episodes: Option<i32>,
}

impl AnimeZoneList {
    pub fn from_entries(entries: impl IntoIterator<Item = AnimeZoneEntry>) -> Self {
        let mut map = IndexMap::new();

        for entry in entries {
            map.entry(entry.id)
                .and_modify(|existing: &mut AnimeZoneEntry| {
                    if entry.section.precedence() > existing.section.precedence() {
                        *existing = entry.clone();
                    } else {
                        existing.merge_missing_detail(&entry);
                    }
                })
                .or_insert(entry);
        }

        map.sort_unstable_keys();
        Self { entries: map }
    }

    pub fn direct_mal_matches(&self) -> impl Iterator<Item = (AnimeId, AnimeId)> + '_ {
        self.entries
            .iter()
            .filter_map(|(&id, entry)| entry.mal_id.map(|mal_id| (id, mal_id)))
    }

    pub fn entries_without_mal_id(&self) -> impl Iterator<Item = &AnimeZoneEntry> {
        self.entries.values().filter(|entry| entry.mal_id.is_none())
    }

    pub fn missing_mal_id_count(&self) -> usize { self.entries_without_mal_id().count() }
}

impl AnimeZoneEntry {
    pub fn id(&self) -> AnimeId { self.id }
    pub fn slug(&self) -> &str { &self.slug }
    pub fn title(&self) -> &str { &self.title }
    pub fn alternative_title(&self) -> Option<&str> { self.alternative_title.as_deref() }
    pub fn mal_id(&self) -> Option<AnimeId> { self.mal_id }
    pub fn anime_status(&self) -> Option<AnimeStatus> { self.anime_status }
    pub fn anime_type(&self) -> Option<AnimeType> { self.anime_type }
    pub fn year(&self) -> Option<i32> { self.year }
    pub fn episodes(&self) -> Option<i32> { self.episodes }
    pub fn watch_status(&self) -> WatchStatus { self.watch_status }
    pub fn section(&self) -> AnimeZoneSection { self.section }
    pub fn score(&self) -> Option<i32> { self.score }

    pub(super) fn from_scraped(item: AnimeZoneListItem, detail: Option<AnimeZoneDetail>) -> Self {
        let detail = detail.unwrap_or_default();
        let metadata = title_processor::process(&item.title);
        let normalized_title = normalize_str(&item.title);

        Self {
            id: animezone_id(&item.slug),
            slug: item.slug,
            title: item.title,
            normalized_title,
            metadata,
            alternative_title: detail.alternative_title,
            mal_id: detail.mal_id,
            anime_status: detail.anime_status.or(item.site_status),
            anime_type: detail.anime_type,
            year: detail.year,
            episodes: detail.episodes,
            watch_status: item.section.watch_status(),
            section: item.section,
            score: item.score,
        }
    }

    fn merge_missing_detail(&mut self, other: &Self) {
        self.alternative_title = self
            .alternative_title
            .clone()
            .or_else(|| other.alternative_title.clone());
        self.mal_id = self.mal_id.or(other.mal_id);
        self.anime_status = self.anime_status.or(other.anime_status);
        self.anime_type = self.anime_type.or(other.anime_type);
        self.year = self.year.or(other.year);
        self.episodes = self.episodes.or(other.episodes);
        self.score = self.score.or(other.score);
    }
}

impl AnimeZoneSection {
    pub const fn default_sections() -> &'static [Self; 3] { &[Self::Rated, Self::Watching, Self::Plans] }

    pub const fn path_segment(self) -> &'static str {
        match self {
            Self::Rated => "rated",
            Self::Watching => "watching",
            Self::Plans => "plans",
        }
    }

    pub const fn watch_status(self) -> WatchStatus {
        match self {
            Self::Rated => WatchStatus::Completed,
            Self::Watching => WatchStatus::Watching,
            Self::Plans => WatchStatus::PlanToWatch,
        }
    }

    const fn precedence(self) -> u8 {
        match self {
            Self::Rated => 3,
            Self::Watching => 2,
            Self::Plans => 1,
        }
    }
}

impl MatchView for AnimeZoneEntry {
    fn title(&self) -> &str { &self.title }
    fn normalized_title(&self) -> &str { &self.normalized_title }
    fn title_metadata(&self) -> Option<&TitleMetadata> { Some(&self.metadata) }
    fn year(&self) -> Option<Option<i32>> { Some(self.year) }
    fn anime_type(&self) -> Option<AnimeType> { self.anime_type }
    fn status(&self) -> Option<AnimeStatus> { self.anime_status }
    fn episodes(&self) -> Option<i32> { self.episodes }
}

impl ExportView for AnimeZoneEntry {
    fn watched_episodes(&self) -> i32 {
        match self.watch_status {
            WatchStatus::Completed => self.episodes.unwrap_or_default(),
            _ => 0,
        }
    }

    fn start_date(&self) -> Option<NaiveDate> { None }
    fn finish_date(&self) -> Option<NaiveDate> { None }
    fn score(&self) -> i32 { self.score.unwrap_or_default() }
    fn status(&self) -> WatchStatus { self.watch_status }
}

fn animezone_id(slug: &str) -> AnimeId {
    let digest = Sha256::digest(slug.as_bytes());
    u64::from_be_bytes(
        digest[..8]
            .try_into()
            .expect("sha256 digest always has at least eight bytes"),
    )
}

pub(super) fn clean_cell_text(value: &str) -> Option<CompactString> {
    let value = value.trim();
    (!value.is_empty() && !value.eq_ignore_ascii_case("brak")).then(|| value.to_compact_string())
}
