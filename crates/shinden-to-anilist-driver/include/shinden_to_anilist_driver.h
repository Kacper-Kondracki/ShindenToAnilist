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

typedef struct StaSearchItem {
    uint64_t id;
    float score;
} StaSearchItem;

typedef struct StaSearchResult {
    struct StaSearchItem *items;
    uintptr_t len;
} StaSearchResult;

typedef struct StaScoreBreakdown {
    float search_score;
    float season_score;
    float year_score;
    float type_score;
    float status_score;
    float seasonal_score;
    float episodes_score;
    float final_score;
} StaScoreBreakdown;

typedef struct StaScoredCandidate {
    uint64_t id;
    struct StaScoreBreakdown score;
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
 * Safe if takes ownership and consumes the object.
 */
void sta_driver_free(struct StaDriver *driver);

struct StaDriver *sta_driver_new(void);

void sta_driver_abort(struct StaDriver *driver);

/**
 * # Safety
 * Safe if takes ownership and consumes the object.
 */
void sta_string_free(char *value);

/**
 * # Safety
 * Safe if takes ownership and consumes all strings inside `value`.
 */
void sta_database_info_free(struct StaDatabaseInfo value);

/**
 * # Safety
 * Safe if takes ownership and consumes the database entry arrays. String pointers are borrowed.
 */
void sta_anime_database_free(struct StaAnimeDatabase value);

/**
 * # Safety
 * Safe if takes ownership and consumes the list entry array. String pointers are borrowed.
 */
void sta_shinden_list_free(struct StaShindenList value);

/**
 * # Safety
 * Safe if takes ownership and consumes the search result array.
 */
void sta_search_result_free(struct StaSearchResult value);

/**
 * # Safety
 * Safe if takes ownership and consumes the match result arrays.
 */
void sta_match_result_free(struct StaMatchResult value);

/**
 * # Safety
 * Safe if takes ownership and consumes the match list result arrays.
 */
void sta_match_list_result_free(struct StaMatchListResult value);

/**
 * # Safety
 * Safe if takes ownership and consumes all strings inside `value`.
 */
void sta_export_result_free(struct StaExportResult value);

/**
 * # Safety
 * `path` must be valid C string.
 */
struct StaError sta_driver_ensure_database(struct StaDriver *driver,
                                           const char *path,
                                           struct StaDatabaseInfo *out);

struct StaError sta_driver_load_shinden_list(struct StaDriver *driver,
                                             uint64_t user_id,
                                             struct StaShindenList *out);

struct StaError sta_driver_get_anime_database(struct StaDriver *driver,
                                              struct StaAnimeDatabase *out);

/**
 * # Safety
 * `ids` must point to `len` entries or be null when `len` is 0.
 */
struct StaError sta_driver_get_anime_database_entries(struct StaDriver *driver,
                                                      const uint64_t *ids,
                                                      uintptr_t len,
                                                      struct StaAnimeDatabase *out);

/**
 * # Safety
 * `query` must be valid C string.
 */
struct StaError sta_driver_search_anime(struct StaDriver *driver,
                                        const char *query,
                                        struct StaSearchOptions options,
                                        struct StaSearchResult *out);

/**
 * # Safety
 * `query` must be valid C string.
 */
struct StaError sta_driver_match_query(struct StaDriver *driver,
                                       const char *query,
                                       struct StaMatchQueryOptions options,
                                       struct StaMatchResult *out);

struct StaError sta_driver_match_loaded_shinden_list(struct StaDriver *driver,
                                                     struct StaMatchOptions options,
                                                     struct StaMatchListResult *out);

/**
 * # Safety
 * `path` must be valid C string. `selections` must point to `len` entries or be null when `len` is 0.
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
