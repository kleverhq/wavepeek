#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=./codex_env_common.sh
source "$SCRIPT_DIR/codex_env_common.sh"

ensure_codex_tooling

log "Codex resume check complete."
