#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
# shellcheck source=../../.devcontainer/env_contract.sh
source "$repo_root/.devcontainer/env_contract.sh"

hand_fixtures_dir="$repo_root/tests/fixtures/hand"
fsdb_fixtures_dir="$repo_root/tests/fixtures/fsdb"
rtl_artifacts_dir="$RTL_ARTIFACTS_DIR"
tmp_root="$repo_root/tmp/fsdb-fixtures"
mode="all"
rtl_filter=""

usage() {
  cat >&2 <<'EOF'
usage: prepare_fsdb_fixtures.sh [--hand-only | --rtl-only] [--rtl-filter <regex>]

By default, prepare both hand-written VCD-derived FSDB test fixtures and RTL
FST-derived FSDB benchmark artifacts. Use --rtl-filter with --rtl-only, or the
default mode, to restrict RTL FST basenames matched for benchmark smoke paths.
EOF
}

parse_args() {
  while [ "$#" -gt 0 ]; do
    case "$1" in
      --hand-only)
        if [ "$mode" = "rtl" ]; then
          printf '%s\n' "error: fsdb fixture: --hand-only and --rtl-only are mutually exclusive" >&2
          exit 2
        fi
        mode="hand"
        ;;
      --rtl-only)
        if [ "$mode" = "hand" ]; then
          printf '%s\n' "error: fsdb fixture: --hand-only and --rtl-only are mutually exclusive" >&2
          exit 2
        fi
        mode="rtl"
        ;;
      --rtl-filter)
        shift
        if [ "$#" -eq 0 ] || [ -z "$1" ]; then
          printf '%s\n' "error: fsdb fixture: --rtl-filter requires a non-empty regex" >&2
          exit 2
        fi
        rtl_filter="$1"
        ;;
      -h | --help)
        usage
        exit 0
        ;;
      *)
        printf '%s\n' "error: fsdb fixture: unknown argument: $1" >&2
        usage
        exit 2
        ;;
    esac
    shift
  done

  if [ "$mode" = "hand" ] && [ -n "$rtl_filter" ]; then
    printf '%s\n' "error: fsdb fixture: --rtl-filter cannot be used with --hand-only" >&2
    exit 2
  fi
}

require_tool() {
  local tool="$1"
  local hint="$2"
  if ! command -v "$tool" >/dev/null 2>&1; then
    printf '%s\n' "error: fsdb fixture: $tool not found on PATH; $hint" >&2
    exit 1
  fi
}

cleanup_converter_logs() {
  local work_dir="$1"
  rm -rf "$work_dir/vcd2fsdbLog"
}

run_converter() {
  local label="$1"
  local stdout_log="$2"
  local stderr_log="$3"
  local work_dir
  shift 3

  rm -f "$stdout_log" "$stderr_log"
  work_dir="$(mktemp -d "$tmp_root/converter.XXXXXX")"
  cleanup_converter_logs "$work_dir"
  if (cd "$work_dir" && "$@") >"$stdout_log" 2>"$stderr_log"; then
    rm -f "$stdout_log" "$stderr_log"
    cleanup_converter_logs "$work_dir"
    rm -rf "$work_dir"
    return 0
  fi

  printf '%s\n' "error: fsdb fixture: conversion failed for $label" >&2
  if [ -s "$stdout_log" ]; then
    printf '%s\n' "--- converter stdout ---" >&2
    cat "$stdout_log" >&2
  fi
  if [ -s "$stderr_log" ]; then
    printf '%s\n' "--- converter stderr ---" >&2
    cat "$stderr_log" >&2
  fi
  rm -f "$stdout_log" "$stderr_log"
  cleanup_converter_logs "$work_dir"
  rm -rf "$work_dir"
  return 1
}

convert_vcd_to_fsdb() {
  local source="$1"
  local output="$2"
  local output_dir
  local tmp
  local stdout_log
  local stderr_log

  output_dir="$(dirname "$output")"
  mkdir -p "$output_dir"

  if [ -f "$output" ] && [ "$output" -nt "$source" ]; then
    printf '%s\n' "info: fsdb fixture: up to date $output"
    return 0
  fi

  tmp="$output.tmp.$$"
  stdout_log="$tmp.stdout.log"
  stderr_log="$tmp.stderr.log"
  rm -f "$tmp" "$stdout_log" "$stderr_log"

  if ! run_converter "$source" "$stdout_log" "$stderr_log" vcd2fsdb "$source" -o "$tmp"; then
    rm -f "$tmp"
    exit 1
  fi

  mv "$tmp" "$output"
  printf '%s\n' "info: fsdb fixture: converted $source -> $output"
}

convert_fst_to_fsdb() {
  local source="$1"
  local output="$2"
  local tmp_dir
  local tmp_vcd
  local tmp_fsdb
  local fst_stdout_log
  local fst_stderr_log
  local vcd_stdout_log
  local vcd_stderr_log

  if [ -f "$output" ] && [ "$output" -nt "$source" ]; then
    printf '%s\n' "info: fsdb fixture: up to date $output"
    return 0
  fi

  mkdir -p "$tmp_root/rtl-artifacts"
  tmp_dir="$(mktemp -d "$tmp_root/rtl-artifacts/convert.XXXXXX")"
  tmp_vcd="$tmp_dir/$(basename "${source%.fst}").vcd"
  tmp_fsdb="$output.tmp.$$"
  fst_stdout_log="$tmp_dir/fst2vcd.stdout.log"
  fst_stderr_log="$tmp_dir/fst2vcd.stderr.log"
  vcd_stdout_log="$tmp_dir/vcd2fsdb.stdout.log"
  vcd_stderr_log="$tmp_dir/vcd2fsdb.stderr.log"

  rm -f "$tmp_fsdb"
  if ! run_converter "$source" "$fst_stdout_log" "$fst_stderr_log" fst2vcd -f "$source" -o "$tmp_vcd"; then
    rm -rf "$tmp_dir"
    rm -f "$tmp_fsdb"
    exit 1
  fi
  if ! run_converter "$tmp_vcd" "$vcd_stdout_log" "$vcd_stderr_log" vcd2fsdb "$tmp_vcd" -o "$tmp_fsdb"; then
    rm -rf "$tmp_dir"
    rm -f "$tmp_fsdb"
    exit 1
  fi

  mv "$tmp_fsdb" "$output"
  rm -rf "$tmp_dir"
  printf '%s\n' "info: fsdb fixture: converted $source -> $output"
}

convert_vcd_fixtures() {
  local sources=()
  local source
  local base
  local output
  declare -A seen_outputs=()

  mkdir -p "$fsdb_fixtures_dir"
  while IFS= read -r -d '' source; do
    sources+=("$source")
  done < <(find "$hand_fixtures_dir" -type f -name '*.vcd' -print0 | sort -z)

  if [ "${#sources[@]}" -eq 0 ]; then
    printf '%s\n' "error: fsdb fixture: no VCD fixtures found under tests/fixtures/hand" >&2
    exit 1
  fi

  for source in "${sources[@]}"; do
    base="$(basename "${source%.vcd}")"
    output="$fsdb_fixtures_dir/$base.fsdb"
    if [ -n "${seen_outputs[$output]:-}" ]; then
      printf '%s\n' "error: fsdb fixture: duplicate FSDB fixture output basename for $source and ${seen_outputs[$output]}" >&2
      exit 1
    fi
    seen_outputs[$output]="$source"
    convert_vcd_to_fsdb "$source" "$output"
  done
}

convert_rtl_fst_artifacts() {
  local rtl_dir
  local sources=()
  local filtered_sources=()
  local source
  local output

  rtl_dir="$rtl_artifacts_dir"
  if [ ! -d "$rtl_dir" ]; then
    printf '%s\n' "error: fsdb fixture: RTL artifact directory does not exist: $rtl_dir" >&2
    printf '%s\n' "error: fsdb fixture: rebuild or enter the wavepeek devcontainer before preparing RTL FSDB benchmark fixtures" >&2
    exit 1
  fi

  while IFS= read -r -d '' source; do
    sources+=("$source")
  done < <(find "$rtl_dir" -maxdepth 1 -type f -name '*.fst' -print0 | sort -z)

  if [ -n "$rtl_filter" ]; then
    for source in "${sources[@]}"; do
      if [[ "$(basename "$source")" =~ $rtl_filter ]]; then
        filtered_sources+=("$source")
      fi
    done
    sources=("${filtered_sources[@]}")
  fi

  if [ "${#sources[@]}" -eq 0 ]; then
    if [ -n "$rtl_filter" ]; then
      printf '%s\n' "error: fsdb fixture: no RTL FST artifacts under $rtl_dir matched filter: $rtl_filter" >&2
      exit 1
    fi
    printf '%s\n' "info: fsdb fixture: no RTL FST artifacts found under $rtl_dir"
    return 0
  fi

  if [ ! -w "$rtl_dir" ]; then
    printf '%s\n' "error: fsdb fixture: RTL artifact directory is not writable: $rtl_dir" >&2
    printf '%s\n' "error: fsdb fixture: rebuild or enter the wavepeek devcontainer before preparing RTL FSDB benchmark fixtures" >&2
    exit 1
  fi

  require_tool fst2vcd "install GTKWave tools or use the devcontainer before preparing RTL FSDB benchmark fixtures"

  for source in "${sources[@]}"; do
    output="${source%.fst}.fsdb"
    convert_fst_to_fsdb "$source" "$output"
  done
}

main() {
  parse_args "$@"
  require_tool vcd2fsdb "load the Verdi environment before preparing FSDB fixtures"
  mkdir -p "$tmp_root"
  case "$mode" in
    all)
      convert_vcd_fixtures
      convert_rtl_fst_artifacts
      ;;
    hand)
      convert_vcd_fixtures
      ;;
    rtl)
      convert_rtl_fst_artifacts
      ;;
    *)
      printf '%s\n' "error: fsdb fixture: internal invalid mode: $mode" >&2
      exit 2
      ;;
  esac
}

main "$@"
