use shinden_to_anilist_core::database::updater::{
    DatabaseUpdateStatus,
    update_latest_jsonl_zst_from_github,
};

#[tokio::main(flavor = "current_thread")]
async fn main() {
    match update_latest_jsonl_zst_from_github(reqwest::Client::new(), "anime-offline-database.jsonl.zst")
        .await
        .unwrap()
    {
        DatabaseUpdateStatus::UpToDate { release, sha256 } => {
            println!("anime-offline-database.jsonl.zst is up to date ({release}, {sha256})");
        },
        DatabaseUpdateStatus::Updated {
            release,
            sha256,
            path,
        } => {
            println!("updated {} from {release} ({sha256})", path.display());
            
        },
    }
}
