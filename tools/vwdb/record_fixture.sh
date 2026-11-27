#!/usr/bin/env bash
# Record source-backed CLI fixtures and print the new run directory.
set -euo pipefail

root=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)
mkdir -p "$root/tmp/vwdb-fixtures"
run_dir=$(mktemp -d "$root/tmp/vwdb-fixtures/run.XXXXXX")
xrun="$XCELIUM_HOME/tools/bin/64bit/xrun"
for name in "$@"; do
    work="$run_dir/$name"
    mkdir "$work"
    (
        cd "$work"
        printf '%s\n' 'call xmDumpvars top 0 +all' run exit > record.tcl
        "$xrun" -64bit -top top -access +rwc "$root/tests/fixtures/source/$name.v" \
            -input record.tcl > simulate.log 2>&1 &&
        test -s ida.db/ida.vwdb/vwdb_info.json
    ) || {
        printf 'error: vwdb: recording failed; see %s/simulate.log\n' "$work" >&2
        exit 1
    }
done
printf '%s\n' "$run_dir"
