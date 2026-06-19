use std::{
    fs,
    path::Path,
};

use tauri::{
    AppHandle,
    Manager,
};

use crate::{
    dto::AppPathsDto,
    state::PRODUCT_NAME,
};

#[tauri::command]
pub(crate) fn app_paths(app: AppHandle) -> Result<AppPathsDto, String> {
    let base = app
        .path()
        .data_dir()
        .map_err(|err| err.to_string())?
        .join(PRODUCT_NAME);

    fs::create_dir_all(&base).map_err(|err| err.to_string())?;
    let export_dir = app.path().document_dir().unwrap_or_else(|_| base.clone());

    Ok(AppPathsDto {
        database: display_path(&base.join("database.jsonl")),
        export: display_path(&export_dir.join("export.xml")),
        base: display_path(&base),
    })
}

fn display_path(path: &Path) -> String { path.to_string_lossy().into_owned() }
