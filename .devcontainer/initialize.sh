#!/usr/bin/env bash
set -euo pipefail

# Create host-side mount sources up front so devcontainer bind mounts do not
# fail or create root-owned placeholders when the paths are missing.
mkdir -p \
    "$HOME/.config/opencode" \
    "$HOME/.config/gh" \
    "$HOME/.local/share/opencode" \
    "$HOME/.cache/opencode" \
    "$HOME/.claude" \
    "$HOME/.codex" \
    "$HOME/.pi"

# Claude Code may keep top-level state/config here; bind mounts need the file to exist.
if [ ! -e "$HOME/.claude.json" ]; then
    printf '{}\n' > "$HOME/.claude.json"
fi
