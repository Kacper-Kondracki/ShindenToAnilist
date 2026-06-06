# Go driver examples

These mirror `crates/shinden-to-anilist-core/examples` while exercising the stateful Go driver API.

Run examples from the repository root:

```sh
go run ./examples/go/update_offline_database
go run ./examples/go/load_mmap_database
go run ./examples/go/searcher_init
go run ./examples/go/searcher_title
go run ./examples/go/request_shinden -user-id 196402
go run ./examples/go/match_fuzzy_query
go run ./examples/go/match_shinden_list -user-id 196402
go run ./examples/go/xml_exporter -user-id 196402
```

Common flags:

- `-database anime-offline-database.jsonl`
- `-user-id 196402`

Notes:

- `match_shinden_list` and `xml_exporter` load Shinden through the driver rather than reading `shinden-test.json`, because the driver owns Shinden state internally.
- `optimize_matcher` reports the strict preset score; arbitrary matcher weights are not exposed by the driver API.
- `update_offline_database_zst` is a stub that exits with an explanatory error because zstd updates are not exposed by the driver API.
