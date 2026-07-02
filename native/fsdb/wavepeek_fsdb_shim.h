#ifndef WAVEPEEK_FSDB_SHIM_H
#define WAVEPEEK_FSDB_SHIM_H

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef enum wp_fsdb_status {
    WP_FSDB_STATUS_OK = 0,
    WP_FSDB_STATUS_ERROR = 1
} wp_fsdb_status;

typedef enum wp_fsdb_tree_event {
    WP_FSDB_TREE_EVENT_BEGIN_TREE = 0,
    WP_FSDB_TREE_EVENT_SCOPE = 1,
    WP_FSDB_TREE_EVENT_SIGNAL = 2,
    WP_FSDB_TREE_EVENT_UPSCOPE = 3,
    WP_FSDB_TREE_EVENT_END_TREE = 4,
    WP_FSDB_TREE_EVENT_END_ALL_TREE = 5,
    WP_FSDB_TREE_EVENT_DATATYPE = 6
} wp_fsdb_tree_event;

typedef enum wp_fsdb_scope_kind {
    WP_FSDB_SCOPE_KIND_MODULE = 0,
    WP_FSDB_SCOPE_KIND_TASK = 1,
    WP_FSDB_SCOPE_KIND_FUNCTION = 2,
    WP_FSDB_SCOPE_KIND_BEGIN = 3,
    WP_FSDB_SCOPE_KIND_FORK = 4,
    WP_FSDB_SCOPE_KIND_GENERATE = 5,
    WP_FSDB_SCOPE_KIND_STRUCT = 6,
    WP_FSDB_SCOPE_KIND_UNION = 7,
    WP_FSDB_SCOPE_KIND_CLASS = 8,
    WP_FSDB_SCOPE_KIND_INTERFACE = 9,
    WP_FSDB_SCOPE_KIND_PACKAGE = 10,
    WP_FSDB_SCOPE_KIND_PROGRAM = 11,
    WP_FSDB_SCOPE_KIND_UNKNOWN = 12
} wp_fsdb_scope_kind;

typedef enum wp_fsdb_signal_kind {
    WP_FSDB_SIGNAL_KIND_EVENT = 0,
    WP_FSDB_SIGNAL_KIND_INTEGER = 1,
    WP_FSDB_SIGNAL_KIND_PARAMETER = 2,
    WP_FSDB_SIGNAL_KIND_REAL = 3,
    WP_FSDB_SIGNAL_KIND_REG = 4,
    WP_FSDB_SIGNAL_KIND_SUPPLY0 = 5,
    WP_FSDB_SIGNAL_KIND_SUPPLY1 = 6,
    WP_FSDB_SIGNAL_KIND_TIME = 7,
    WP_FSDB_SIGNAL_KIND_TRI = 8,
    WP_FSDB_SIGNAL_KIND_TRIAND = 9,
    WP_FSDB_SIGNAL_KIND_TRIOR = 10,
    WP_FSDB_SIGNAL_KIND_TRIREG = 11,
    WP_FSDB_SIGNAL_KIND_TRI0 = 12,
    WP_FSDB_SIGNAL_KIND_TRI1 = 13,
    WP_FSDB_SIGNAL_KIND_WAND = 14,
    WP_FSDB_SIGNAL_KIND_WIRE = 15,
    WP_FSDB_SIGNAL_KIND_WOR = 16,
    WP_FSDB_SIGNAL_KIND_STRING = 17,
    WP_FSDB_SIGNAL_KIND_PORT = 18,
    WP_FSDB_SIGNAL_KIND_SPARSE_ARRAY = 19,
    WP_FSDB_SIGNAL_KIND_REAL_TIME = 20,
    WP_FSDB_SIGNAL_KIND_REAL_PARAMETER = 21,
    WP_FSDB_SIGNAL_KIND_BIT = 22,
    WP_FSDB_SIGNAL_KIND_LOGIC = 23,
    WP_FSDB_SIGNAL_KIND_INT = 24,
    WP_FSDB_SIGNAL_KIND_SHORT_INT = 25,
    WP_FSDB_SIGNAL_KIND_LONG_INT = 26,
    WP_FSDB_SIGNAL_KIND_BYTE = 27,
    WP_FSDB_SIGNAL_KIND_ENUM = 28,
    WP_FSDB_SIGNAL_KIND_SHORT_REAL = 29,
    WP_FSDB_SIGNAL_KIND_BOOLEAN = 30,
    WP_FSDB_SIGNAL_KIND_BIT_VECTOR = 31,
    WP_FSDB_SIGNAL_KIND_UNKNOWN = 32
} wp_fsdb_signal_kind;

typedef enum wp_fsdb_datatype_kind {
    WP_FSDB_DATATYPE_KIND_ENUM = 0,
    WP_FSDB_DATATYPE_KIND_LOGIC = 1,
    WP_FSDB_DATATYPE_KIND_BIT = 2,
    WP_FSDB_DATATYPE_KIND_INT = 3,
    WP_FSDB_DATATYPE_KIND_UINT = 4,
    WP_FSDB_DATATYPE_KIND_SHORT_INT = 5,
    WP_FSDB_DATATYPE_KIND_SHORT_UINT = 6,
    WP_FSDB_DATATYPE_KIND_LONG_INT = 7,
    WP_FSDB_DATATYPE_KIND_LONG_UINT = 8,
    WP_FSDB_DATATYPE_KIND_BYTE = 9,
    WP_FSDB_DATATYPE_KIND_UBYTE = 10,
    WP_FSDB_DATATYPE_KIND_REAL = 11,
    WP_FSDB_DATATYPE_KIND_SHORT_REAL = 12,
    WP_FSDB_DATATYPE_KIND_TIME = 13,
    WP_FSDB_DATATYPE_KIND_STRING = 14,
    WP_FSDB_DATATYPE_KIND_EVENT = 15,
    WP_FSDB_DATATYPE_KIND_UNKNOWN = 16
} wp_fsdb_datatype_kind;

typedef enum wp_fsdb_value_encoding {
    WP_FSDB_VALUE_ENCODING_BIT_VECTOR = 0,
    WP_FSDB_VALUE_ENCODING_UNSUPPORTED = 1,
    WP_FSDB_VALUE_ENCODING_DATATYPE_CANDIDATE = 2
} wp_fsdb_value_encoding;

typedef struct wp_fsdb_reader wp_fsdb_reader;
typedef struct wp_fsdb_signal_session wp_fsdb_signal_session;

typedef struct wp_fsdb_metadata {
    char *scale_unit;
    uint64_t time_start_raw;
    uint64_t time_end_raw;
    uint32_t xtag_type;
} wp_fsdb_metadata;

typedef struct wp_fsdb_scope_record {
    const char *name;
    uint32_t kind;
    int hidden;
} wp_fsdb_scope_record;

typedef struct wp_fsdb_signal_record {
    const char *name;
    uint64_t idcode;
    int has_bit_range;
    int32_t left;
    int32_t right;
    int has_datatype_id;
    uint32_t datatype_id;
    uint32_t kind;
    int packed_component;
    uint32_t value_encoding;
} wp_fsdb_signal_record;

typedef struct wp_fsdb_enum_label_record {
    const char *name;
    const char *bits;
} wp_fsdb_enum_label_record;

typedef struct wp_fsdb_datatype_record {
    uint32_t idcode;
    uint32_t kind;
    const char *name;
    int has_bit_width;
    uint32_t bit_width;
    int has_is_signed;
    int is_signed;
    size_t enum_label_count;
    const wp_fsdb_enum_label_record *enum_labels;
} wp_fsdb_datatype_record;

typedef struct wp_fsdb_sample_record {
    uint64_t idcode;
    int has_value;
    uint32_t bit_width;
    uint64_t value_time_raw;
    char *bits;
} wp_fsdb_sample_record;

typedef struct wp_fsdb_time_list {
    uint64_t *times;
    size_t count;
} wp_fsdb_time_list;

typedef struct wp_fsdb_value_change_record {
    uint64_t time_raw;
    char *bits;
} wp_fsdb_value_change_record;

typedef struct wp_fsdb_signal_timeline_record {
    uint64_t idcode;
    uint32_t bit_width;
    wp_fsdb_value_change_record *changes;
    size_t change_count;
} wp_fsdb_signal_timeline_record;

typedef struct wp_fsdb_signal_timeline_list {
    wp_fsdb_signal_timeline_record *signals;
    size_t count;
} wp_fsdb_signal_timeline_list;

typedef int (*wp_fsdb_tree_callback)(
    uint32_t event,
    const wp_fsdb_scope_record *scope,
    const wp_fsdb_signal_record *signal,
    const wp_fsdb_datatype_record *datatype,
    void *user
);

wp_fsdb_status wp_fsdb_probe(const char *path, int *is_fsdb, char **error_message);
wp_fsdb_status wp_fsdb_open(const char *path, wp_fsdb_reader **out, char **error_message);
void wp_fsdb_close(wp_fsdb_reader *reader);
wp_fsdb_status wp_fsdb_read_metadata(
    wp_fsdb_reader *reader,
    wp_fsdb_metadata *out,
    char **error_message
);
wp_fsdb_status wp_fsdb_read_scope_var_tree(
    wp_fsdb_reader *reader,
    wp_fsdb_tree_callback callback,
    void *user,
    char **error_message
);
wp_fsdb_status wp_fsdb_sample_signal_values(
    wp_fsdb_reader *reader,
    const uint64_t *idcodes,
    size_t count,
    uint64_t query_time_raw,
    wp_fsdb_sample_record **out,
    char **error_message
);
wp_fsdb_status wp_fsdb_collect_signal_change_times(
    wp_fsdb_reader *reader,
    const uint64_t *idcodes,
    size_t count,
    uint64_t from_raw,
    uint64_t to_raw,
    wp_fsdb_time_list *out,
    char **error_message
);
wp_fsdb_status wp_fsdb_signal_event_occurred(
    wp_fsdb_reader *reader,
    uint64_t idcode,
    uint64_t query_time_raw,
    int *occurred,
    char **error_message
);
wp_fsdb_status wp_fsdb_open_signal_session(
    wp_fsdb_reader *reader,
    const uint64_t *idcodes,
    size_t count,
    wp_fsdb_signal_session **out,
    char **error_message
);
wp_fsdb_status wp_fsdb_signal_session_sample(
    wp_fsdb_signal_session *session,
    const uint64_t *idcodes,
    size_t count,
    uint64_t query_time_raw,
    wp_fsdb_sample_record **out,
    char **error_message
);
wp_fsdb_status wp_fsdb_signal_session_collect_change_times(
    wp_fsdb_signal_session *session,
    const uint64_t *idcodes,
    size_t count,
    uint64_t from_raw,
    uint64_t to_raw,
    wp_fsdb_time_list *out,
    char **error_message
);
wp_fsdb_status wp_fsdb_signal_session_read_value_changes(
    wp_fsdb_signal_session *session,
    const uint64_t *idcodes,
    size_t count,
    uint64_t from_raw,
    uint64_t to_raw,
    wp_fsdb_signal_timeline_list *out,
    char **error_message
);
wp_fsdb_status wp_fsdb_signal_session_event_occurred(
    wp_fsdb_signal_session *session,
    uint64_t idcode,
    uint64_t query_time_raw,
    int *occurred,
    char **error_message
);
void wp_fsdb_close_signal_session(wp_fsdb_signal_session *session);
void wp_fsdb_free_samples(wp_fsdb_sample_record *samples, size_t count);
void wp_fsdb_free_time_list(wp_fsdb_time_list *list);
void wp_fsdb_free_signal_timelines(wp_fsdb_signal_timeline_list *list);
void wp_fsdb_free_string(char *value);
void wp_fsdb_free_error(char *value);

#ifdef __cplusplus
}
#endif

#endif
