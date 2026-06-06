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

#ifdef __cplusplus
}  // extern "C"
#endif  // __cplusplus

#endif  /* SHINDEN_TO_ANILIST_DRIVER_H */
