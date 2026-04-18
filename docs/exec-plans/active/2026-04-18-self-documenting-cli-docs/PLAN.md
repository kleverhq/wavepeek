# Implement layered help and embedded CLI docs

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

Note that this document must be maintained in accordance with the `exec-plan` skill.

## Purpose / Big Picture

Today the installed `wavepeek` binary can explain command syntax, but it cannot yet serve as the complete, version-matched entry point for quick syntax lookup, deeper command reference, narrative workflow guidance, and troubleshooting. The current CLI also forces one help style everywhere because `src/cli/mod.rs` rewires both `-h` and `--help` to the same long output, while the proposal in `docs/cmd_docs_proposal.md` explicitly asks for progressive disclosure instead.

After this plan is implemented, users and agents will be able to stay inside the installed binary for normal documentation needs. `wavepeek -h` will give compact lookup help, `wavepeek --help` and `wavepeek help <command-path...>` will give detailed command reference, and `wavepeek docs ...` will expose embedded Markdown topics, search, export, and the packaged agent skill. The visible proof is direct: `wavepeek -h`, `wavepeek help change`, `wavepeek docs show commands/change`, `wavepeek docs search transitions`, and `wavepeek docs export /tmp/wavepeek-docs` will all work offline and will describe the same installed version.

This plan also migrates the accepted product semantics out of the temporary proposal file and into the canonical design corpus. The exact CLI surface will remain code-first in `src/cli/`, `wavepeek --help`, and `wavepeek <command> --help`; the durable semantics for layered help and embedded docs will move into `docs/design/contracts/`; and `docs/design/reference/cli.md` will stay a thin map rather than regrowing into a second command reference.

## Non-Goals

This plan does not add pager integration, colored or rich terminal rendering, localization, online documentation hosting, or fuzzy search beyond deterministic case-insensitive token and substring matching. It does not change waveform-query semantics for `info`, `scope`, `signal`, `value`, `change`, `property`, or `schema` beyond help text, cross-links, and the new `docs topics --json` / `docs search --json` machine outputs.

This plan does not resurrect retired command names from older documents. The installed command surface today is `info`, `scope`, `signal`, `value`, `change`, `property`, and `schema`; the progressive-disclosure and narrative-doc contracts in this plan apply to that shipped surface.

This plan does not treat `docs/cmd_docs_proposal.md` as a new permanent source of truth. That file is a staging proposal only and should become a thin historical pointer once the canonical contracts and implementation land.

## Progress

- [x] (2026-04-18 12:36Z) Reviewed `docs/cmd_docs_proposal.md`, the exec-plan template, `docs/exec-plans/AGENTS.md`, the historical CLI-help and docs-decomposition plans, and the current canonical design docs to capture repo-specific plan requirements.
- [x] (2026-04-18 12:36Z) Mapped the current implementation anchors in `src/cli/mod.rs`, `src/engine/mod.rs`, `src/output.rs`, `src/schema_contract.rs`, `schema/wavepeek.json`, `tests/cli_contract.rs`, and `tests/schema_cli.rs`, plus the current shipped skill at `.opencode/skills/wavepeek/SKILL.md`.
- [x] (2026-04-18 12:36Z) Resolved the main planning ambiguities up front: use the current shipped command surface (`property`, not the retired `when`), store shipped narrative-doc source in a package-included docs tree instead of only under `.opencode/`, and migrate durable semantics into a new canonical design contract file.
- [x] (2026-04-18 12:36Z) Drafted this active ExecPlan with TDD-first milestones, explicit file targets, review lanes, and commit boundaries.
- [x] (2026-04-18 12:44Z) Ran docs and architecture review lanes on the plan; the docs lane was clean and the architecture lane identified three issues to fix before execution: stale machine-output contract migration, an over-broad per-command topic requirement, and a non-atomic suggested commit split.
- [x] (2026-04-18 12:44Z) Revised the plan to migrate `docs/design/contracts/machine_output.md`, constrain `commands/*` topics to narrative-only adjuncts with a smaller required seed corpus, and replace the suggested commits with build-safe atomic slices.
- [x] (2026-04-18 12:52Z) Ran two fresh control reviews on the revised plan; they surfaced remaining gaps in canonical contract migration, export completeness, TDD step ordering, explicit `wavepeek help` coverage, subcommand layered-help regression checks, and living-plan bookkeeping.
- [x] (2026-04-18 12:52Z) Applied the control-pass fixes by extending the canonical-doc migration to `command_model.md`, requiring export to preserve original Markdown bytes and explicitly exclude the skill from `docs export`, moving the remaining contract tests into the TDD step list, and recording the final review round in the living-plan sections.
- [x] (2026-04-18 12:57Z) Ran a fresh final control review on the fully revised plan; it returned clean with no remaining substantive issues.
- [x] (2026-04-18 13:26Z) Reconfirmed the legacy baseline with the existing targeted tests: the top-level and per-command parity tests still pass, `help`/`docs` are absent from the shipped surface, and `wavepeek schema` still exposes only the pre-docs JSON branches.
- [x] (2026-04-18 13:26Z) Completed the TDD contract-capture step by rewriting `tests/cli_contract.rs`, adding `tests/docs_cli.rs`, extending `tests/schema_cli.rs`, and running those suites to confirm the new contract fails on the legacy implementation for the expected reasons: missing `help`/`docs` commands, unchanged short-vs-long help behavior, absent packaged docs assets, and missing schema branches.
- [x] (2026-04-18 13:41Z) Created the packaged docs source tree under `docs/cli/`, added the required breadcrumb `AGENTS.md` files, added the seed Markdown corpus plus canonical packaged skill source, and synchronized `.opencode/skills/wavepeek/SKILL.md` to the packaged skill text.
- [x] (2026-04-18 13:41Z) Implemented the embedded docs runtime in `src/docs/` with YAML front-matter parsing, H1/title validation, deterministic indexing, deterministic search and suggestions, and managed-root export that preserves authored Markdown bytes.
- [x] (2026-04-18 13:41Z) Reworked clap help handling to restore native short-vs-long help, exposed the built-in `help` subcommand, added the `docs` command family, generalized nested parse-error help hints, and wired the docs surface through engine dispatch, output rendering, and the schema contract.
- [x] (2026-04-18 13:41Z) Validated the implementation slice with `cargo test docs:: --lib`, `cargo test --test cli_contract`, `cargo test --test docs_cli`, and `cargo test --test schema_cli`; all targeted suites are now green.
- [x] (2026-04-18 13:50Z) Migrated the accepted semantics into canonical docs by adding `docs/design/contracts/documentation_surface.md`, updating the related design/reference breadcrumbs, refreshing README/development/changelog wording, and shrinking `docs/cmd_docs_proposal.md` into a historical pointer.
- [x] (2026-04-18 13:50Z) Ran full validation after the docs migration: `cargo test -q`, `make check`, and `make ci` all passed, and the manual smoke commands for layered help, docs JSON, docs search, and docs export produced the expected signatures and export manifest.
- [x] (2026-04-18 14:23Z) Ran the mandatory multi-lane review workflow: docs, code, and architecture lanes in parallel, followed by two fresh independent control passes. The first lane round and first control pass found stale packaged guidance plus subtle docs-search contract mismatches; those fixes landed in follow-up commits without rewriting history.
- [x] (2026-04-18 14:23Z) Re-ran the impacted review lanes after the fix commits, then ran a final fresh control pass on the consolidated `HEAD~7..HEAD` diff; the docs lane, code lane, architecture lane, and final control pass all returned clean.
- [ ] Implement the milestones below, keeping this plan updated after every milestone, review pass, and follow-up fix.

## Surprises & Discoveries

- Observation: the proposal examples do not fully match the currently shipped command surface.
  Evidence: `src/cli/mod.rs` and `tests/cli_contract.rs` list `property`, while `docs/cmd_docs_proposal.md` does not name `property` and should not be read as the authoritative command inventory.

- Observation: the current CLI intentionally forces `-h` and `--help` to be identical, which directly conflicts with the proposal's layered-help contract.
  Evidence: `src/cli/mod.rs` installs one global help arg with `clap::ArgAction::HelpLong`, and `tests/cli_contract.rs` contains parity tests such as `top_level_short_and_long_help_are_identical` and `short_and_long_help_are_identical_for_shipped_commands`.

- Observation: the packaged skill cannot safely be embedded from `.opencode/skills/wavepeek/SKILL.md` alone.
  Evidence: `Cargo.toml` excludes `.opencode/*` from the package payload, so a build from the packaged source would not see that file.

- Observation: the current parse-error help hint logic only reconstructs one subcommand token and will under-specify nested docs help hints.
  Evidence: `help_hint_for_rendered_clap_error()` in `src/cli/mod.rs` stops after the first non-option token after `wavepeek`, which is not sufficient for usages such as `wavepeek docs show <TOPIC>`.

- Observation: the current machine-output schema and engine command model have no room for docs-command JSON branches.
  Evidence: `schema/wavepeek.json` enumerates only `info`, `scope`, `signal`, `value`, `change`, and `property`, while `src/engine/mod.rs` defines no docs-related `Command`, `CommandName`, or `CommandData` variants.

- Observation: the TDD-first docs integration tests currently fail not only on missing CLI surface, but also on missing packaged source assets, which usefully proves the test suite will guard the package-safe docs-tree requirement from the start.
  Evidence: the first `cargo test --test docs_cli` run failed both on `unrecognized subcommand 'docs'` and on missing `docs/cli/wavepeek-skill.md`, matching the intended milestone ordering.

- Observation: clap's native short-help usage lines keep `[OPTIONS]` ahead of positional arguments even after layered help is restored, so exact short-help guidance for `docs show <TOPIC>` is most reliable when included explicitly in `after_help` text rather than inferred from the usage line alone.
  Evidence: `cargo run -- docs show -h` prints `Usage: wavepeek docs show [OPTIONS] <TOPIC>`, while the compact next-step guidance added through `after_help` can still expose the simpler `wavepeek docs show <TOPIC>` shape required by the contract tests.

- Observation: the only full-gate validation blocker after implementation was unrelated pre-existing formatting drift in `tests/common/expr_cases.rs` and `tests/common/expr_runtime.rs`, which had to be temporarily stashed so `make check` / `make ci` could evaluate the current feature work without rewriting someone else's unstaged changes.
  Evidence: the first direct `make check` run failed at `cargo fmt -- --check` on those two unstaged files only, while the rerun with those paths temporarily stashed completed successfully through both `make check` and `make ci`.

- Observation: the review cycle surfaced several non-obvious search-contract edges that were easy to miss while implementing the happy path: topic-ID token matching, repeated-token scoring, canonical normalized `data.query`, exact-title `match_kind` precedence, and referential integrity for `see_also` links.
  Evidence: the code lane and successive control passes each found one of those cases in `src/docs/mod.rs`, and each issue became a targeted regression test in `tests/docs_cli.rs` or `src/docs/mod.rs` before the final clean control pass.

## Decision Log

- Decision: implement the proposal against the current shipped top-level command surface and do not reintroduce retired names such as `when`.
  Rationale: `src/cli/` and generated help remain the exact authority for command names, so the plan must extend the real shipped surface instead of historical examples.
  Date/Author: 2026-04-18 / OpenCode

- Decision: add a new canonical design contract file at `docs/design/contracts/documentation_surface.md` for layered help, embedded-doc topic rules, search ranking, export ownership, and the docs-command JSON support matrix.
  Rationale: these semantics do not fit cleanly inside the existing waveform-only `command_model.md`, and the proposal explicitly says durable outcomes must move into the canonical design corpus.
  Date/Author: 2026-04-18 / OpenCode

- Decision: store shipped narrative-doc source in a package-included tree rooted at `docs/cli/`, with Markdown topic files as the canonical authoring source and a package-included `docs/cli/wavepeek-skill.md` as the canonical shipped skill source.
  Rationale: this keeps prose assets in Markdown, preserves cargo-package buildability, and avoids making `.opencode/` the only packaged source of a user-visible product surface.
  Date/Author: 2026-04-18 / OpenCode

- Decision: keep `.opencode/skills/wavepeek/SKILL.md` as a derived runtime asset for OpenCode, but add a regression check that it stays synchronized with the package-included skill source.
  Rationale: the installed binary and packaged source need a stable source file that is not excluded from cargo packaging, but agent workflows still need the OpenCode skill path.
  Date/Author: 2026-04-18 / OpenCode

- Decision: prefer clap's native short-help versus long-help behavior for `-h`, `--help`, and the `help` subcommand, while preserving the existing no-args alias to top-level `--help` and only falling back to a thin custom resolver if tests prove clap cannot satisfy nested help-path aliases.
  Rationale: the current custom `ArgAction::HelpLong` wiring is the direct source of the parity contract that now needs to be removed; returning to clap's native model is the lowest-risk route to progressive disclosure.
  Date/Author: 2026-04-18 / OpenCode

- Decision: implement docs-topic discovery, suggestions, and search ranking with deterministic lowercase normalization, whitespace tokenization, prefix and substring checks, and explicit structural buckets rather than fuzzy matching libraries.
  Rationale: this matches the proposal's deterministic non-goals, keeps ranking explainable, and avoids introducing a heavyweight search dependency.
  Date/Author: 2026-04-18 / OpenCode

- Decision: start the export manifest at `export_format_version = 1` and treat that integer as part of the managed-root overwrite contract.
  Rationale: the first stable export layout needs a concrete version marker so later implementations can refuse unknown roots instead of guessing overwrite compatibility.
  Date/Author: 2026-04-18 / OpenCode

- Decision: implement layered help with clap's native short-help and long-help support plus `after_help` / `after_long_help` guidance strings, while keeping the built-in `help` subcommand visible and authoritative.
  Rationale: removing the old global `HelpLong` override restored the native split cleanly, and targeted `after_help` text covered the remaining contract-sensitive navigation cues without introducing a custom help renderer.
  Date/Author: 2026-04-18 / OpenCode

- Decision: treat docs-search token scoring as distinct-token scoring, canonicalize the reported JSON `query` by lowercasing and collapsing internal whitespace, allow topic-ID token matching in the default search scope, and preserve `title_exact` as the strongest non-ID-exact bucket when the whole normalized query matches the title exactly.
  Rationale: the review cycle showed that search behavior needed a sharper separation between overall whole-query buckets and per-token coverage to stay deterministic, schema-valid, and aligned with the documented ranking contract.
  Date/Author: 2026-04-18 / OpenCode

- Decision: validate cross-topic `see_also` references during catalog load and fail fast on unknown targets.
  Rationale: once `see_also` is exposed through JSON output and exported manifests, dead references become shipped contract drift rather than harmless prose mistakes, so referential integrity belongs in the loader guardrails.
  Date/Author: 2026-04-18 / OpenCode

## Outcomes & Retrospective

Current status: the full planned implementation is complete and the mandatory review workflow is clean. Packaged docs assets, embedded docs runtime, layered clap help, `help`/`docs` CLI wiring, engine/output integration, schema support, canonical-doc migration, packaged skill synchronization, follow-up review fixes, and the fresh final control pass are all done. Only final reporting remains.

The main outcome is that the installed binary is now the version-matched offline documentation entry point the proposal described. The planning work fixed three repo-specific gaps that would otherwise cause drift during implementation: it states that the real command inventory comes from `src/cli/`, it assigns a package-safe home for shipped Markdown and skill assets, and it introduces a new canonical design-contract file so the accepted semantics do not remain trapped in `docs/cmd_docs_proposal.md`. The TDD suites then converted those decisions into guardrails across help rendering, docs runtime behavior, export safety, skill synchronization, and schema coverage, and the implementation now satisfies those guardrails across both code and prose collateral.

The review cycle added value beyond merely signing off. It exposed subtle search-contract edges that the initial happy-path tests did not cover: topic-ID token matching, distinct-token scoring, canonical normalized `data.query`, exact-title precedence, and `see_also` referential integrity. Each of those cases is now covered by a regression test or loader guardrail. The implementation validated the broader design end to end: the tests initially failed exactly where the repo was incomplete, they now pass after the runtime and CLI work landed, the canonical docs now own the accepted semantics, and the manual smoke runs confirm the installed binary behaves as the primary offline docs surface. The main residual risk is outside this change set: unrelated unstaged formatting-only drift in `tests/common/*.rs`, which required temporary stashing for `make check` / `make ci` but was deliberately left untouched.

## Context and Orientation

`wavepeek` CLI parsing lives in `src/cli/mod.rs`. That file currently defines the top-level command enum, the top-level `about` and `long_about` text, parse-error normalization, and the custom help-flag builder that forces `-h` and `--help` to render the same long help. The shipped command-specific argument structs live in `src/cli/info.rs`, `src/cli/scope.rs`, `src/cli/signal.rs`, `src/cli/value.rs`, `src/cli/change.rs`, `src/cli/property.rs`, and `src/cli/schema.rs`. There is no `help` subcommand today, no `docs` command family, and no embedded-doc runtime.

The runtime command model lives in `src/engine/mod.rs`. That module defines the dispatch enum `Command`, the output discriminator `CommandName`, and the serializable payload enum `CommandData`. Human and JSON rendering live in `src/output.rs`. The schema URL constant and canonical schema bytes live in `src/schema_contract.rs`, while `schema/wavepeek.json` is the human-reviewed machine contract checked by `tests/schema_cli.rs` and `scripts/check_schema_contract.py`.

The current help contract is already enforced by tests. `tests/cli_contract.rs` asserts that `wavepeek` with no args matches `wavepeek --help`, that `-h` and `--help` are byte-identical at top level and subcommand scope, that `help` is absent from the visible command list, and that waveform-command help points readers to `wavepeek schema` for JSON details. Those tests are the first place that must change when adopting layered help and the new docs family.

The current canonical prose docs are split under `docs/design/`. `docs/design/index.md` explains that exact command names and flags are code-first in `src/cli/` and generated help. `docs/design/contracts/command_model.md` and `docs/design/contracts/machine_output.md` currently describe waveform-command semantics and JSON/error behavior. `docs/design/reference/cli.md` is intentionally thin and must remain a guide, not a second source of truth. `docs/cmd_docs_proposal.md` is explicitly temporary staging text and says that accepted semantics must move into the canonical design corpus.

Two terms in this plan need to stay concrete. “Progressive disclosure” means short help for quick lookup, long help for full command reference, and `wavepeek docs` for longer narrative guidance. A “topic ID” means the stable slash-separated name exposed to users, such as `commands/change` or `troubleshooting/empty-results`; it is not a raw filesystem path argument. A “managed export root” means a target directory whose root `manifest.json` contains `{"kind":"wavepeek-docs-export"}` and a recognized `export_format_version`, making it safe for `wavepeek docs export --force` to replace the full exported tree.

## Open Questions

No blocking product questions remain before implementation starts. The proposal-level ambiguities that would have changed code structure have been resolved in this plan.

There is one implementation-time escape hatch to keep explicit. The default path is to rely on clap's native `-h`, `--help`, and `help` behavior after removing the custom global `HelpLong` override. If targeted tests prove that clap's built-in `help <command-path...>` behavior cannot satisfy the exact nested-path alias contract, the fallback is to add a thin custom `help` subcommand that resolves the target command node from `build_cli_command()` and prints that node's long help. Do not invent a different layered-help model.

## Plan of Work

Milestone 1 is TDD and contract capture. Start by rewriting the existing CLI help contract tests so they describe the desired layered behavior instead of the old parity behavior. Expand `tests/cli_contract.rs` to cover compact top-level short help, detailed top-level long help, `wavepeek` no-args parity with top-level long help only, visible `help` and `docs` subcommands, nested `wavepeek help <command-path...>` aliases, and nested parse-error help hints such as `See 'wavepeek docs show --help'.` Add a new integration suite `tests/docs_cli.rs` that uses the installed embedded corpus to lock `docs`, `docs topics`, `docs show`, `docs search`, `docs export`, and `docs skill`, including the `--json` support matrix, deterministic ordering, ranking buckets, summary mode, unknown-topic suggestions, and export-root overwrite safety. Extend `tests/schema_cli.rs` so the canonical schema is required to include `docs topics` and `docs search` branches. This milestone is complete when the new tests fail for the current code because the feature is not implemented yet, not because the assertions are vague.

Milestone 2 creates the shipped docs corpus and the embedding runtime. Create a new package-included source tree under `docs/cli/`. That tree must include `docs/cli/AGENTS.md`, the canonical topic Markdown files under `docs/cli/topics/`, and a package-included `docs/cli/wavepeek-skill.md` that is the canonical source for the installed skill text. If `docs/cli/topics/` uses durable family subdirectories such as `concepts/`, `commands/`, `workflows/`, or `troubleshooting/`, add local `AGENTS.md` breadcrumbs there in the same change to satisfy the repository breadcrumb policy. The `commands/*` topic family is allowed only for narrative companion material: concepts, caveats, workflows, and longer examples that complement help. Those topic files must not duplicate clap syntax blocks, flag tables, defaults, or authoritative reference semantics. Add a new runtime module rooted at `src/docs/` that embeds the topic tree, parses YAML front matter plus Markdown bodies, validates that each body starts with an H1 heading matching the canonical `title`, indexes topics by stable `id`, extracts headings for search, and exposes deterministic lookup, search, suggestion, and export helpers. Keep the source of truth in Markdown files only; do not duplicate topic metadata as hand-maintained Rust literals.

Milestone 3 reworks clap help behavior and adds the new CLI surface. Update `src/cli/mod.rs` so top-level no-args invocation still routes through synthetic `--help`, but restore layered short versus long help instead of forcing `HelpLong` everywhere. Remove the tests and code that require `-h == --help` parity, make `help` a visible supported subcommand, and add a new nested docs command module at `src/cli/docs.rs`. The bare `wavepeek docs` command is an action, not a help alias: it should print the short docs index/orientation text described by the proposal. `wavepeek docs -h` and `wavepeek docs --help` must still follow the layered help rules like every other command node. Update top-level and per-command long help text so long help ends with purposeful `See also:` links into `wavepeek docs show <topic>` where narrative guidance exists, while short help stays compact and points to `--help`, `help`, and `docs` for the next layer. Generalize `help_hint_for_rendered_clap_error()` so it reconstructs the full nested command path from clap's rendered `Usage:` line instead of only the first subcommand token.

Milestone 4 wires the docs runtime into engine dispatch, output rendering, and the schema contract. Extend `src/engine/mod.rs` with the docs command family and new command names for the JSON-producing docs subcommands. Add `src/engine/docs.rs` to run `docs topics`, `docs show`, `docs search`, `docs export`, and `docs skill`, delegating the corpus operations to `src/docs/`. Extend `CommandData` and `src/output.rs` so human-only docs outputs can emit raw text/Markdown, while `docs topics --json` and `docs search --json` serialize through the standard JSON envelope with the exact `command` values `docs topics` and `docs search`. `docs export` must write the canonical authored topic Markdown files with YAML front matter preserved verbatim, plus the managed-root `manifest.json`; it intentionally excludes the separately routed skill asset, which remains available only through `wavepeek docs skill`. Only `docs show` strips front matter for stdout display. Update `schema/wavepeek.json` with new `$defs` for topic summaries, search matches, and their `match_kind` enum, then align `src/schema_contract.rs`, `wavepeek schema`, and schema tests. Unsupported combinations such as `wavepeek docs show --json` or `wavepeek docs export --json` must fail as argument errors, not silently switch modes.

Milestone 5 migrates the accepted semantics into canonical docs and trims staging text. Create `docs/design/contracts/documentation_surface.md` and populate it from the accepted parts of `docs/cmd_docs_proposal.md`: layered help semantics, `help` aliasing, docs-topic metadata/front-matter rules, search ranking, `--json` support matrix, export manifest contract, and the single-source-of-truth ownership split. Update `docs/design/contracts/machine_output.md` so its normative JSON-envelope language covers all stable JSON-producing commands, including `docs topics --json` and `docs search --json`, rather than only waveform commands. Update `docs/design/contracts/command_model.md` so it is explicit that its `--waves`, time-window, and ordering rules apply to waveform-inspection commands only, while the new `docs` command family is a separate non-waveform surface governed by `documentation_surface.md`. Update `docs/design/index.md`, `docs/design/contracts/AGENTS.md`, and `docs/design/reference/cli.md` so they point readers to the new canonical contract while keeping `reference/cli.md` thin. Update `docs/DEVELOPMENT.md`, `README.md`, `.opencode/skills/wavepeek/SKILL.md`, and `CHANGELOG.md` so live prose no longer promises universal `-h == --help` parity and instead describes the layered-help plus `wavepeek docs` model. Once those canonical files are in place, reduce `docs/cmd_docs_proposal.md` to a short accepted-history pointer that links to the new canonical contract and the relevant code-first authorities instead of remaining a second full contract document.

Milestone 6 is full validation, review, and follow-up fixes. Run the targeted test suites first, then the full repository gates. After the implementation diff is committed in atomic units, load `ask-review` and run three focused lanes in parallel: a docs lane for wording, cross-links, Markdown corpus quality, and proposal-to-contract migration clarity; a code lane for correctness, ranking rules, export overwrite safety, and tests; and an architecture lane for package-safe asset ownership, source-of-truth boundaries, and schema/output layering. Fix findings in follow-up commits without amending. Then run one fresh independent control pass on the consolidated diff and close the plan only when that control review is clean.

### Concrete Steps

Run all commands from `/workspaces/docs-rework`.

1. Reconfirm the current help and schema surface before editing code.

       cargo test --test cli_contract help_lists_expected_subcommands -- --exact
       cargo test --test cli_contract top_level_short_and_long_help_are_identical -- --exact
       cargo test --test cli_contract short_and_long_help_are_identical_for_shipped_commands -- --exact
       cargo test --test schema_cli schema_command_output_is_valid_json -- --exact

   Expected before implementation: the parity tests pass and there is no docs-command coverage yet.

2. Add the failing tests for the new contract first.

   Add or rename targeted integration tests so these commands exist and fail on the legacy implementation:

        cargo test --test cli_contract top_level_short_help_is_compact_and_points_to_next_layers -- --exact
        cargo test --test cli_contract top_level_long_help_describes_help_and_docs_entrypoints -- --exact
        cargo test --test cli_contract help_subcommand_matches_top_level_long_help -- --exact
        cargo test --test cli_contract help_subcommand_aliases_nested_long_help -- --exact
        cargo test --test cli_contract change_help_is_layered_and_links_to_docs -- --exact
        cargo test --test cli_contract docs_command_help_is_layered -- --exact
        cargo test --test cli_contract docs_show_help_is_layered -- --exact
        cargo test --test cli_contract nested_parse_errors_point_to_full_help_path -- --exact
        cargo test --test docs_cli docs_topics_are_sorted_lexicographically -- --exact
        cargo test --test docs_cli docs_command_prints_orientation_index -- --exact
        cargo test --test docs_cli docs_topics_json_uses_standard_envelope -- --exact
        cargo test --test docs_cli docs_search_ranks_matches_deterministically -- --exact
        cargo test --test docs_cli docs_search_empty_query_is_argument_error -- --exact
        cargo test --test docs_cli docs_show_unknown_topic_suggests_close_matches -- --exact
        cargo test --test docs_cli unsupported_docs_json_modes_are_argument_errors -- --exact
        cargo test --test docs_cli docs_export_force_requires_managed_root -- --exact
        cargo test --test docs_cli docs_export_force_rejects_unrecognized_manifest_version -- --exact
        cargo test --test docs_cli docs_export_preserves_front_matter -- --exact
        cargo test --test docs_cli docs_export_excludes_skill_asset -- --exact
        cargo test --test docs_cli docs_export_manifest_matches_contract -- --exact
        cargo test --test docs_cli docs_export_replaces_stale_managed_files -- --exact
        cargo test --test docs_cli docs_skill_prints_packaged_skill_markdown -- --exact
        cargo test --test schema_cli schema_command_includes_docs_command_branches -- --exact

   Expected before implementation: the new help and docs tests fail because `help`/`docs` do not exist yet and because the current CLI still forces `-h` and `--help` to be identical.

3. Create the shipped docs source tree and the loader.

   Create and populate these paths first so later code can compile against a stable corpus:

   - `docs/cli/AGENTS.md`
   - `docs/cli/topics/AGENTS.md`
   - `docs/cli/topics/intro.md`
   - `docs/cli/topics/concepts/AGENTS.md`
   - `docs/cli/topics/concepts/time.md`
   - `docs/cli/topics/concepts/selectors.md`
   - `docs/cli/topics/commands/AGENTS.md`
   - `docs/cli/topics/commands/change.md`
   - `docs/cli/topics/commands/property.md`
   - `docs/cli/topics/commands/help.md`
   - `docs/cli/topics/commands/docs.md`
   - `docs/cli/topics/workflows/AGENTS.md`
   - `docs/cli/topics/workflows/find-first-change.md`
   - `docs/cli/topics/troubleshooting/AGENTS.md`
   - `docs/cli/topics/troubleshooting/empty-results.md`
   - `docs/cli/wavepeek-skill.md`

   Then add the runtime modules and unit tests:

       cargo test docs:: --lib

   The first topic files must follow one exact shape so the loader is deterministic:

       ---
       id: commands/change
       title: Change command
       summary: Inspect value transitions across a bounded time range.
       section: commands
       see_also:
         - concepts/time
         - workflows/find-first-change
       ---
       # Change command

   `docs show` will later print only the body, so the embedded loader must keep the front matter and body split explicitly.

4. Implement the layered help surface and docs command family.

        cargo test --test cli_contract top_level_short_help_is_compact_and_points_to_next_layers -- --exact
        cargo test --test cli_contract top_level_long_help_describes_help_and_docs_entrypoints -- --exact
        cargo test --test cli_contract help_subcommand_matches_top_level_long_help -- --exact
        cargo test --test cli_contract help_subcommand_aliases_nested_long_help -- --exact
        cargo test --test cli_contract change_help_is_layered_and_links_to_docs -- --exact
        cargo test --test cli_contract docs_command_help_is_layered -- --exact
        cargo test --test cli_contract docs_show_help_is_layered -- --exact
        cargo test --test cli_contract nested_parse_errors_point_to_full_help_path -- --exact

   Acceptance after this step: `wavepeek` still matches `wavepeek --help`, but `wavepeek -h` is shorter, `wavepeek help` is visible and works for nested paths, and top-level help advertises `wavepeek docs` instead of claiming global `-h == --help` parity.

5. Implement docs runtime, schema updates, and export safety.

       cargo test --test docs_cli docs_topics_json_uses_standard_envelope -- --exact
       cargo test --test docs_cli docs_topics_are_sorted_lexicographically -- --exact
       cargo test --test docs_cli docs_command_prints_orientation_index -- --exact
       cargo test --test docs_cli docs_search_ranks_matches_deterministically -- --exact
       cargo test --test docs_cli docs_search_empty_query_is_argument_error -- --exact
       cargo test --test docs_cli docs_show_unknown_topic_suggests_close_matches -- --exact
       cargo test --test docs_cli docs_export_force_requires_managed_root -- --exact
       cargo test --test docs_cli docs_export_force_rejects_unrecognized_manifest_version -- --exact
       cargo test --test docs_cli docs_export_preserves_front_matter -- --exact
       cargo test --test docs_cli docs_export_excludes_skill_asset -- --exact
       cargo test --test docs_cli docs_export_manifest_matches_contract -- --exact
       cargo test --test docs_cli docs_export_replaces_stale_managed_files -- --exact
       cargo test --test docs_cli docs_skill_prints_packaged_skill_markdown -- --exact
       cargo test --test schema_cli schema_command_includes_docs_command_branches -- --exact
       cargo test --test schema_cli schema_command_prints_canonical_artifact_bytes -- --exact

   Acceptance after this step: the docs command family works end to end, `wavepeek schema` exposes the docs JSON branches, and unsupported `--json` combinations fail as `error: args:`.

6. Migrate the accepted semantics into canonical docs and update live collateral.

       cargo test --test cli_contract
       cargo test --test docs_cli
       cargo test --test schema_cli
       cargo test -q
       make check

   If the container has the required external fixtures, also run:

       make ci

   Then manually exercise the full progressive-disclosure path:

        cargo run -- -h
        cargo run -- --help
        cargo run -- help
        cargo run -- help change
        cargo run -- help docs show
        cargo run -- docs
       cargo run -- docs topics
       cargo run -- docs topics --summary
       cargo run -- docs topics --json
       cargo run -- docs show intro
       cargo run -- docs show commands/change --summary
       cargo run -- docs search transitions
       cargo run -- docs search ready valid --full-text --json
       cargo run -- docs skill
       cargo run -- docs export /tmp/wavepeek-docs-plan-check

   Expected signatures include:

       wavepeek local docs
       Start here when you need more than command syntax.

   and:

       "$schema": "https://raw.githubusercontent.com/kleverhq/wavepeek/v0.4.0/schema/wavepeek.json"
       "command": "docs topics"

   and the exported root manifest containing:

       "kind": "wavepeek-docs-export"
       "export_format_version": 1
       "cli_name": "wavepeek"
       "cli_version": "0.4.0"

7. Commit in atomic units without rewriting history.

   Suggested commit split:

       git add Cargo.toml docs/cli src/docs src/cli/mod.rs src/cli/docs.rs src/engine/docs.rs src/engine/mod.rs src/output.rs schema/wavepeek.json src/schema_contract.rs tests/cli_contract.rs tests/docs_cli.rs tests/schema_cli.rs
       git commit -m "feat(cli): add layered help and embedded docs"

       git add docs/design/index.md docs/design/contracts/documentation_surface.md docs/design/contracts/machine_output.md docs/design/contracts/AGENTS.md docs/design/reference/cli.md docs/DEVELOPMENT.md README.md CHANGELOG.md docs/cmd_docs_proposal.md .opencode/skills/wavepeek/SKILL.md
       git commit -m "docs: migrate self-documenting CLI contracts"

   If you prefer a smaller split, only split further when each commit still builds, passes its relevant targeted tests, and leaves the repo in a reviewable state. If review finds issues, fix them in one or more follow-up commits. Do not amend or squash.

8. Run the mandatory review workflow.

   Load `ask-review` and prepare one concise context packet with the scope summary, commit range, validation already run, and the remaining risks. Run three focused lanes in parallel:

   - Docs lane: help wording, cross-links, topic quality, proposal-to-contract migration, and README/changelog accuracy.
   - Code lane: layered help correctness, ranking logic, unknown-topic suggestions, export-root safety, unsupported `--json` rejection, and regression coverage.
   - Architecture lane: package-safe asset ownership, schema/output layering, duplication risk between `docs/cli/`, `docs/design/`, and `.opencode/`, and breadcrumb-policy compliance.

   After fixing findings and committing them, run one fresh independent control review on the consolidated diff. Close the plan only when that control pass is clean.

### Validation and Acceptance

The implementation is acceptable only when all of the following behaviors are observable.

- `wavepeek` with no arguments prints the same bytes as `wavepeek --help`.
- `wavepeek -h` is materially shorter than `wavepeek --help` and explicitly points readers to `wavepeek --help`, `wavepeek help <command>`, and `wavepeek docs`.
- `wavepeek help` is byte-identical to top-level long help, and `wavepeek help <command-path...>` is byte-equivalent to `<command-path> --help` for nested paths such as `change` and `docs show`.
- At least one waveform command and one docs subcommand prove the layered-help contract directly: for example, `wavepeek change -h` is shorter than `wavepeek change --help`, `wavepeek change --help` includes a valid `See also: wavepeek docs show commands/change` link, and `wavepeek docs show -h` stays compact while `wavepeek docs show --help` gives the fuller reference.
- `wavepeek docs` prints the orientation index text, not clap help output.
- `wavepeek docs -h` stays compact while `wavepeek docs --help` provides the fuller reference for the docs command family.
- `wavepeek docs topics` lists all embedded topics in lexicographic topic-ID order, while `wavepeek docs topics --json` emits the standard JSON envelope with `command = "docs topics"` and `data.topics[...]` entries containing `id`, `title`, `summary`, `section`, and optional `see_also`.
- `wavepeek docs show <topic>` prints raw Markdown body with no YAML front matter, while `--summary` prints only the stored summary text.
- Unknown topics fail with a non-zero exit code and deterministic close-match suggestions when matches exist.
- `wavepeek docs search <query>` returns all matches in deterministic ranked order, honors `--full-text`, succeeds with an empty result set when nothing matches, and fails as an argument error when normalization leaves zero tokens.
- `wavepeek docs search --json` emits the standard JSON envelope with `command = "docs search"`, the normalized `query`, `full_text`, and `matches[...]` entries carrying the strongest `match_kind`.
- Unsupported `--json` cases (`docs show`, `docs skill`, `docs export`) fail as argument errors and never print a human-mode payload on stdout.
- `wavepeek docs export <out-dir>` writes one Markdown file per topic using the stable topic path with its YAML front matter preserved verbatim, plus a deterministic `manifest.json` whose required fields are `kind = "wavepeek-docs-export"`, `export_format_version = 1`, `cli_name = "wavepeek"`, `cli_version`, and lexicographically ordered `topics`; it refuses non-empty unmanaged targets by default, allows `--force` only for empty or previously managed export roots with a recognized `export_format_version`, rejects previously managed roots whose manifest version is unrecognized, and does not export the separately routed skill file.
- `wavepeek docs skill` prints the packaged skill Markdown that also keeps `.opencode/skills/wavepeek/SKILL.md` synchronized.
- `wavepeek schema` includes the docs JSON branches and remains byte-identical to `schema/wavepeek.json`.
- Live prose docs (`docs/design/contracts/documentation_surface.md`, `docs/design/contracts/command_model.md`, `docs/design/contracts/machine_output.md`, `docs/design/reference/cli.md`, `docs/DEVELOPMENT.md`, `README.md`, `CHANGELOG.md`) all describe the layered-help model and no live document still claims universal `-h == --help` parity.

### Idempotence and Recovery

The implementation steps are meant to be repeatable. Re-running the tests, schema validation, and human-verification commands is always safe. Re-running the docs-loader build path is also safe because the loader should fail fast on duplicate IDs, invalid front matter, missing headings, or title-heading mismatches before any runtime behavior ships.

For export safety, keep writes staged through a temporary sibling directory when the host platform allows it, then rename into place only after the full tree and `manifest.json` are ready. If an export attempt fails before the final rename, delete the temporary directory and rerun. If the target directory already exists and is non-empty, do not manually delete it unless it is a managed export root and the user requested replacement with `--force`.

For skill synchronization, make the package-included `docs/cli/wavepeek-skill.md` the canonical source, derive or copy `.opencode/skills/wavepeek/SKILL.md` from it in the same change, and add a test that fails if they drift. If that test fails during a retry, update the derived skill file and rerun the same targeted tests.

### Artifacts and Notes

Keep the most important evidence small and local to the changed files. The implementation should leave these concise artifacts in the commit history and, where helpful, in test fixtures:

    wavepeek -h
    wavepeek --help
    wavepeek help change
    wavepeek docs
    wavepeek docs topics --json
    wavepeek docs search ready valid --full-text --json
    wavepeek docs export /tmp/wavepeek-docs

The docs corpus should contain short but real narrative topics, not placeholders. Each long-help page should cross-link only to topics that actually exist. A minimum viable seed corpus is `intro`, the time and selector concept topics, narrative companion topics for `commands/change`, `commands/property`, `commands/help`, and `commands/docs`, one workflow topic, and one troubleshooting topic. Additional `commands/<name>.md` topics are optional and must stay narrative-only; clap help remains the sole command-reference authority.

When shrinking `docs/cmd_docs_proposal.md`, leave one short note that it was an accepted staging proposal and point readers to `docs/design/contracts/documentation_surface.md`, `docs/design/reference/cli.md`, `src/cli/`, and generated help. Do not leave the proposal as a second full contract after the canonical docs are updated.

### Interfaces and Dependencies

Add the smallest new dependencies needed to embed and parse Markdown assets cleanly. In `Cargo.toml`, add `include_dir` (or an equivalent directory-embedding crate) and `serde_yaml` for YAML front matter parsing. Do not introduce a full static-site generator or a fuzzy-search library.

In `src/docs/mod.rs`, define the runtime catalog surface that the rest of the code will call. At the end of the milestone, this module must expose a deterministic API equivalent to:

    pub struct TopicSummary {
        pub id: String,
        pub title: String,
        pub summary: String,
        pub section: String,
        pub see_also: Vec<String>,
    }

    pub struct TopicRecord {
        pub summary: TopicSummary,
        pub raw_markdown: String,
        pub body: String,
        pub headings: Vec<String>,
        pub source_relpath: String,
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
    #[serde(rename_all = "snake_case")]
    pub enum MatchKind {
        IdExact,
        IdPrefix,
        TitleExact,
        TitleOrSummary,
        Heading,
        Body,
    }

    pub struct SearchMatch {
        pub topic: TopicSummary,
        pub match_kind: MatchKind,
        pub matched_tokens: usize,
    }

    pub fn embedded_catalog() -> Result<&'static DocsCatalog, WavepeekError>;
    pub fn lookup_topic(id: &str) -> Option<&TopicRecord>;
    pub fn search_topics(query: &str, full_text: bool) -> Result<Vec<SearchMatch>, WavepeekError>;
    pub fn suggest_topics(input: &str, limit: usize) -> Vec<TopicSummary>;
    pub fn export_catalog(out_dir: &std::path::Path, force: bool) -> Result<ExportSummary, WavepeekError>;

Use `BTreeMap` or another deterministic ordered structure inside the catalog rather than hash-map iteration for user-visible ordering. `raw_markdown` must retain the original authored UTF-8 bytes, including YAML front matter formatting, key order, and comments, so `export_catalog()` can write the topic files back unchanged.

In `src/cli/docs.rs`, define the user-facing clap surface explicitly. The final shape should be equivalent to:

    #[derive(Debug, clap::Args)]
    pub struct DocsArgs {
        #[command(subcommand)]
        pub command: Option<DocsCommand>,
    }

    #[derive(Debug, clap::Subcommand)]
    pub enum DocsCommand {
        Topics(DocsTopicsArgs),
        Show(DocsShowArgs),
        Search(DocsSearchArgs),
        Export(DocsExportArgs),
        Skill(DocsSkillArgs),
    }

`DocsArgs` with `command = None` must route to the orientation index output. `DocsTopicsArgs` and `DocsSearchArgs` are the only docs subcommands that support `--json`. Model their `--summary`, `--json`, and `--full-text` relationships with clap conflicts rather than post-parse guesswork.

In `src/engine/mod.rs` and `src/engine/docs.rs`, extend the runtime surface so JSON-producing docs subcommands have stable command names and data types. The final engine-level additions must be equivalent to:

    pub enum CommandName {
        // existing variants...
        DocsTopics,
        DocsSearch,
    }

    impl CommandName {
        pub const fn as_str(self) -> &'static str {
            // ...
            Self::DocsTopics => "docs topics",
            Self::DocsSearch => "docs search",
        }
    }

    pub enum CommandData {
        // existing variants...
        Text(String),
        DocsTopics(DocsTopicsData),
        DocsSearch(DocsSearchData),
    }

Define `DocsTopicsData` as an object containing `topics: Vec<TopicSummary>` and `DocsSearchData` as an object containing `query: String`, `full_text: bool`, and `matches: Vec<DocsSearchMatchData>`. Keep the top-level JSON envelope unchanged: `$schema`, `command`, `data`, and `warnings` remain the only outer keys.

Define the export manifest explicitly so its serialized field order is deterministic:

    pub struct ExportManifest {
        pub kind: String,
        pub export_format_version: u32,
        pub cli_name: String,
        pub cli_version: String,
        pub topics: Vec<TopicSummary>,
    }

Set `export_format_version` to `1` for this initial implementation and keep `topics` lexicographically ordered by topic ID in both the manifest and `docs topics` output.

Update `schema/wavepeek.json` and the docs contract files together. The schema must gain `$defs` for topic summaries, search matches, docs-topics data, and docs-search data, plus `allOf` branches that bind `command = "docs topics"` and `command = "docs search"` to the correct `data` shapes. Keep `wavepeek schema` as the sole machine-contract authority for the precise JSON schema bytes.

Revision Note: 2026-04-18 / OpenCode - Initial active ExecPlan created from `docs/cmd_docs_proposal.md`, with repo-specific decisions added for the current `property` command surface, a package-safe shipped-docs source tree, and canonical migration into `docs/design/contracts/documentation_surface.md`.

Revision Note: 2026-04-18 / OpenCode - Updated after review to keep the canonical contracts fully aligned (`machine_output.md` and `command_model.md`), require front-matter-preserving docs export, narrow the required `commands/*` topic seed set to narrative-only adjuncts, and replace the suggested commit split with build-safe atomic commits.

Revision Note: 2026-04-18 / OpenCode - Updated after the final control review to make bare `wavepeek help` an explicit tested alias, add direct subcommand layered-help regression checks, and require the docs catalog to retain original Markdown bytes so export can preserve front matter verbatim.

Revision Note: 2026-04-18 / OpenCode - Updated after the follow-up control review to move the remaining docs contract checks into the TDD-first step list, clarify that `docs export` excludes the skill asset, and record the last review round in `Progress` and `Outcomes & Retrospective`.

Revision Note: 2026-04-18 / OpenCode - Updated after TDD contract capture to record the baseline confirmation runs, the newly failing layered-help/docs/schema tests, and the discovery that the package-safe docs asset requirement is already enforced by the new integration suite.

Revision Note: 2026-04-18 / OpenCode - Updated after the first implementation slice to record the new packaged docs corpus, embedded runtime, layered help wiring, schema/output integration, targeted validation success, and the clap-specific short-help discovery that required explicit compact-shape guidance in `after_help`.

Revision Note: 2026-04-18 / OpenCode - Updated after canonical-doc migration and full validation to record the new documentation-surface contract, the successful full-gate/manual verification pass, and the temporary stash workaround used to avoid rewriting unrelated unstaged formatting-only changes during `make check` / `make ci`.

Revision Note: 2026-04-18 / OpenCode - Updated after the mandatory review workflow to record the lane findings, the follow-up fix commits for docs-search and guidance edge cases, and the final clean control pass.
