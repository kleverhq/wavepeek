# Type AXI extract output by profile and channel

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

Note that this document must be maintained in accordance with the `exec-plan` skill.

## Purpose / Big Picture

`wavepeek extract axi` exists so downstream tools can consume AXI traffic as AXI-shaped data rather than protocol-neutral rows. The current JSON and JSONL output contains AXI channel names and payload objects, but the output schema describes those fields as generic strings and arbitrary objects. A visualizer or analyzer cannot read the schema and know which profiles exist, which channels exist, or which payload keys may appear for a channel.

After this change, each AXI transfer row will carry its `profile`, including JSONL item rows, and the generated output and stream schemas will enumerate AXI profiles, channels, and channel payload keys. A consumer can validate that `axi4-lite` write-address rows use only `awaddr` and `awprot`, that AXI3 write-data rows may use `wid`, and that AXI4 write-data rows cannot use `wid`.

## Non-Goals

This change does not reconstruct AXI bursts, ordering, outstanding state, or complete transactions. It does not add AXI5 or ACE. It does not require every optional or unmapped signal to appear in a row; payload keys remain present only when the waveform mapping included that signal. It does not change generic extract output.

## Progress

- [x] (2026-07-06T05:49Z) Confirmed the gap: current schemas use `profile: string`, `channel: string`, and `payload` with arbitrary keys for AXI transfer rows.
- [x] (2026-07-06T05:49Z) Created this ExecPlan for the typed AXI output contract fix.
- [x] (2026-07-06T06:05Z) Exposed AXI profile/channel metadata from the runtime so the schema generator uses the same profile tables as extraction.
- [x] (2026-07-06T06:05Z) Added `profile` to every AXI transfer row, including JSONL item rows.
- [x] (2026-07-06T06:05Z) Generated typed AXI schema branches keyed by `profile` and `channel`, with payload keys enumerated per profile and channel.
- [x] (2026-07-06T06:05Z) Added tests that prove valid profile/channel payload combinations pass and invalid combinations fail.
- [x] (2026-07-06T06:05Z) Regenerated schema artifacts and updated public docs for the `profile` row field and typed AXI schema behavior.
- [x] (2026-07-06T06:05Z) Ran focused tests and lint: `cargo test -q --test extract_axi_cli`, `cargo test -q --test schema_cli`, `just check-schema`, and `cargo clippy --all-targets -- -D warnings` passed.
- [x] (2026-07-06T06:16Z) Ran final gate: `just check` passed.
- [x] (2026-07-06T06:16Z) Ran focused subagent review for typed AXI schema/output changes; it returned no substantive findings.
- [x] (2026-07-06T06:17Z) Committed the typed AXI output change as `72b4804 fix(extract): type AXI transfer output`.
- [x] (2026-07-06T06:18Z) Pushed the typed AXI output commit to `origin/feat-extract-axi`.
- [x] (2026-07-06T18:35Z) Reviewed maintainability concern: typed AXI schema generation works but leaves too much manual builder code in `src/contract/schema.rs`.
- [x] (2026-07-06T18:40Z) Moved AXI-specific schema builder code into `src/contract/axi_schema.rs` without changing generated schema output.
- [x] (2026-07-06T18:40Z) Added a unit test proving AXI transfer schema definitions follow runtime AXI profile/channel metadata.
- [x] (2026-07-06T18:40Z) Ran focused checks: `cargo test -q axi_schema::tests::transfer_defs_follow_runtime_profiles_and_channels`, `cargo test -q --test schema_cli schema_output_validator_enforces_axi_profile_channel_payloads`, and `just check-schema` passed.
- [x] (2026-07-06T18:40Z) Ran final gate: `just check` passed.
- [x] (2026-07-06T18:40Z) Ran focused subagent review for the schema refactor; it returned no substantive findings.
- [x] (2026-07-06T18:41Z) Committed the cleanup as `6e50560 refactor(contract): isolate AXI schema generation`.
- [x] (2026-07-06T18:42Z) Pushed the cleanup commit to `origin/feat-extract-axi`.

## Surprises & Discoveries

- Observation: The runtime already owns the necessary AXI profile tables.
  Evidence: `src/engine/axi.rs` defines AXI3, AXI4, and AXI4-Lite channel signal arrays based on Arm IHI 0022H.c tables A2-1 through A2-6 and B1-1.

- Observation: The generated schemas do not currently expose that runtime knowledge.
  Evidence: `schema/output.json` defines `extractAxiTransfer.channel` as `type: string` and `extractAxiTransfer.payload` as an object with arbitrary keys whose values are sampled values.

- Observation: A schema regression test fails against the old schema and passes after regenerating typed AXI schemas.
  Evidence: Before `just update-schema`, `schema_output_validator_enforces_axi_profile_channel_payloads` accepted an AXI4-Lite AW payload containing `awlen`; after regeneration it rejects that payload and accepts `awaddr`/`awprot`.

- Observation: JSONL row typing now has independent profile context.
  Evidence: `tests/extract_axi_cli.rs::extract_axi_source_jsonl_includes_begin_context` validates that the first item row includes `item.profile == "axi4-lite"`.

## Decision Log

- Decision: Put `profile` directly on each AXI transfer row rather than creating a JSONL-only wrapper.
  Rationale: JSONL records are validated independently, so each item row needs its own profile for schema branches to be self-contained. Reusing the same transfer shape for JSON and JSONL is simpler than maintaining separate row contracts.
  Date/Author: 2026-07-06 / agent

- Decision: Enumerate allowed payload keys but do not make them required.
  Rationale: A profile defines which signals may exist on a channel, but a waveform mapping can include only a subset. Requiring every optional or unmapped signal would reject valid extraction output.
  Date/Author: 2026-07-06 / agent

## Outcomes & Retrospective

The typed output behavior and maintainability cleanup are implemented, committed, and pushed. AXI transfer rows include `profile`; generated output and stream schemas enumerate profile/channel payload branches. AXI-specific schema generation is isolated in `src/contract/axi_schema.rs`, generated schema output remains stable, focused schema and AXI CLI tests, `just check-schema`, clippy, `just check`, and focused subagent review passed.

## Context and Orientation

The repository is a Rust CLI named `wavepeek`. The AXI extractor lives in `src/engine/axi.rs`. It maps waveform signals to lowercase AXI standard signal names such as `awvalid`, `awaddr`, and `rdata`, builds generic extract sources internally, then converts generic rows into AXI transfer rows.

The output contract types live in `src/contract/output.rs`. These Rust structs are serialized for JSON output and are also fed to `schemars` to generate JSON Schema definitions. JSONL stream wrappers live in `src/contract/stream.rs`; JSONL begin records can carry context and item records carry one row each. Schema assembly lives in `src/contract/schema.rs`, which combines generated schema definitions and manual command-specific wrappers into `schema/output.json` and `schema/stream.json`.

An AXI profile means one supported protocol class: `axi3`, `axi4`, or `axi4-lite`. A channel means one AXI ready/valid channel: `aw`, `w`, `b`, `ar`, or `r`. A payload key is a lowercase AXI standard signal name sampled on a successful ready/valid transfer after excluding the channel handshake signals and global `aclk`/`aresetn`.

The supported payload keys are derived from Arm IHI 0022H.c. AXI3 and AXI4 use tables A2-1 through A2-6. AXI4-Lite uses table B1-1. Runtime constants already encode these lists.

## Open Questions

There are no open product questions. The implementation should keep the current schema family version unless the branch maintainer requests a new version, because this work is still on the feature branch that introduced schema v2.2.

## Plan of Work

For the cleanup milestone, move the AXI-specific schema construction helpers from `src/contract/schema.rs` into a new focused module such as `src/contract/axi_schema.rs`. Keep `src/contract/schema.rs` responsible for top-level schema assembly only. The new module should expose small functions that mutate the `$defs` map for output and stream schemas, and it should continue to source profile/channel/payload metadata from `src/engine/axi.rs`. The generated `schema/output.json` and `schema/stream.json` should not change except for ordering if unavoidable; ideally `just check-schema` proves they are byte-stable.

Add a unit test in the AXI schema module that compares emitted transfer branch counts and representative payload properties against `axi::profile_specs()` so future profile-table changes cannot silently desynchronize schema construction. Re-run schema and CLI tests after the refactor.

Original typed-output implementation plan follows for context. First, expose a crate-local read-only view of the AXI profile metadata from `src/engine/axi.rs`. The schema generator should not duplicate the signal lists by hand; it should consume the same runtime tables to avoid drift.

Next, add `profile` to `crate::engine::axi::AxiTransfer` and set it in `GenericToAxiSink::emit` from the current `AxiContext`. Because `src/contract/output.rs::ExtractAxiTransfer` converts from the engine transfer, adding the field there will make both JSON array rows and JSONL item rows include `profile`.

Then replace or post-process the generated `extractAxiTransfer` schema in `src/contract/schema.rs` with a manual `oneOf` over every supported profile/channel pair. Each branch must require `time`, `sample_time`, `profile`, `channel`, and `payload`; set `profile` and `channel` as `const`; and define `payload.properties` as the allowed payload keys for that profile and channel. Payload properties should reference `sampledValue`. Payload objects should reject unknown keys so the schema is useful for typed consumers.

If it remains simple, also type `extractAxiData.profile` and `extractAxiContext.profile` as an enum of supported profiles, and type `mappings` keys with the known standard names for the profile. Do not add speculative abstractions or a new dependency.

Finally, update tests in `tests/extract_axi_cli.rs` and `tests/schema_cli.rs`, regenerate schema artifacts with `just update-schema`, run focused checks and `just check`, ask a subagent for focused review, commit, and push.

## Concrete Steps

Run all commands from `/workspaces/wavepeek/.worktrees/feat-extract-axi`.

1. Inspect the current working tree:

    git status --short --branch

   Keep untracked local inspection files such as `axi.json` and `axi.jsonl` out of commits unless the user explicitly asks to add them.

2. Edit `src/engine/axi.rs` to expose metadata and add `profile` to `AxiTransfer`.

3. Edit `src/contract/output.rs` so `ExtractAxiTransfer` serializes `profile`.

4. Edit `src/contract/schema.rs` so generated output and stream schemas use profile/channel-specific AXI transfer definitions.

5. Update tests:

    cargo test -q --test extract_axi_cli
    cargo test -q --test schema_cli schema_output_validator_enforces_axi_profile_channel_payloads
    cargo test -q --test schema_cli schema_stream_validator_rejects_command_payload_mismatches

6. Regenerate schemas and run checks:

    just update-schema
    just check-schema
    cargo test -q --test extract_axi_cli
    cargo test -q --test schema_cli
    cargo clippy --all-targets -- -D warnings
    just check

7. Request a focused read-only subagent review covering runtime output shape, schema strictness, JSONL self-contained profile rows, and tests.

8. Commit only the intended tracked files and push to `origin/feat-extract-axi`.

## Validation and Acceptance

Acceptance requires observable behavior. Running `wavepeek extract axi --jsonl` on the AXI CLI fixture should produce item records whose `item.profile` is `axi4-lite` or the selected profile. Running `wavepeek schema --stream` should show `extractAxiTransfer` as profile/channel-specific schema branches. The new schema tests must reject an AXI4-Lite AW row with `awlen` and reject an AXI4 W row with `wid`, while accepting the corresponding valid profile/channel rows.

The final validation gate is `just check` passing after schema regeneration. Focused tests must include `tests/extract_axi_cli.rs` and `tests/schema_cli.rs`.

## Idempotence and Recovery

Schema regeneration with `just update-schema` is safe to rerun. If a schema edit produces invalid artifacts, rerun `just update-schema` after fixing Rust schema generation code rather than editing `schema/*.json` manually. If a test fails because the output shape changed, update the test expectation only when the changed shape matches this plan. Keep untracked local inspection files out of `git add`.

## Artifacts and Notes

Current problematic schema excerpt before the fix:

    "extractAxiTransfer": {
      "properties": {
        "channel": { "type": "string" },
        "payload": {
          "additionalProperties": { "$ref": "#/$defs/sampledValue" },
          "type": "object"
        }
      }
    }

Expected shape after the fix, shown schematically:

    "extractAxiTransfer": {
      "oneOf": [
        {
          "properties": {
            "profile": { "const": "axi4-lite" },
            "channel": { "const": "aw" },
            "payload": {
              "properties": {
                "awaddr": { "$ref": "#/$defs/sampledValue" },
                "awprot": { "$ref": "#/$defs/sampledValue" }
              },
              "additionalProperties": false
            }
          }
        }
      ]
    }

## Interfaces and Dependencies

Use only existing dependencies: `serde`, `schemars`, and `serde_json`. Do not add a new crate.

At the end, `crate::engine::axi::AxiTransfer` must contain a `profile: String` field. `crate::contract::output::ExtractAxiTransfer` must serialize that field. `src/contract/schema.rs` must have helper functions that build AXI profile/channel schema branches from `crate::engine::axi` metadata instead of hard-coded duplicate tables.

## Revision Notes

- 2026-07-06: Initial plan created after identifying that AXI output schema does not expose profile-dependent channel and payload contracts.
- 2026-07-06: Added cleanup milestone after code review concern that `src/contract/schema.rs` now contains too much AXI-specific manual schema builder code. The cleanup should preserve output behavior and generated schema artifacts.
- 2026-07-06: Updated after completing and pushing the schema-generation cleanup.
