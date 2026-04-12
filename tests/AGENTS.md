# Test Suite Guide

This directory contains integration and fixture-driven tests.

## Parent Map

- Repository agent map: [`../AGENTS.md`](../AGENTS.md)

## Child Maps

- Command-runtime fixture manifests: [`fixtures/cli/AGENTS.md`](fixtures/cli/AGENTS.md)

## Source of Truth

- Testing workflow and conventions: [`../docs/DEVELOPMENT.md`](../docs/DEVELOPMENT.md)
- Product behavior and output contracts: [`../docs/design/contracts/command_model.md`](../docs/design/contracts/command_model.md), [`../docs/design/contracts/machine_output.md`](../docs/design/contracts/machine_output.md)
- Expression behavior for trigger and eval surfaces: [`../docs/design/contracts/expression_lang.md`](../docs/design/contracts/expression_lang.md)

Tests should validate current contracts from those documents.

## Expression Integration Policy

- Capability suites stay split by engine surface in `tests/expression_parse.rs`, `tests/expression_event_runtime.rs`, `tests/expression_integral_boolean.rs`, and `tests/expression_rich_types.rs`.
- Shared expression manifests live under `tests/fixtures/expr/` and deserialize through the common contracts in `tests/common/expr_cases.rs`.
- Positive manifests use one tagged `cases` array with `event_parse`, `logical_eval`, and `event_eval` rows.
- Negative manifests use one `cases` array with explicit `entrypoint`, `layer`, `code`, `span`, optional `snapshot`, required `host_profile` for `entrypoint: "logical"` or `entrypoint: "event"`, optional inline `signals` only for `host_profile: "custom"`, and required `timestamp` for runtime failures.
- `entrypoint: "parse"` negatives must not declare `host_profile`, inline `signals`, or `timestamp`.
- Shared manifests currently reject `entrypoint: "event"` with `layer: "runtime"`; keep those event-runtime failures as code-only tests until the schema grows explicit event-eval frame support.
- Shared runtime fixtures and assertion helpers live in `tests/common/expr_runtime.rs`; use `SignalFixture`, `SignalSample`, `TypeFixture`, `ExpectedValueFixture`, enum metadata (`EnumLabelFixture`, `enum_type_id`, `enum_labels`), and the named baseline host profiles there instead of suite-local host models.

## Code-Only Exceptions

- Keep generated or instrumented checks in Rust when JSON would hide the reason the test exists.
- Current intentional code-only exceptions are the parser no-panic corpus in `tests/expression_parse.rs`, the event short-circuit and change-command parity checks in `tests/expression_event_runtime.rs`, and the logical short-circuit sample-trap check in `tests/expression_integral_boolean.rs`.

## Snapshot Policy

- Keep an Insta snapshot only when rendered diagnostic text, notes, or source echo are part of the contract.
- Routine negative cases should rely on shared structured assertions for layer, code, and span.
- `tests/expression_fixture_contract.rs` is the durable guardrail: it verifies every expression manifest uses the shared schema, every referenced snapshot exists, and no `tests/snapshots/expression_*.snap` file is orphaned.
