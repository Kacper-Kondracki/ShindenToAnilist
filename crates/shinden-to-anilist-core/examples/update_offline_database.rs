use shinden_to_anilist_core::database::updater::{
    DatabaseReleaseInfo,
    latest_database_archive_asset_blocking,
    update_latest_jsonl_from_github_blocking,
};

fn main() {
    let status = update_latest_jsonl_from_github_blocking(
        shinden_to_anilist_core::BlockingHttpClient::new(),
        "anime-offline-database.jsonl",
    )
    .unwrap();
    let latest =
        latest_database_archive_asset_blocking(shinden_to_anilist_core::BlockingHttpClient::new()).unwrap();

    dbg!(&status);
    dbg!(&latest);
    dbg!(DatabaseReleaseInfo::from(status) == DatabaseReleaseInfo::from(latest));
}
