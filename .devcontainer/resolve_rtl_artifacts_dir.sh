#!/usr/bin/env sh
set -eu

if [ -n "${WAVEPEEK_RTL_ARTIFACTS_DIR:-}" ]; then
    printf '%s\n' "$WAVEPEEK_RTL_ARTIFACTS_DIR"
elif [ -n "${RTL_ARTIFACTS_DIR:-}" ]; then
    printf '%s\n' "$RTL_ARTIFACTS_DIR"
elif [ -d /opt/rtl-artifacts ] && [ -r /opt/rtl-artifacts ]; then
    printf '/opt/rtl-artifacts\n'
else
    printf '%s/.cache/wavepeek/rtl-artifacts\n' "$HOME"
fi
