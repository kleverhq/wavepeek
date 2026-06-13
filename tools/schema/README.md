# Schema Tools

This group owns validation for the current major JSON schema artifact, such as `schema/wavepeek_v1.json`.

Normal entrypoints:

    just update-schema
    just check-schema

`just check-schema` runs `tools/schema/check_schema_contract.py` and verifies that the committed current-major schema matches `wavepeek schema`, the schema artifact's envelope `$schema` URL pattern, and the runtime `$schema` URL contract.
