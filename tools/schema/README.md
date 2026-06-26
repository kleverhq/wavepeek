# Schema Tools

This group owns validation for generated JSON schema snapshots. Current snapshots are `schema/output.json`, `schema/stream.json`, and `schema/catalog.json`; historical artifacts such as `schema/wavepeek_v1.json` remain available for published contracts.

Normal entrypoints:

    just update-schema
    just check-schema

`just check-schema` runs `tools/schema-gen` into `tmp/schema-check`, then runs `tools/schema/check_schema_contract.py`. The check verifies generated snapshot freshness, `wavepeek schema` and `wavepeek schema --stream` byte matches, catalog URLs, and runtime `$schema` values.
