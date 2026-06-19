use shinden_to_anilist_grpc::pb;
use tauri::State;

use crate::{
    dto::{
        FuzzyMatchResponseDto,
        FuzzySearchResponseDto,
        MatchShindenListResponseDto,
        MatchSourceListResponseDto,
        SearchOptionsDto,
        WireNumberDto,
        command_error,
    },
    state::AppState,
};

#[tauri::command(rename_all = "camelCase")]
pub(crate) async fn fuzzy_search(
    state: State<'_, AppState>,
    query: String,
    options: Option<SearchOptionsDto>,
) -> Result<FuzzySearchResponseDto, String> {
    state
        .service
        .fuzzy_search(pb::FuzzySearchRequest {
            query,
            options: Some(options.unwrap_or_default().into()),
        })
        .await
        .map(|response| FuzzySearchResponseDto {
            database_version: response.database_version.into(),
            results: response.results.into_iter().map(Into::into).collect(),
        })
        .map_err(command_error)
}

#[tauri::command(rename_all = "camelCase")]
pub(crate) async fn fuzzy_match(
    state: State<'_, AppState>,
    query: String,
    options: Option<SearchOptionsDto>,
    shinden_id: Option<WireNumberDto>,
    source_id: Option<WireNumberDto>,
) -> Result<FuzzyMatchResponseDto, String> {
    state
        .service
        .fuzzy_match(pb::FuzzyMatchRequest {
            query,
            options: Some(options.unwrap_or_default().into()),
            shinden_id: shinden_id.map(Into::into),
            source_id: source_id.map(Into::into),
        })
        .await
        .map(|response| FuzzyMatchResponseDto {
            database_version: response.database_version.into(),
            results: response.results.into_iter().map(Into::into).collect(),
        })
        .map_err(command_error)
}

#[tauri::command(rename_all = "camelCase")]
pub(crate) async fn match_shinden_list(
    state: State<'_, AppState>,
    options: Option<SearchOptionsDto>,
) -> Result<MatchShindenListResponseDto, String> {
    state
        .service
        .match_shinden_list(pb::MatchShindenListRequest {
            options: Some(options.unwrap_or_default().into()),
        })
        .await
        .map(|response| MatchShindenListResponseDto {
            shinden_version: response.shinden_version.into(),
            database_version: response.database_version.into(),
            results: response.results.into_iter().map(Into::into).collect(),
        })
        .map_err(command_error)
}

#[tauri::command(rename_all = "camelCase")]
pub(crate) async fn match_source_list(
    state: State<'_, AppState>,
    options: Option<SearchOptionsDto>,
) -> Result<MatchSourceListResponseDto, String> {
    state
        .service
        .match_source_list(pb::MatchSourceListRequest {
            options: Some(options.unwrap_or_default().into()),
        })
        .await
        .map(|response| MatchSourceListResponseDto {
            source_version: response.source_version.into(),
            database_version: response.database_version.into(),
            results: response.results.into_iter().map(Into::into).collect(),
        })
        .map_err(command_error)
}
