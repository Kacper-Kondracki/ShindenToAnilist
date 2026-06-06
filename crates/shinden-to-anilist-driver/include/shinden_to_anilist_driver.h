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

typedef struct StaOptionalI32 {
    int32_t value;
    bool has_value;
} StaOptionalI32;

typedef struct StaStringView {
    const char *ptr;
    uintptr_t len;
} StaStringView;

typedef struct StaOptionalDate {
    int32_t year;
    uint32_t month;
    uint32_t day;
    bool has_value;
} StaOptionalDate;

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

typedef struct StaError {
    enum StaStatus status;
    char *message;
} StaError;

#ifdef __cplusplus
extern "C" {
#endif // __cplusplus

struct StaDriver *sta_driver_new(void);

/**
 * # Safety
 * Safe if takes ownership and consumes the object.
 */
void sta_driver_free(struct StaDriver *driver);

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
 * Safe if takes ownership and consumes the list entry array. String pointers are borrowed.
 */
void sta_shinden_list_free(struct StaShindenList value);

struct StaError sta_driver_counter_value(struct StaDriver *driver, int64_t *out);

struct StaError sta_driver_counter_increment(struct StaDriver *driver,
                                             uint32_t amount,
                                             int64_t *out);

/**
 * # Safety
 * `path` must be valid C string
 */
struct StaError sta_driver_ensure_database(struct StaDriver *driver,
                                           const char *path,
                                           struct StaDatabaseInfo *out);

struct StaError sta_driver_load_shinden_list(struct StaDriver *driver,
                                             uint64_t user_id,
                                             struct StaShindenList *out);

#ifdef __cplusplus
}  // extern "C"
#endif  // __cplusplus

#endif  /* SHINDEN_TO_ANILIST_DRIVER_H */
