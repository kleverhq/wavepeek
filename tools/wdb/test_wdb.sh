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

# Released ondas-fixtures wdb0305-enum-core. Use its native enum and name metadata.
enum_wdb="$output/enum-core.wdb"
cache="$PWD/tmp/wdb/wdb0305-enum-core.wdb"
digest=96c2cb9d88fdd22296b26aac5b15d8826f1818bddf50433d82055d32665427dc
if [[ -f "$cache" ]]; then
    cp -- "$cache" "$enum_wdb"
else
    release=https://github.com/kleverhq/ondas-fixtures/releases/download/files-4aec75b57bad
    curl -fsSL --retry 3 --connect-timeout 15 --max-time 60 \
        "$release/wdb0305-enum-core.$digest.wdb.gz" | gzip -dc > "$enum_wdb"
fi
printf '%s  %s\n' "$digest" "$enum_wdb" | sha256sum -c
if [[ ! -f "$cache" ]]; then
    cp -- "$enum_wdb" "$output/enum-core.cached"
    mv -fT -- "$output/enum-core.cached" "$cache"
fi

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
