#ifndef FSDB_DEMO_BRIDGE_API_H
#define FSDB_DEMO_BRIDGE_API_H

#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct FsdbProbeResult {
    int32_t code;
    uint64_t signal_count;
    uint64_t end_time_raw;
    char scale_unit[32];
    char message[256];
} FsdbProbeResult;

const char *fsdb_bridge_kind(void);
int32_t fsdb_probe_file(const char *path, FsdbProbeResult *result);

#ifdef __cplusplus
}
#endif

#endif
