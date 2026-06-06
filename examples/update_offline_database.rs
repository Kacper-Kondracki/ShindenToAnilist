use shinden_to_anilist_core::database::updater::{
    update_latest_jsonl_from_github_blocking, DatabaseUpdateStatus,
};

fn main() {
    match update_latest_jsonl_from_github_blocking(
        shinden_to_anilist_core::BlockingHttpClient::new(),
        "anime-offline-database.jsonl",
    )
    .unwrap()
    {
        DatabaseUpdateStatus::UpToDate { release, sha256 } => {
            println!("anime-offline-database.jsonl is up to date ({release}, {sha256})");
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
