# Expression Implementation Roadmap

This document records the phased delivery that took the expression surface from
partial implementation to the full `docs/expression_lang.md` contract now
shipped in the repository.

It is a sequencing plan, not a language-spec replacement. Source-of-truth
syntax and semantics remain in `docs/expression_lang.md`, and the phase
breakdown below is retained as rollout history.

## Purpose

- Start point: staged support where standalone engine milestones can land before
  command integration.
- End point: full expression-language contract implemented, validated, and
  integrated in command runtimes with deterministic behavior.

## Current Baseline (post-C5 integration)

Current implementation snapshot:

- Standalone typed runtime in `src/expr/` supports the full delivered event and
  logical expression surface (`parse_*_ast(...)`, `bind_*_ast(...)`,
  `event_matches_at(...)`, `eval_logical_expr_at(...)`).
- `change --on` and `property` both execute through the shared typed command
  runtime path.
- Schema, parity, and benchmark gates cover the command-integrated surface.
- The phase breakdown below remains the historical rollout plan for how that
  boundary was reached.

Current delivered boundary is `C5`.

## Final Target (C5)

At `C5`, all language sections from `docs/expression_lang.md` are fully
implemented and command-integrated:

- Event expressions: complete grammar + runtime semantics.
- Boolean expressions: complete operator/type/cast/conversion semantics.
- Deterministic parser/semantic/runtime diagnostics.
- One shared expression engine used by `change` and `property`.
- Full conformance tests and benchmark regression gates.

## Release Alignment Note

`docs/ROADMAP.md` is version-oriented, while this document is contract-level and
phase-oriented (`C0`..`C5`). For expression work planning:

- this document is authoritative for sequencing and scope boundaries,
- each phase execution plan must state its intended roadmap/milestone mapping,
- roadmap milestone text should be updated when phase boundaries shift.

## Phase Overview

| Phase | Contract Level | Focus | Integration posture |
|---|---|---|---|
| 1 | C1 | Engine architecture + strict parser/diagnostics foundation | Standalone engine |
| 2 | C2 | Event-expression runtime + bounded `iff` logical subset | Standalone engine |
| 3 | C3 | Core integral boolean contract (excluding rich/event-member forms) | Standalone engine |
| 4 | C4 | Rich types, full grammar closure, full `iff logical_expr` surface | Standalone engine |
| 5 | C5 | Command integration (`change`/`property`) + parity/perf hardening | Production command paths |

## Phase Gate Standard (applies to every phase)

Each phase must define and pass all of the following artifacts:

- one execution plan in `docs/exec-plans/active/` that targets exactly one
  phase boundary,
- one locked positive test manifest and one locked negative test manifest (what
  must still fail deterministically at that phase),
- one benchmark baseline artifact and one compare run,
- deterministic diagnostics snapshots for new/changed error classes.

Default benchmark gate (unless a phase plan explicitly tightens it):

- mean regression <= 15% and median regression <= 15% versus approved baseline
  on fixed fixture set and fixed run profile.

## Phase Details

### Phase 1 (C1): Engine Foundation

Contract scope at phase exit:

- Stable internal architecture exists for:
  - lexer/token stream with spans,
  - parser/AST,
  - semantic binding/type-check entry points,
  - evaluator API,
  - host interface for symbol/type/value lookup.
- Parser strictness debt for malformed event syntax is resolved:
  unmatched `(` / `)`, empty `iff`, and broken union segmentation are rejected
  deterministically.
- Diagnostics contract is defined and tested for parse/semantic/runtime layers.

Out of scope:

- Event runtime evaluation semantics.
- Boolean operator semantics.
- Command-surface behavior changes in `change`/`property`.

Boundary invariants (must still fail deterministically):

- Any runtime expression evaluation path.
- Any `property` runtime execution path.

Exit criteria:

- Positive/negative parser matrices pass from locked manifests.
- Parser fuzz/no-panic tests pass.
- Diagnostic snapshots pass with stable formatting/category rules.
- Parse/tokenization benchmarks pass default regression gate.

Traceability:

- Design refs: `docs/DESIGN.md` (expression sections and current staged status).
- Backlog refs: parser strictness debt and lexer-scaffolding debt in
  `docs/BACKLOG.md`.

### Phase 2 (C2): Event Contract Core + `iff` Subset

Contract scope at phase exit:

- Event-expression runtime semantics are implemented for Section `1` event forms:
  `*`, named, edge terms, union (`or`, `,`), precedence, and `iff` term binding.
- `iff` predicate evaluation is enabled with a bounded logical subset:
  - operand references,
  - integral literals,
  - `!`, `<`, `<=`, `>`, `>=`, `==`, `!=`, `&&`, `||`, parentheses,
  - 4-state boolean behavior and short-circuit semantics for this subset.

Out of scope:

- Full Section `2` operator/type surface.
- Rich type domains (`enum`, `real`, `string`, `triggered()`).
- Command integration.

Boundary invariants (must still fail deterministically):

- Any Section `2` feature not listed above when used in `iff`.
- Full `property` runtime behavior.

Exit criteria:

- Event runtime conformance tests pass for Section `1` event semantics.
- Dedicated `iff` binding/evaluation subset tests pass.
- Unsupported-in-C2 `iff` features fail with deterministic diagnostics.
- Event-evaluation benchmarks pass default regression gate.

Traceability:

- Design refs: `docs/DESIGN.md` `change` behavior and staged `iff` note.
- Backlog refs: deferred `iff` runtime debt in `docs/BACKLOG.md`.

### Phase 3 (C3): Core Integral Boolean Contract

Contract scope at phase exit:

- Section `2` is complete for integral-family core semantics:
  - integral operand types/forms,
  - enum values participating as integral operands through underlying
    bit-vector semantics,
  - selection,
  - logical/bitwise/reduction,
  - exponentiation/arithmetic/shifts,
  - comparisons/equalities for integral + allowed mixed numeric forms,
  - conditional `?:` and `inside` (integral scope),
  - concatenation/replication,
  - precedence/associativity for operators in scope,
  - integral implicit conversions/common-type rules,
  - integral-target explicit casts.

Out of scope:

- Full `real` semantics.
- Full `string` semantics.
- Enum label-introspection semantics requiring full metadata pipeline
  (`type(enum_operand_reference)::LABEL`).
- Raw-event `triggered()` semantics.

Boundary invariants (must still fail deterministically):

- Any rich-type feature deferred to C4.
- `triggered()` semantics.

Exit criteria:

- Integral operator-matrix conformance tests pass.
- Integral cast/conversion conformance tests pass.
- Unknown-flow (`x`/`z`) regression suite passes.
- Evaluator throughput benchmarks pass default regression gate.

Traceability:

- Design refs: expression language references in `docs/DESIGN.md`.

### Phase 4 (C4): Rich Types + Full Surface Closure

Contract scope at phase exit:

- Remaining Section `2` semantics are implemented:
  - advanced enum semantics (labels, label-preservation behavior, and
    enum-specific cast/introspection rules),
  - `type(operand_reference)'(expr)`,
  - `type(enum_operand_reference)::LABEL`,
  - `real` semantics and mixed numeric pathways,
  - `string` semantics and restrictions,
  - `.triggered()` raw-event method-like semantics.
- Full parser grammar surface in Section `2.7` is implemented.
- Full `iff logical_expr` surface is available because full Section `2` parsing
  and semantics are now implemented.
- Metadata plumbing and deterministic fallback behavior are defined for missing
  or unsupported dump metadata (deterministic `error: expr:` paths).

Out of scope:

- Production command integration.
- Any behavior change in default command output/exit codes.

Boundary invariants (must still hold):

- If optional shadow wiring is used for validation, it is test-only and does
  not change default CLI output, exit code, or schema artifacts.

Exit criteria:

- Full Section `2` conformance suite passes.
- Full `iff` surface tests pass (no C2 subset restrictions remain).
- Metadata availability/fallback tests pass for VCD/FST fixtures.
- Rich-type benchmark suites pass default regression gate.

Traceability:

- Design refs: expression contract references in `docs/DESIGN.md`.

### Phase 5 (C5): Command Integration + Hardening

Contract scope at phase exit:

- Shared expression engine is the only runtime path for command consumers:
  - `change --on` uses the unified expression runtime,
  - `property --eval` runtime is implemented end-to-end with capture modes.
- Command behavior remains deterministic in human/json modes.
- JSON/schema artifacts are updated to reflect implemented property behavior.
- Any remaining contract mismatch between `docs/expression_lang.md` and command
  surfaces is resolved in the same phase (by implementation or by explicit
  contract/doc alignment).

Out of scope:

- New language features beyond `docs/expression_lang.md`.

Exit criteria:

- Integration tests pass for expression-driven `change` and `property` flows.
- VCD/FST parity tests pass for expression runtime behavior.
- Command-level performance suites (`bench/e2e`) pass default regression gate.
- `make ci` passes with expression suites enabled.

Traceability:

- Design refs: `docs/DESIGN.md` sections for `change` and `property`.
- Backlog refs: command integration and `property` runtime debt in `docs/BACKLOG.md`.

## Coverage Map (Spec -> Phase)

| Contract Area (`docs/expression_lang.md`) | Phase Ownership |
|---|---|
| Section 1.1-1.3 event forms and runtime semantics | 2 |
| Section 1.4 `iff` binding | 2 |
| Section 1.4 full `iff logical_expr` surface | 4 |
| Section 1.5 event-level precedence/grouping | 2 |
| Section 1.6 event grammar sketch | 2 |
| Section 2.1 operand types (`bit-vector`, `integer-like`, `enum` core value semantics) | 3 |
| Section 2.1 operand types (`real`, `string`, `event`) | 4 |
| Section 2.2 operand forms (integral forms) | 3 |
| Section 2.2 enum-label, rich-type, `.triggered()` forms | 4 |
| Section 2.3 casts (integral core) | 3 |
| Section 2.3 rich casts/introspection (`type(...)`, enum labels, real/string/event details) | 4 |
| Section 2.4 implicit conversions (integral/common-type core) | 3 |
| Section 2.4 rich-type conversion rules (`enum` label-preservation nuances, `real`, `string`) | 4 |
| Section 2.5 operator matrix (integral/core subsets) | 3 |
| Section 2.5 remaining rich-type/operator-specific semantics | 4 |
| Section 2.6 full precedence/associativity across full surface | 4 |
| Section 2.7 full parser-level grammar | 4 |
| Command runtime integration (`change`, `property`, expression-surface reconciliation) | 5 |

## Planning Rules For Follow-up Work

Every smaller execution plan must:

- target exactly one phase boundary (`C1`..`C5`),
- enumerate explicit in-scope spec clauses from `docs/expression_lang.md`,
- enumerate explicit out-of-scope clauses and required deterministic failures,
- define test manifests + benchmark profile used for phase-gate evaluation,
- live in `docs/exec-plans/active/` and follow the exec-plan template
  (`.opencode/skills/exec-plan/references/plan-template.md`).
