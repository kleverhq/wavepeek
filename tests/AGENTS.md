# Test Suite Guidance

## Source of Truth

- Testing workflow and conventions: `../docs/dev/testing.md`
- Quality gates: `../docs/dev/quality.md`
- Product behavior and output contracts: `../docs/public/reference/command-model.md`, `../docs/public/reference/machine-output.md`
- Expression behavior for trigger and eval surfaces: `../docs/public/reference/expression-language.md`

## Command Fixture Policy

- Command-runtime fixture manifests live under `fixtures/cli/`.
- Shared loader and manifest validation live in `common/command_cases.rs`.
- Fixture contract coverage lives in `command_fixture_contract.rs`.
- Keep command manifests small, explicit, and limited to durable command contracts.

## Expression Integration Policy

- Capability suites stay split by engine surface in `expression_parse.rs`, `expression_event_runtime.rs`, `expression_integral_boolean.rs`, and `expression_rich_types.rs`.
- Shared expression manifests live under `fixtures/expr/` and deserialize through `common/expr_cases.rs`.
- Positive manifests use one tagged `cases` array with `event_parse`, `logical_eval`, and `event_eval` rows.
- Negative manifests use one `cases` array with explicit `entrypoint`, `layer`, `code`, `span`, optional `snapshot`, required `host_profile` for logical or event entrypoints, optional inline `signals` only for `host_profile: "custom"`, and required `timestamp` for runtime failures.
- `entrypoint: "parse"` negatives must not declare `host_profile`, inline `signals`, or `timestamp`.
- Shared manifests currently reject `entrypoint: "event"` with `layer: "runtime"`; keep those event-runtime failures as code-only tests until the schema grows explicit event-eval frame support.
- Shared runtime fixtures and assertion helpers live in `common/expr_runtime.rs`; use those named host profiles instead of suite-local host models.

## Snapshot Policy

Keep an Insta snapshot only when rendered diagnostic text, notes, or source echo are part of the contract. Routine negative cases should rely on shared structured assertions. `expression_fixture_contract.rs` verifies manifest schema usage, referenced snapshots, and orphaned `snapshots/expression_*.snap` files.
