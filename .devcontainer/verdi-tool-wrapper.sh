#!/usr/bin/env bash
set -euo pipefail

cmd="$(basename "$0")"
verdi_home="${VERDI_HOME:-/opt/verdi}"
tool="$verdi_home/bin/$cmd"

if [ ! -e "$tool" ]; then
    printf 'error: %s: %s not found; set VERDI_HOME\n' "$cmd" "$tool" >&2
    exit 127
fi

# Some Verdi distributions install command symlinks that point at a shared
# launcher containing bash arrays despite a /bin/sh shebang. Invoke it with
# bash explicitly; the launcher still sees the symlink name via $0 and dispatches
# to the real tool.
exec /bin/bash "$tool" "$@"
