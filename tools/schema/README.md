# Schema Tools

This group owns validation for generated JSON schema snapshots. Current snapshots are `schema/output.json`, `schema/stream.json`, `schema/input.json`, and `schema/catalog.json`. Historical schema artifacts remain available from release tags and GitHub Pages rather than duplicate files in the current `schema/` directory.

Normal entrypoints:

    just update-schema
    just check-schema

`just check-schema` runs `tools/schema-gen` into `tmp/schema-check`, then runs `tools/schema/check_schema_contract.py`. The check verifies generated snapshot freshness, `wavepeek schema`, `wavepeek schema --stream`, and `wavepeek schema --input` byte matches, catalog URLs, and runtime `$schema` values.
