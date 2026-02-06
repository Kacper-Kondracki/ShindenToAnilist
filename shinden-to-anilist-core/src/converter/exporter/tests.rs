use crate::converter::database;

#[test]
fn xml_exporter_test() {
    let _database = database::get_from_mmap("anime-offline-database.jsonl").unwrap();
}
