#!/usr/bin/env bash
set -euo pipefail

# Expose only the selected reader, without Xcelium's bundled C++ runtime.
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
library="$(python3 -B "$root/tools/shm/check_shm_env.py" --require --print-library)"
runtime="$root/tmp/shm-runtime"
mkdir -p "$runtime"
ln -sfn "$library" "$runtime/libsst2.so"
export LD_LIBRARY_PATH="$runtime${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
exec "$@"
