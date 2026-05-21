#!/usr/bin/env bash
set -euo pipefail

has_headers() {
    local home="$1"
    [ -f "$home/share/FsdbReader/ffrAPI.h" ] && \
    [ -f "$home/share/FsdbReader/ffrKit.h" ] && \
    [ -f "$home/share/FsdbReader/fsdbShr.h" ]
}

selected_libdir() {
    local home="$1"
    if [ -n "${WAVEPEEK_FSDB_READER_LIBDIR:-}" ]; then
        printf '%s\n' "$WAVEPEEK_FSDB_READER_LIBDIR"
    elif [ -n "${WAVEPEEK_FSDB_ABI:-}" ]; then
        printf '%s/share/FsdbReader/%s\n' "$home" "$WAVEPEEK_FSDB_ABI"
    else
        printf '%s/share/FsdbReader/linux64\n' "$home"
    fi
}

has_reader_library() {
    local home="$1"
    local libdir
    libdir="$(selected_libdir "$home")"
    [ -f "$libdir/libnffr.so" ]
}

for candidate in "${VERDI_HOME:-}" /opt/verdi; do
    if [ -n "$candidate" ] && has_headers "$candidate" && has_reader_library "$candidate"; then
        printf '%s\n' "$candidate"
        exit 0
    fi
done

exit 0
