#ifndef SHINDEN_TO_ANILIST_DRIVER_H
#define SHINDEN_TO_ANILIST_DRIVER_H

#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

typedef enum StaStatus {
    StaStatusOk = 0,
    StaStatusNullPointer = 1,
    StaStatusPanic = 2,
    StaStatusError = 3,
} StaStatus;

typedef struct StaDriver StaDriver;

typedef struct StaDatabaseInfo {
    char *last_update;
    char *release;
    char *sha256;
    char *path;
    bool updated;
} StaDatabaseInfo;

typedef struct StaOptionalDate {
    int32_t year;
    uint32_t month;
    uint32_t day;
    bool has_value;
} StaOptionalDate;

typedef struct StaOptionalF32 {
    float value;
    bool has_value;
} StaOptionalF32;

typedef struct StaConsolidatedMetadata {
    struct StaOptionalF32 season;
    struct StaOptionalF32 part;
    struct StaOptionalF32 episode;
    bool is_final_season;
    bool is_final_part;
    bool is_final_episode;
} StaConsolidatedMetadata;

typedef struct StaStringView {
    const char *ptr;
    uintptr_t len;
} StaStringView;

typedef struct StaStringViewArray {
    struct StaStringView *entries;
    uintptr_t len;
} StaStringViewArray;

typedef struct StaTitleMetadata {
    struct StaOptionalF32 season;
    struct StaOptionalF32 part;
    struct StaOptionalF32 episode;
    bool has_season_keyword;
    bool has_part_keyword;
    bool has_episode_keyword;
} StaTitleMetadata;

typedef struct StaOptionalI32 {
    int32_t value;
    bool has_value;
} StaOptionalI32;

typedef struct StaDatabaseEntry {
    uint64_t id;
    struct StaConsolidatedMetadata consolidated_metadata;
    struct StaStringViewArray sources;
    struct StaStringView title;
    struct StaStringView normalized_title;
    struct StaTitleMetadata metadata;
    struct StaStringView anime_type;
    int32_t episodes;
    struct StaStringView status;
    struct StaStringView season;
    struct StaOptionalI32 year;
    struct StaStringView picture;
    struct StaStringView thumbnail;
    struct StaOptionalI32 duration;
    struct StaStringViewArray synonyms;
    struct StaStringViewArray normalized_synonyms;
    struct StaStringViewArray studios;
    struct StaStringViewArray producers;
    struct StaStringViewArray related_anime;
    struct StaStringViewArray tags;
} StaDatabaseEntry;

typedef struct StaAnimeDatabase {
    struct StaOptionalDate last_update;
    struct StaDatabaseEntry *entries;
    uintptr_t len;
} StaAnimeDatabase;

typedef struct StaShindenEntry {
    uint64_t id;
    struct StaOptionalI32 cover_id;
    struct StaStringView title;
    struct StaStringView anime_status;
    struct StaStringView anime_type;
    struct StaOptionalDate premiere_date;
    struct StaOptionalDate finish_date;
    struct StaOptionalI32 episodes;
    bool is_favourite;
    struct StaStringView watch_status;
    int32_t watched_episodes;
    struct StaOptionalI32 score;
    struct StaStringView note;
    struct StaStringView description;
} StaShindenEntry;

typedef struct StaShindenList {
    struct StaShindenEntry *entries;
    uintptr_t len;
} StaShindenList;

typedef struct StaIdList {
    uint64_t *entries;
    uintptr_t len;
} StaIdList;

typedef struct StaSearchItem {
    uint64_t id;
    float score;
} StaSearchItem;

typedef struct StaSearchResult {
    struct StaSearchItem *items;
    uintptr_t len;
} StaSearchResult;

typedef struct StaScoredCandidate {
    uint64_t id;
    float score;
} StaScoredCandidate;

typedef struct StaMatchWinner {
    struct StaScoredCandidate item;
    bool has_value;
} StaMatchWinner;

typedef struct StaMatchResult {
    struct StaScoredCandidate *items;
    uintptr_t items_len;
    struct StaScoredCandidate *top;
    uintptr_t top_len;
    struct StaMatchWinner winner;
} StaMatchResult;

typedef struct StaShindenMatchResult {
    uint64_t shinden_id;
    struct StaMatchResult result;
} StaShindenMatchResult;

typedef struct StaMatchListResult {
    struct StaShindenMatchResult *entries;
    uintptr_t len;
    uintptr_t total;
    uintptr_t winners;
    uintptr_t has_top;
    uintptr_t unmatched;
} StaMatchListResult;

typedef struct StaExportResult {
    char *path;
    uintptr_t exported_count;
} StaExportResult;

typedef struct StaError {
    enum StaStatus status;
    char *message;
} StaError;

typedef struct StaSearchOptions {
    struct StaStringView mode;
    uintptr_t limit;
    float threshold;
} StaSearchOptions;

typedef struct StaMatchQueryOptions {
    struct StaSearchOptions search;
    uintptr_t result_limit;
    bool has_result_limit;
} StaMatchQueryOptions;

typedef struct StaMatchOptions {
    uintptr_t candidate_limit;
    float search_threshold;
    uintptr_t result_limit;
    bool has_result_limit;
} StaMatchOptions;

typedef struct StaMatchSelection {
    uint64_t shinden_id;
    uint64_t database_id;
} StaMatchSelection;

#ifdef __cplusplus
extern "C" {
#endif // __cplusplus

/**
 * # Safety
 * `driver` must be null or a pointer returned by [`sta_driver_new`]. After this
 * call, the pointer is consumed and must not be used again.
 */
void sta_driver_free(struct StaDriver *driver);

struct StaDriver *sta_driver_new(void);

/**
 * # Safety
 * `driver` must be null or a live pointer returned by [`sta_driver_new`].
 */
void sta_driver_abort(struct StaDriver *driver);

/**
 * # Safety
 * `value` must be null or a string allocated by this library. After this call,
 * the pointer is consumed and must not be used again.
 */
void sta_string_free(char *value);

/**
 * # Safety
 * `value` must be a database-info result returned by this library. This call
 * consumes all owned strings inside `value`.
 */
void sta_database_info_free(struct StaDatabaseInfo value);

/**
 * # Safety
 * `value` must be an anime-database result returned by this library. This call
 * consumes the entry arrays; string views are borrowed from those entries and
 * become invalid after the free call.
 */
void sta_anime_database_free(struct StaAnimeDatabase value);

/**
 * # Safety
 * `value` must be a Shinden-list result returned by this library. This call
 * consumes the entry array; string views are borrowed from those entries and
 * become invalid after the free call.
 */
void sta_shinden_list_free(struct StaShindenList value);

/**
 * # Safety
 * `value` must be an id-list result returned by this library. This call
 * consumes the id array.
 */
void sta_id_list_free(struct StaIdList value);

/**
 * # Safety
 * `value` must be a search result returned by this library. This call consumes
 * the result array.
 */
void sta_search_result_free(struct StaSearchResult value);

/**
 * # Safety
 * `value` must be a match result returned by this library. This call consumes
 * all result arrays.
 */
void sta_match_result_free(struct StaMatchResult value);

/**
 * # Safety
 * `value` must be a match-list result returned by this library. This call
 * consumes all nested result arrays.
 */
void sta_match_list_result_free(struct StaMatchListResult value);

/**
 * # Safety
 * `value` must be an export result returned by this library. This call consumes
 * all owned strings inside `value`.
 */
void sta_export_result_free(struct StaExportResult value);

/**
 * # Safety
 * `driver` must be a live pointer returned by [`sta_driver_new`]. `path` must
 * be a valid UTF-8 C string. `out` must be non-null and writable; on success it
 * must be released with [`sta_database_info_free`].
 */
struct StaError sta_driver_ensure_database(struct StaDriver *driver,
                                           const char *path,
                                           struct StaDatabaseInfo *out);

/**
 * # Safety
 * `driver` must be a live pointer returned by [`sta_driver_new`]. `out` must
 * be non-null and writable; on success it must be released with
 * [`sta_id_list_free`].
 */
struct StaError sta_driver_load_shinden_list(struct StaDriver *driver,
                                             uint64_t user_id,
                                             struct StaIdList *out);

/**
 * # Safety
 * `driver` must be a live pointer returned by [`sta_driver_new`]. `view` must
 * be a valid UTF-8 C string naming a loaded-list view. `out` must be non-null
 * and writable; on success it must be released with [`sta_id_list_free`].
 */
struct StaError sta_driver_get_loaded_shinden_entry_ids(struct StaDriver *driver,
                                                        const char *view,
                                                        struct StaIdList *out);

/**
 * # Safety
 * `driver` must be a live pointer returned by [`sta_driver_new`]. `ids` must
 * point to `len` entries or be null when `len` is 0. `out` must be non-null and
 * writable; on success it must be released with [`sta_shinden_list_free`].
 */
struct StaError sta_driver_get_loaded_shinden_entries(struct StaDriver *driver,
                                                      const uint64_t *ids,
                                                      uintptr_t len,
                                                      struct StaShindenList *out);

/**
 * # Safety
 * `driver` must be a live pointer returned by [`sta_driver_new`]. `ids` must
 * point to `len` entries or be null when `len` is 0. `out` must be non-null and
 * writable; on success it must be released with [`sta_anime_database_free`].
 */
struct StaError sta_driver_get_anime_database_entries(struct StaDriver *driver,
                                                      const uint64_t *ids,
                                                      uintptr_t len,
                                                      struct StaAnimeDatabase *out);

/**
 * # Safety
 * `driver` must be a live pointer returned by [`sta_driver_new`]. `query` and
 * `options.mode` must be valid UTF-8 C/string views for the duration of this
 * call. `out` must be non-null and writable; on success it must be released
 * with [`sta_search_result_free`].
 */
struct StaError sta_driver_search_anime(struct StaDriver *driver,
                                        const char *query,
                                        struct StaSearchOptions options,
                                        struct StaSearchResult *out);

/**
 * # Safety
 * `driver` must be a live pointer returned by [`sta_driver_new`]. `query` and
 * `options.search.mode` must be valid UTF-8 C/string views for the duration of
 * this call. `out` must be non-null and writable; on success it must be
 * released with [`sta_match_result_free`].
 */
struct StaError sta_driver_match_query(struct StaDriver *driver,
                                       const char *query,
                                       struct StaMatchQueryOptions options,
                                       struct StaMatchResult *out);

/**
 * # Safety
 * `driver` must be a live pointer returned by [`sta_driver_new`]. `out` must
 * be non-null and writable; on success it must be released with
 * [`sta_match_list_result_free`].
 */
struct StaError sta_driver_match_loaded_shinden_list(struct StaDriver *driver,
                                                     struct StaMatchOptions options,
                                                     struct StaMatchListResult *out);

/**
 * # Safety
 * `driver` must be a live pointer returned by [`sta_driver_new`]. `path` must
 * be a valid UTF-8 C string. `selections` must point to `len` entries or be
 * null when `len` is 0. `out` must be non-null and writable; on success it must
 * be released with [`sta_export_result_free`].
 */
struct StaError sta_driver_export_matches(struct StaDriver *driver,
                                          const char *path,
                                          const struct StaMatchSelection *selections,
                                          uintptr_t len,
                                          struct StaExportResult *out);

#ifdef __cplusplus
}  // extern "C"
#endif  // __cplusplus

#endif  /* SHINDEN_TO_ANILIST_DRIVER_H */
