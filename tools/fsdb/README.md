# FSDB Tools

This group owns local FSDB environment probes, fixture preparation, and benchmark artifact checks surfaced through `just` recipes. The full FSDB development contract lives in `../../docs/fsdb.md`.

Normal entrypoints:

    just check-fsdb-env
    just prepare-fsdb-fixtures
    just prepare-fsdb-test-fixtures
    just check-fsdb-ondas-fixtures
    just test-fsdb
    just bench-e2e-fsdb-smoke

Focused tests:

    python3 -B -m unittest discover -s tools/fsdb -p "test_*.py"

Keep these helpers deterministic, non-interactive, and free of proprietary Verdi payloads.

Local FSDB inputs use `tests/fixtures/fsdb/`. The E2E catalog points to that directory for source-backed fixtures.

External inputs use the declared Ondas fixture directories under `ONDAS_FIXTURES_DIR`. Derived VCD/FSDB files stay beside `waveform.fst`; `--rtl-filter` matches fixture directory IDs.
