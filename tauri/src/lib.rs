mod commands;
mod dto;
mod state;

use commands::{
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
    },
    source::{
        cancel_source_list_fetch,
        fetch_source_list,
        get_source_full,
        get_source_ids,
    },
};
use state::AppState;
use tracing_subscriber::{
    EnvFilter,
    fmt,
};

fn init_tracing() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        EnvFilter::new(
            "shinden_to_anilist_core=info,shinden_to_anilist_grpc=info,shinden_to_anilist_tauri=info",
        )
    });

    let _ = fmt().with_env_filter(filter).try_init();
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    init_tracing();

    let client = reqwest::Client::builder()
        .cookie_store(true)
        .build()
        .expect("failed to build HTTP client");

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .manage(AppState::new(client))
        .invoke_handler(tauri::generate_handler![
            app_paths,
            fetch_source_list,
            cancel_source_list_fetch,
            get_source_ids,
            get_source_full,
            fetch_shinden_list,
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
