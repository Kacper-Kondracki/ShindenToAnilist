use shinden_to_anilist_grpc::pb;
use tauri::State;

use crate::{
    dto::{
        ExportXmlResponseDto,
        SourceIdPairDto,
        WireNumberDto,
        command_error,
    },
    state::AppState,
};

#[tauri::command(rename_all = "camelCase")]
pub(crate) async fn export_xml(
    state: State<'_, AppState>,
    path: String,
    matches: Vec<SourceIdPairDto>,
    expected_source_version: Option<WireNumberDto>,
    expected_database_version: Option<WireNumberDto>,
) -> Result<ExportXmlResponseDto, crate::dto::CommandErrorDto> {
    state
        .service
        .export_xml(pb::ExportXmlRequest {
            path,
            matches: matches.into_iter().map(Into::into).collect(),
            expected_source_version: expected_source_version.map(Into::into),
            expected_database_version: expected_database_version.map(Into::into),
        })
        .await
        .map(|response| ExportXmlResponseDto {
            source_version: response.source_version.into(),
            shinden_version: response.shinden_version.into(),
            path: response.path,
        })
        .map_err(command_error)
}
