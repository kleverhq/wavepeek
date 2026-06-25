# Schema Guidance

## Source of Truth

- Schema generation and validation entrypoints: `../justfile` (`update-schema`, `check-schema`)
- Schema contract checker: `../tools/schema/check_schema_contract.py`
- Machine-output contract: `../docs/public/reference/machine-output.md`

## Local Guidance

Regenerate current schema artifacts with `just update-schema` when the runtime owns the change, and always validate with `just check-schema`. Historical v0/v1 schema artifacts are named `wavepeek_vN.json` by tool major version; v2+ artifacts are named by exact major.minor version, such as `wavepeek_v2.0.json`. Keep prior schema artifacts in this directory; they are public contracts that must remain available in the repository and documentation site. Deliberate edits to current contract artifacts are allowed when the runtime embeds those artifacts; prove the runtime schemas match afterward and do not modify prior artifacts for current changes.
