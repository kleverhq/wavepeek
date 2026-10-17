# SHM helpers

`check_shm_env.py` probes the optional reader selected by `XCELIUM_HOME`. An unset
installation returns 77; an explicitly configured missing library returns 1.
Ondas owns ABI validation during the build.

`with_shm_env.sh COMMAND ...` exposes the selected `libsst2.so` through an ignored
symlink directory. It avoids adding Xcelium's bundled C++ runtime to the loader path.

`prepare_shm_fixtures.py` converts selected test VCDs and benchmark FSTs into
ignored SHM directories. It uses the 64-bit `simvisdbutil` launcher and requires a
local Cadence license. The rich-value fixture is recorded directly with `xrun`
from `tests/fixtures/source/shm_rich.sv` because conversion loses string types. A source/converter stamp enables reuse. Failed conversions
preserve prior output; directories without an ownership stamp are never replaced.

Run these helpers through root `just` recipes in `./dev`; see `docs/shm.md`.
