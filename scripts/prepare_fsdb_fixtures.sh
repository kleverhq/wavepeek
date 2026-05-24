#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
fixtures_dir="$repo_root/tests/fixtures/fsdb"
mkdir -p "$fixtures_dir"

if ! command -v vcd2fsdb >/dev/null 2>&1; then
  echo "error: vcd2fsdb not found on PATH; load the Verdi environment before preparing FSDB fixtures" >&2
  exit 1
fi

sources=()
while IFS= read -r -d '' source; do
  sources+=("$source")
done < <(find "$repo_root/tests/fixtures/hand" -type f -name '*.vcd' -print0 | sort -z)

if [ "${#sources[@]}" -eq 0 ]; then
  echo "error: no VCD fixtures found under tests/fixtures/hand" >&2
  exit 1
fi

for source in "${sources[@]}"; do
  base="$(basename "${source%.vcd}")"
  output="$fixtures_dir/$base.fsdb"
  tmp="$output.tmp.$$"
  rm -f "$tmp"
  log_dir="$repo_root/vcd2fsdbLog"
  converter_log="$tmp.log"
  rm -rf "$log_dir"
  if ! vcd2fsdb "$source" -o "$tmp" >"$converter_log" 2>&1; then
    cat "$converter_log" >&2
    rm -f "$tmp" "$converter_log"
    rm -rf "$log_dir"
    exit 1
  fi
  rm -f "$converter_log"
  rm -rf "$log_dir"
  mv "$tmp" "$output"
done
