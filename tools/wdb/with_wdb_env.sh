#!/usr/bin/env bash
# Run a command with the build-selected Vivado runtime.
set -euo pipefail

sdk=$(realpath -e "${XILINX_VIVADO:?set XILINX_VIVADO to a compatible Vivado installation}")
if [[ ! -d "$sdk/data" || ! -d "$sdk/lib/lnx64.o" ]]; then
    printf '%s\n' 'error: wdb: XILINX_VIVADO must provide a data directory and 64-bit reader libraries' >&2
    exit 1
fi
export XILINX_VIVADO="$sdk"
export RDI_DATADIR="$sdk/data"
export LD_LIBRARY_PATH="$sdk/lib/lnx64.o${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
exec "$@"
