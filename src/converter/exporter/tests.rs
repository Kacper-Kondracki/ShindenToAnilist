use crate::database::{
    AnimeDatabase,
    AnimeDatabaseLoad,
};

#[test]
fn xml_exporter_test() {
    let _database = AnimeDatabase::get_from_mmap("anime-offline-database.jsonl").unwrap();
}
