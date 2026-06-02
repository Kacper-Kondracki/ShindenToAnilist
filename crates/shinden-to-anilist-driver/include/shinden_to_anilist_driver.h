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

typedef struct StaError {
    enum StaStatus status;
    char *message;
} StaError;

#ifdef __cplusplus
extern "C" {
#endif // __cplusplus

struct StaDriver *sta_driver_new(void);

void sta_driver_free(struct StaDriver *driver);

void sta_string_free(char *value);

struct StaError sta_driver_counter_value(struct StaDriver *driver, int64_t *out);

struct StaError sta_driver_counter_increment(struct StaDriver *driver,
                                             uint32_t amount,
                                             int64_t *out);

#ifdef __cplusplus
}  // extern "C"
#endif  // __cplusplus

#endif  /* SHINDEN_TO_ANILIST_DRIVER_H */
