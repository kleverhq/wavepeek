# JSON schema data-field detail hardening

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

## Purpose / Big Picture

After this change, `wavepeek schema` will stop being a bag of vaguely typed strings for the most important command payload fields. Machine clients and LLM workflows will be able to see which `scope` and `signal` `kind` values are intentionally supported, and every payload field under `data` will explain itself in the schema instead of forcing readers to reverse-engineer meaning from examples.

A contributor can see the change working by running `wavepeek schema`, inspecting the richer `$defs` and field `description` text, and verifying that schema checks fail if the waveform adapter’s stable kind inventory and the checked-in schema enum lists drift apart.

## Non-Goals

This plan does not redesign the one-schema-per-binary approach or split contracts into per-command schema files.

This plan does not tighten canonical path syntax, time-string syntax, or sampled-value syntax with new regex restrictions. Those strings should become better documented, not more fragile.

This plan does not add GHW support, VHDL support, or new waveform formats. Instead, it explicitly keeps GHW- and VHDL-specific `kind` literals out of the stable machine contract.

This plan does not change JSON bytes for the current shipped contract fixtures. If unsupported backend-specific VHDL or GHW aliases can leak through today, the fix is to normalize them into the stable inventory rather than expanding the schema enums to advertise those backend-specific literals.

## Progress

- [x] (2026-05-16 00:00Z) Audited the backlog item in `docs/BACKLOG.md`, the current schema in `schema/wavepeek.json`, and the current command/runtime docs in `docs/public/reference/machine-output.md` and `docs/ARCHITECTURE.md`.
- [x] (2026-05-16 00:00Z) Located the current runtime sources for `scope` and `signal` kind strings in `src/waveform/mod.rs` (`scope_type_alias` and `var_type_alias`) and confirmed that the checked-in schema still models those fields as generic strings.
- [x] (2026-05-16 00:00Z) Resolved the user-requested policy that schema enums must exclude GHW- and VHDL-related kind values even though `wellen` exposes them.
- [x] (2026-05-16 00:00Z) Wrote the first draft of this implementation plan in `docs/exec-plans/active/2026-05-16-json-schema-data-field-detail-hardening/PLAN.md`.
- [x] (2026-05-16 00:20Z) Ran peer review with separate Grins on plan quality and technical correctness, then revised the plan to require runtime/schema alignment, alias-path drift protection, and mandatory runtime regression validation.
- [ ] Implementation is not started. Remaining work is to normalize the stable alias surface, harden the schema artifact, add drift protection, validate, and keep this plan current while coding.

## Surprises & Discoveries

- Observation: the current canonical schema already hardens `property.data[].kind` with an enum, but `scope.data[].kind` and `signal.data[].kind` are still plain strings.
  Evidence: `schema/wavepeek.json` defines `propertyData.items.properties.kind.enum`, while `scopeData.items.properties.kind` and `signalData.items.properties.kind` are only `{ "type": "string" }`.

- Observation: the runtime adapter already has explicit alias functions, but they include dormant VHDL and GHW values that do not match the intended stable contract for this backlog item.
  Evidence: `src/waveform/mod.rs` maps `ScopeType::Vhdl*`, `ScopeType::GhwGeneric`, `VarType::StdLogic*`, and `VarType::StdULogic*` to output strings.

- Observation: the existing `make check-schema` path only proves artifact freshness, byte equality, and envelope URL shape; it does not prove that schema enum inventories still match the waveform adapter’s stable alias inventory.
  Evidence: `scripts/check_schema_contract.py` compares `wavepeek schema` against `schema/wavepeek.json` and inspects `$schema`, but it never compares schema enum arrays with the adapter’s emitted stable kind inventory.

- Observation: `docs/public/reference/machine-output.md` treats `schema/wavepeek.json` as the exact contract for stable JSON output, so a curated schema enum that excludes literals still emitted at runtime would be dishonest rather than “extra strict.”
  Evidence: the reference says “The exact JSON shapes for every command are defined by `schema/wavepeek.json` and by `wavepeek schema`.”

- Observation: `make check` is not enough by itself to prove that `scope` and `signal` runtime JSON stayed stable, because `make check` does not include integration tests.
  Evidence: `docs/DEVELOPMENT.md` defines `make check` as format/lint/schema/build plus commit-message validation, while `make ci` is the test-inclusive gate.

## Decision Log

- Decision: treat schema-enumerated `scope` and `signal` kinds as an intentionally curated stable inventory, not as a mirror of every `wellen` enum variant.
  Rationale: the backlog item asks for stronger contracts, and the user explicitly requested that GHW- and VHDL-related kinds stay out of schema. Including dormant parser/backend variants would advertise support the product does not intend to stabilize here.
  Date/Author: 2026-05-16 / Grin

- Decision: the runtime adapter must emit only values from that stable inventory for stable JSON surfaces; excluded VHDL and GHW backend aliases must be normalized, not merely hidden from the schema.
  Rationale: `schema/wavepeek.json` is documented as the exact JSON contract. A schema enum that rejects real runtime output would be broken on arrival.
  Date/Author: 2026-05-16 / Grin

- Decision: keep `path`, `time`, `time_unit`, and sampled `value` fields as strings with concise descriptions instead of adding new regex or pattern constraints.
  Rationale: these fields are not closed sets, the repository still carries open design questions around path canonicalization, and over-constraining them would create brittle false failures without improving real interoperability much.
  Date/Author: 2026-05-16 / Grin

- Decision: add drift protection in two layers and run both layers from the schema gate: one focused schema test that compares canonical schema enums against the stable inventory, and one exhaustive waveform-adapter test that proves alias functions emit the exact stable mapping for every supported inventory entry plus the excluded backend-specific variants.
  Rationale: a manual constant compared only against the schema would still allow the real alias functions to drift. The tests must cover both the schema document and the actual alias-producing code path, and both must run under `make check-schema` so the standard gate catches regressions.
  Date/Author: 2026-05-16 / Grin

- Decision: keep `unknown` as a valid `scope` kind in the stable schema inventory, but do not add a new `unknown` signal kind unless implementation proves it is required.
  Rationale: `scope_type_alias` already has a stable fallback shape, while signal-type normalization can use existing generic kinds for the currently excluded VHDL literals.
  Date/Author: 2026-05-16 / Grin

- Decision: make runtime regression validation mandatory with targeted integration tests for `scope`, `signal`, and `schema`, not merely `make check`.
  Rationale: this work claims not to perturb current shipped JSON outputs. That claim must be tested directly.
  Date/Author: 2026-05-16 / Grin

## Outcomes & Retrospective

Planning is complete and implementation is pending. After one review loop, the plan is stricter and less hand-wavy: it now tells the implementer to make the runtime and schema agree, not to fake agreement with a curated enum and a disconnected constant.

The remaining risk is the usual one: someone will be tempted to patch only `schema/wavepeek.json` and call it a day. Do not. The whole point of this plan is to make the stable contract honest and self-checking.

## Context and Orientation

`wavepeek` ships one checked-in canonical JSON schema at `schema/wavepeek.json`. The top-level `schema` command does not generate anything dynamically; `src/engine/schema.rs` returns `crate::schema_contract::CANONICAL_SCHEMA_JSON`, and `src/schema_contract.rs` embeds the file with `include_str!`. That means every schema hardening change in this plan is first a source edit to `schema/wavepeek.json`, then a validation problem.

The two payload families that matter most here are `scope` and `signal`. Their JSON rows are constructed in `src/engine/scope.rs` and `src/engine/signal.rs`, but the `kind` strings themselves come from the waveform adapter in `src/waveform/mod.rs`. `scope_type_alias` converts `wellen::ScopeType` to strings. `var_type_alias` converts `wellen::VarType` to strings. Today those functions expose more backend-specific literals than the stable machine contract should admit, so implementation must make them emit only the stable surface described below.

The current schema under-documents many payload fields. For example, `infoData.time_unit`, `scopeData[].path`, `signalData[].path`, `valueData.time`, `changeData[].signals[].value`, `docsSearchData.query`, and several others are only typed as strings or integers with no explanation. Machine consumers can validate shape, but they still have to guess semantics.

This repository’s public product scope matters. `docs/public/intro.md` says the tool supports VCD and FST dumps. `docs/BACKLOG.md` keeps GHW support as an open design question, not a shipped contract. `docs/public/reference/machine-output.md` says `schema/wavepeek.json` defines the exact JSON shapes for stable JSON output. Therefore this plan intentionally treats the schema enum lists as the stable contract for the intended surface and requires runtime normalization so the emitted JSON stays inside that contract.

For this plan, use the following stable `scope.data[].kind` inventory, in deterministic order:

    module
    task
    function
    begin
    fork
    generate
    struct
    union
    class
    interface
    package
    program
    unknown

Excluded backend-specific scope literals must not appear in the schema enum and must not be emitted by the stable runtime surface. Normalize all of the following to `unknown`:

    vhdl_architecture
    vhdl_procedure
    vhdl_function
    vhdl_record
    vhdl_process
    vhdl_block
    vhdl_for_generate
    vhdl_if_generate
    vhdl_generate
    vhdl_package
    vhdl_array
    ghw_generic

For this plan, use the following stable `signal.data[].kind` inventory, in deterministic order:

    event
    integer
    parameter
    real
    reg
    supply0
    supply1
    time
    tri
    triand
    trior
    trireg
    tri0
    tri1
    wand
    wire
    wor
    string
    port
    sparse_array
    real_time
    real_parameter
    bit
    logic
    int
    short_int
    long_int
    byte
    enum
    short_real
    boolean
    bit_vector

Excluded backend-specific signal literals must not appear in the schema enum and must not be emitted by the stable runtime surface. Normalize them as follows:

    std_logic -> logic
    std_ulogic -> logic
    std_logic_vector -> bit_vector
    std_ulogic_vector -> bit_vector

These normalization rules are intentionally narrow. They exist only to keep the stable machine contract free of backend-specific VHDL naming while preserving useful coarse metadata.

## Open Questions

There are no blocking design questions left for this plan. If implementation discovers that one of the prescribed signal normalization mappings above is technically wrong for the current product surface, record the discovery here and pick a different mapping deliberately. Do not silently expand the schema enum with VHDL spellings just because `wellen` happens to know them.

## Plan of Work

Start in `src/waveform/mod.rs`. Add two crate-visible constants near the existing alias functions: one for stable schema scope kinds and one for stable schema signal kinds. These constants are the curated stable inventory, not a duplicate of every `wellen` variant. Add a short comment above them stating that they intentionally exclude GHW/VHDL backend spellings from the stable machine contract.

Then change `scope_type_alias` and `var_type_alias` so the stable JSON-producing runtime surface emits only those stable values. Keep the existing shipped fixture behavior unchanged, but normalize the currently excluded backend-specific literals according to the rules in the Context section. For scope kinds, that means collapsing the listed VHDL and GHW variants to `unknown`. For signal kinds, that means collapsing the listed VHDL spellings to `logic` or `bit_vector`. Add an exhaustive table-driven unit test in `src/waveform/mod.rs` that covers every stable `ScopeType` and `VarType` mapping represented in the schema inventory, plus the excluded backend-specific variants, and proves the alias helpers never emit literals outside the stable inventory.

After the runtime surface is honest, harden `schema/wavepeek.json`. Keep the top-level envelope structure intact, but enrich `$defs` and payload properties so the schema explains the command data instead of only describing shape. Add reusable definitions where they actually reduce duplication rather than for sport. A good minimum is to introduce reusable `$defs` for `scopeKind`, `signalKind`, and repeated object row shapes such as scope entries, signal entries, sampled signal rows, and change rows. Wire `scope.data[].kind` and `signal.data[].kind` to the new enum definitions so the schema now reflects the stable runtime inventory.

While editing the schema, add concise `description` text to payload fields under `$defs`. Keep the writing short and operational. Fields that should no longer be bare primitives include `infoData.time_unit`, `infoData.time_start`, `infoData.time_end`, `scopeData[].path`, `scopeData[].depth`, `scopeData[].kind`, `signalData[].name`, `signalData[].path`, `signalData[].kind`, `signalData[].width`, `valueData.time`, `valueData.signals[].path`, `valueData.signals[].value`, `changeData[].time`, `changeData[].signals[].path`, `changeData[].signals[].value`, `propertyData[].time`, `propertyData[].kind`, `topicSummary` properties, `docsSearchMatch.match_kind`, and `docsSearchData.query`. Do not waste effort adding descriptions to fields whose meaning is already fully conveyed by stable JSON Schema keywords alone unless the field is part of command payload semantics.

Once the schema is richer, add drift protection in `src/schema_contract.rs`. Parse `CANONICAL_SCHEMA_JSON`, extract the enum arrays used by `scopeKind` and `signalKind`, and compare them exactly to the stable inventory constants from `src/waveform/mod.rs`. Also assert that the excluded backend-specific literals are absent from those enum arrays. Name the test something durable, such as `schema_kind_alias_inventory_matches_canonical_schema`.

Do not stop there. Add or extend waveform adapter unit tests so the real alias functions are checked against the same rules. Use an exhaustive test such as `stable_schema_kind_aliases_cover_full_inventory` that calls the alias helpers directly for every supported stable mapping and every excluded `ScopeType` and `VarType` variant, then asserts that the returned literals stay inside the stable inventory and match the exact mapping rules from this plan. That is the missing guardrail between a curated constant and the real runtime path.

After the tests exist, wire both focused Rust tests into `make check-schema`. Update `Makefile` so `check-schema` still runs `scripts/check_schema_contract.py` and then runs the focused schema-inventory test plus the exhaustive waveform-adapter alias test by name. Keep both targeted tests inside `check-schema` rather than relying on the broader `cargo test` phase, because this backlog item is specifically about schema drift. The Python script does not need to parse Rust; let it keep doing byte-identity and `$schema` URL checks.

Finish by extending `tests/schema_cli.rs`. Add assertions that `wavepeek schema` exposes the expected `scope` and `signal` enum inventories, that representative payload fields carry non-empty descriptions, and that excluded VHDL/GHW literals are absent from the stable enums. Keep `tests/scope_cli.rs` and `tests/signal_cli.rs` behavior expectations intact unless implementation discovers that new targeted assertions are needed to prove existing JSON output stayed unchanged for the shipped fixtures. If such assertions are added, they should prove stability, not normalize new behavior into the fixtures by accident.

## Concrete Steps

Work from the repository root, `/workspaces/wavepeek`.

1. Edit `src/waveform/mod.rs`.

   Add the stable `scope` and `signal` kind inventory constants and update `scope_type_alias` plus `var_type_alias` so they emit only stable contract values.

2. Add exhaustive waveform adapter tests in `src/waveform/mod.rs`.

   Add a table-driven test that proves every stable `ScopeType` and `VarType` mapping in the schema inventory still maps to the expected stable literal, and that the excluded backend-specific variants collapse to `unknown`, `logic`, or `bit_vector` as specified above.

3. Edit `schema/wavepeek.json`.

   Expected shape after this step:

       jq '."$defs" | keys' schema/wavepeek.json

   should include new reusable definitions for the hardened `scope` and `signal` kinds, and:

       jq '."$defs".scopeKind.enum' schema/wavepeek.json
       jq '."$defs".signalKind.enum' schema/wavepeek.json

   should print exactly the stable inventories listed in the Context section.

4. Add the focused schema-vs-inventory test in `src/schema_contract.rs`.

5. Update `Makefile` so `check-schema` runs `scripts/check_schema_contract.py`, the focused Rust schema test, and the exhaustive waveform-adapter alias test.

6. Extend `tests/schema_cli.rs` with schema-metadata assertions.

7. Run focused validation first:

       cargo test --lib schema_kind_alias_inventory_matches_canonical_schema
       cargo test --lib stable_schema_kind_aliases_cover_full_inventory
       cargo test --test schema_cli --test scope_cli --test signal_cli

   Expected result: all pass. `schema_cli` proves the schema metadata is richer, and `scope_cli` plus `signal_cli` prove shipped fixture JSON did not change.

8. Run the repository schema and local gates:

       make check-schema
       make check

   Expected result: `make check-schema` passes without drift complaints, and `make check` stays green.

9. Run the test-inclusive parity gate before closing the work:

       make ci

   Expected result: the repository-level test, schema, and build gate stays green after the schema hardening and runtime normalization changes.

## Validation and Acceptance

Acceptance is behavioral, not decorative.

Run:

    cargo test --lib schema_kind_alias_inventory_matches_canonical_schema

and expect one focused library test to prove that the canonical schema enum lists exactly match the stable waveform-adapter inventories and exclude the backend-specific literals listed in the Context section.

Run:

    cargo test --lib stable_schema_kind_aliases_cover_full_inventory

and expect an exhaustive adapter test to prove both that every supported stable `ScopeType` and `VarType` still maps to the expected stable literal and that excluded backend-specific variants are normalized into the stable runtime inventory rather than leaking backend-specific spellings.

Run:

    cargo test --test schema_cli --test scope_cli --test signal_cli

and expect integration coverage to prove both halves of the claim: `wavepeek schema` exposes richer field metadata, and the current shipped `scope` plus `signal` outputs for existing fixtures still match their old JSON expectations.

Run:

    make check-schema

and expect it to fail if any of the following drift conditions are introduced: the checked-in schema stops matching `wavepeek schema` byte-for-byte, the envelope `$schema` URL is malformed, the legacy `schema_version` key reappears, the stable schema enum inventories stop matching the adapter’s stable alias inventory, or the real alias helpers drift from the full stable mapping promised by the schema.

Run:

    make check

and expect the normal repository gate to stay green.

Run:

    make ci

and expect the test-inclusive repository gate to stay green.

The change is done when `schema/wavepeek.json` is richer, `wavepeek schema` shows those richer definitions, `scope.data[].kind` and `signal.data[].kind` are enum-backed with the exact stable inventories above, excluded GHW/VHDL literals are absent from those stable enums and from the stable runtime output, and the schema gate plus the test-inclusive repository gate catch future drift automatically.

## Idempotence and Recovery

All steps in this plan are safe to repeat. Editing the schema artifact, rerunning the focused tests, and rerunning `make check-schema` are idempotent from a clean tree.

If `make check-schema` fails after the schema edit, fix the mismatch before touching anything else. The correct recovery order is: first make the exhaustive adapter alias test pass, then make the focused schema-inventory test pass, then make `wavepeek schema` and `schema/wavepeek.json` agree byte-for-byte, then rerun the broader gate.

If implementation accidentally changes `scope` or `signal` JSON output for the shipped fixtures, stop and inspect `tests/scope_cli.rs` or `tests/signal_cli.rs` failures before updating expectations. This plan does not authorize casual contract drift.

If a reviewer finds that a new description overstates semantics or promises unsupported formats, trim the wording in the schema rather than inventing runtime behavior to justify the prose. The schema exists to document the contract, not to write fan fiction about it.

## Artifacts and Notes

Useful source locations while implementing:

    docs/BACKLOG.md
    schema/wavepeek.json
    src/waveform/mod.rs
    src/schema_contract.rs
    src/engine/schema.rs
    scripts/check_schema_contract.py
    tests/schema_cli.rs
    tests/scope_cli.rs
    tests/signal_cli.rs

A concise success spot-check after implementation should look roughly like this:

    $ wavepeek schema | jq '."$defs".scopeKind.enum'
    [
      "module",
      "task",
      "function",
      "begin",
      "fork",
      "generate",
      "struct",
      "union",
      "class",
      "interface",
      "package",
      "program",
      "unknown"
    ]

and:

    $ wavepeek schema | jq -r '."$defs".signalData.items.properties.kind.description'
    Stable signal kind alias emitted by wavepeek for the selected signal.

If the exact wording differs, that is fine. What is not fine is leaving those fields undocumented or silently reintroducing excluded backend-specific literals.

## Interfaces and Dependencies

Keep using the existing static-schema architecture. `src/engine/schema.rs` must continue returning `crate::schema_contract::CANONICAL_SCHEMA_JSON` unchanged in shape. `src/schema_contract.rs` should remain the place that embeds the canonical artifact and owns schema-focused tests.

The implementation should continue depending on:

- `wellen` for waveform backend enums in `src/waveform/mod.rs`;
- `serde_json`, already present in the crate, for parsing schema JSON inside tests;
- the existing `Makefile` and `scripts/check_schema_contract.py` gate structure for schema validation.

At the end of implementation, the repository should contain one stable waveform-adapter kind inventory that both the runtime alias helpers and the schema tests rely on, a canonical schema artifact whose `scope` and `signal` kind fields point at stable enum definitions, and a `check-schema` gate that exercises both artifact freshness and alias-inventory drift.

Revision Note: 2026-05-16 / Grin — Initial plan authored from the backlog item, current schema artifact, waveform adapter alias functions, and the user-requested exclusion of GHW/VHDL literals. Revised after peer review to require runtime/schema alignment, exhaustive adapter-path drift tests wired into `make check-schema`, mandatory runtime regression validation, and a required `make ci` completion gate.