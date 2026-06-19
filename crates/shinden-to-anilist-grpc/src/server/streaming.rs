use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tonic::Status;
use tracing::{
    info,
    warn,
};

const STREAM_CHUNK_SIZE: usize = 500;

pub(super) fn stream_batches<T, R>(
    version: u64,
    entries: Vec<T>,
    into_response: impl Fn(u64, Vec<T>) -> R + Send + 'static,
) -> ReceiverStream<Result<R, Status>>
where
    T: Send + 'static,
    R: Send + 'static,
{
    let total_entries = entries.len();
    let (tx, rx) = mpsc::channel(64);

    tokio::spawn(async move {
        let mut iter = entries.into_iter();
        let mut batches = 0usize;
        loop {
            let batch = iter.by_ref().take(STREAM_CHUNK_SIZE).collect::<Vec<_>>();
            if batch.is_empty() {
                break;
            }

            batches += 1;
            if tx.send(Ok(into_response(version, batch))).await.is_err() {
                warn!(
                    version,
                    total_entries,
                    batches_sent = batches,
                    "response stream receiver dropped"
                );
                break;
            }
        }

        info!(
            version,
            total_entries,
            batches_sent = batches,
            "response stream finished"
        );
    });

    ReceiverStream::new(rx)
}
