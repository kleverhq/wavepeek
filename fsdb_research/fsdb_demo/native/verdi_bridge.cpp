// Based on public FsdbReader usage examples such as fsdb-parse's print_header.cpp
// and dump_embedding.cpp, but narrowed to a tiny C ABI that is easy to lazy-load.

#ifdef NOVAS_FSDB
#undef NOVAS_FSDB
#endif

#include "bridge_api.h"

#include <cstring>
#include <string>

#if __has_include("ffrAPI.h")
#include "ffrAPI.h"
#define FSDB_DEMO_HAS_VERDI_API 1
#else
#define FSDB_DEMO_HAS_VERDI_API 0
#endif

namespace {

void clear_result(FsdbProbeResult *result) {
    std::memset(result, 0, sizeof(FsdbProbeResult));
}

void copy_into(char *destination, std::size_t destination_size, const std::string &value) {
    if (destination_size == 0) {
        return;
    }

    std::strncpy(destination, value.c_str(), destination_size - 1);
    destination[destination_size - 1] = '\0';
}

#if FSDB_DEMO_HAS_VERDI_API

struct TreeStats {
    uint64_t signal_count;
};

bool_T tree_callback(fsdbTreeCBType cb_type, void *client_data, void *tree_cb_data) {
    (void)tree_cb_data;

    TreeStats *stats = static_cast<TreeStats *>(client_data);
    if (stats == nullptr) {
        return static_cast<bool_T>(0);
    }

    if (cb_type == FSDB_TREE_CBT_VAR) {
        stats->signal_count += 1;
    }

    return static_cast<bool_T>(1);
}

std::string scale_unit_from(ffrObject *fsdb_obj) {
    str_T raw_scale = fsdb_obj->ffrGetScaleUnit();
    uint_T digits = 0;
    char *unit = nullptr;
    ffrObject::ffrExtractScaleUnit(raw_scale, digits, unit);

    if (unit == nullptr) {
        return "unknown";
    }

    return std::to_string(static_cast<unsigned int>(digits)) + unit;
}

uint64_t max_time_raw(ffrObject *fsdb_obj) {
    fsdbTag64 max_tag;
    std::memset(&max_tag, 0, sizeof(max_tag));
    fsdb_obj->ffrGetMaxFsdbTag64(&max_tag);
    return (static_cast<uint64_t>(max_tag.H) << 32) | static_cast<uint64_t>(max_tag.L);
}

#endif

}  // namespace

extern "C" const char *fsdb_bridge_kind(void) {
    return "verdi";
}

extern "C" int32_t fsdb_probe_file(const char *path, FsdbProbeResult *result) {
    if (result == nullptr) {
        return -1;
    }

    clear_result(result);

#if !FSDB_DEMO_HAS_VERDI_API
    (void)path;
    result->code = -98;
    copy_into(
        result->message,
        sizeof(result->message),
        "ffrAPI.h was unavailable while compiling the Verdi bridge"
    );
    return result->code;
#else

    if (path == nullptr) {
        result->code = -2;
        copy_into(result->message, sizeof(result->message), "null waveform path");
        return result->code;
    }

    str_T fsdb_path = const_cast<str_T>(path);
    if (ffrObject::ffrIsFSDB(fsdb_path) == static_cast<bool_T>(0)) {
        result->code = -3;
        copy_into(result->message, sizeof(result->message), "path is not recognized as FSDB by FsdbReader");
        return result->code;
    }

    ffrObject *fsdb_obj = ffrObject::ffrOpen3(fsdb_path);
    if (fsdb_obj == nullptr) {
        result->code = -4;
        copy_into(result->message, sizeof(result->message), "ffrOpen3 returned null");
        return result->code;
    }

    TreeStats stats = {0};
    fsdb_obj->ffrSetTreeCBFunc(tree_callback, &stats);
    fsdb_obj->ffrReadScopeVarTree();

    result->code = 0;
    result->signal_count = stats.signal_count;
    result->end_time_raw = max_time_raw(fsdb_obj);
    copy_into(result->scale_unit, sizeof(result->scale_unit), scale_unit_from(fsdb_obj));
    copy_into(result->message, sizeof(result->message), "opened FSDB and traversed hierarchy");

    fsdb_obj->ffrClose();
    return result->code;
#endif
}
