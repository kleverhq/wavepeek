# Schema Guidance

## Source of Truth

- Schema generation and validation entrypoints: `../justfile` (`update-schema`, `check-schema`)
- Schema contract checking interface: `../justfile` (`check-schema`)
- Machine-output contract: `../docs/public/reference/machine-output.md`

## Local Guidance

Regenerate schema artifacts with `just update-schema` and validate with `just check-schema`. Do not hand-edit `wavepeek.json` unless you are deliberately repairing generated output and can prove the runtime schema matches afterward.
