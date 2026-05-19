#!/usr/bin/env bash
set -euo pipefail

HOST_VERDI_HOME="${VERDI_HOME-}"
if [ -z "$HOST_VERDI_HOME" ] && [ -n "${SHELL-}" ] && [ -x "$SHELL" ]; then
    HOST_VERDI_HOME="$($SHELL -lc 'printf "%s" "${VERDI_HOME-}"')"
fi

WAVEPEEK_STATE_DIR="$HOME/.cache/wavepeek"
WAVEPEEK_VERDI_MOUNT_SOURCE="$WAVEPEEK_STATE_DIR/verdi"

# Create host-side mount sources up front so devcontainer bind mounts do not
# fail or create root-owned placeholders when the paths are missing.
mkdir -p \
    "$HOME/.config/opencode" \
    "$HOME/.config/gh" \
    "$HOME/.local/share/opencode" \
    "$HOME/.cache/opencode" \
    "$HOME/.claude" \
    "$HOME/.codex" \
    "$HOME/.pi" \
    "$WAVEPEEK_STATE_DIR"

# Claude Code may keep top-level state/config here; bind mounts need the file to exist.
if [ ! -e "$HOME/.claude.json" ]; then
    printf '{}\n' > "$HOME/.claude.json"
fi

rm -rf "$WAVEPEEK_VERDI_MOUNT_SOURCE"

if [ -n "$HOST_VERDI_HOME" ] && [ -d "$HOST_VERDI_HOME" ]; then
    ln -s "$HOST_VERDI_HOME" "$WAVEPEEK_VERDI_MOUNT_SOURCE"
else
    mkdir -p "$WAVEPEEK_VERDI_MOUNT_SOURCE"
fi
