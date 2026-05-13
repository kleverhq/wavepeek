# Promote the packaged skill from `wavepeek docs skill` to `wavepeek skill`

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

Note that this document must be maintained in accordance with the `exec-plan` skill.

## Purpose / Big Picture

Today the packaged agent skill is printed through `wavepeek docs skill`. That makes one non-documentation surface live under the documentation command family, which is awkward for users, awkward for top-level help, and awkward for repository collateral because the same skill is treated as both a docs subcommand and a standalone installable asset.

After this change, users and agents will run `wavepeek skill` to print the packaged skill Markdown directly. `wavepeek docs` will stay focused on topic discovery, topic display, search, and export. The visible proof is simple: `wavepeek skill` prints the exact bytes from `docs/skills/wavepeek.md`, `wavepeek --help` lists `skill` as a top-level helper command, `wavepeek docs --help` and `wavepeek docs` stop advertising a `skill` subcommand, and `wavepeek docs show commands/skill` becomes the narrative topic that explains the new entrypoint.

## Non-Goals

This plan does not change the packaged skill's substantive guidance unless a wording update is required only to keep command references accurate after the move.

This plan does not add JSON output for the skill surface. `skill` remains a human or Markdown printing command.

This plan does not change docs export semantics. `wavepeek docs export` must continue to export public topics only and must continue to exclude the packaged skill asset.

This plan does not implement the feature during plan authoring. The current branch work for this request is limited to producing and reviewing this ExecPlan, then committing the plan-only result.

## Progress

- [x] (2026-05-12 19:53Z) Read the repository maps and planning rules in `AGENTS.md`, `docs/AGENTS.md`, `docs/exec-plans/AGENTS.md`, and the `exec-plan` skill before drafting.
- [x] (2026-05-12 19:53Z) Mapped the current shipped skill surface across `src/cli/docs.rs`, `src/cli/mod.rs`, `src/engine/docs.rs`, `src/engine/mod.rs`, `src/docs/mod.rs`, `docs/public/commands/docs.md`, `docs/public/commands/overview.md`, `docs/public/intro.md`, `README.md`, `docs/DEVELOPMENT.md`, `docs/skills/AGENTS.md`, `tests/docs_cli.rs`, and `tests/cli_contract.rs`.
- [x] (2026-05-12 19:53Z) Searched live repository collateral for `docs skill`, `packaged skill`, `agent skill`, and related phrases to separate required collateral from no-change surfaces.
- [x] (2026-05-12 19:53Z) Created the active ExecPlan at `docs/exec-plans/active/2026-05-12-root-skill-command/PLAN.md` and committed it as `a535bdd` (`docs: plan root skill command promotion`).
- [x] (2026-05-12 20:04Z) Attempted focused read-only subagent review lanes for docs, code, and architecture. The helper wrapper mis-handled prompts containing backticks and the word `skill`, so those runs did not return usable findings.
- [x] (2026-05-12 20:04Z) Completed a main-session review fallback and identified three plan fixes: add an explicit regression test for rejecting `wavepeek docs skill`, include `docs/public/commands/help.md` and `docs/public/commands/AGENTS.md` in the collateral list, and clarify that `CHANGELOG.md` must revise the existing unreleased docs-surface wording rather than only append a new line.
- [x] (2026-05-12 20:10Z) Applied the review fixes in this plan file, recorded the failed subagent attempt plus the manual-review fallback, and finalized the reviewed plan revision for the second plan-only commit.
- [x] (2026-05-13 06:20Z) Rewrote the CLI and engine command tree so `skill` is a top-level helper command implemented by `src/cli/skill.rs` and `src/engine/skill.rs`, while `docs` now owns only topics, show, search, and export.
- [x] (2026-05-13 06:24Z) Updated public docs, maintainer collateral, breadcrumbs, README, changelog, and development guidance so all live command references point to `wavepeek skill` and `commands/skill`.
- [x] (2026-05-13 06:28Z) Moved contract coverage to the new surface: added `tests/skill_cli.rs`, updated `tests/docs_cli.rs` and `tests/cli_contract.rs`, and adjusted embedded-doc export assertions for the additional public topic.
- [x] (2026-05-13 06:31Z) Validated the implementation with `cargo test --test docs_cli --test skill_cli --test cli_contract --test schema_cli` and `make check`; all passed.
- [x] (2026-05-13 06:37Z) Attempted focused read-only implementation review lanes again, but the subagent helper still produced empty print-mode logs. Completed a main-session review fallback on the final diff and found no substantive follow-up fixes beyond the already-applied branch changes.

## Surprises & Discoveries

- Observation: the current packaged skill content is already source-of-truth data in `docs/skills/wavepeek.md`; the command move is mostly a surface-routing and collateral problem, not a skill-content migration.
  Evidence: `src/docs/mod.rs` exposes `packaged_skill_markdown()` from `docs/skills/wavepeek.md`, and the current docs-path wiring only decides how that text is reached.

- Observation: the current live collateral is broader than only code and the docs topic that names `docs skill`.
  Evidence: repository search shows the old surface in `README.md`, `CHANGELOG.md`, `docs/DEVELOPMENT.md`, `docs/skills/AGENTS.md`, `docs/public/commands/docs.md`, `docs/public/commands/overview.md`, `docs/public/intro.md`, `tests/docs_cli.rs`, `tests/cli_contract.rs`, and the docs orientation text in `src/engine/docs.rs`.

- Observation: some likely-adjacent files are intentionally no-change surfaces for this migration.
  Evidence: the same search does not require behavioral edits in `schema/wavepeek.json`, `tests/schema_cli.rs`, `docs/ROADMAP.md`, `docs/RELEASE.md`, or `docs/BACKLOG.md`; they either do not mention the command path or they describe broader concepts that remain true after the promotion.

- Observation: adding the new `commands/skill` topic changed both docs-topic ordering and exported-topic count, so two separate guards had to move together.
  Evidence: `tests/docs_cli.rs` needed `TOPIC_IDS` to grow from 19 to 20 and `src/docs/mod.rs` needed `summary.topics.len()` to grow from 19 to 20 for export validation.

- Observation: the read-only subagent review workflow remained unreliable even after switching to prompt files.
  Evidence: the tmux sessions launched successfully, but `subagent.py tail` and the corresponding log files stayed empty while `subagent.py capture` showed only the wrapper banner and full prompt text, so the implementation review fell back to the main session again.

## Decision Log

- Decision: make `wavepeek skill` the only public command surface for printing the packaged skill, and remove `wavepeek docs skill` instead of keeping a long-lived compatibility alias.
  Rationale: this repository prefers clear canonical command names over compatibility aliases, and the move is specifically intended to lift the skill out of the docs command family rather than merely duplicate it.
  Date/Author: 2026-05-12 / Pi

- Decision: keep the packaged skill content source at `docs/skills/wavepeek.md` and keep `src/docs/mod.rs` as the byte source for the printed Markdown.
  Rationale: the change is about command topology, not about moving the asset again. Keeping one packaged Markdown source avoids content drift and keeps the already-tested byte-for-byte behavior intact.
  Date/Author: 2026-05-12 / Pi

- Decision: add a new public topic `docs/public/commands/skill.md` and remove skill-specific narrative content from `docs/public/commands/docs.md`.
  Rationale: after the command move, `commands/docs` should describe only the docs command family, while the top-level command corpus should still have one topic per durable top-level command family.
  Date/Author: 2026-05-12 / Pi

- Decision: treat schema and docs export as no-change contracts for this feature.
  Rationale: `wavepeek skill` is not a JSON envelope command and should not appear in `schema/wavepeek.json`, while `wavepeek docs export` must keep exporting only topic Markdown and excluding packaged skills.
  Date/Author: 2026-05-12 / Pi

## Outcomes & Retrospective

The migration is implemented and validated. `wavepeek skill` is now the canonical packaged-skill command, `wavepeek docs` stays limited to topic discovery/show/search/export, the public docs corpus contains `commands/skill`, and the schema plus docs-export contracts remained unchanged apart from the additional public topic count.

The implementation confirmed the plan’s original scoping insight: the risky part was not the Rust routing itself, but the breadth of user-visible collateral. The code move was small, while the help text, README, topic corpus, breadcrumbs, changelog, and integration tests were the surfaces most likely to drift if handled casually.

The main remaining tooling lesson is unchanged from planning and reinforced by implementation: the current subagent review wrapper is not dependable for this repository’s review prompts. Even prompt-file based invocations produced empty logs, so the effective review path for this task was a main-session fallback after validation.

## Context and Orientation

The repository root is `/workspaces/wavepeek`. The project is a Rust command-line tool named `wavepeek`. The current packaged skill surface lives under the docs command family. `src/cli/docs.rs` defines a nested `Skill(DocsSkillArgs)` variant under `DocsCommand`. `src/engine/docs.rs` handles that nested subcommand and prints `docs::packaged_skill_markdown()`. `src/engine/mod.rs` names the surface `docs skill` through `CommandName::DocsSkill`. The Markdown bytes themselves come from `src/docs/mod.rs`, which embeds `docs/skills/wavepeek.md`.

A packaged skill is a Markdown file that an external coding agent can install as task guidance. In this repository the packaged skill source is `docs/skills/wavepeek.md`. The installed CLI prints that file verbatim today, but it currently does so through `wavepeek docs skill`.

A helper command is a non-waveform top-level command such as `schema`, `docs`, or `help`. This plan promotes `skill` into that same top-level helper layer so the command tree reflects what the surface actually does.

The current collateral falls into five groups.

The first group is command wiring in `src/cli/docs.rs`, `src/cli/mod.rs`, `src/engine/docs.rs`, and `src/engine/mod.rs`. Those files define the visible CLI tree, top-level help template, nested help text, and human output routing.

The second group is packaged docs runtime support in `src/docs/mod.rs`. That module should keep returning the same packaged skill bytes and should not grow topic-export behavior for the skill.

The third group is public docs corpus content under `docs/public/`. Today `docs/public/commands/docs.md` still says the docs command prints the packaged skill, `docs/public/commands/overview.md` describes `docs` as including the skill, and `docs/public/intro.md` tells readers `wavepeek docs` covers skill text. There is currently no `docs/public/commands/skill.md` topic.

The fourth group is maintainer collateral and packaged-skill breadcrumbs. `README.md` describes installation through `wavepeek docs skill`; `docs/DEVELOPMENT.md` says the packaged skill is emitted verbatim by `wavepeek docs skill`; `docs/skills/AGENTS.md` says the directory contains skill Markdown emitted by `wavepeek docs skill`; and `CHANGELOG.md` currently records the unreleased docs surface as including the shipped agent skill under `wavepeek docs`.

The fifth group is tests. `tests/docs_cli.rs` currently treats `docs skill` as part of the docs command family, including help text, unsupported `--json` cases, and exact byte printing. `tests/cli_contract.rs` expects the nested docs help surfaces for `skill` to behave like other docs subcommands. `tests/schema_cli.rs` is a guardrail that should remain unchanged because `skill` does not add a JSON schema branch.

The plan must also preserve the repository's documented style: exact syntax remains in generated help and `src/cli/`; public docs topics stay routing-oriented; packaged skills stay short and routing-oriented; and `make check` is the pre-handoff validation gate.

## Open Questions

No blocking product question remains for planning. This plan resolves the main contract question by removing `docs skill` rather than keeping a compatibility alias.

One implementation detail should still be validated during coding: whether the top-level helper-command order should become `schema`, `docs`, `skill`, `help` or `schema`, `skill`, `docs`, `help`. This plan chooses `schema`, `docs`, `skill`, `help` so the narrative docs surface stays grouped before the packaged-skill print surface, and `help` remains the catch-all command at the end.

A second implementation detail is test organization. This plan chooses to move exact `wavepeek skill` behavior checks into a dedicated integration file `tests/skill_cli.rs` so `tests/docs_cli.rs` can focus only on the docs command family. If implementation finds that keeping the byte-for-byte check in `tests/docs_cli.rs` is materially simpler without making ownership confusing, record that change in the Decision Log and keep the rest of the collateral split intact.

## Plan of Work

Milestone 1 is test-first contract movement for the public surface. Start by changing the tests so they describe the intended command tree before changing the implementation. In `tests/cli_contract.rs`, add `skill` to the visible top-level command list, update the top-level help expectations so `skill` appears as a helper command, remove the nested `docs skill` help assertions from the docs-subcommand section, add top-level help-surface assertions for `wavepeek skill -h`, `wavepeek skill --help`, and `wavepeek help skill`, and add a regression that the old path `wavepeek docs skill` is rejected with a help hint back to `wavepeek docs --help`. In `tests/docs_cli.rs`, remove `docs skill` from docs-family help expectations, remove the unsupported `docs skill --json` case from docs-only argument checks, stop reading the packaged skill through `successful_stdout(["docs", "skill"])`, and add `commands/skill` to the expected topic list. Create a new integration file `tests/skill_cli.rs` that proves `wavepeek skill` prints the exact bytes from `docs/skills/wavepeek.md`, rejects `--json` as an argument error, and keeps any runtime-capability wording checks that currently live under docs tests. `tests/schema_cli.rs` should stay semantically unchanged and continues to prove that only JSON-producing docs subcommands appear in the schema contract.

Milestone 2 is command-tree rewiring. Create a new CLI module `src/cli/skill.rs` with `SkillArgs` and help text `Print the packaged agent skill Markdown for wavepeek.` Register that module in `src/cli/mod.rs` as a top-level helper command, update the top-level help template to list `skill`, and remove the nested `DocsCommand::Skill` variant plus `DocsSkillArgs` from `src/cli/docs.rs`. In the engine layer, add a new `src/engine/skill.rs` that returns the same text payload currently produced by `src/engine/docs.rs`. Register it through `src/engine/mod.rs` as `Command::Skill(...)` with a new `CommandName::Skill`. Remove `CommandName::DocsSkill`, the `skill()` handler from `src/engine/docs.rs`, and the `wavepeek docs` orientation copy that suggests `wavepeek docs skill`. `src/docs/mod.rs` should keep `packaged_skill_markdown()` unchanged, because that function is still the single byte source for the printed Markdown.

Milestone 3 is public-docs and breadcrumb collateral. Add `docs/public/commands/skill.md` with proper front matter, an H1 that matches the title, a short explanation that `wavepeek skill` prints the packaged skill Markdown, and pointers to `wavepeek help skill`, `wavepeek docs topics`, and the reference topics that the skill routes agents toward. Update `docs/public/commands/docs.md` so its front-matter summary and body describe topic discovery, topic display, search, and export only; remove the packaged-skill section entirely. Update `docs/public/commands/help.md` so its examples reflect the new top-level helper surface, for example by including `wavepeek help skill` alongside existing examples. Update `docs/public/commands/overview.md` so `docs` no longer claims to print the skill and so the helper-command overview mentions `skill` separately. Update `docs/public/intro.md` so the getting-help and docs-surface text no longer says `wavepeek docs` includes skill text, and add `commands/skill` to `see_also` if the cross-link map benefits from it. Update `docs/public/commands/AGENTS.md` so the command-topics breadcrumb names `skill.md` alongside the other durable command-family topics. Update `docs/public/reference/command-model.md` so non-waveform surfaces explicitly include `skill`. Update `docs/public/reference/machine-output.md` only enough to keep the human-only surface description accurate, for example by noting that human-only commands such as `skill` and most docs subcommands reject `--json` at argument parsing time. Do not add `skill` to schema-facing JSON examples or schema topic lists because it is not a JSON envelope command.

Milestone 4 is repository-level collateral outside the embedded topic corpus. Update `README.md` so the Agentic Flows section points to `wavepeek skill`, add `skill` to the command table as a separate helper command, and stop describing `wavepeek docs` as the place that prints skill text. Update `docs/DEVELOPMENT.md` so the packaged skill emission rule says `wavepeek skill`, not `wavepeek docs skill`. Update `docs/skills/AGENTS.md` so it describes the directory as the source emitted by `wavepeek skill`, and if `src/engine/skill.rs` is added, point the runtime-surface bullet at that new module instead of `src/engine/docs.rs`. Update `CHANGELOG.md` by revising the existing Unreleased docs-surface wording that currently says `wavepeek docs` exposes the shipped skill, and add any new entry needed to describe the command-surface promotion cleanly. Leave `docs/ROADMAP.md`, `docs/RELEASE.md`, `docs/BACKLOG.md`, and `schema/wavepeek.json` untouched unless implementation discovers a direct stale command-path reference that this plan's repository search missed.

Milestone 5 is validation, review, and branch closure for the implementation diff. After the code and collateral changes are in place, run the focused suites first: `cargo test --test docs_cli`, `cargo test --test skill_cli`, `cargo test --test cli_contract`, and `cargo test --test schema_cli`. Then run the repository gate `make check`, and if time allows or the diff grows beyond the planned surface, run `make ci`. After the branch is green, use the `ask-review` skill with read-only subagents. At minimum run a code lane for CLI tree and regression risks plus a docs lane for embedded topics, README, changelog, and breadcrumb wording. Because this migration changes command topology and no-change contracts, also run an architecture lane focused on surface boundaries: `docs` should still own topic operations, `skill` should own only the packaged skill print surface, and schema/export should remain unchanged. Fix findings in follow-up commits, rerun the affected validation, then run one fresh control-pass review on the consolidated diff before closing the plan.

## Concrete Steps

Run all commands from `/workspaces/wavepeek`.

1. Lock the public-surface tests before implementation.

       cargo test --test docs_cli docs_command_without_subcommand_prints_help -- --exact
       cargo test --test docs_cli docs_topics_use_logical_section_order -- --exact
       cargo test --test cli_contract nested_docs_help_surfaces_are_aligned_and_trimmed -- --exact

   These commands should fail after the tests are updated but before the command-tree implementation is done. The failure proves the test change is asserting the new surface rather than restating the current behavior.

2. Rewire the CLI and engine to lift `skill` to the top level.

       cargo test --test skill_cli skill_prints_packaged_skill_markdown -- --exact
       cargo test --test cli_contract nested_docs_help_surfaces_are_aligned_and_trimmed -- --exact
       cargo test --test cli_contract top_level_help_lists_expected_subcommands -- --exact

   After the new modules and help wiring exist, these targeted tests should pass.

3. Update public docs topics and repository collateral.

       cargo run -- skill
       cargo run -- skill --help
       cargo run -- docs --help
       cargo run -- docs show commands/skill
       cargo run -- docs show commands/docs

   The expected observations are: `wavepeek skill` prints the packaged Markdown; `wavepeek docs --help` no longer lists `skill`; `commands/skill` explains the new surface; and `commands/docs` no longer claims to print the packaged skill.

4. Run the full planned validation gate.

       cargo test --test docs_cli
       cargo test --test skill_cli
       cargo test --test cli_contract
       cargo test --test schema_cli
       make check

   If any of these fail, fix the surface mismatch before attempting review.

5. Run focused plan-completion review on the implementation diff.

       tmux -V
       command -v pi
       ~/.pi/agent/skills/subagents/scripts/subagent.py spawn --name root-skill-code --cwd /workspaces/wavepeek --tools read,bash --prompt '<code lane prompt>'
       ~/.pi/agent/skills/subagents/scripts/subagent.py spawn --name root-skill-docs --cwd /workspaces/wavepeek --tools read,bash --prompt '<docs lane prompt>'
       ~/.pi/agent/skills/subagents/scripts/subagent.py spawn --name root-skill-arch --cwd /workspaces/wavepeek --tools read,bash --prompt '<architecture lane prompt>'

   Each prompt must state that the subagent is read-only, must inspect the implementation diff, and must return concrete findings with severity and `file:line` references. After fixes, run one fresh independent control pass on the final diff.

## Validation and Acceptance

Acceptance is entirely behavior-based.

`wavepeek skill` must print the exact bytes from `docs/skills/wavepeek.md`. A byte-for-byte integration test must guard this.

`wavepeek --help` must list `skill` as a top-level helper command, and `wavepeek skill --help` must match `wavepeek help skill`. The first help line should remain `Print the packaged agent skill Markdown for wavepeek.`

`wavepeek docs` and `wavepeek docs --help` must no longer expose a `skill` subcommand. `wavepeek docs skill` must fail as an argument error or unrecognized nested subcommand and must point the user back to `wavepeek docs --help`.

`wavepeek docs topics` must include `commands/skill`, and `wavepeek docs show commands/skill` must explain the new entrypoint. `wavepeek docs show commands/docs` must no longer mention packaged-skill printing.

`wavepeek schema` output must remain byte-for-byte identical to `schema/wavepeek.json`, proving that the new top-level `skill` command did not accidentally alter JSON contracts.

`wavepeek docs export <out-dir>` must still exclude the packaged skill file and must still export only public topic Markdown plus `manifest.json`.

## Idempotence and Recovery

The implementation should proceed in small slices so the branch never sits in a half-migrated state for long. Update tests first, then CLI wiring, then docs collateral, then full validation.

If the docs runtime stops loading after adding `commands/skill.md`, first verify the topic front matter and the required H1 match. The embedded docs loader rejects missing or mismatched topic metadata, so this is the safest first recovery check.

If `wavepeek docs skill` still appears anywhere after the main code changes, rerun the collateral search before continuing:

    rg -n "docs skill|packaged skill|agent skill|wavepeek skill|skill Markdown" README.md CHANGELOG.md docs src tests schema -g '!docs/exec-plans/**'

Treat `docs/exec-plans/completed/` as historical context, not a blocker for live-surface cleanup.

## Artifacts and Notes

Use this repository search as the starting map for live collateral. It was gathered during plan authoring and should be refreshed before implementation changes are declared complete.

    rg -n "docs skill|packaged skill|agent skill|wavepeek skill|skill Markdown" README.md CHANGELOG.md docs src tests schema -g '!docs/exec-plans/**'

At plan-authoring time, the expected live hits included at least these files:

    README.md
    CHANGELOG.md
    docs/DEVELOPMENT.md
    docs/skills/AGENTS.md
    docs/public/commands/docs.md
    docs/public/commands/overview.md
    docs/public/intro.md
    src/cli/docs.rs
    src/engine/docs.rs
    src/engine/mod.rs
    tests/docs_cli.rs
    tests/cli_contract.rs

The same search should not force changes in these no-change guardrails unless implementation introduces them by mistake:

    schema/wavepeek.json
    tests/schema_cli.rs
    docs/ROADMAP.md
    docs/RELEASE.md
    docs/BACKLOG.md

## Interfaces and Dependencies

In `src/cli/skill.rs`, define:

    #[derive(Debug, clap::Args, Default)]
    pub struct SkillArgs {}

and expose one top-level helper subcommand with help text equivalent to:

    Print the packaged agent skill Markdown for wavepeek.

In `src/engine/skill.rs`, define a public runner with this shape:

    pub fn run(args: crate::cli::skill::SkillArgs) -> Result<crate::engine::CommandResult, crate::error::WavepeekError>

That runner should return `CommandData::Text(crate::docs::packaged_skill_markdown().to_string())` through the normal human text rendering path.

In `src/engine/mod.rs`, add a new top-level command variant and command name:

    Command::Skill(crate::cli::skill::SkillArgs)
    CommandName::Skill => "skill"

and remove the nested `DocsSkill` command name.

In `src/cli/docs.rs`, remove:

    Skill(DocsSkillArgs)
    pub struct DocsSkillArgs {}

so the docs command family owns only topic-oriented subcommands.

In the test layer, add a dedicated integration file `tests/skill_cli.rs` that uses the existing `tests/common` helpers and the canonical packaged-skill source at `docs/skills/wavepeek.md`.

Revision Note: 2026-05-12 / Pi - Initial active ExecPlan created for promoting the packaged skill from `wavepeek docs skill` to top-level `wavepeek skill`, with explicit collateral mapping and no implementation work performed yet.
Revision Note: 2026-05-12 / Pi - Updated after plan review fallback in the main session to record the initial plan commit, note the failed subagent review attempt, add the explicit old-path rejection test, include `docs/public/commands/help.md` and `docs/public/commands/AGENTS.md` in collateral scope, and clarify that `CHANGELOG.md` must revise existing unreleased wording instead of only appending a new line.
