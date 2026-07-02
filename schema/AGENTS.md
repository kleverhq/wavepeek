# Schema Guidance

## Source of Truth

- Schema generation and validation entrypoints: `../justfile` (`update-schema`, `check-schema`)
- Schema contract checker: `../tools/schema/check_schema_contract.py`
- Machine-output contract: `../docs/public/reference/machine-output.md`

## Local Guidance

Regenerate current schema snapshots with `just update-schema` when the Rust contract owns the change, and always validate with `just check-schema`. Current snapshots are `output.json`, `stream.json`, `input.json`, and `catalog.json`; do not edit them manually. Historical schema artifacts remain public contracts through release tags and GitHub Pages; do not duplicate them in the current `schema/` directory.
