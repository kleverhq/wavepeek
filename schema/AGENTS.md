# Schema Guide

This directory contains canonical JSON schema artifacts shipped with the
repository.

## Parent Map

- Repository map: `../AGENTS.md`

## Source of Truth

- Schema generation and validation entrypoints: `../justfile` (`update-schema`, `check-schema`)
- Schema contract checker: `../scripts/check_schema_contract.py`
- CLI schema command and JSON envelope contract: `../docs/public/reference/machine-output.md`

Prefer updating schema artifacts via `just update-schema` and validating with
`just check-schema`.
