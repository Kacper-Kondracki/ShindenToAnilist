mod commands;
mod dto;
mod state;

use commands::{
    browser_session::open_shinden_cloudflare_verification,
    database::{
        check_database_update,
        download_database,
        get_database_entries,
        get_database_full,
        get_database_ids,
        get_database_metadata,
        load_database,
    },
    export::export_xml,
    matching::{
        fuzzy_match,
        fuzzy_search,
        match_shinden_list,
        match_source_list,
    },
    paths::app_paths,
    shinden::{
        fetch_shinden_list,
        get_shinden_entries,
        get_shinden_full,
        get_shinden_ids,
        set_shinden_cloudflare_clearance,
    },
    source::{
        cancel_source_list_fetch,
        fetch_source_list,
        get_source_full,
        get_source_ids,
    },
};
use state::AppState;
#[tauri::command]
fn report_renderer_failure(operation: &str, detail: &str) {
    if !operation.is_empty()
        && operation.len() <= 64
        && operation
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
    {
        let detail: String = detail.chars().take(5000).collect();
        tracing::error!(operation, ?detail, "renderer operation failed");
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    shinden_to_anilist_grpc::diagnostics::init("tauri", env!("CARGO_PKG_VERSION"));

    tauri::Builder::default()
        .plugin(tauri_plugin_wayland_nvidia_quirk::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .manage(AppState::new().expect("failed to build app HTTP clients"))
        .invoke_handler(tauri::generate_handler![
            app_paths,
            report_renderer_failure,
            fetch_source_list,
            cancel_source_list_fetch,
            get_source_ids,
            get_source_full,
            fetch_shinden_list,
            set_shinden_cloudflare_clearance,
            open_shinden_cloudflare_verification,
            get_shinden_ids,
            get_shinden_entries,
            get_shinden_full,
            check_database_update,
            download_database,
            load_database,
            get_database_metadata,
            get_database_ids,
            get_database_entries,
            get_database_full,
            fuzzy_search,
            fuzzy_match,
            match_source_list,
            match_shinden_list,
            export_xml
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
