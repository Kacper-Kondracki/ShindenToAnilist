use shinden_to_anilist_core::common::MatchView;
use tokio_stream::{
    Stream,
    StreamExt,
};
use tokio_util::sync::CancellationToken;
use tonic::Status;

use crate::{
    error::IntoStatus,
    pb::{
        SourceFetchPhase,
        SourceFetchProgress,
        SourceProvider,
    },
};

pub(super) trait ScrapedSourceFetchEvent {
    type Entry: MatchView + Send;

    fn into_parts(self) -> ScrapedSourceFetchEventParts<Self::Entry>;
}

pub(super) enum ScrapedSourceFetchEventParts<Entry> {
    Started {
        total_entries: usize,
    },
    Entry {
        current: usize,
        total_entries: usize,
        entry: Entry,
    },
}

pub(super) async fn collect_entries<E, S, Err>(
    provider: SourceProvider,
    mut stream: S,
    cancellation_token: CancellationToken,
    emit_progress: &mut impl FnMut(SourceFetchProgress) -> Result<(), Status>,
) -> Result<(Vec<E::Entry>, u64), Status>
where
    E: ScrapedSourceFetchEvent,
    S: Stream<Item = Result<E, Err>> + Unpin,
    Err: IntoStatus,
{
    emit_progress(super::super::source::source_progress(
        provider,
        SourceFetchPhase::FetchingList,
        0,
        0,
        "",
    ))?;

    let mut entries = Vec::new();
    let mut total_entries = 0u64;
    loop {
        let event = tokio::select! {
            () = cancellation_token.cancelled() => {
                return Err(Status::cancelled("source fetch cancelled"));
            },
            event = stream.next() => event,
        };

        let Some(event) = event else {
            break;
        };

        match event.map_err(IntoStatus::into_status)?.into_parts() {
            ScrapedSourceFetchEventParts::Started { total_entries: total } => {
                total_entries = total as u64;
                emit_progress(super::super::source::source_progress(
                    provider,
                    SourceFetchPhase::FetchingDetails,
                    0,
                    total_entries,
                    "",
                ))?;
            },
            ScrapedSourceFetchEventParts::Entry {
                current,
                total_entries: total,
                entry,
            } => {
                total_entries = total as u64;
                let latest_title = entry.title().to_string();
                entries.push(entry);
                emit_progress(super::super::source::source_progress(
                    provider,
                    SourceFetchPhase::FetchingDetails,
                    current as u64,
                    total_entries,
                    latest_title,
                ))?;
            },
        }
    }

    Ok((entries, total_entries))
}
