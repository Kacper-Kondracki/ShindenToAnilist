use shinden_to_anilist_grpc::pb;
use tauri::State;

use crate::{
    dto::{
        ExportXmlResponseDto,
        SourceIdPairDto,
        command_error,
    },
    state::AppState,
};

#[tauri::command(rename_all = "camelCase")]
pub(crate) async fn export_xml(
    state: State<'_, AppState>,
    path: String,
    matches: Vec<SourceIdPairDto>,
) -> Result<ExportXmlResponseDto, String> {
    state
        .service
        .export_xml(pb::ExportXmlRequest {
            path,
            matches: matches.into_iter().map(Into::into).collect(),
        })
        .await
        .map(|response| ExportXmlResponseDto {
            source_version: response.source_version.into(),
            shinden_version: response.shinden_version.into(),
            path: response.path,
        })
        .map_err(command_error)
}
