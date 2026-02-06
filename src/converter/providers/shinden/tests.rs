use std::{
    fs::File,
    io::Write,
    time::Instant,
};

use crate::converter::{
    common::AnimeList,
    providers::shinden::request,
};

#[tokio::test]
async fn request_shinden_test() {
    let now = Instant::now();
    let shinden = request(196402, 999999, 0).await.unwrap();
    let elapsed = now.elapsed();

    println!("{} entries", shinden.len());
    println!("took {:.2?}", elapsed);

    File::options()
        .create_new(true)
        .write(true)
        .open("shinden-test.json")
        .and_then(|mut f| f.write_all(&serde_json::to_vec_pretty(&shinden).unwrap()))
        .ok();
}
