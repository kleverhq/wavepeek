#!/usr/bin/env bash
set -euo pipefail

# This script is a manual Codex-specific projection of the repository's
# devcontainer image. Keep `.devcontainer/`, especially
# `.devcontainer/env_contract.sh`, as the canonical source of truth and
# update this wrapper when that contract or the image contents move.

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=./codex_env_common.sh
source "$SCRIPT_DIR/codex_env_common.sh"

ensure_codex_tooling

log "Codex resume check complete. Non-dev just recipes are ready."
