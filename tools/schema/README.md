# Schema Tools

This group owns validation for the current JSON schema artifacts. Historical v0/v1 artifacts use major-only names such as `schema/wavepeek_v1.json`; current v2+ artifacts use exact major.minor names such as `schema/wavepeek_v2.0.json`.

Normal entrypoints:

    just update-schema
    just check-schema

`just check-schema` runs `tools/schema/check_schema_contract.py` and verifies that the committed current schema artifacts match `wavepeek schema` and `wavepeek schema --stream`, the artifacts' `$schema` URL patterns, and the runtime `$schema` URL contract.
