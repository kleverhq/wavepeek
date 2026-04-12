# Decompose design docs under `docs/design/`

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

Note that this document must be maintained in accordance with `exec-plan` skill.

## Purpose / Big Picture

After this change, readers will start at `docs/design/index.md` instead of opening one oversized `docs/DESIGN.md`. The design material will be split into three clearly named layers: rationale and navigation in `docs/design/index.md` plus nearby design notes, normative contracts in `docs/design/contracts/`, and a thin code-first CLI reference in `docs/design/reference/`. Someone new to the repository will be able to answer three practical questions quickly: what wavepeek is for, which documents are normative, and where the command-line surface is actually defined.

The visible proof is straightforward. Opening `docs/design/index.md` must give a compact overview, scope, design principles, and links to architecture, contracts, and reference material. The expression-language contract must live under `docs/design/contracts/expression_lang.md`. The repository breadcrumbs in `AGENTS.md` files must point to the new canonical locations. The old paths `docs/DESIGN.md` and `docs/expression_lang.md` must remain as thin compatibility stubs with prominent working links to the new files so historical plans and old links do not break.

This plan also resolves the source-of-truth question for CLI documentation. The full command-line surface, including flags, defaults, requiredness, and examples, will be treated as code-first through `src/cli/`, `wavepeek --help`, and `wavepeek <command> --help`, while the machine-readable output contract will remain machine-checked through `schema/wavepeek.json` and `wavepeek schema`. The new docs should preserve only the semantics that code or schema alone do not explain well, such as time normalization rules, naming and resolution rules, output contracts, and the meaning of the command families.

## Non-Goals

This plan does not change any Rust behavior, JSON schema behavior, CLI flags, command help strings, tests, or benchmarks. It does not introduce automatic documentation generation, prompt-driven documentation generation, or a new documentation build pipeline. It does not rewrite the historical files under `docs/exec-plans/completed/`; those records may continue to mention the old paths and will rely on compatibility stubs. It does not attempt a prose rewrite of every existing design paragraph; the goal is decomposition, clarification of ownership, and removal of duplicated CLI surface detail, not a wholesale editorial refresh.

## Progress

- [x] (2026-04-12 19:57Z) Reviewed `docs/DESIGN.md`, `docs/expression_lang.md`, the exec-plan template, `docs/exec-plans/AGENTS.md`, and representative completed plans to capture the current document shape and the required exec-plan structure.
- [x] (2026-04-12 19:57Z) Reviewed `src/cli/mod.rs`, `src/cli/change.rs`, and `src/cli/property.rs` to confirm that the CLI help surface is already rich enough to serve as the source of truth for flag-level command reference.
- [x] (2026-04-12 19:57Z) Mapped live references to `docs/DESIGN.md` and `docs/expression_lang.md` across breadcrumbs, docs, and support files to define the migration and compatibility-stub strategy.
- [x] (2026-04-12 19:57Z) Drafted this active ExecPlan with the target `docs/design/` tree, the code-first CLI documentation model, the normative contract split, and the compatibility strategy for old paths.
- [x] (2026-04-12 20:03Z) Ran focused docs and architecture review lanes on the plan, then revised it to add explicit remap targets, correct the compatibility-stub link semantics, exclude exec-plan files from the validation sweep, and keep the suggested commit split internally complete.
- [x] (2026-04-12 20:08Z) Ran a fresh control review pass on the consolidated diff; no substantive issues remained, so the plan is now ready for execution.
- [x] (2026-04-12 20:38Z) Created `docs/design/`, `docs/design/contracts/`, and `docs/design/reference/` with local `AGENTS.md` breadcrumbs that follow the repository breadcrumb policy and point to the new ownership split.
- [x] (2026-04-12 20:38Z) Split the monolith into `docs/design/index.md`, `docs/design/architecture.md`, `docs/design/open_questions.md`, `docs/design/contracts/command_model.md`, `docs/design/contracts/machine_output.md`, `docs/design/contracts/expression_lang.md`, and `docs/design/reference/cli.md`, keeping the CLI surface code-first and the semantics docs prose-first.
- [x] (2026-04-12 20:49Z) Replaced `docs/DESIGN.md` with a thin compatibility stub after the new canonical design corpus was populated.
- [x] (2026-04-12 20:38Z) Replaced `docs/expression_lang.md` with a thin compatibility stub after moving the full canonical contract to `docs/design/contracts/expression_lang.md`.
- [x] (2026-04-12 20:49Z) Updated live breadcrumbs and live docs so they point directly at `docs/design/index.md`, `docs/design/contracts/*`, or `docs/design/reference/cli.md` instead of the retired canonical paths.
- [x] (2026-04-12 21:02Z) Ran focused docs and architecture review lanes on the implementation diff; both lanes were mostly clean, but they found two follow-up issues: top-level breadcrumb entrypoints were still non-clickable path text, and several docs overstated `wavepeek schema` as authority for the general CLI surface instead of the machine-output contract only.
- [x] (2026-04-12 21:07Z) Fixed the review findings by converting the root and docs breadcrumb entrypoints to real markdown links, adding the missing parent breadcrumb in `docs/AGENTS.md`, and narrowing the `wavepeek schema` authority statements to the machine-output contract.
- [x] (2026-04-12 21:18Z) Ran a fresh control review pass on the consolidated diff; it found one remaining migration issue: several touched `AGENTS.md` files under `src/`, `tests/`, `schema/`, and `bench/` were still using non-clickable path text instead of real markdown links.
- [x] (2026-04-12 21:22Z) Fixed the control-pass finding by converting the touched `AGENTS.md` files in `src/`, `tests/`, `schema/`, `bench/`, `bench/e2e/`, and `bench/expr/` to real markdown links and by narrowing `bench/e2e/AGENTS.md` back to normative contract links only.
- [x] (2026-04-12 21:31Z) Reran validation after the last review-fix commits (`make check` plus the retired-path `rg` sweep) and got a clean final control review pass on the consolidated implementation diff.

## Surprises & Discoveries

- Observation: the CLI already carries a strong human-readable contract in code, so the design docs no longer need to repeat flag tables and per-command parameter matrices to stay trustworthy.
  Evidence: `src/cli/mod.rs` defines detailed top-level and per-command `about` and `long_about` text, and the command argument structs in `src/cli/*.rs` already encode requiredness, defaults, and value shapes.

- Observation: the migration risk is dominated by path churn, not by content extraction.
  Evidence: live references to `docs/DESIGN.md` exist in the repository root breadcrumb, multiple `AGENTS.md` files under `bench/`, `schema/`, `src/`, and `tests/`, plus `docs/BACKLOG.md` and `CHANGELOG.md`, while `docs/expression_lang.md` is referenced from `docs/AGENTS.md`, `bench/expr/AGENTS.md`, and `docs/DESIGN.md`.

- Observation: the repository breadcrumb policy applies to the new documentation subdirectories.
  Evidence: the root `AGENTS.md` requires a local `AGENTS.md` when creating a new durable directory with tracked files, which means `docs/design/`, `docs/design/contracts/`, and `docs/design/reference/` must each gain a small navigation file in the same change.

- Observation: compatibility stubs are the lowest-risk way to preserve archived-plan usability without rewriting historical records.
  Evidence: many completed plans under `docs/exec-plans/completed/` mention `docs/DESIGN.md` and `docs/expression_lang.md`, and those files are explicitly treated as historical records in `docs/exec-plans/AGENTS.md`.

- Observation: the expression-language move was safest as a literal path move plus stub, not a rewrite.
  Evidence: `docs/expression_lang.md` was already a standalone contract with no decomposition needed, so moving it intact to `docs/design/contracts/expression_lang.md` preserved the reviewed text while keeping the compatibility pointer tiny.

- Observation: the new design breadcrumbs needed real markdown links, not just code-styled path text.
  Evidence: the acceptance criteria require readers to follow prominent links from `docs/design/index.md` and the new `AGENTS.md` files in the repository viewer, so bare backticked paths would not have been sufficient.

- Observation: it is easy to over-ascribe `wavepeek schema` when describing the code-first documentation model.
  Evidence: the first architecture review pass flagged that `wavepeek schema` is authoritative for the machine-readable output contract, but not for general command names, flags, defaults, or help examples, which remain owned by `src/cli/` and `wavepeek --help`.

- Observation: breadcrumb migrations are only complete once every touched `AGENTS.md` uses actual markdown links, not just the newly created maps.
  Evidence: the first control pass stayed red because the canonical-path updates in `src/AGENTS.md`, `tests/AGENTS.md`, `schema/AGENTS.md`, `bench/AGENTS.md`, `bench/e2e/AGENTS.md`, and `bench/expr/AGENTS.md` still rendered as inert code text in repository viewers.

## Decision Log

- Decision: `docs/design/index.md` will become the canonical design entrypoint.
  Rationale: the user explicitly requested an `index.md` entrypoint under `docs/design/`, and a dedicated index cleanly separates navigation and overview material from contracts and internal architecture.
  Date/Author: 2026-04-12 / OpenCode

- Decision: the new documentation tree will distinguish rationale, normative contracts, and derived reference material.
  Rationale: this split matches how the repository already treats `docs/expression_lang.md` as a normative contract while making it clear that command help and schema are stronger authorities for flag-level CLI detail than a large prose document.
  Date/Author: 2026-04-12 / OpenCode

- Decision: CLI surface documentation will be code-first, not prompt-first.
  Rationale: the exact command names, flags, defaults, requiredness, and examples are already encoded in `src/cli/` and exposed through `wavepeek --help`, which makes them reviewable, deterministic, and harder to let drift than a prompt-generated document. Prompts may help author docs later, but they are not an acceptable canonical contract.
  Date/Author: 2026-04-12 / OpenCode

- Decision: the new normative contract set will keep only the cross-cutting semantics that code and schema do not express clearly enough on their own.
  Rationale: retaining the semantic rules for time handling, name resolution, output contracts, warnings, and expression language avoids losing important guarantees while eliminating duplicated clap-level parameter prose.
  Date/Author: 2026-04-12 / OpenCode

- Decision: keep `docs/DESIGN.md` and `docs/expression_lang.md` as thin compatibility stubs after moving the canonical content.
  Rationale: this allows live breadcrumbs to move to the new paths without forcing a noisy rewrite of archived exec plans and any external references that still use the old paths.
  Date/Author: 2026-04-12 / OpenCode

- Decision: keep `docs/design/open_questions.md` as a separate file in the first pass.
  Rationale: an explicit file makes the decomposition easier to scan and keeps unresolved design issues out of the stable architecture and contract documents. If it later proves too small to justify its own file, it can be folded back after the decomposition settles.
  Date/Author: 2026-04-12 / OpenCode

- Decision: describe `wavepeek schema` as the machine-output authority, not as the general CLI-surface authority.
  Rationale: the schema command is authoritative for JSON payload shape and schema linkage, but exact command names, flags, defaults, and help examples remain owned by `src/cli/` plus the generated help text. Keeping those buckets separate prevents future docs from blurring command-surface ownership with machine-output ownership.
  Date/Author: 2026-04-12 / OpenCode

## Outcomes & Retrospective

Current status: complete. The new canonical `docs/design/` tree is in place, both legacy top-level docs are compatibility stubs only, live breadcrumbs point to the new canonical files, validation is clean, and the focused review lanes plus final control pass are clean.

The implementation completed the intended source-of-truth split. The repository now has a dedicated design entrypoint, a separate architecture note, explicit open questions, normative contracts under `docs/design/contracts/`, and a thin CLI-family guide under `docs/design/reference/`. The old `docs/DESIGN.md` and `docs/expression_lang.md` paths are compatibility pointers only, while the live breadcrumb network points at the new canonical locations.

The main lesson from the implementation is that the decomposition itself is straightforward once the ownership boundaries are explicit, but both wording and link mechanics matter. The review cycle exposed three subtle failure modes: navigation regresses if breadcrumbs are not real markdown links, authority boundaries blur if `wavepeek schema` is described as a catch-all CLI authority instead of the more precise machine-output authority, and touched legacy breadcrumb files need the same link-quality treatment as the new maps. Capturing those points in the final docs and in this completed plan should make future documentation refactors lower risk.

## Context and Orientation

`wavepeek` currently keeps most product design, CLI behavior, architecture notes, and testing strategy inside one large file at `docs/DESIGN.md`. That file mixes several kinds of information. Its early sections describe product intent, design principles, and scope. Its middle section contains a detailed command-by-command CLI specification with parameter tables and examples. Its later sections document non-functional requirements, module structure, dependencies, change-command execution internals, error handling, and testing strategy. The result is useful but hard to maintain because the file bundles together information with different ownership and different rates of change.

The repository already contains one important counterexample: `docs/expression_lang.md`. That file is a specialized contract for the expression language used by `change --on` and `property --eval`. It is intentionally more normative than the rest of `docs/DESIGN.md`, because the exact syntax and semantics matter independently of implementation details. That split is the model for the new `docs/design/contracts/` directory.

Several files already act as stronger authorities than prose for the CLI surface. `src/cli/mod.rs` defines the top-level command descriptions and dispatch. The files under `src/cli/` such as `src/cli/change.rs`, `src/cli/property.rs`, `src/cli/info.rs`, `src/cli/scope.rs`, `src/cli/signal.rs`, and `src/cli/value.rs` define the flags, value kinds, defaults, and requiredness for each command. `schema/wavepeek.json` and the `wavepeek schema` command define the machine-readable JSON contract. In this plan, “code-first CLI reference” means that code and generated help own the exact command surface, schema owns the exact JSON payload contract, and docs explain the semantics and where to look.

This repository also uses breadcrumb files named `AGENTS.md` as local navigation maps. The root `AGENTS.md` says that when a new durable directory with tracked files is introduced, that directory should gain its own `AGENTS.md` with concise links back to the parent map, sideways to canonical documents, and forward to any child maps. Because this plan creates `docs/design/`, `docs/design/contracts/`, and `docs/design/reference/`, each of those directories must gain an `AGENTS.md` in the same implementation.

The migration must account for existing references. Live repository files currently point to `docs/DESIGN.md` from `AGENTS.md`, `docs/AGENTS.md`, `src/AGENTS.md`, `tests/AGENTS.md`, `schema/AGENTS.md`, `bench/AGENTS.md`, `bench/e2e/AGENTS.md`, `docs/BACKLOG.md`, and `CHANGELOG.md`. Live files currently point to `docs/expression_lang.md` from `docs/AGENTS.md` and `bench/expr/AGENTS.md`, and `docs/DESIGN.md` itself links there as part of the current design contract. Numerous historical plans under `docs/exec-plans/completed/` also mention the old paths. A thin compatibility stub means a very small markdown file at the old path whose only job is to tell readers that the canonical content moved and to link to the new location.

The target document tree for this plan is fixed up front. `docs/design/index.md` will hold Overview, Scope, design principles, and a map of the design docs. `docs/design/architecture.md` will hold internal engineering material such as non-functional requirements, architecture, module structure, dependencies, error handling, execution strategy, and testing strategy. `docs/design/open_questions.md` will carry unresolved design questions. `docs/design/contracts/command_model.md` will hold cross-cutting command semantics such as time handling, naming and scope resolution, bounded output, and ordering. `docs/design/contracts/machine_output.md` will hold stdout and stderr rules, JSON envelope behavior, schema linkage, warnings, and exit codes. `docs/design/contracts/expression_lang.md` will become the canonical home of the expression-language contract. `docs/design/reference/cli.md` will be a thin operator’s guide to command families and will point readers to `wavepeek --help` rather than duplicating clap-level detail.

One subtle term in this plan is “normative.” A normative document is one whose statements must be treated as binding behavior. Another subtle term is “derived reference.” A derived reference is documentation that summarizes a surface but intentionally defers exact truth to code or schema. In this repository, `reference/cli.md` is derived, while `contracts/command_model.md`, `contracts/machine_output.md`, and `contracts/expression_lang.md` are normative. This distinction is the key to preventing the new documentation tree from regrowing into another monolith.

## Open Questions

There are no blocking product questions left before implementation starts, but two implementation-time judgment calls should stay explicit.

The first is how much detail `docs/design/reference/cli.md` should contain. The default answer in this plan is “as little as possible while still being a useful map”: one short section per command family, a short note on when to use each command, and direct pointers to `wavepeek --help` and the relevant contracts. If the draft starts repeating per-flag tables or default values from `src/cli/`, trim it back.

The second is how minimal the compatibility stubs should be. The default answer is “one short paragraph plus a direct link to the new canonical file.” If a live in-repo link depends on an old anchor that cannot be updated immediately, fix that live link rather than reproducing legacy anchor structure inside the stub.

## Plan of Work

Milestone 1 creates the new documentation skeleton and the navigation contract before any large content move happens. Start by creating the directories `docs/design/`, `docs/design/contracts/`, and `docs/design/reference/`, each with a local `AGENTS.md` that follows the repository breadcrumb policy. Then create `docs/design/index.md`, `docs/design/architecture.md`, and `docs/design/open_questions.md` as empty-but-structured destinations so subsequent content moves have stable landing points. This milestone is complete when a new reader can open `docs/design/index.md`, see the target doc map, and follow links to every planned design subdocument even if some sections still carry placeholder prose during the migration.

Milestone 2 performs the actual decomposition of `docs/DESIGN.md` into the new tree. Move Overview, Scope, and the design principles into `docs/design/index.md`. Move the architecture, dependency, error-handling, change-execution, and testing sections into `docs/design/architecture.md`. Move the current open questions into `docs/design/open_questions.md`. Rebuild the functional-requirements section rather than copying it verbatim: extract only the cross-cutting semantics into `docs/design/contracts/command_model.md` and `docs/design/contracts/machine_output.md`, and write `docs/design/reference/cli.md` as a thin guide that names the command families and points to `wavepeek --help` for exact command surface details and `wavepeek schema` for the machine-readable output contract.

Milestone 3 moves the expression-language contract and stabilizes compatibility. Copy the full current content of `docs/expression_lang.md` into `docs/design/contracts/expression_lang.md`, then update all live documentation and breadcrumb links to point at the new canonical location. Only after the new files are complete should `docs/DESIGN.md` and `docs/expression_lang.md` be rewritten into thin compatibility stubs. This copy-then-trim order matters because it prevents temporary information loss and gives reviewers a clean diff showing the new canonical files before the old files collapse to compatibility pointers.

Milestone 4 finishes the migration and hardens the new source-of-truth story. Update live breadcrumbs and support docs that still mention the old canonical paths, including `AGENTS.md`, `docs/AGENTS.md`, `src/AGENTS.md`, `tests/AGENTS.md`, `schema/AGENTS.md`, `bench/AGENTS.md`, `bench/e2e/AGENTS.md`, `bench/expr/AGENTS.md`, `docs/BACKLOG.md`, and any current `CHANGELOG.md` references. Make sure `docs/design/index.md` explicitly states that exact command-surface truth lives in `src/cli/`, `wavepeek --help`, and `wavepeek <command> --help`, while `schema/wavepeek.json` and `wavepeek schema` define the machine-readable output contract and the documents under `docs/design/contracts/` define normative semantics that code or schema do not express clearly enough. Do not repoint everything mechanically to `docs/design/index.md`. High-level product and navigation references should point to `docs/design/index.md`; architecture and testing references should point to `docs/design/architecture.md`; expression-language references should point to `docs/design/contracts/expression_lang.md`; output-contract references should point to `docs/design/contracts/machine_output.md`; cross-cutting command semantics such as time and resolution should point to `docs/design/contracts/command_model.md`; and operator-facing command inventory references should point to `docs/design/reference/cli.md`. Leave historical completed plans untouched unless a tiny wording fix is truly required, because the compatibility stubs already preserve usability.

Milestone 5 is validation and review. Because this is a docs-only change, test-driven development is not the right tool here; instead, the implementation should use documentation-first migration with command-level verification and repository quality checks. Validate that the new docs point to real command surfaces by running representative help and schema commands. Validate that live breadcrumbs now point at the new canonical files and that only the intended compatibility stubs remain at the old paths. Then run focused review lanes in parallel: one docs lane for wording, navigation, and contract clarity, and one architecture lane for ownership boundaries, source-of-truth consistency, and breadcrumb completeness. After fixing any findings in follow-up commits, run one fresh control pass on the consolidated diff and close the plan only when the review comes back clean.

### Concrete Steps

Run all commands from `/workspaces/wavepeek`.

1. Reconfirm the current split between code-first CLI surface and normative semantics before editing docs.

   Read these files in full before drafting the new docs:

   - `docs/DESIGN.md`
   - `docs/expression_lang.md`
   - `docs/AGENTS.md`
   - `AGENTS.md`
   - `src/cli/mod.rs`
   - representative command arg files under `src/cli/`
   - `schema/wavepeek.json`

   The goal of this read is to keep only the semantics that need prose. If an exact flag default or parameter list already lives in clap or schema, do not preserve it as normative prose unless the semantics would become ambiguous without it.

2. Create the new documentation directories and breadcrumb files first.

   Commands:

       mkdir -p docs/design/contracts docs/design/reference

   Then create and populate these files:

   - `docs/design/AGENTS.md`
   - `docs/design/index.md`
   - `docs/design/architecture.md`
   - `docs/design/open_questions.md`
   - `docs/design/contracts/AGENTS.md`
   - `docs/design/contracts/command_model.md`
   - `docs/design/contracts/machine_output.md`
   - `docs/design/contracts/expression_lang.md`
   - `docs/design/reference/AGENTS.md`
   - `docs/design/reference/cli.md`

   Expected initial tree:

       docs/design/
       ├── AGENTS.md
       ├── index.md
       ├── architecture.md
       ├── open_questions.md
       ├── contracts/
       │   ├── AGENTS.md
       │   ├── command_model.md
       │   ├── machine_output.md
       │   └── expression_lang.md
       └── reference/
           ├── AGENTS.md
           └── cli.md

3. Populate `docs/design/index.md` with the entrypoint content and doc map.

   `docs/design/index.md` must contain, in this order, the Overview material, the project Scope material, the design principles, and a short “document map” section that explains which files are normative and which files are derived reference. It must explicitly say that `src/cli/`, `wavepeek --help`, and `wavepeek <command> --help` are authoritative for exact command-surface shape, while `schema/wavepeek.json` and `wavepeek schema` are authoritative for the machine-readable output contract.

4. Split the old monolith into architecture, contracts, and reference.

   Move the internal engineering sections from `docs/DESIGN.md` into `docs/design/architecture.md`. Move the current open questions into `docs/design/open_questions.md`. Write `docs/design/contracts/command_model.md` from the cross-cutting command rules only: waveform-file requirements, time token and range semantics, naming and scope resolution, bounded output, deterministic ordering, and the distinction between human-readable and machine-readable modes. Write `docs/design/contracts/machine_output.md` from the output contract only: stdout versus stderr, JSON envelope behavior, `schema` output behavior, warning handling, error format, and exit codes. Write `docs/design/reference/cli.md` as a thin guide with short sections for `schema`, `info`, `scope`, `signal`, `value`, `change`, and `property`, but do not recreate the old parameter tables.

5. Move the expression-language contract and then collapse the old files to compatibility stubs.

   Copy the full content of `docs/expression_lang.md` into `docs/design/contracts/expression_lang.md`, adjust local links if needed, and only then replace `docs/expression_lang.md` with a short compatibility notice whose markdown link target is `[design/contracts/expression_lang.md](design/contracts/expression_lang.md)`. Apply the same pattern to `docs/DESIGN.md`, replacing the large monolith with a short compatibility notice whose markdown link target is `[design/index.md](design/index.md)`. The stubs should be intentionally small enough that future readers cannot mistake them for the canonical content.

6. Update all live breadcrumbs and live docs that reference the old canonical paths.

   Search commands:

       rg -n "docs/DESIGN\.md|docs/expression_lang\.md" AGENTS.md docs/AGENTS.md docs/BACKLOG.md docs/design bench src tests schema CHANGELOG.md

   Update every live hit so it points at the new canonical path unless the hit is inside `docs/DESIGN.md` or `docs/expression_lang.md` themselves as part of the compatibility stub text. Do not edit this active ExecPlan or any file under `docs/exec-plans/completed/` just to satisfy this search; those files are intentionally outside the validation sweep.

7. Validate the new navigation and the claimed CLI authorities.

   Commands:

       cargo run -- --help
       cargo run -- info --help
       cargo run -- schema > /tmp/wavepeek-schema.json
       python3 -m json.tool /tmp/wavepeek-schema.json > /dev/null
       rg -n "docs/DESIGN\.md|docs/expression_lang\.md" AGENTS.md docs/AGENTS.md docs/BACKLOG.md docs/design bench src tests schema CHANGELOG.md
       make check

   Then manually open these files in the repository viewer or your editor and follow each prominent markdown link once:

   - `docs/design/index.md`
   - `docs/DESIGN.md`
   - `docs/expression_lang.md`
   - `docs/design/AGENTS.md`
   - `docs/design/contracts/AGENTS.md`
   - `docs/design/reference/AGENTS.md`

   Expected signatures:

       wavepeek is a command-line tool for RTL waveform inspection.
       General conventions:

   and:

       {
         "$schema": "https://json-schema.org/draft/2020-12/schema",

   The final `rg` output should be empty. If it is not empty, either a live breadcrumb still points at an old canonical path or one of the new canonical docs still mentions the retired path names.

8. Commit the work in atomic units without rewriting history.

   Suggested split:

       git add docs/design/AGENTS.md docs/design/index.md docs/design/architecture.md docs/design/open_questions.md docs/design/contracts/AGENTS.md docs/design/contracts/command_model.md docs/design/contracts/machine_output.md docs/design/contracts/expression_lang.md docs/design/reference/AGENTS.md docs/design/reference/cli.md
       git commit -m "docs(design): create decomposed design doc tree"

       git add docs/DESIGN.md docs/expression_lang.md AGENTS.md docs/AGENTS.md src/AGENTS.md tests/AGENTS.md schema/AGENTS.md bench/AGENTS.md bench/e2e/AGENTS.md bench/expr/AGENTS.md docs/BACKLOG.md CHANGELOG.md
       git commit -m "docs(design): repoint breadcrumbs to canonical paths"

   If review finds issues, fix them in one or more follow-up commits. Do not amend or squash.

9. Run the mandatory review workflow.

   Load `ask-review` skill and prepare a concise context packet with the scope summary, the commit range, the validation commands already run, and the remaining risks. Run two focused lanes in parallel:

   - Docs lane: wording clarity, doc-map quality, normative-versus-derived labeling, stub clarity, and breadcrumb readability.
   - Architecture lane: source-of-truth boundaries, completeness of file moves, breadcrumb policy compliance, and migration risk.

   After fixing findings, rerun `make check` if any review fix touched tracked docs, then run one fresh control pass on the consolidated diff. The plan is complete only when both focused lanes and the control pass are clean, or every finding has been fixed, committed, and rechecked.

### Validation and Acceptance

The implementation is successful when a new reader can start at `docs/design/index.md` and navigate the entire design corpus without needing `docs/DESIGN.md` as a monolith. `docs/design/index.md` must clearly identify `docs/design/contracts/` as the normative semantics layer and `docs/design/reference/cli.md` as a thin guide, not the authoritative flag-by-flag CLI contract.

The change must preserve two compatibility behaviors. Opening `docs/DESIGN.md` must immediately show a prominent compatibility pointer with a working relative markdown link to `design/index.md`, and opening `docs/expression_lang.md` must immediately show a prominent compatibility pointer with a working relative markdown link to `design/contracts/expression_lang.md`. Live breadcrumbs must point directly at the new canonical files rather than relying on the stubs.

The change must also make the new source-of-truth story observable. Running `cargo run -- --help` and `cargo run -- info --help` must still show the authoritative CLI help surface, and `cargo run -- schema` must still emit valid JSON. The prominent links from `docs/design/index.md`, the two compatibility stubs, and the three new `AGENTS.md` files must each resolve to the intended target when opened in the normal repository viewer. `make check` must pass, proving the repository remains healthy after the docs-only reorganization.

### Idempotence and Recovery

This work is safe to stage incrementally because it is documentation-only. Directory creation with `mkdir -p` is idempotent. The safest editing order is copy-then-trim: create the new canonical files first, move content into them, update links, and only then replace the old files with compatibility stubs. That order means a partial interruption still leaves the original content in place.

If a step fails midway, use `git status` to see which files are partially edited. If the new files exist but the stubs have not yet been written, continue forward rather than deleting work. If the stubs have already been written but review reveals missing content in the new files, restore the lost section from `git diff` or `git checkout -- docs/DESIGN.md docs/expression_lang.md` and retry the copy-then-trim sequence. Because no generated artifacts need to be committed, cleanup is limited to removing `/tmp/wavepeek-schema.json` if desired.

### Artifacts and Notes

Use small, obvious compatibility stubs. A good stub for `docs/DESIGN.md` looks like this:

    # Design Documentation Moved

    The canonical design entrypoint is now [`design/index.md`](design/index.md).
    This file remains only as a compatibility pointer for older links.

The new index should include a short map that makes the ownership split impossible to miss. A concise example is:

    - `docs/design/index.md` — overview, scope, principles, and navigation
    - `docs/design/architecture.md` — internal architecture and testing strategy
    - `docs/design/contracts/` — normative semantics contracts
    - `docs/design/reference/cli.md` — thin CLI guide; exact surface lives in `wavepeek --help`

The new CLI reference should stay intentionally short. A good section shape is one short paragraph per command family that says what the command is for and where to find exact help text. If a draft starts reproducing `--max`, `--json`, or per-command parameter tables from `src/cli/`, cut that text back and move the semantic part into the contracts instead.

### Interfaces and Dependencies

This plan changes documentation interfaces, not Rust interfaces. No public Rust type, trait, function, or schema file should change as part of the decomposition. In particular, `src/cli/mod.rs`, the other files under `src/cli/`, and `schema/wavepeek.json` are dependencies of this documentation change and should be treated as the factual authorities the docs point at, not as files to edit for the sake of the reorganization.

At the end of the implementation, these repository paths must exist and have stable roles:

- `docs/design/index.md` as the canonical entrypoint for design documentation.
- `docs/design/architecture.md` as the canonical internal architecture note.
- `docs/design/open_questions.md` as the canonical home for unresolved design questions.
- `docs/design/contracts/command_model.md` as the canonical command-semantics contract.
- `docs/design/contracts/machine_output.md` as the canonical output-contract document.
- `docs/design/contracts/expression_lang.md` as the canonical expression-language contract.
- `docs/design/reference/cli.md` as the thin derived CLI guide.
- `docs/design/AGENTS.md`, `docs/design/contracts/AGENTS.md`, and `docs/design/reference/AGENTS.md` as the required breadcrumb maps for the new directories.
- `docs/DESIGN.md` and `docs/expression_lang.md` as compatibility stubs only.

The live breadcrumb files that currently reference the old paths are also part of the required surface and must be updated in the same change: `AGENTS.md`, `docs/AGENTS.md`, `src/AGENTS.md`, `tests/AGENTS.md`, `schema/AGENTS.md`, `bench/AGENTS.md`, `bench/e2e/AGENTS.md`, `bench/expr/AGENTS.md`, `docs/BACKLOG.md`, and any live `CHANGELOG.md` path references that still point to the old canonical locations.

Revision note: finalized on 2026-04-12 after rerunning validation, recording the clean final control pass, and moving the completed plan to `docs/exec-plans/completed/`.
