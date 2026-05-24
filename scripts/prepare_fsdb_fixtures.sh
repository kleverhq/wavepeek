#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
fixtures_dir="$repo_root/tests/fixtures/fsdb"
mkdir -p "$fixtures_dir"

if ! command -v vcd2fsdb >/dev/null 2>&1; then
  echo "error: vcd2fsdb not found on PATH; load the Verdi environment before preparing FSDB fixtures" >&2
  exit 1
fi

sources=(
  "$repo_root/tests/fixtures/hand/scope_mixed_kinds.vcd"
  "$repo_root/tests/fixtures/hand/signal_recursive_depth.vcd"
)

for source in "${sources[@]}"; do
  base="$(basename "${source%.vcd}")"
  output="$fixtures_dir/$base.fsdb"
  tmp="$output.tmp.$$"
  rm -f "$tmp"
  log_dir="$repo_root/vcd2fsdbLog"
  rm -rf "$log_dir"
  vcd2fsdb "$source" -o "$tmp"
  rm -rf "$log_dir"
  mv "$tmp" "$output"
done
