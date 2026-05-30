# Coverage Tools

This group owns the source-coverage summary checker used by `just coverage-src` and `just coverage-src-check`.

Normal entrypoints:

    just coverage-src
    just coverage-src-check

Focused tests:

    python3 -B -m unittest tools/coverage/test_check_coverage.py
