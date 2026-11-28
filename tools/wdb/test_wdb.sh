#!/usr/bin/env bash
# Record fixtures, run the WDB CLI tests, and check the loader outside Cargo.
set -euo pipefail

sdk=$(realpath -e "${XILINX_VIVADO:?set XILINX_VIVADO to an XSim installation}")
for tool in xvlog xelab xsim; do
    if [[ ! -x "$sdk/bin/$tool" ]]; then
        printf 'error: wdb: XILINX_VIVADO does not provide executable %s\n' "$tool" >&2
        exit 1
    fi
done

export XILINX_VIVADO="$sdk"
cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../.."
mkdir -p tmp/wdb
output=$(mktemp -d "$PWD/tmp/wdb/fixtures.XXXXXX")
export WAVEPEEK_WDB_FIXTURES="$output"
export CARGO_TARGET_DIR=target/wdb
source_dir="$PWD/tests/fixtures/source"

# Use the native enum and name metadata from the pinned fixture corpus.
enum_wdb="$output/enum-core.wdb"
digest=96c2cb9d88fdd22296b26aac5b15d8826f1818bddf50433d82055d32665427dc
source_wdb="${ONDAS_FIXTURES_DIR:?set ONDAS_FIXTURES_DIR to the pinned corpus}/wdb/wdb0305-enum-core/waveform.wdb"
if [[ ! -f "$source_wdb" ]]; then
    printf 'error: wdb: missing %s (rebuild the devcontainer image and run ./dev --recreate true)\n' "$source_wdb" >&2
    exit 1
fi
cp -- "$source_wdb" "$enum_wdb"
printf '%s  %s\n' "$digest" "$enum_wdb" | sha256sum -c

for name in m2_core extract_apb3; do
    mkdir "$output/$name"
    (
        cd "$output/$name"
        printf '%s\n' 'log_wave -recursive /top/*' 'run all' 'quit' > capture.tcl
        "$sdk/bin/xvlog" "$source_dir/$name.v"
        "$sdk/bin/xelab" -debug typical work.top -s fixture
        "$sdk/bin/xsim" fixture -tclbatch capture.tcl -wdb waveform.wdb
        test -s waveform.wdb
    )
done

bash tools/wdb/with_wdb_env.sh cargo test --locked --features wdb --test wdb_cli

result="$output/direct-value.json"
bash tools/wdb/with_wdb_env.sh target/wdb/debug/wavepeek value \
    --waves "$output/m2_core/waveform.wdb" --signals top.cpu.valid --at 5ns --json > "$result"
python3 -B - "$result" <<'PY'
import json
import sys

with open(sys.argv[1], encoding="utf-8") as stream:
    value = json.load(stream)["data"][0]["signals"][0]["value"]
if value != "1'h1":
    raise SystemExit(f"error: unexpected WDB value at 5ns: {value}")
print("ok: WDB direct value at 5ns")
PY

rm -r -- "$output"
