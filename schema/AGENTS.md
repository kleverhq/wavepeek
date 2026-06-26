# Schema Guidance

## Source of Truth

- Schema generation and validation entrypoints: `../justfile` (`update-schema`, `check-schema`)
- Schema contract checker: `../tools/schema/check_schema_contract.py`
- Machine-output contract: `../docs/public/reference/machine-output.md`

## Local Guidance

Regenerate current schema snapshots with `just update-schema` when the Rust contract owns the change, and always validate with `just check-schema`. Current snapshots are `output.json`, `stream.json`, and `catalog.json`; do not edit them manually. Historical artifacts such as `wavepeek_v1.json` and `wavepeek_v2.0.json` remain public contracts and must stay available in the repository and documentation site.
