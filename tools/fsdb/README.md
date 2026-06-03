# FSDB Tools

This group owns local FSDB environment probes, fixture preparation, benchmark catalog generation, and benchmark artifact checks surfaced through `just` recipes.

Normal entrypoints:

    just check-fsdb-env
    just prepare-fsdb-fixtures
    just update-bench-e2e-fsdb-catalog
    just check-bench-e2e-fsdb-catalog
    just test-fsdb
    just bench-e2e-fsdb-smoke-commit

Focused tests:

    python3 -B -m unittest discover -s tools/fsdb -p "test_*.py"

Keep these helpers deterministic, non-interactive, and free of proprietary Verdi payloads.
