use shinden_to_anilist_grpc::pb;
use tauri::State;

use crate::{
    dto::{
        CheckDatabaseUpdateResponseDto,
        DownloadDatabaseResponseDto,
        GetDatabaseEntriesResponseDto,
        GetDatabaseFullResponseDto,
        GetDatabaseIdsResponseDto,
        GetDatabaseMetadataResponseDto,
        LoadDatabaseResponseDto,
        WireNumberDto,
        command_error,
        wire_numbers,
    },
    state::AppState,
};

#[tauri::command(rename_all = "camelCase")]
pub(crate) async fn check_database_update(
    state: State<'_, AppState>,
    path: String,
) -> Result<CheckDatabaseUpdateResponseDto, String> {
    state
        .service
        .check_database_update(pb::CheckDatabaseUpdateRequest { path })
        .await
        .map(|response| CheckDatabaseUpdateResponseDto {
            status: response.status.map(Into::into),
        })
        .map_err(command_error)
}

#[tauri::command(rename_all = "camelCase")]
pub(crate) async fn download_database(
    state: State<'_, AppState>,
    path: String,
) -> Result<DownloadDatabaseResponseDto, String> {
    state
        .service
        .download_database(pb::DownloadDatabaseRequest { path })
        .await
        .map(|response| DownloadDatabaseResponseDto {
            status: response.status.map(Into::into),
        })
        .map_err(command_error)
}

#[tauri::command(rename_all = "camelCase")]
pub(crate) async fn load_database(
    state: State<'_, AppState>,
    path: String,
) -> Result<LoadDatabaseResponseDto, String> {
    state
        .service
        .load_database(pb::LoadDatabaseRequest { path })
        .await
        .map(|response| LoadDatabaseResponseDto {
            database_version: response.database_version.into(),
        })
        .map_err(command_error)
}

#[tauri::command(rename_all = "camelCase")]
pub(crate) async fn get_database_metadata(
    state: State<'_, AppState>,
    path: String,
) -> Result<GetDatabaseMetadataResponseDto, String> {
    state
        .service
        .get_database_metadata(pb::GetDatabaseMetadataRequest { path })
        .await
        .map(|response| GetDatabaseMetadataResponseDto {
            metadata: response.metadata.map(Into::into),
        })
        .map_err(command_error)
}

#[tauri::command(rename_all = "camelCase")]
pub(crate) async fn get_database_ids(
    state: State<'_, AppState>,
    sorted_by: Option<i32>,
) -> Result<GetDatabaseIdsResponseDto, String> {
    state
        .service
        .get_database_ids(pb::GetDatabaseIdsRequest {
            sorted_by: sorted_by.unwrap_or_default(),
        })
        .await
        .map(|response| GetDatabaseIdsResponseDto {
            database_version: response.database_version.into(),
            ids: wire_numbers(response.ids),
        })
        .map_err(command_error)
}

#[tauri::command(rename_all = "camelCase")]
pub(crate) async fn get_database_entries(
    state: State<'_, AppState>,
    ids: Vec<WireNumberDto>,
) -> Result<GetDatabaseEntriesResponseDto, String> {
    state
        .service
        .get_database_entries(pb::GetDatabaseEntriesRequest {
            ids: ids.into_iter().map(Into::into).collect(),
        })
        .await
        .map(|response| GetDatabaseEntriesResponseDto {
            database_version: response.database_version.into(),
            entries: response.entries.into_iter().map(Into::into).collect(),
        })
        .map_err(command_error)
}

#[tauri::command]
pub(crate) async fn get_database_full(
    state: State<'_, AppState>,
) -> Result<GetDatabaseFullResponseDto, String> {
    state
        .service
        .get_database_full(pb::GetDatabaseFullRequest {})
        .await
        .map(|response| GetDatabaseFullResponseDto {
            database_version: response.database_version.into(),
            entries: response.entries.into_iter().map(Into::into).collect(),
        })
        .map_err(command_error)
}
