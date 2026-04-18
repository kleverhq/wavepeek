#include "bridge_api.h"

#include <cstring>
#include <string>

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

}  // namespace

extern "C" const char *fsdb_bridge_kind(void) {
    return "mock";
}

extern "C" int32_t fsdb_probe_file(const char *path, FsdbProbeResult *result) {
    if (result == nullptr) {
        return -1;
    }

    clear_result(result);

    if (path == nullptr) {
        result->code = -2;
        copy_into(result->message, sizeof(result->message), "mock bridge received a null waveform path");
        return result->code;
    }

    result->code = 0;
    result->signal_count = 7;
    result->end_time_raw = 4242;
    copy_into(result->scale_unit, sizeof(result->scale_unit), "1ps");
    copy_into(
        result->message,
        sizeof(result->message),
        std::string("mock bridge opened ") + path
    );
    return result->code;
}
