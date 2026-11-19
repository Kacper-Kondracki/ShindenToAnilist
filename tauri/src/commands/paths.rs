use std::{
    fs,
    path::Path,
};

use shinden_to_anilist_grpc::{
    error::ServiceError,
    pb::ErrorKind,
};
use tauri::{
    AppHandle,
    Manager,
};

use crate::{
    dto::{
        AppPathsDto,
        CommandErrorDto,
        command_error,
    },
    state::PRODUCT_NAME,
};

#[tauri::command]
pub(crate) fn app_paths(app: AppHandle) -> Result<AppPathsDto, CommandErrorDto> {
    let base = app
        .path()
        .data_dir()
        .map_err(|err| {
            command_error(ServiceError::caused_by(
                ErrorKind::AppPathsFailed,
                "resolve application data directory",
                err,
            ))
        })?
        .join(PRODUCT_NAME);

    fs::create_dir_all(&base).map_err(|err| {
        command_error(ServiceError::caused_by(
            ErrorKind::AppPathsFailed,
            format!("create {}", base.display()),
            err,
        ))
    })?;
    let export_dir = app.path().document_dir().unwrap_or_else(|_| base.clone());

    Ok(AppPathsDto {
        database: display_path(&base.join("database.jsonl")),
        export: display_path(&export_dir.join("export.xml")),
        base: display_path(&base),
    })
}

fn display_path(path: &Path) -> String { path.to_string_lossy().into_owned() }
