# Schema Guidance

## Source of Truth

- Schema generation and validation entrypoints: `../justfile` (`update-schema`, `check-schema`)
- Schema contract checker: `../tools/schema/check_schema_contract.py`
- Machine-output contract: `../docs/public/reference/machine-output.md`

## Local Guidance

Regenerate the current major schema artifact with `just update-schema` when the schema generator owns the change, and always validate with `just check-schema`. Schema artifacts are named `wavepeek_vN.json` by tool major version. Keep prior major schema artifacts in this directory; they are public contracts that must remain available in the repository and documentation site. Deliberate edits to the current-major contract artifact are allowed when the runtime embeds that artifact; prove the runtime schema matches afterward and do not modify prior-major artifacts for current-major changes.
