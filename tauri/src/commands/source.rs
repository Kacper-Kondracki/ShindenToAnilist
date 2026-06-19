use shinden_to_anilist_grpc::pb;
use tauri::{
    State,
    ipc::Channel,
};
use tokio_util::sync::CancellationToken;
use tonic::Status;

use crate::{
    dto::{
        FetchSourceListResponseDto,
        GetSourceFullResponseDto,
        GetSourceIdsResponseDto,
        SourceFetchProgressDto,
        command_error,
        wire_numbers,
    },
    state::AppState,
};

#[tauri::command(rename_all = "camelCase")]
pub(crate) async fn fetch_source_list(
    state: State<'_, AppState>,
    request_id: u64,
    provider: i32,
    user: String,
    on_progress: Channel<SourceFetchProgressDto>,
) -> Result<FetchSourceListResponseDto, String> {
    let cancellation_token = CancellationToken::new();
    {
        let mut cancellations = state
            .source_fetch_cancellations
            .lock()
            .map_err(|err| err.to_string())?;

        if let Some(previous) = cancellations.insert(request_id, cancellation_token.clone()) {
            previous.cancel();
        }
    }

    let result = state
        .service
        .fetch_source_list_with_progress(
            pb::FetchSourceListRequest { provider, user },
            cancellation_token,
            move |progress| {
                on_progress
                    .send(progress.into())
                    .map_err(|err| Status::internal(err.to_string()))
            },
        )
        .await
        .map(|response| FetchSourceListResponseDto {
            source_version: response.source_version.into(),
        })
        .map_err(command_error);

    state
        .source_fetch_cancellations
        .lock()
        .map_err(|err| err.to_string())?
        .remove(&request_id);

    result
}

#[tauri::command(rename_all = "camelCase")]
pub(crate) fn cancel_source_list_fetch(state: State<'_, AppState>, request_id: u64) -> Result<(), String> {
    if let Some(cancellation_token) = state
        .source_fetch_cancellations
        .lock()
        .map_err(|err| err.to_string())?
        .remove(&request_id)
    {
        cancellation_token.cancel();
    }

    Ok(())
}

#[tauri::command(rename_all = "camelCase")]
pub(crate) async fn get_source_ids(
    state: State<'_, AppState>,
    sorted_by: Option<i32>,
) -> Result<GetSourceIdsResponseDto, String> {
    state
        .service
        .get_source_ids(pb::GetSourceIdsRequest {
            sorted_by: sorted_by.unwrap_or_default(),
        })
        .await
        .map(|response| GetSourceIdsResponseDto {
            source_version: response.source_version.into(),
            ids: wire_numbers(response.ids),
        })
        .map_err(command_error)
}

#[tauri::command]
pub(crate) async fn get_source_full(state: State<'_, AppState>) -> Result<GetSourceFullResponseDto, String> {
    state
        .service
        .get_source_full(pb::GetSourceFullRequest {})
        .await
        .map(|response| GetSourceFullResponseDto {
            source_version: response.source_version.into(),
            entries: response.entries.into_iter().map(Into::into).collect(),
        })
        .map_err(command_error)
}
