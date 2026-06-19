use shinden_to_anilist_grpc::pb;
use tauri::State;

use crate::{
    dto::{
        FetchShindenListResponseDto,
        GetShindenEntriesResponseDto,
        GetShindenFullResponseDto,
        GetShindenIdsResponseDto,
        WireNumberDto,
        command_error,
        wire_numbers,
    },
    state::AppState,
};

#[tauri::command(rename_all = "camelCase")]
pub(crate) async fn fetch_shinden_list(
    state: State<'_, AppState>,
    id: WireNumberDto,
) -> Result<FetchShindenListResponseDto, String> {
    state
        .service
        .fetch_shinden_list(pb::FetchShindenListRequest { id: id.into() })
        .await
        .map(|response| FetchShindenListResponseDto {
            shinden_version: response.shinden_version.into(),
        })
        .map_err(command_error)
}

#[tauri::command(rename_all = "camelCase")]
pub(crate) async fn get_shinden_ids(
    state: State<'_, AppState>,
    sorted_by: Option<i32>,
) -> Result<GetShindenIdsResponseDto, String> {
    state
        .service
        .get_shinden_ids(pb::GetShindenIdsRequest {
            sorted_by: sorted_by.unwrap_or_default(),
        })
        .await
        .map(|response| GetShindenIdsResponseDto {
            shinden_version: response.shinden_version.into(),
            ids: wire_numbers(response.ids),
        })
        .map_err(command_error)
}

#[tauri::command(rename_all = "camelCase")]
pub(crate) async fn get_shinden_entries(
    state: State<'_, AppState>,
    ids: Vec<WireNumberDto>,
) -> Result<GetShindenEntriesResponseDto, String> {
    state
        .service
        .get_shinden_entries(pb::GetShindenEntriesRequest {
            ids: ids.into_iter().map(Into::into).collect(),
        })
        .await
        .map(|response| GetShindenEntriesResponseDto {
            shinden_version: response.shinden_version.into(),
            entries: response.entries.into_iter().map(Into::into).collect(),
        })
        .map_err(command_error)
}

#[tauri::command]
pub(crate) async fn get_shinden_full(
    state: State<'_, AppState>,
) -> Result<GetShindenFullResponseDto, String> {
    state
        .service
        .get_shinden_full(pb::GetShindenFullRequest {})
        .await
        .map(|response| GetShindenFullResponseDto {
            shinden_version: response.shinden_version.into(),
            entries: response.entries.into_iter().map(Into::into).collect(),
        })
        .map_err(command_error)
}
