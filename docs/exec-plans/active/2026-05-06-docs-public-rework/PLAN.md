# Rework installed and repository documentation into a public docs corpus

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

Note that this document must be maintained in accordance with the `exec-plan` skill.

## Purpose / Big Picture

Today the repository has two overlapping documentation layers. The installed command `wavepeek docs` embeds Markdown from `docs/cli/topics/`, while the richer semantic contracts live under `docs/design/contracts/` and are treated as internal design material. This makes important user-facing behavior, such as time normalization, scope/name resolution, JSON output, errors, and the expression language, harder for installed users and agents to discover.

After this change, the installed documentation corpus will be the canonical user-facing documentation. Users will be able to run `wavepeek docs topics`, see public topics grouped under `intro`, `commands`, `workflows`, `troubleshooting`, and `reference`, and open detailed reference topics such as `wavepeek docs show reference/command-model`, `wavepeek docs show reference/machine-output`, and `wavepeek docs show reference/expression-language`. Contributors will still have internal documentation, but it will be clearly separated into files such as `docs/ARCHITECTURE.md`, `docs/DEVELOPMENT.md`, and execution plans rather than mixed into a directory named `design`.

The visible proof is straightforward. Running `wavepeek docs topics` after implementation should list the new public topic IDs, including all top-level commands and reference topics. Running `wavepeek docs show intro` should show a user-facing introduction with a public topic map and help guidance. Running `wavepeek docs show reference/expression-language` should show the expression-language contract that was previously under `docs/design/contracts/expression_lang.md`. Running `wavepeek docs skill` should still print the packaged agent skill, but that skill should be a short router that points agents into `wavepeek help` and `wavepeek docs` instead of copying the detailed reference content.

## Non-Goals

This plan does not change waveform query behavior, JSON data shapes, command names, command flags, or schema semantics. It changes where documentation lives, how embedded documentation topics are organized, and how prose points users to the existing code-first help and schema surfaces.

This plan does not add nested topic files for `docs` subcommands such as `commands/docs/show` or `commands/docs/search`. The `docs` command family will be covered by one top-level topic, `commands/docs`, until there is enough real content to justify finer-grained topics.

This plan does not make command-topic files into a second flag reference. Exact command names, flags, defaults, requiredness, and help examples remain authoritative in `src/cli/`, `wavepeek -h`, `wavepeek --help`, `wavepeek help <command-path...>`, and `wavepeek docs --help`. Command topics may explain intent, caveats, and workflows, but the initial command topics will mostly be useful placeholders that point to the appropriate help command.

This plan does not update historical completed execution plans unless a future maintainer explicitly chooses to add a historical note. Completed plans under `docs/exec-plans/completed/` may continue to mention paths that were canonical at the time they were written. Live repository docs, source breadcrumbs, tests, and shipped docs must be updated.

## Progress

- [x] (2026-05-06 21:09Z) Read the current documentation map, embedded docs implementation, docs tests, schema artifact, README, changelog, and the scratch TODO at `tmp/docs_todo.md` to capture the desired reorganization.
- [x] (2026-05-06 21:09Z) Resolved the core source-of-truth decision for this plan: `docs/public/` will become the normal embedded topic corpus, `docs/skills/wavepeek.md` will become the packaged skill source, and internal contributor material will live outside the public corpus.
- [x] (2026-05-06 21:09Z) Created this active ExecPlan at `docs/exec-plans/active/2026-05-06-docs-public-rework/PLAN.md` as the first implementation artifact.
- [ ] Commit the initial ExecPlan draft as an intermediate checkpoint.
- [ ] Run focused read-only subagent reviews of this plan before implementation, with at least docs, code/tests, and architecture/link-integrity lanes.
- [ ] Revise this plan from review findings and commit the reviewed plan as a second checkpoint.
- [ ] Implement the documentation tree migration and embedded-doc runtime path changes.
- [ ] Update live links, breadcrumbs, tests, and changelog.
- [ ] Run full validation and a final multi-lane subagent review on the implementation diff.
- [ ] Move this plan to `docs/exec-plans/completed/2026-05-06-docs-public-rework/PLAN.md` after implementation, validation, review, and reporting are complete.

## Surprises & Discoveries

- Observation: the current embedded docs runtime already supports topic files at any relative path under the embedded root, as long as each file path matches the topic ID plus `.md`.
  Evidence: `src/docs/mod.rs` embeds `docs/cli/topics`, skips only `AGENTS.md`, and validates each topic with `canonical_source_relpath(&metadata.id)`, which returns `{id}.md`. This means changing the embedded root to `docs/public` can support `docs/public/reference/command-model.md` without changing the topic path validator.

- Observation: the current runtime has a hard-coded logical section order that includes `concepts` and does not include `reference`.
  Evidence: `topic_section_rank()` in `src/docs/mod.rs` orders `intro`, `concepts`, `commands`, `workflows`, and `troubleshooting`, then gives unknown sections the fallback rank. This must change when the `concepts` section is removed and `reference` is added.

- Observation: the current test suite assumes exactly nine topics and points directly at `docs/cli`.
  Evidence: `tests/docs_cli.rs` defines `TOPIC_IDS` with `concepts/time` and `concepts/selectors`, and its `docs_root()` helper returns `docs/cli`. These tests will be updated to enforce the new public corpus instead of the old corpus.

- Observation: the current packaged skill source is embedded from `docs/cli/wavepeek-skill.md` and contains detailed command recipes and rules that duplicate material better suited for docs topics.
  Evidence: `src/docs/mod.rs` uses `include_str!(... "/docs/cli/wavepeek-skill.md")`, and `docs/cli/wavepeek-skill.md` includes long recipe, rules, and failure-recovery sections.

## Decision Log

- Decision: rename the packaged user-facing topic corpus from `docs/cli/topics/` to `docs/public/`, not to `docs/public/topics/`.
  Rationale: the target user-facing paths discussed for the rework are `docs/public/intro.md`, `docs/public/commands/...`, and `docs/public/reference/...`. Keeping topic files directly under `docs/public/` makes repository paths line up with installed topic IDs such as `intro`, `commands/change`, and `reference/command-model`.
  Date/Author: 2026-05-06 / Pi

- Decision: remove the current `concepts` topic section and replace it with `reference` for normative semantic documents.
  Rationale: the existing `concepts/time.md` and `concepts/selectors.md` are short summaries of `docs/design/contracts/command_model.md`. Moving the full semantic contracts into `reference` avoids stale duplicate summaries and gives users access to the real behavior contract through `wavepeek docs`.
  Date/Author: 2026-05-06 / Pi

- Decision: move `docs/design/contracts/command_model.md`, `docs/design/contracts/machine_output.md`, and `docs/design/contracts/expression_lang.md` into `docs/public/reference/` with hyphenated file names.
  Rationale: these documents describe user-visible behavior and machine-facing guarantees, so they belong in the installed public docs corpus. Hyphenated names produce friendlier topic IDs: `reference/command-model`, `reference/machine-output`, and `reference/expression-language`.
  Date/Author: 2026-05-06 / Pi

- Decision: do not move `docs/design/contracts/documentation_surface.md` as one public reference topic.
  Rationale: that document mixes user-facing behavior with maintainer source-of-truth rules. The user-facing parts should be split into `commands/help`, `commands/docs`, and possibly `intro`, while maintainer-only rules should live in contributor documentation such as `docs/DEVELOPMENT.md` or local `AGENTS.md` breadcrumbs.
  Date/Author: 2026-05-06 / Pi

- Decision: move internal engineering architecture from `docs/design/architecture.md` to `docs/ARCHITECTURE.md`.
  Rationale: this file documents implementation layers, dependencies, module boundaries, and testing strategy. It is useful to contributors but is not part of the installed user-facing docs corpus.
  Date/Author: 2026-05-06 / Pi

- Decision: convert `docs/design/reference/cli.md` into `docs/public/commands/overview.md` rather than `docs/public/reference/cli.md`.
  Rationale: the existing file is an operator guide for choosing command families, not an exact CLI reference. Putting it under `commands/overview` keeps `reference` reserved for stable semantics and contracts.
  Date/Author: 2026-05-06 / Pi

- Decision: keep the packaged agent skill out of `docs/public/` and move it to `docs/skills/wavepeek.md`.
  Rationale: the skill has a different front matter shape (`name` and `description`) from normal docs topics (`id`, `title`, `summary`, `section`). Keeping it outside `docs/public/` prevents the topic loader from treating it as a normal topic while preserving `wavepeek docs skill` as the command that prints it verbatim.
  Date/Author: 2026-05-06 / Pi

## Outcomes & Retrospective

No implementation outcome exists yet. The current outcome is a reviewed planning target: move public user-facing docs into `docs/public/`, move internal architecture to `docs/ARCHITECTURE.md`, move the packaged skill to `docs/skills/wavepeek.md`, and update code/tests/links so the installed `wavepeek docs` surface exposes the richer public corpus without duplicating it.

## Context and Orientation

The repository root is `/workspaces/docs-rework`. The project is a Rust command-line tool named `wavepeek`. It already has an installed documentation command family, `wavepeek docs`, implemented by `src/cli/docs.rs`, `src/engine/docs.rs`, and `src/docs/mod.rs`. The runtime currently embeds Markdown topics from `docs/cli/topics/` and embeds the packaged agent skill from `docs/cli/wavepeek-skill.md`.

A topic is one Markdown file with YAML front matter. The current topic front matter requires `id`, `title`, `summary`, and `section`, with optional `see_also`. The topic body must start with an H1 heading that matches `title`. The topic ID is the stable slash-separated public name shown to users, such as `commands/change`; it is not the raw filesystem path with `.md`, although the loader currently requires the file path under the embedded root to match the ID plus `.md`.

The current public-but-thin embedded topic corpus lives under `docs/cli/topics/`. It contains `intro`, two short `concepts` topics, four command topics, one workflow, and one troubleshooting topic. The richer semantic contracts live under `docs/design/contracts/`. Those contracts are currently not embedded into `wavepeek docs`, even though they describe user-visible behavior.

The current internal architecture document is `docs/design/architecture.md`. It explains Rust dependencies, execution layers, module structure, expression-engine implementation, error handling, the `change` command execution architecture, and testing strategy. It should remain available to contributors but should not be treated as an installed user topic.

The current command-family guide is `docs/design/reference/cli.md`. It is explicitly derived documentation and says that exact command syntax, defaults, requiredness, and examples are authoritative in generated help and `src/cli/`. It should become a user-facing overview topic under `commands/overview`, not a reference contract.

The current documentation-surface contract is `docs/design/contracts/documentation_surface.md`. It defines layered help, the `docs` command family, topic metadata, docs search, docs export, and ownership split. Its user-facing content belongs in public command/help topics, while its maintainer-only source-of-truth rules belong in contributor docs or breadcrumbs.

The important implementation files are:

- `src/docs/mod.rs`, which embeds and validates Markdown topics, lists topics, searches topics, suggests close matches, exports topics, and returns the packaged skill Markdown.
- `src/cli/docs.rs`, which defines the `wavepeek docs` command family and its arguments.
- `src/engine/docs.rs`, which implements docs command behavior and calls into `src/docs/mod.rs`.
- `src/output.rs`, which renders human output and JSON envelopes.
- `schema/wavepeek.json`, which is the canonical schema returned by `wavepeek schema` and already includes `docs topics` and `docs search` JSON branches.
- `tests/docs_cli.rs`, which checks the embedded docs behavior and currently assumes the old `docs/cli` paths and old topic list.
- `tests/cli_contract.rs`, which checks help behavior and visible command help.
- `tests/schema_cli.rs`, which checks schema behavior.

The important live documentation files that contain links to current paths are:

- `AGENTS.md`, `docs/AGENTS.md`, `src/AGENTS.md`, `tests/AGENTS.md`, and `schema/AGENTS.md`.
- `README.md`, especially the Agentic Flows section that currently points at `docs/cli/wavepeek-skill.md`.
- `CHANGELOG.md`, especially the unreleased entries that mention `docs/design/index.md` and `docs/design/contracts/expression_lang.md`.
- `docs/DEVELOPMENT.md`, which contains CLI help and docs maintenance guidance.
- `docs/BACKLOG.md`, which contains references to `docs/design/` and related contracts.

## Open Questions

No blocking product decision remains for starting implementation. The plan chooses `reference` rather than `concepts` for semantic contracts, keeps command topics as placeholders until they are useful, and keeps the skill outside the topic corpus.

One implementation detail must be verified during the test update: whether `docs topics` should list `reference` before or after `commands`. This plan chooses the order `intro`, `commands`, `workflows`, `troubleshooting`, `reference`, because users should see practical docs first and normative reference material last. If review finds that reference should appear earlier, update the Decision Log, `topic_section_rank()`, `docs/design`-to-public migration text, and tests together.

Another detail is whether to leave tiny compatibility stubs under `docs/design/` after moving all canonical content out. The default implementation in this plan is to remove `docs/design/` from live source-of-truth docs and update all live links directly. If a reviewer identifies a strong compatibility reason to keep a stub such as `docs/design/index.md`, it must be clearly marked as a non-canonical pointer and must not duplicate public docs content.

## Plan of Work

Milestone 1 commits and reviews this plan. The plan itself is part of the work because this refactor touches embedded assets, tests, schemas, breadcrumbs, README, and many links. First commit this plan as an intermediate checkpoint. Then run read-only subagent reviews with focused lanes for docs/link integrity, code/tests/runtime behavior, and architecture/source-of-truth boundaries. Apply review feedback by editing this plan, updating the living sections, and committing the reviewed plan. This milestone is complete when the plan has at least one clean review cycle recorded in `Progress`, `Surprises & Discoveries`, and `Decision Log`.

Milestone 2 builds the new public documentation tree while keeping tests and code temporarily able to fail. Create `docs/public/` as the new embedded topic root. Move and rewrite `docs/design/index.md` into `docs/public/intro.md` as a user-facing entrypoint. The new intro should explain what wavepeek is, scope, principles, public topic groups, and getting help through `wavepeek -h`, `wavepeek --help`, `wavepeek help <command-path...>`, `wavepeek docs --help`, `wavepeek docs topics`, and optionally `wavepeek docs search <query>`. Move the semantic contracts into `docs/public/reference/command-model.md`, `docs/public/reference/machine-output.md`, and `docs/public/reference/expression-language.md`, adjusting headings, links, and references to say that they are public reference topics while exact flags remain code-first. Convert `docs/design/reference/cli.md` into `docs/public/commands/overview.md` as a guide for choosing command families.

Milestone 3 replaces the old command and concept topics. Remove the old `concepts/time` and `concepts/selectors` topics entirely. Create one topic file per top-level command under `docs/public/commands/`: `help.md`, `docs.md`, `schema.md`, `info.md`, `scope.md`, `signal.md`, `value.md`, `change.md`, and `property.md`, plus the `overview.md` from Milestone 2. For now, each per-command topic should have valid topic front matter, an H1, a clear `Detailed guide coming soon` notice, and a pointer to the authoritative help command. For example, `commands/info` should tell the reader to run `wavepeek help info`; `commands/docs` should tell the reader to run `wavepeek docs --help`; and `commands/help` should explain the layered help model. Do not create separate nested topics for `docs` subcommands. Update workflow and troubleshooting topics so their `see_also` links no longer point to removed `concepts` topics; use `reference/command-model` and command topics instead.

Milestone 4 moves internal and skill documentation to their new homes. Move `docs/design/architecture.md` to `docs/ARCHITECTURE.md` and update its internal links from old contract paths to public reference topic paths or other internal paths as appropriate. Split `docs/design/contracts/documentation_surface.md`: move user-facing behavior into `docs/public/commands/help.md`, `docs/public/commands/docs.md`, and `docs/public/intro.md`; move maintainer-only source-of-truth rules into `docs/DEVELOPMENT.md` and local `AGENTS.md` breadcrumbs. Move `docs/cli/wavepeek-skill.md` to `docs/skills/wavepeek.md` and rewrite it as a short agent router. The skill should preserve critical safety rules, especially that waveform files should be treated as CLI inputs and `.fst` files must not be read directly, but it should point to `wavepeek help`, `wavepeek docs topics`, `reference/command-model`, `reference/machine-output`, and `reference/expression-language` instead of copying detailed command recipes and semantic rules.

Milestone 5 updates code and tests to embed the new tree. In `src/docs/mod.rs`, change the topic embed root from `docs/cli/topics` to `docs/public`, and change the skill include path from `docs/cli/wavepeek-skill.md` to `docs/skills/wavepeek.md`. Update `topic_section_rank()` so the logical order is `intro`, `commands`, `workflows`, `troubleshooting`, and `reference`; remove the special `concepts` rank. If any unit tests inside `src/docs/mod.rs` assert old topic counts, paths, or topic IDs, update them to the new topic set. In `tests/docs_cli.rs`, change the canonical docs root helper from `docs/cli` to `docs/public`, add a separate canonical skill helper for `docs/skills/wavepeek.md`, update `TOPIC_IDS`, update topic summary expectations, and keep tests that assert export preserves front matter and excludes the skill. In `tests/cli_contract.rs`, update any strings that mention concepts or old docs descriptions if the public docs wording changes. `schema/wavepeek.json` should not need a semantic schema change, because `docs topics` and `docs search` already expose generic topic metadata, but run schema checks to prove it.

Milestone 6 updates live breadcrumbs and non-embedded docs. Update `docs/AGENTS.md` so canonical user docs point to `public/intro.md`, embedded public docs point to `public/AGENTS.md`, the skill path points to `skills/wavepeek.md`, and internal architecture points to `ARCHITECTURE.md`. Remove child-map references to deleted `docs/cli/` and `docs/design/`. Add or update concise `AGENTS.md` files for durable documentation nodes where they materially improve navigation, especially `docs/public/AGENTS.md`, `docs/public/commands/AGENTS.md`, `docs/public/reference/AGENTS.md`, and `docs/skills/AGENTS.md`. Update `src/AGENTS.md`, `tests/AGENTS.md`, and `schema/AGENTS.md` so they point to `docs/public/reference/...` and `docs/public/` instead of old design or cli paths. Update `README.md` so the Agentic Flows section points to `docs/skills/wavepeek.md`, and update any instructions that describe embedded docs as `concepts`-based. Update `CHANGELOG.md` with an Unreleased Changed entry for the public docs corpus reorganization. Update `docs/BACKLOG.md` and `docs/DEVELOPMENT.md` to remove live references to `docs/design/` and `docs/cli/` as canonical locations.

Milestone 7 removes retired paths and validates the repository. Delete `docs/cli/` and `docs/design/` after their content has been moved, unless a reviewed decision adds tiny compatibility stubs. Run repository searches to catch stale live links. Do not treat references inside `docs/exec-plans/completed/` as blockers unless they are referenced by live docs; those files are historical. Run targeted tests first, then full gates. Commit implementation in atomic slices, for example public docs tree, code/tests path switch, live link cleanup, and validation fixes. After the implementation commits, run the mandatory review workflow again with read-only subagents over the implementation diff, fix findings in follow-up commits, and close the plan only after the final review pass is clean.

### Concrete Steps

Run all commands from `/workspaces/docs-rework`.

1. Commit the initial plan checkpoint.

       git status --short
       git add docs/exec-plans/active/2026-05-06-docs-public-rework/PLAN.md
       git commit -m "docs: plan public documentation rework"

   Expected result: `git status --short` shows only unrelated pre-existing work, if any, and the plan commit is recorded. Do not use `--no-verify`.

2. Run review subagents on the plan before implementation. Use read-only tools only.

       tmux -V
       command -v pi
       ~/.pi/agent/skills/subagents/scripts/subagent.py spawn --name docs-public-plan-docs --cwd /workspaces/docs-rework --tools read,bash --prompt '<docs lane prompt>'
       ~/.pi/agent/skills/subagents/scripts/subagent.py spawn --name docs-public-plan-code --cwd /workspaces/docs-rework --tools read,bash --prompt '<code/tests lane prompt>'
       ~/.pi/agent/skills/subagents/scripts/subagent.py spawn --name docs-public-plan-arch --cwd /workspaces/docs-rework --tools read,bash --prompt '<architecture/link lane prompt>'

   The prompts must tell reviewers not to edit files, to inspect `docs/exec-plans/active/2026-05-06-docs-public-rework/PLAN.md`, and to return concrete findings with file and line references. Capture their logs with `subagent.py tail <printed-name>` or `subagent.py capture <printed-name>`. Update this plan from findings and commit the reviewed revision.

3. Create the new public docs tree with copy-then-trim movement to avoid information loss.

       mkdir -p docs/public/commands docs/public/reference docs/public/workflows docs/public/troubleshooting docs/skills
       cp docs/design/contracts/command_model.md docs/public/reference/command-model.md
       cp docs/design/contracts/machine_output.md docs/public/reference/machine-output.md
       cp docs/design/contracts/expression_lang.md docs/public/reference/expression-language.md
       cp docs/design/reference/cli.md docs/public/commands/overview.md
       cp docs/cli/topics/workflows/find-first-change.md docs/public/workflows/find-first-change.md
       cp docs/cli/topics/troubleshooting/empty-results.md docs/public/troubleshooting/empty-results.md
       cp docs/design/architecture.md docs/ARCHITECTURE.md
       cp docs/cli/wavepeek-skill.md docs/skills/wavepeek.md

   Then edit the copied files into their final form. Do not delete the old source files until the code and tests have been switched and validated.

4. Create or rewrite the public command topics. Each file must start with YAML front matter and then an H1 matching `title`.

   The minimal shape for `docs/public/commands/info.md` is:

       ---
       id: commands/info
       title: Info command
       summary: Show waveform metadata.
       section: commands
       see_also:
         - commands/overview
         - reference/command-model
       ---
       # Info command

       > Detailed guide coming soon.

       For exact syntax and flags, run:

           wavepeek help info

   Use the same pattern for `schema`, `scope`, `signal`, `value`, `change`, and `property`. For `commands/docs`, point to `wavepeek docs --help`; for `commands/help`, explain `wavepeek -h`, `wavepeek --help`, and `wavepeek help <command-path...>`; for `commands/change` and `commands/property`, include `reference/expression-language` in `see_also`.

5. Switch the embedded runtime paths.

   In `src/docs/mod.rs`, change:

       static TOPICS_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/docs/cli/topics");

   to:

       static TOPICS_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/docs/public");

   Change the skill include path from `/docs/cli/wavepeek-skill.md` to `/docs/skills/wavepeek.md`. Update `topic_section_rank()` to order `intro`, `commands`, `workflows`, `troubleshooting`, then `reference`.

6. Update tests and run targeted checks.

       cargo test docs:: --lib
       cargo test --test docs_cli
       cargo test --test cli_contract
       cargo test --test schema_cli

   Expected result after code and test updates: all four targeted commands pass. Before the path switch is complete, failures should be limited to expected missing-topic, stale-path, or stale-string assertions.

7. Search for stale live references and fix them.

       rg -n "docs/cli|docs/design|design/contracts|design/index|design/architecture|reference/cli|concepts/|wavepeek-skill" AGENTS.md README.md CHANGELOG.md docs src tests schema scripts Cargo.toml -g '!docs/exec-plans/completed/**' -g '!*.fst'

   Expected result after cleanup: no stale live references remain except intentional historical notes or freshly updated compatibility references if a compatibility-stub decision was made and recorded.

8. Run full validation.

       make check
       make ci

   Expected result: both commands pass inside the devcontainer or CI image. If the environment is not inside the container and `make` refuses to run, use targeted Cargo tests for local iteration but record that full gates still need container execution before handoff.

9. Run final implementation review and close the plan.

   Spawn read-only subagents for docs, code/tests, and architecture/link-integrity lanes over the implementation diff. Fix substantive findings in follow-up commits. Update this plan's `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective`. Move the plan directory from `docs/exec-plans/active/2026-05-06-docs-public-rework/` to `docs/exec-plans/completed/2026-05-06-docs-public-rework/` only after implementation, validation, review, and final reporting are complete.

### Validation and Acceptance

The implementation is acceptable when the installed docs surface demonstrates the new organization:

- `cargo run -- docs topics` lists topic IDs in this logical section order: `intro`, all `commands/*` topics, `workflows/*`, `troubleshooting/*`, and `reference/*`. It must not list `concepts/time` or `concepts/selectors`.
- `cargo run -- docs show intro` prints a user-facing introduction that explains wavepeek, scope, principles, public topic groups, and getting help through short help, long help, the `help` command, and the `docs` command.
- `cargo run -- docs show reference/command-model` prints the command semantics reference moved from the old design contract.
- `cargo run -- docs show reference/machine-output` prints the machine output reference moved from the old design contract.
- `cargo run -- docs show reference/expression-language` prints the expression-language reference moved from the old design contract.
- `cargo run -- docs show commands/info` prints a valid placeholder with `Detailed guide coming soon` and `wavepeek help info`.
- `cargo run -- docs skill` prints the new short skill from `docs/skills/wavepeek.md`; it must preserve waveform safety guidance and route agents to help/docs rather than copying long reference content.
- `cargo run -- docs export /tmp/wavepeek-docs-public-check` exports the topic corpus from `docs/public/`, preserves YAML front matter, writes `manifest.json`, and does not export `docs/skills/wavepeek.md`.
- `cargo run -- docs topics --json` and `cargo run -- docs search expression --json` still emit the standard JSON envelope with `command` values `docs topics` and `docs search`.
- `cargo run -- schema` still emits the canonical schema and `cargo test --test schema_cli` passes, proving JSON schema behavior did not drift.

The repository is acceptable when live links and breadcrumbs no longer point to deleted canonical paths. Run the stale-reference `rg` command above and inspect any remaining hits. Completed execution plans may remain historical and are not blockers.

The final handoff is acceptable only after `make check` and `make ci` pass in the intended container environment, or after the inability to run them is explicitly recorded with the reason and all targeted Cargo tests are green.

### Idempotence and Recovery

The migration should use copy-then-trim rather than delete-then-create. Copying source files into `docs/public/`, `docs/ARCHITECTURE.md`, and `docs/skills/` is safe to repeat if the destination has not yet been manually edited. After manual edits begin, use `git diff` before repeating copy commands so edited work is not overwritten.

If the embedded topic loader fails after the path switch, run `cargo test docs:: --lib` first. Loader failures usually name the exact Markdown file, topic ID, missing H1, invalid front matter, duplicate ID, bad `see_also`, or path mismatch. Fix the named topic before continuing to broader integration tests.

If docs export tests fail, check whether `source_relpath` still matches the topic ID and whether the test helper points to `docs/public` rather than `docs/cli`. The export contract should continue to export only topic files and should continue to exclude the packaged skill.

If a stale-link search finds many historical hits under `docs/exec-plans/completed/`, rerun the search with the completed-plans exclusion shown in the concrete steps. Historical plans are allowed to preserve old paths. Live docs and source/test breadcrumbs are not.

If full `make` gates fail because the current environment is not the devcontainer or CI image, do not bypass the gate. Record the failure reason in `Surprises & Discoveries`, run the targeted Cargo tests locally, and state that full validation remains pending until the correct container environment is available.

### Artifacts and Notes

Current high-risk stale-reference roots identified before implementation include these live files:

       README.md
       CHANGELOG.md
       docs/AGENTS.md
       docs/BACKLOG.md
       docs/DEVELOPMENT.md
       src/AGENTS.md
       tests/AGENTS.md
       schema/AGENTS.md
       src/docs/mod.rs
       tests/docs_cli.rs
       docs/design/contracts/documentation_surface.md
       docs/design/index.md
       docs/design/reference/cli.md
       docs/cli/AGENTS.md
       docs/cli/topics/AGENTS.md

The current old topic inventory is:

       intro
       concepts/selectors
       concepts/time
       commands/change
       commands/docs
       commands/help
       commands/property
       workflows/find-first-change
       troubleshooting/empty-results

The target topic inventory must include at least:

       intro
       commands/overview
       commands/help
       commands/docs
       commands/schema
       commands/info
       commands/scope
       commands/signal
       commands/value
       commands/change
       commands/property
       workflows/find-first-change
       troubleshooting/empty-results
       reference/command-model
       reference/machine-output
       reference/expression-language

Additional public topics may be added only if they have real content and stable IDs. Do not add empty nested docs-subcommand topics during this plan.

### Interfaces and Dependencies

The implementation should keep using the existing dependencies already present in `Cargo.toml`: `include_dir` for embedding Markdown directories, `serde` and `serde_json` for JSON output, `serde_yaml` for topic front matter, and `regex` for front-matter parsing and search helpers. No new dependency is required for this reorganization.

At the end of implementation, `src/docs/mod.rs` must still expose these public functions and types used by the engine and tests:

       pub fn lookup_topic(id: &str) -> Result<Option<&'static TopicRecord>, WavepeekError>;
       pub fn list_topics() -> Result<Vec<TopicSummary>, WavepeekError>;
       pub fn normalize_search_query(query: &str) -> Result<String, WavepeekError>;
       pub fn search_topics(query: &str, full_text: bool) -> Result<Vec<SearchMatch>, WavepeekError>;
       pub fn suggest_topics(input: &str, limit: usize) -> Vec<TopicSummary>;
       pub fn export_catalog(out_dir: &Path, force: bool) -> Result<ExportSummary, WavepeekError>;
       pub fn packaged_skill_markdown() -> &'static str;

The bodies of those functions should continue to source metadata from embedded Markdown files. Do not replace topic metadata with hand-maintained Rust arrays.

`src/engine/docs.rs` should not need a new public interface. It should continue to implement the existing `docs` subcommands by delegating to `src/docs/mod.rs`. The behavior of `docs show`, `docs topics`, `docs search`, `docs export`, and `docs skill` should remain the same except for the new corpus paths, topic IDs, and skill content.

`schema/wavepeek.json` should remain the authority for the precise JSON schema. Because the docs JSON payload shape is generic topic metadata and search matches, the new topic IDs and sections should not require a schema shape change unless the implementation changes serialized fields. If schema bytes do change, use the repository schema workflow and validate with `make check-schema` or the relevant `make` gate.

Revision Note: 2026-05-06 / Pi - Initial active ExecPlan created from the scratch TODO and discussion decisions. It records the target public docs corpus, reference-topic migration, command-topic placeholder strategy, skill relocation, internal architecture move, code/test/link update scope, validation gates, and mandatory subagent review workflow.
