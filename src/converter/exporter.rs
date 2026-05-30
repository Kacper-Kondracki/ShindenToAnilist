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

/// Trait for converting matched anime entries into a target export format.
///
/// # Example
///
/// See [`xml::XmlExporter`] for the built-in MAL XML implementation.
pub trait Exporter {
    /// The error type produced by this exporter.
    type Error: Error;

    /// Writes the matched entries to `writer` in the exporter's format.
    ///
    /// `entries` is an iterator of `(shinden_id, database_id)` pairs that
    /// have been successfully matched. The exporter uses `shinden_id` to
    /// look up watch data from `anime_list` and `database_id` as the
    /// external identifier in the output.
    fn export(
        &self,
        anime_list: &impl AnimeList<Entry = impl ExportView>,
        entries: impl Iterator<Item = (AnimeId, AnimeId)>,
        writer: impl Write,
    ) -> Result<(), Self::Error>;
}

/// Convenience extension trait that lets any [`AnimeList`] with [`ExportView`]
/// entries call `.export(...)` directly.
///
/// This is blanket-implemented for all `T: AnimeList<Entry = impl ExportView>`,
/// so you never need to implement it manually.
pub trait ExportExt<E: ExportView>: AnimeList<Entry = E> + Sized {
    /// Delegates to [`Exporter::export`] with `self` as the anime list.
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

/// The user's watch status for an anime entry.
#[derive(Serialize, Deserialize, Debug, Copy, Clone, Eq, PartialEq)]
pub enum WatchStatus {
    /// The user dropped the anime.
    Dropped,
    /// The user finished watching.
    Completed,
    /// The user is currently watching.
    Watching,
    /// The user paused watching.
    OnHold,
    /// The user intends to watch in the future.
    PlanToWatch,
}
