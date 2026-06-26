use std::io::Write;

use shinden_to_anilist_core::{
    common::{
        AnimeId,
        AnimeList,
        ExportView,
        MatchView,
    },
    exporter::{
        ExportExt,
        xml::{
            XmlExportError,
            XmlExporter,
        },
    },
    providers::{
        animezone::AnimeZoneList,
        ogladajanime::OgladajAnimeList,
        shinden::ShindenList,
    },
};

use crate::pb::{
    SourceEntry,
    SourceProvider,
};

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

    pub(crate) fn ids_by_urgency(&self) -> Vec<AnimeId> {
        match self {
            Self::Shinden(list) => {
                let mut ids = list
                    .iter()
                    .map(|(id, entry)| (id, entry.premiere_date()))
                    .collect::<Vec<_>>();
                ids.sort_by(|(_, date_a), (_, date_b)| match (date_a, date_b) {
                    (None, None) => std::cmp::Ordering::Equal,
                    (None, Some(_)) => std::cmp::Ordering::Less,
                    (Some(_), None) => std::cmp::Ordering::Greater,
                    (Some(date_a), Some(date_b)) => date_b.cmp(date_a),
                });
                ids.into_iter().map(|(id, _)| id).collect()
            },
            Self::AnimeZone(list) => list.keys().collect(),
            Self::OgladajAnime(list) => list.keys().collect(),
        }
    }

    pub(crate) fn entries(&self) -> Vec<SourceEntry> {
        match self {
            Self::Shinden(list) => list.values().map(SourceEntry::from).collect(),
            Self::AnimeZone(list) => list.values().map(SourceEntry::from).collect(),
            Self::OgladajAnime(list) => list.values().map(SourceEntry::from).collect(),
        }
    }

    pub(crate) fn match_view(&self, id: AnimeId) -> Option<&dyn MatchView> {
        match self {
            Self::Shinden(list) => list.get(id).map(|entry| entry as &dyn MatchView),
            Self::AnimeZone(list) => list.get(id).map(|entry| entry as &dyn MatchView),
            Self::OgladajAnime(list) => list.get(id).map(|entry| entry as &dyn MatchView),
        }
    }

    pub(crate) fn write_xml(
        &self,
        matches: impl Iterator<Item = (AnimeId, AnimeId)>,
        writer: impl Write,
    ) -> Result<(), XmlExportError> {
        match self {
            Self::Shinden(list) => write_xml(list, matches, writer),
            Self::AnimeZone(list) => write_xml(list, matches, writer),
            Self::OgladajAnime(list) => write_xml(list, matches, writer),
        }
    }
}

fn write_xml<E>(
    source: &impl AnimeList<Entry = E>,
    matches: impl Iterator<Item = (AnimeId, AnimeId)>,
    writer: impl Write,
) -> Result<(), XmlExportError>
where
    E: ExportView + Send + Sync,
{
    source.export(&XmlExporter {}, matches, writer)
}
