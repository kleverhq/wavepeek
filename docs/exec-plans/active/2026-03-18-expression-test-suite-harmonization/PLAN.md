# Expression Test Suite Harmonization

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

Note that this document must be maintained in accordance with `exec-plan` skill.

## Purpose / Big Picture

After this plan is implemented, a contributor will be able to answer two practical questions without reverse-engineering four separate test files: where a new `src/expr` regression belongs, and which fixture shape to use when adding it. The expression integration suites under `tests/` will keep their current capability-oriented split (`parse`, `event runtime`, `integral boolean`, and `rich types`), but they will share one positive manifest vocabulary, one negative manifest vocabulary, one runtime fixture vocabulary, and one explicit rule for which checks stay as code-only tests.

The result will be observable in three ways. First, the four existing capability suites will still pass, but their local serde structs and ad hoc host-fixture shapes will be gone. Second, a new manifest-contract test will prove that every expression manifest deserializes through the shared schema in `tests/common/expr_cases.rs` and `tests/common/expr_runtime.rs`. Third, the remaining snapshots under `tests/snapshots/` will be obviously purposeful: they will lock representative rendered diagnostics, while routine negative cases will rely on structured assertions for layer, code, and span.

This work is a test-surface cleanup, not an expression-engine feature change. `src/expr/` semantics, command routing, benchmark targets, and roadmap boundaries from `docs/expression_roadmap.md` must remain unchanged. The cleanup only changes how current coverage is organized, expressed, and maintained.

## Non-Goals

This plan does not change any expression parsing, binding, evaluation, or CLI behavior. It does not replace the current capability-oriented split with a top-level split such as "all positives" versus "all negatives," because the four existing suites intentionally map to capability areas that mirror the live engine surface. It does not remove the unit tests that already live inside `src/expr/*.rs`, because those tests protect low-level lexer, parser, binder, evaluator, and legacy-wrapper invariants that are awkward to express as integration fixtures. It does not add benchmark work, and it does not broaden snapshot coverage to every negative case.

## Progress

- [x] (2026-03-18 20:09Z) Reviewed `docs/expression_roadmap.md`, `docs/DEVELOPMENT.md`, `tests/AGENTS.md`, `src/AGENTS.md`, the four expression integration suites, the shared `tests/common/` helpers, the `tests/fixtures/expr/` manifests, the existing diagnostic snapshots, and the recent completed cleanup plan that introduced the capability-oriented suite names.
- [x] (2026-03-18 20:09Z) Audited the current test surface and recorded the end-state policy for this cleanup: keep the four capability suites, collapse the ad hoc manifest and runtime fixture shapes into shared contracts, move only truly data-driven code checks into manifests, and keep snapshots only where rendered diagnostic text is part of the contract.
- [x] (2026-03-18 20:22Z) Completed focused docs and architecture review lanes on the draft plan, then revised the contract-test rollout, explicit negative-case host context, runtime-negative timestamp rule, snapshot-reference checks, and final validation gates so every committed milestone stays compatible with repository hooks.
- [x] (2026-03-18 20:22Z) Completed a fresh control pass on the revised plan; no substantive remaining issues were reported.
- [x] (2026-03-19 19:55Z) Re-ran the baseline capability suites before editing; `cargo test --test expression_parse --test expression_event_runtime --test expression_integral_boolean --test expression_rich_types` passed cleanly, so the refactor started from a green expression surface.
- [x] (2026-03-19 19:55Z) Added the shared manifest/runtime contract in `tests/common/expr_cases.rs`, `tests/common/expr_runtime.rs`, and the new `tests/expression_fixture_contract.rs`, then captured the intended red phase where the contract test rejected the old parse/event manifest shapes.
- [x] (2026-03-19 19:55Z) Migrated `tests/expression_parse.rs`, `tests/expression_event_runtime.rs`, and their four manifests to the shared contract; removed suite-local serde structs plus the event-runtime-only signal fixtures; and restored snapshot stability through the shared assertion helper so `cargo test --test expression_parse --test expression_event_runtime --test expression_fixture_contract` now passes.
- [x] (2026-03-19 20:06Z) Migrated `tests/expression_integral_boolean.rs`, `tests/expression_rich_types.rs`, and their four manifests to the shared contracts; widened `tests/expression_fixture_contract.rs` from the temporary allowlist to the final full-directory scan; and moved the manifest-friendly logical/event regression tables out of Rust code and into shared JSON fixtures.
- [x] (2026-03-19 20:08Z) Audited the remaining `tests/snapshots/expression_*.snap` files, kept the current set because each one still locks rendered diagnostics that structured assertions do not cover alone, documented the end-state policy in `tests/AGENTS.md`, and passed `INSTA_UPDATE=always cargo test --test expression_parse --test expression_event_runtime --test expression_integral_boolean --test expression_rich_types`, `cargo test --test expression_fixture_contract`, and `make check`.
- [ ] Run the final multi-lane review workflow, resolve any findings, rerun the affected validation, and close the branch with `make ci`.

## Surprises & Discoveries

- Observation: the current top-level split is already capability-oriented rather than purely historical.
  Evidence: `tests/expression_parse.rs`, `tests/expression_event_runtime.rs`, `tests/expression_integral_boolean.rs`, and `tests/expression_rich_types.rs` match the feature taxonomy chosen in `docs/exec-plans/completed/2026-03-13-expression-cohesion-cleanup/PLAN.md`.

- Observation: the manifest surface already uses capability filenames, but not a shared schema.
  Evidence: `tests/fixtures/expr/` contains eight files with at least five distinct shapes: parser positive, generic negative, event-runtime positive with a narrowed signal schema, mixed logical/event positive, and rich negative with per-case host overrides.

- Observation: the event-runtime suite still carries a duplicate signal fixture model even after the earlier cohesion cleanup.
  Evidence: `tests/expression_event_runtime.rs` defines `EventRuntimeSignalFixture` and `EventRuntimeSignalSample`, while `tests/common/expr_runtime.rs` already exposes the richer shared `SignalFixture`, `SignalSample`, and `TypeFixture` types used by the other suites.

- Observation: several code-only tests are pure data tables in disguise, while others genuinely require code because they depend on instrumentation or external behavior.
  Evidence: `tests/expression_integral_boolean.rs` and `tests/expression_rich_types.rs` contain pure `eval_expr_at(...)` assertions that can become fixture rows, but `tests/expression_parse.rs` no-panic corpus generation, `tests/expression_event_runtime.rs` legacy CLI parity, and the short-circuit trap tests depend on generated inputs, sampling traps, or spawned commands.

- Observation: snapshot usage is already intentionally sparse rather than blanket-applied.
  Evidence: there are many more negative cases than snapshots in `tests/snapshots/`, and the existing `.snap` files mostly protect rendered message and note text for parse grouping, metadata fallback, and runtime cast failures rather than generic "wrong signal" cases.

- Observation: centralizing snapshot assertions changes Insta's default output path and identifier unless the helper pins both settings explicitly.
  Evidence: the first shared-helper run created pending files under `tests/common/snapshots/` such as `expression_parse__common__expr_cases__parse_unmatched_open.snap.new` instead of reusing `tests/snapshots/expression_parse__parse_unmatched_open.snap`.

- Observation: logical expressions involving `real` truthiness still report a four-state 1-bit result type, while `.triggered` stays two-state.
  Evidence: the first rich-type manifest migration failed on `real_truthiness_not` because the actual `ExprType` reported `is_four_state = true`, whereas the already-existing `.triggered` positive manifest rows continue to pass with `is_four_state = false`.

- Observation: the post-migration snapshot set was already minimal once the structured negative assertions and full-directory snapshot reference check were in place.
  Evidence: after the contract test began enforcing both snapshot existence and orphan detection, the remaining directory still contained only the nine long-lived `expression_*.snap` files, all referenced by the negative manifests that intentionally protect rendered notes or source echoes.

## Decision Log

- Decision: keep the four existing capability suites as the top-level organization.
  Rationale: those file names already describe the current engine surface better than a global positive/negative split would, and they line up with the intended taxonomy from the March 13 cohesion cleanup.
  Date/Author: 2026-03-18 / OpenCode

- Decision: standardize on two shared manifest contracts, one positive and one negative, while keeping the current capability-specific manifest file names.
  Rationale: the user confusion comes from ad hoc schema drift, not from the filenames themselves. Keeping `parse_positive_manifest.json`, `event_runtime_positive_manifest.json`, and the analogous negative files preserves easy navigation while eliminating schema churn.
  Date/Author: 2026-03-18 / OpenCode

- Decision: make the shared positive manifest a tagged union rather than one giant record with many optional fields.
  Rationale: parser-positive cases, logical-evaluation cases, and event-evaluation cases all need different expected payloads. A tagged union keeps the schema explicit and novice-readable while still counting as one positive contract.
  Date/Author: 2026-03-18 / OpenCode

- Decision: move only manifest-friendly data-driven assertions into fixtures and keep generated, instrumented, and external-parity checks as code.
  Rationale: forcing short-circuit trap tests, no-panic corpus generation, or spawned-CLI parity through JSON would hide the reason those tests exist and would produce a weaker design than the current code.
  Date/Author: 2026-03-18 / OpenCode

- Decision: keep snapshots only for representative rendered-diagnostic families and push routine diagnostics through shared structured assertions.
  Rationale: snapshots are valuable because they lock user-facing diagnostic text, notes, and source rendering, but snapshotting every negative row would add churn without protecting a stronger contract.
  Date/Author: 2026-03-18 / OpenCode

- Decision: document the end-state testing policy locally in `tests/AGENTS.md` rather than broadening repository-wide workflow docs.
  Rationale: this cleanup changes the organization of expression integration tests specifically. The repository-wide testing guidance in `docs/DEVELOPMENT.md` is already sufficient, while the local test map is the right place to explain fixture shapes, code-only exceptions, and snapshot policy.
  Date/Author: 2026-03-18 / OpenCode

- Decision: roll the new fixture-contract test out in two committable stages: an initial migrated-manifest allowlist and a final full-directory scan.
  Rationale: repository hooks run `cargo test` at commit time, so no atomic milestone may rely on a knowingly failing contract test. The temporary allowlist keeps early migration milestones green, while the final directory scan still enforces the durable all-manifest contract.
  Date/Author: 2026-03-18 / OpenCode

- Decision: require non-parse negatives to declare `host_profile`, and require runtime negatives to declare `timestamp` explicitly.
  Rationale: the main ambiguity in the current negative fixtures is hidden execution context. Making host and runtime time selection explicit removes suite-local defaults from the contract and makes the manifests self-contained.
  Date/Author: 2026-03-18 / OpenCode

- Decision: route snapshot assertions through one shared helper, but pin Insta to `../snapshots` and explicit suite-prefixed snapshot identifiers.
  Rationale: the helper should remove duplicated assertion code without renaming or relocating the committed `expression_*.snap` files that already represent the user-facing diagnostic contract.
  Date/Author: 2026-03-19 / OpenCode

- Decision: encode the observed result-type details in the migrated manifests instead of keeping type-light code-only assertions for rich-type regressions.
  Rationale: the harmonized suite is more valuable when the manifests spell out the engine's actual type contract, including the non-obvious four-state result type for `real` truthiness/logical operators, rather than leaving that detail implicit in ad hoc Rust assertions.
  Date/Author: 2026-03-19 / OpenCode

## Outcomes & Retrospective

Current status: implementation and pre-review validation are complete. The branch has finished the contract migration, documented the end-state testing policy, and is waiting only on the required review workflow plus final `make ci` closeout.

The work so far confirms the migration shape from the plan. The full-directory contract test now catches both schema drift and stale snapshot references immediately, the integral/rich suites no longer carry local manifest structs or duplicated eval/type helpers, and the final docs pass makes the remaining code-only exceptions explicit for the next contributor. The last remaining work is quality gating through the mandated review lanes and final CI validation.

## Context and Orientation

`wavepeek` is a single-crate Rust repository. The expression engine lives in `src/expr/`, with low-level unit tests embedded directly in files such as `src/expr/lexer.rs`, `src/expr/parser.rs`, `src/expr/sema.rs`, and `src/expr/eval.rs`. Those unit tests protect lexer spans, parser acceptance, binder invariants, evaluator behavior, and the legacy event-expression wrapper. They are not the target of this refactor except to leave them in place and keep them green.

The higher-level expression conformance coverage lives under `tests/`. Four integration suites matter here. `tests/expression_parse.rs` covers parser-only behavior for event expressions, including AST normalization, negative parse diagnostics, and a generated no-panic corpus. `tests/expression_event_runtime.rs` covers standalone event binding and matching, including bounded `iff` behavior, negative parse and semantic diagnostics, short-circuit preservation, and parity with the legacy `wavepeek change --on` path. `tests/expression_integral_boolean.rs` covers the integral-family logical surface and event `iff` reuse. `tests/expression_rich_types.rs` covers the richer `real`, `string`, enum-label, and `.triggered` surface plus event cases that depend on that richer logical engine.

Two shared helper modules already exist. `tests/common/expr_cases.rs` loads manifests from `tests/fixtures/expr/` and converts string layers such as `parse` and `semantic` into `wavepeek::expr::DiagnosticLayer`. `tests/common/expr_runtime.rs` defines an in-memory `ExpressionHost` implementation plus shared signal, sample, type, and enum-label fixtures. The helper split is useful, but the suites still duplicate too much around those helpers.

For this plan, a "manifest" means a committed JSON file under `tests/fixtures/expr/` that drives one or more integration tests. A "positive manifest" means rows that must parse, bind, and evaluate successfully and therefore need expected outputs. A "negative manifest" means rows that must fail deterministically with a known diagnostic layer, code, and span. A "runtime fixture" means the JSON or Rust structure that describes signal types, sampled values, and raw event timestamps for the in-memory expression host. A "code-only regression" means a test that stays in Rust because it depends on generated inputs, sample-count instrumentation, panic-catching, or an external process. A "snapshot" means an Insta `.snap` file that locks the rendered user-facing diagnostic string rather than only the structured layer/code/span fields.

The current pain points are concrete. `tests/expression_event_runtime.rs` still defines its own `EventRuntimeSignalFixture` even though the other suites use the common signal fixture model. Each integration file declares its own `PositiveManifest` and `NegativeManifest` structs even when the fields are nearly identical. `tests/expression_integral_boolean.rs` and `tests/expression_rich_types.rs` both duplicate type-assertion and evaluation helper logic. The JSON manifests express similar ideas in different shapes, which makes it hard to know whether a new regression belongs in a fixture, a suite-local helper, or a code-only test.

The roadmap context matters only as a boundary reminder. `docs/expression_roadmap.md` says the standalone engine has already delivered through `C4`; this cleanup must not reintroduce phase-coded naming or turn the suites back into rollout buckets. The completed March 13 cohesion cleanup intentionally moved the live tests to capability-oriented names. This plan preserves that decision while making the underlying contracts simpler and more uniform.

## Open Questions

There are no blocking product questions. The cleanup is structural and the target behavior is clear.

One implementation-time choice should be resolved conservatively: if a suite-by-suite migration is easier with temporary serde compatibility shims that accept both the old and new positive-manifest shapes, that is acceptable during the migration, but the durable `tests/expression_fixture_contract.rs` test must not be weakened by those shims. The final branch must prove two things at once: every current manifest file uses the new shared schema, and representative legacy payloads are rejected.

## Plan of Work

Milestone 1 establishes the target contract before the suite files are rewritten. Add one durable integration test, `tests/expression_fixture_contract.rs`, that imports `tests/common/mod.rs` and exercises the shared helper types. In the first committed milestone, that test should cover representative inline positive and negative payloads, explicit rejection of legacy payloads, and an explicit allowlist of the manifest files migrated in the same milestone. Expand `tests/common/expr_cases.rs` so it owns the common manifest schema and negative-diagnostic assertion helpers. Expand `tests/common/expr_runtime.rs` so it owns the shared runtime fixture schema, event-match collection helpers, and shared type/value assertion helpers. In the final committed milestone, widen the contract test from the temporary allowlist to a full scan of `tests/fixtures/expr/` so newly added manifests cannot be silently skipped.

Milestone 2 migrates the parser and event-runtime suites onto the shared contract. `tests/expression_parse.rs` should stop declaring suite-local serde structs and should load shared positive and negative manifests through `tests/common/expr_cases.rs`. `tests/expression_event_runtime.rs` should stop declaring `EventRuntimeSignalFixture` and should use the shared `SignalFixture` and helper functions from `tests/common/expr_runtime.rs`. Keep the suite-local responsibilities intact: parser AST normalization and no-panic generation stay in `tests/expression_parse.rs`, while event matching, short-circuit preservation, and legacy CLI parity stay in `tests/expression_event_runtime.rs`.

Milestone 3 migrates the integral-boolean and rich-types suites and then moves manifest-friendly code checks out of Rust. `tests/expression_integral_boolean.rs` and `tests/expression_rich_types.rs` should stop declaring local manifest structs and shared assertion helpers that duplicate `tests/common/expr_cases.rs` or `tests/common/expr_runtime.rs`. Split mixed code-only tests if needed: move pure data-driven `eval_expr_at(...)` or event-match checks into the positive or negative manifests, but keep the no-panic corpus, short-circuit sample-trap tests, and legacy CLI parity as code. At the end of this milestone, the remaining code-only tests should be small, obviously special, and easy to justify.

Milestone 4 applies the snapshot and documentation policy and closes validation. Review every remaining snapshot under `tests/snapshots/` that begins with `expression_`. Keep a snapshot only if it protects rendered diagnostic message, note text, source echo, or note absence that is not already covered by layer/code/span assertions. Remove or avoid redundant snapshots for routine failures once shared structured assertions cover them. The contract test should also verify that every snapshot referenced from a negative manifest resolves to a committed snapshot file and that there are no unreferenced `expression_*.snap` leftovers. Update `tests/AGENTS.md` so a contributor can see the final policy in one place: capability suites at the top level, shared positive and negative manifest contracts in `tests/common/expr_cases.rs`, runtime fixtures in `tests/common/expr_runtime.rs`, code-only exceptions for generated or instrumented behavior, and selective snapshot usage for rendered diagnostics.

Milestone 5 closes the branch through validation, review, and cleanup. Run the new fixture-contract test, the four capability suites, and then the repository validation gates from this repository's documented workflow. After the branch is green, run the mandatory review workflow by loading the `ask-review` skill and requesting at least a code lane for correctness and regressions plus an architecture lane for helper boundaries and fixture policy. If `tests/AGENTS.md` changes materially, add a docs lane as well. Resolve findings, rerun the affected validation, and then make a final clean control pass on the consolidated diff before merging.

### Concrete Steps

Run all commands from `/workspaces/feat-cmd-property`.

1. Confirm the current baseline before touching helpers or fixtures.

       cargo test --test expression_parse --test expression_event_runtime --test expression_integral_boolean --test expression_rich_types

   Expect all four integration suites to pass before the refactor starts. If any suite is already failing for unrelated reasons, stop and record that discovery in this plan before restructuring tests.

2. Add the shared contract test and helper interfaces first.

   Create `tests/expression_fixture_contract.rs`. Its first committed form should cover representative inline positive and negative rows, representative legacy payloads that must be rejected, and a temporary explicit allowlist containing only the manifests that will be migrated in the same milestone. At the same time, extend `tests/common/expr_cases.rs` so it defines the end-state manifest contract and shared diagnostic-assertion helpers, and extend `tests/common/expr_runtime.rs` so it defines the end-state runtime fixture helpers.

   The positive manifest contract should have one root shape:

       {
         "cases": [
           {
             "kind": "event_parse",
             "name": "iff_binds_to_preceding_term_only",
             "source": "negedge clk iff rstn or ready",
             "terms": [ ... ]
           },
           {
             "kind": "logical_eval",
             "name": "real_mixed_numeric_result",
             "source": "1.5 + count",
             "signals": [ ... ],
             "timestamp": 0,
             "expected_type": { ... },
             "expected_result": { ... }
           },
           {
             "kind": "event_eval",
             "name": "iff_inside_and_concatenation",
             "source": "posedge clk iff ({a,b}[3:0] inside {[4'd9:4'd10]})",
             "tracked_signals": ["clk"],
             "signals": [ ... ],
             "probes": [0, 4, 7, 9],
             "matches": [4]
           }
         ]
       }

   The negative manifest contract should have one root shape:

       {
         "cases": [
           {
             "name": "invalid_const_part_select_bound",
             "entrypoint": "logical",
             "source": "a[idx:0]",
             "layer": "semantic",
             "code": "EXPR-SEMANTIC-CONST-REQUIRED",
             "span": { "start": 2, "end": 5 },
             "snapshot": "semantic_const_required_part_select",
             "host_profile": "custom",
             "signals": [ ... required when host_profile is custom ... ],
             "timestamp": 0
           }
         ]
       }

   Parse-only negative cases may omit `host_profile`, `signals`, and `timestamp`. Any non-parse negative case must make its execution context explicit. Use a stable named `host_profile` such as `event_runtime_baseline`, `integral_boolean_baseline`, or `rich_types_baseline` when a shared default host is intended, and use `host_profile: "custom"` plus inline `signals` when a case needs its own host data. Any case that is expected to fail at runtime must provide an explicit `timestamp`; the shared helper should reject runtime-negative rows that omit it. Do not keep suite-local hidden defaults that are only discoverable by reading Rust setup code.

   Red-phase command:

       cargo test --test expression_fixture_contract

   Expected red evidence: the contract test fails because the parse and event-runtime manifests named in its initial allowlist still use the old suite-local JSON shape, because a representative legacy payload is still accepted when it should now be rejected, or because a runtime-negative row without `timestamp` is incorrectly allowed.

3. Migrate the parse and event-runtime suites to the shared contract.

   Update `tests/expression_parse.rs`, `tests/expression_event_runtime.rs`, `tests/common/expr_cases.rs`, `tests/common/expr_runtime.rs`, `tests/common/mod.rs`, and the four parser and event-runtime manifests under `tests/fixtures/expr/`. Remove suite-local manifest structs and the event-runtime-only signal fixture structs. Update the contract test's allowlist so it names exactly those four migrated manifests and passes against them. If temporary serde compatibility is useful while the JSON files move, keep it confined to the shared helper module and delete it before the milestone commit lands.

   Commands after this step:

       cargo test --test expression_parse
       cargo test --test expression_event_runtime
       cargo test --test expression_fixture_contract

   Acceptance for this step: `tests/expression_event_runtime.rs` no longer defines `EventRuntimeSignalFixture`, both suites load manifests through shared helpers, and all three commands pass. The contract test should still cover only the four migrated manifests plus inline rejection cases at this point, so the milestone remains committable under the repository's enforced hooks.

   Commit this milestone as one atomic unit. Recommended message:

       test(expr): unify parse and event fixture contracts

4. Migrate the integral-boolean and rich-types suites and move manifest-friendly code checks out of Rust.

   Update `tests/expression_integral_boolean.rs`, `tests/expression_rich_types.rs`, the remaining four manifests under `tests/fixtures/expr/`, and the shared helper modules. Move pure data-driven logical-evaluation and event-evaluation checks into manifests. Keep only the generated no-panic corpus, sample-trap short-circuit checks, legacy CLI parity, and any other clearly instrumented or external-process cases as code. In the same milestone, widen `tests/expression_fixture_contract.rs` from the temporary allowlist to the final full-directory scan so all eight manifests, the runtime-negative timestamp rule, the legacy-shape rejection checks, and the snapshot-reference checks are enforced together.

   During this step, remove suite-local duplicates such as repeated `assert_expr_type_eq(...)`, local `eval_expr_at(...)` wrappers that do not encode special behavior, and direct repeated diagnostic assertion blocks if they can be replaced with shared helpers.

   Commands after this step:

       cargo test --test expression_integral_boolean
       cargo test --test expression_rich_types
       cargo test --test expression_fixture_contract

   Acceptance for this step: all manifests deserialize through the shared contract, representative legacy payloads are rejected by the contract test, the four capability suites still exist, and the remaining code-only tests are explainable by behavior rather than by historical accident.

   Commit this milestone as one atomic unit. Recommended message:

       test(expr): migrate logical and rich suites to shared manifests

5. Apply the snapshot policy, document it, and close validation.

   Review `tests/snapshots/expression_*.snap`, the negative-case rows that still reference snapshots, and `tests/AGENTS.md`. Remove any snapshot whose rendered contract is already fully captured by layer/code/span assertions. Keep or add snapshots only when rendered diagnostic text or notes must remain stable. Update `tests/AGENTS.md` so it names the final suite split, the shared positive and negative fixture contracts, the code-only exceptions, and the selective snapshot rule.

   Commands after this step:

       INSTA_UPDATE=always cargo test --test expression_parse --test expression_event_runtime --test expression_integral_boolean --test expression_rich_types
       cargo test --test expression_fixture_contract
       make check

   If snapshot changes occur, review every diff before accepting it. `make check` is the pre-handoff acceptance that the refactor still satisfies the repository's normal non-test quality gate.

   Commit this milestone as one atomic unit. Recommended message:

       test(expr): document fixture policy and prune snapshots

6. Run the mandatory review workflow and final control pass.

   Load the `ask-review` skill and request at least two focused review lanes on the final diff: a code lane for regression risk, fixture migration mistakes, and missing coverage; and an architecture lane for helper boundaries, suite responsibilities, and the fixture-versus-code policy. If `tests/AGENTS.md` materially changes wording, add a docs lane in parallel. Provide each lane with the scope summary, focus files, validation results, and the explicit note that this branch changes tests and manifests but not `src/expr` semantics. After resolving findings, rerun the affected validation commands and then request one fresh independent control pass on the consolidated diff.

   Final commands:

       cargo test --test expression_fixture_contract
       cargo test --test expression_parse --test expression_event_runtime --test expression_integral_boolean --test expression_rich_types
       make ci

   Acceptance for closeout: the review lanes and the control pass report no unresolved substantive findings, all commands pass, and the final branch contains only the shared manifest shapes plus the deliberate code-only exceptions.

### Validation and Acceptance

The feature is complete when a novice can verify five things directly from the working tree and the test results. First, `cargo test --test expression_fixture_contract` passes and proves that every current expression manifest uses the common schema, every retained snapshot is still referenced, representative legacy manifest payloads are rejected, and runtime-negative rows without an explicit `timestamp` are rejected. Second, the four capability suites still pass and still describe the current engine surface through `tests/expression_parse.rs`, `tests/expression_event_runtime.rs`, `tests/expression_integral_boolean.rs`, and `tests/expression_rich_types.rs`. Third, `tests/expression_event_runtime.rs` no longer contains a private signal fixture model, and the other suites no longer declare their own positive or negative manifest structs. Fourth, non-parse negative cases make their host context explicit through a named `host_profile` or a fully inline custom signal set rather than through suite-local hidden defaults. Fifth, the remaining snapshots are easy to justify because they lock user-facing rendered diagnostics rather than duplicating plain structured assertions.

The most important human-readable acceptance checks are:

       rg "struct (PositiveManifest|NegativeManifest|EventRuntimeSignalFixture)" tests/expression_*.rs
       rg "insta::assert_snapshot!" tests/expression_*.rs tests/common

The first command should return no suite-local manifest or event-runtime fixture structs. The second command should show either one shared snapshot helper in `tests/common/` or a very small number of intentional call sites that are easy to explain.

### Idempotence and Recovery

This refactor should be performed additively and suite-by-suite. It is safe to rerun the test commands as often as needed. If a partial migration leaves one suite green and another red, keep the compatibility shim or intermediate helper only long enough to finish the next atomic step, then remove it before the milestone commit lands. Do not leave support for both old and new manifest shapes in the final branch.

If a manifest rewrite accidentally changes semantics rather than structure, recover by restoring the previous JSON content from git for that one file, re-running the affected suite, and then reapplying only the schema translation. If snapshot churn becomes noisy, stop using `INSTA_UPDATE=always` until the structured assertions and the negative rows are stable again, then refresh snapshots deliberately.

### Artifacts and Notes

The durable proof artifacts for this cleanup should be small and committed. The most important artifacts are the new `tests/expression_fixture_contract.rs` file, the simplified helper APIs in `tests/common/expr_cases.rs` and `tests/common/expr_runtime.rs`, the updated JSON manifests in `tests/fixtures/expr/`, and any remaining snapshots in `tests/snapshots/`.

Keep the final helper split simple:

- `tests/common/expr_cases.rs` owns manifest loading, shared positive and negative manifest types, span and layer decoding, and the shared diagnostic assertion path including optional snapshot checks.
- `tests/common/expr_runtime.rs` owns `SignalFixture`, `SignalSample`, `TypeFixture`, the in-memory expression host, event-match collection helpers, logical-evaluation helpers, and shared type or value assertions.
- The four suite files own only capability-specific execution and the code-only regressions that cannot be expressed through the shared fixtures.

If an intermediate step needs a temporary helper or translation shim, record that decision in `Progress`, keep it local to the shared helper modules, and delete it before the final milestone commit.

### Interfaces and Dependencies

The final implementation should keep using the existing repository testing stack: Rust integration tests in `tests/`, serde-driven JSON fixtures, the in-memory `ExpressionHost` in `tests/common/expr_runtime.rs`, and Insta snapshots under `tests/snapshots/`.

In `tests/common/expr_cases.rs`, define a shared positive-manifest API and a shared negative-manifest API. The positive API should be a tagged union with stable case kinds that cover parser-positive, logical-evaluation, and event-evaluation scenarios. The negative API should be one struct shape with `entrypoint`, `source`, `layer`, `code`, `span`, optional `snapshot`, optional `host_profile`, optional `signals`, and optional `timestamp`, with the rules that non-parse negatives must make their host context explicit through a named profile or `custom` inline signals and runtime-negative cases must provide an explicit `timestamp`. That file should also expose one helper that checks `DiagnosticLayer`, diagnostic code, span, and the optional snapshot so suite files do not repeat that logic, plus one contract-test helper that rejects representative legacy shapes and runtime negatives that omit `timestamp`.

In `tests/common/expr_runtime.rs`, keep `SignalFixture`, `SignalSample`, `TypeFixture`, `EnumLabelFixture`, and `InMemoryExprHost` as the canonical runtime-fixture surface. Do not keep a second event-runtime-only fixture type. Add stable helpers for common operations that are currently duplicated across suites, such as collecting event matches over probe timestamps, evaluating a logical expression at a timestamp, comparing `ExprType`, and comparing expected runtime values. This module should also own the stable named host profiles used by negative cases so execution context lives in one place instead of being hidden in suite-local setup blocks.

In `tests/expression_parse.rs`, keep only parser-specific AST normalization and no-panic corpus generation. In `tests/expression_event_runtime.rs`, keep only event-runtime-specific evaluation, short-circuit sample-trap behavior, and legacy CLI parity. In `tests/expression_integral_boolean.rs` and `tests/expression_rich_types.rs`, keep only the suite-local behavior that cannot be expressed by the shared fixture contract after the migration. `tests/AGENTS.md` should be updated to describe exactly that split so future contributors do not recreate the current drift.

Change note: initially drafted on 2026-03-18 after auditing the current expression test surface, then revised the same day after review lanes and a clean control pass. The revision clarified the staged contract-test rollout, explicit negative-case host and timestamp rules, snapshot-reference checks, and repository-aligned validation gates. On 2026-03-19 the plan was updated again after implementation began to record the green baseline, the deliberate red-phase contract test, the completed parse/event migration, the shared snapshot-helper constraint needed to keep existing snapshot artifacts stable, and the later logical/rich migration that finished the full shared-manifest rollout.
