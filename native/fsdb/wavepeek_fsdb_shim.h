#ifndef WAVEPEEK_FSDB_SHIM_H
#define WAVEPEEK_FSDB_SHIM_H

#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef enum wp_fsdb_status {
    WP_FSDB_STATUS_OK = 0,
    WP_FSDB_STATUS_ERROR = 1
} wp_fsdb_status;

typedef struct wp_fsdb_reader wp_fsdb_reader;

typedef struct wp_fsdb_metadata {
    char *scale_unit;
    uint64_t time_start_raw;
    uint64_t time_end_raw;
    uint32_t xtag_type;
} wp_fsdb_metadata;

wp_fsdb_status wp_fsdb_probe(const char *path, int *is_fsdb, char **error_message);
wp_fsdb_status wp_fsdb_open(const char *path, wp_fsdb_reader **out, char **error_message);
void wp_fsdb_close(wp_fsdb_reader *reader);
wp_fsdb_status wp_fsdb_read_metadata(
    wp_fsdb_reader *reader,
    wp_fsdb_metadata *out,
    char **error_message
);
void wp_fsdb_free_string(char *value);
void wp_fsdb_free_error(char *value);

#ifdef __cplusplus
}
#endif

#endif
