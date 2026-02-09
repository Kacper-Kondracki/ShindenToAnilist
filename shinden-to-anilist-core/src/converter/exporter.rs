use std::{
    error::Error,
    io::Write,
};

use serde::{
    Deserialize,
    Serialize,
};

use crate::converter::common::{
    AnimeId,
    AnimeList,
    ExportView,
};

pub mod xml;

#[cfg(test)]
mod tests;

pub trait Exporter {
    type Error: Error;
    fn export(
        &self,
        anime_list: &impl AnimeList<Entry = impl ExportView>,
        entries: impl Iterator<Item = (AnimeId, AnimeId)>,
        writer: impl Write,
    ) -> Result<(), Self::Error>;
}

pub trait ExportExt<E: ExportView>: AnimeList<Entry = E> + Sized {
    fn export<T: Exporter>(
        &self,
        exporter: &T,
        entries: impl Iterator<Item = (AnimeId, AnimeId)>,
        writer: impl Write,
    ) -> Result<(), T::Error> {
        exporter.export(self, entries, writer)
    }
}
impl<E: ExportView, T: AnimeList<Entry = E>> ExportExt<E> for T {}

#[derive(Serialize, Deserialize, Debug, Copy, Clone, Eq, PartialEq)]
pub enum WatchStatus {
    Dropped,
    Completed,
    Watching,
    OnHold,
    PlanToWatch,
}
