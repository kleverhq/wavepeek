# Schema Guide

This directory contains canonical JSON schema artifacts shipped with the
repository.

## Parent Map

- Repository map: [`../AGENTS.md`](../AGENTS.md)

## Source of Truth

- Schema generation and validation entrypoints: [`../Makefile`](../Makefile) (`update-schema`, `check-schema`)
- Schema contract checker: [`../scripts/check_schema_contract.py`](../scripts/check_schema_contract.py)
- CLI schema command and JSON envelope contract: [`../docs/design/contracts/machine_output.md`](../docs/design/contracts/machine_output.md)

Prefer updating schema artifacts via `make update-schema` and validating with
`make check-schema`.
