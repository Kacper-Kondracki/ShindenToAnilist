use shinden_to_anilist_core::{
    common::{
        AnimeId,
        AnimeList,
        MatchView,
    },
    providers::{
        animezone::AnimeZoneList,
        ogladajanime::OgladajAnimeList,
        shinden::ShindenList,
    },
};

use crate::pb::SourceProvider;

#[derive(Debug)]
pub(crate) enum SourceList {
    Shinden(ShindenList),
    AnimeZone(AnimeZoneList),
    OgladajAnime(OgladajAnimeList),
}

impl SourceList {
    pub(crate) fn provider(&self) -> SourceProvider {
        match self {
            Self::Shinden(_) => SourceProvider::Shinden,
            Self::AnimeZone(_) => SourceProvider::AnimeZone,
            Self::OgladajAnime(_) => SourceProvider::OgladajAnime,
        }
    }

    pub(crate) fn len(&self) -> usize {
        match self {
            Self::Shinden(list) => list.len(),
            Self::AnimeZone(list) => list.len(),
            Self::OgladajAnime(list) => list.len(),
        }
    }

    pub(crate) fn ids(&self) -> Vec<AnimeId> {
        match self {
            Self::Shinden(list) => list.keys().collect(),
            Self::AnimeZone(list) => list.keys().collect(),
            Self::OgladajAnime(list) => list.keys().collect(),
        }
    }

    pub(crate) fn match_view(&self, id: AnimeId) -> Option<&dyn MatchView> {
        match self {
            Self::Shinden(list) => list.get(id).map(|entry| entry as &dyn MatchView),
            Self::AnimeZone(list) => list.get(id).map(|entry| entry as &dyn MatchView),
            Self::OgladajAnime(list) => list.get(id).map(|entry| entry as &dyn MatchView),
        }
    }
}
