# Schema Tools

This group owns validation for the canonical JSON schema artifact at `schema/wavepeek.json`.

Normal entrypoints:

    just update-schema
    just check-schema

`just check-schema` runs `tools/schema/check_schema_contract.py` and verifies that the committed schema matches `wavepeek schema` plus the runtime `$schema` URL contract.
