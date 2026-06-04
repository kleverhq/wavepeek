#!/usr/bin/env bash
set -euo pipefail

# Host-side Verdi SDK path. Inside the container VERDI_HOME is /opt/verdi.
HOST_VERDI_HOME="${VERDI_HOME-}"

WAVEPEEK_STATE_DIR="$HOME/.cache/wavepeek"
WAVEPEEK_VERDI_MOUNT_SOURCE="$WAVEPEEK_STATE_DIR/verdi"
WAVEPEEK_GITHUB_CONFIG_DIR="$HOME/.config/wavepeek"
WAVEPEEK_GITHUB_EMPTY_ENV="$WAVEPEEK_GITHUB_CONFIG_DIR/github.empty.env"
WAVEPEEK_GITHUB_ENV="$WAVEPEEK_GITHUB_CONFIG_DIR/github.env"

# Create host-side mount sources up front so devcontainer bind mounts do not
# fail or create root-owned placeholders when the paths are missing.
mkdir -p \
    "$HOME/.config/opencode" \
    "$HOME/.local/share/opencode" \
    "$HOME/.cache/opencode" \
    "$HOME/.claude" \
    "$HOME/.codex" \
    "$HOME/.pi" \
    "$WAVEPEEK_STATE_DIR" \
    "$WAVEPEEK_GITHUB_CONFIG_DIR"

# Optional GitHub auth env-file used by the dev container. This is
# intentionally outside the repository and empty by default.
chmod 700 "$WAVEPEEK_GITHUB_CONFIG_DIR" 2>/dev/null || true

if [ ! -e "$WAVEPEEK_GITHUB_EMPTY_ENV" ]; then
    : > "$WAVEPEEK_GITHUB_EMPTY_ENV"
fi
chmod 600 "$WAVEPEEK_GITHUB_EMPTY_ENV" 2>/dev/null || true

if [ ! -e "$WAVEPEEK_GITHUB_ENV" ] && [ ! -L "$WAVEPEEK_GITHUB_ENV" ]; then
    ln -s "github.empty.env" "$WAVEPEEK_GITHUB_ENV"
fi

# Claude Code may keep top-level state/config here; bind mounts need the file to exist.
if [ ! -e "$HOME/.claude.json" ]; then
    printf '{}\n' > "$HOME/.claude.json"
fi

# Recreate the wavepeek-managed host source mounted as /opt/verdi.
rm -rf "$WAVEPEEK_VERDI_MOUNT_SOURCE"
if [ -n "$HOST_VERDI_HOME" ] && [ -d "$HOST_VERDI_HOME" ]; then
    ln -s "$HOST_VERDI_HOME" "$WAVEPEEK_VERDI_MOUNT_SOURCE"
else
    mkdir -p "$WAVEPEEK_VERDI_MOUNT_SOURCE"
fi
