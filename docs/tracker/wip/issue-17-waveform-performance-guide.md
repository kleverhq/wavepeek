# Add public waveform performance guide

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This document must be maintained in accordance with the `exec-plan` skill.

## Purpose / Big Picture

Users inspecting RTL waveform dumps need a short offline guide that explains why `wavepeek` can be fast for one format and slow for another. After this change, a user can run `wavepeek docs show reference/waveform-performance` and read practical guidance for VCD, FST, and FSDB performance without opening internal benchmark artifacts. The guide should help users decide when to query an existing dump directly, when FST is a better repeated-inspection format, and why one-shot FSDB commands can have a high setup cost.

## Non-Goals

This plan does not change waveform parsing, command execution, caching, or FSDB native code. It does not add or describe a future batch, session, daemon, or persistent-process workflow. It does not publish the full benchmark logs under `tmp/fsdb-eval/`; those remain local investigation artifacts, not public documentation.

## Progress

- [x] (2026-06-23T11:25Z) Created branch `issue-17-waveform-performance-guide` from `main`.
- [x] (2026-06-23T11:25Z) Reviewed issue #17, local benchmark notes in `tmp/fsdb-eval/`, public docs guidance, and docs tests.
- [x] (2026-06-23T11:25Z) Drafted this execution plan.
- [x] (2026-06-23T11:28Z) Ran read-only plan review and added the missing milestone section requested by the reviewer.
- [ ] Commit the reviewed execution plan.
- [x] (2026-06-23T11:35Z) Added `docs/public/reference/waveform-performance.md`, discoverability links, and docs topic tests.
- [x] (2026-06-23T11:36Z) Ran targeted docs validation with `cargo test -q --test docs_cli` and manual `cargo run -q -- docs ...` checks.
- [x] (2026-06-23T11:42Z) Ran read-only implementation review; fixed the stale `src/docs/mod.rs` topic-count assertion it found.
- [x] (2026-06-23T11:45Z) Re-ran targeted tests and `just check` after the review fix; all passed.
- [x] (2026-06-23T11:47Z) Ran a fresh final read-only control pass; it returned no substantive findings.
- [ ] Commit implementation changes.
- [ ] Remove branch-local WIP plan before final handoff unless the maintainer asks to keep it.
- [ ] Push the branch to `origin` and open a pull request linked to issue #17.

## Surprises & Discoveries

- Observation: The existing docs topic order is asserted explicitly in `tests/docs_cli.rs` through `TOPIC_IDS`.
  Evidence: `docs_topics_use_logical_section_order`, `docs_topics_json_uses_standard_envelope`, and `docs_export_manifest_matches_contract` compare runtime topic IDs against that constant.

- Observation: Public docs are embedded from `docs/public/` at compile time.
  Evidence: `src/docs/mod.rs` defines `static TOPICS_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/docs/public");`.

- Observation: `cargo test -q docs_cli` is not the correct way to run the `tests/docs_cli.rs` integration test file; it filters test names and ran zero tests.
  Evidence: the command reported many harnesses with `0 passed` and filtered tests. The correct targeted command is `cargo test -q --test docs_cli`, which ran 24 tests and passed.

- Observation: Adding a docs topic also affects a runtime unit test in `src/docs/mod.rs` that checks exported topic count.
  Evidence: implementation review found `assert_eq!(summary.topics.len(), 21);`; it was updated to `22`, and `cargo test -q export_writes_manifest_and_topics_without_skill_file` passed.

## Decision Log

- Decision: Add the guide as `docs/public/reference/waveform-performance.md` with id `reference/waveform-performance`.
  Rationale: The guide is cross-cutting format behavior rather than a single command recipe. It is not an exact flag reference, but it belongs near other stable explanatory references and can clearly state that performance numbers are rules of thumb rather than guarantees.
  Date/Author: 2026-06-23 / Grin

- Decision: Do not mention a future batch, session, daemon, or persistent-process workflow in the public guide.
  Rationale: The maintainer explicitly said not to discuss future batch/session workflow. Public docs should describe current behavior and durable rationale, not speculative planning.
  Date/Author: 2026-06-23 / Grin

- Decision: Use only a short generic benchmark table derived from `tmp/fsdb-eval/`.
  Rationale: Issue #17 asks for rough order-of-magnitude guidance while keeping detailed benchmark artifacts out of public docs. A compact table with file size, representative command time, and peak memory is enough to explain expectations.
  Date/Author: 2026-06-23 / Grin

## Outcomes & Retrospective

The public docs implementation is complete in the working tree. The new topic is discoverable through `docs topics`, `docs show`, and `docs search`; targeted tests, `just check`, implementation review, and final control review passed. Remaining work is commit cleanup, push, and pull request creation.

## Context and Orientation

`wavepeek` is a Rust command-line tool for deterministic waveform inspection. A waveform dump is a file containing signal values over time. The relevant formats are VCD, FST, and FSDB. VCD means Value Change Dump, a textual waveform format that can become very large. FST means Fast Signal Trace, a compact indexed format commonly produced by GTKWave tooling. FSDB means Fast Signal Database, a proprietary Synopsys format read by `wavepeek` only when the binary is built with the optional `fsdb` Cargo feature and linked against the Verdi FSDB Reader SDK.

Public user documentation lives under `docs/public/`. Each topic is a Markdown file with YAML front matter containing `id`, `title`, `description`, `section`, and optional `see_also`. The path must match the topic id plus `.md`; for example, topic id `reference/command-model` lives at `docs/public/reference/command-model.md`. The body starts with an H1 that exactly matches the front matter title.

The embedded docs runtime in `src/docs/mod.rs` packages `docs/public/` into the binary. The docs CLI tests in `tests/docs_cli.rs` assert topic ordering and export behavior. Adding a topic therefore requires updating the `TOPIC_IDS` constant in that test file.

The local investigation artifacts for issue #17 live under `tmp/fsdb-eval/`. `performance-hypotheses.md`, `timing-comparison.md`, and `fsdb-profile.md` record that VCD was slow because the current Wellen VCD path parses a large text waveform body per process, FST was fast because it can use indexed selective loading, and FSDB one-shot CLI calls spent most time rebuilding and dropping hierarchy state before very fast value sampling.

## Open Questions

There are no blocking open questions. The only judgment call is how much benchmark detail to include; this plan chooses a short table plus qualitative guidance.

## Milestones

The first milestone is the planning milestone. It produces a reviewed execution plan in `docs/tracker/wip/issue-17-waveform-performance-guide.md` that records the goal, constraints, file map, validation commands, and review outcome. This milestone is complete when a read-only reviewer returns no blocking findings or all findings are addressed, and `git diff -- docs/tracker/wip/issue-17-waveform-performance-guide.md` shows the reviewed plan ready to commit.

The second milestone is the documentation milestone. It produces a new embedded public topic at `docs/public/reference/waveform-performance.md`, updates topic discoverability, and updates `tests/docs_cli.rs` so the embedded docs topic list remains deterministic. This milestone is complete when `wavepeek docs show reference/waveform-performance` prints the guide and `wavepeek docs topics` lists the topic in the reference section.

The third milestone is the validation and handoff milestone. It proves that the docs runtime accepts the new topic, that tests covering topic order/export pass, and that repository pre-handoff checks pass or any environment limitation is recorded. This milestone is complete when targeted docs tests and `just check` have been run, implementation review is clean, the branch-local WIP plan has been removed from the final diff unless the maintainer asks otherwise, and the PR links to issue #17.

## Plan of Work

First, review this plan with a read-only docs-focused reviewer. Apply any corrections to file placement, wording scope, or validation. Commit the plan as a branch-local WIP artifact so the requested planning step is recorded.

Next, create `docs/public/reference/waveform-performance.md`. The topic front matter should use id `reference/waveform-performance`, title `Waveform Performance Guide`, description `Understand format-level performance expectations for VCD, FST, and FSDB waveform inspection.`, section `reference`, and see-also links to `reference/command-model`, `commands/overview`, and `commands/info`. The body should explain that performance depends on file size, format indexing, selected command, machine, and cache state, so the page provides expectations rather than guarantees.

The topic should contain concise sections for VCD, FST, FSDB, conversion, and practical selection guidance. VCD guidance should say that large VCD files are textual and can require expensive parsing and high memory use, so even narrow queries may pay setup costs. FST guidance should say that FST is compact and indexed, generally preferred for repeated inspection, and can selectively load requested signals. FSDB guidance should say that the native reader can sample quickly after setup, but short independent CLI invocations may repeatedly pay reader/hierarchy setup costs; it must not mention future batch/session workflows. Conversion guidance should say that converting to FST can be worthwhile for repeated analysis but may not pay off for one-off queries because conversion itself costs time.

Then update discoverability. Add `reference/waveform-performance` to `docs/public/intro.md` `see_also`. Add it to `docs/public/reference/command-model.md` `see_also`, because the command model already explains stateless per-invocation waveform opening. Optionally add one sentence in `commands/overview.md` pointing users to the performance guide when choosing dump formats or diagnosing slow queries; keep this narrative, not a duplicated flag table.

Update `tests/docs_cli.rs` by changing `const TOPIC_IDS: [&str; 21]` to `const TOPIC_IDS: [&str; 22]` and adding `reference/waveform-performance` in lexicographic order within the reference section after `reference/machine-output`.

Finally, run validation. Run a targeted docs test first, then the repository pre-handoff gate. If validation reveals stale docs order, front matter errors, or wording issues, update the docs and this plan before re-running the affected checks.

### Concrete Steps

From `/workspaces/wavepeek`, run the plan review through a read-only subagent. After plan review is clean, commit the plan:

    git status --short
    git add docs/tracker/wip/issue-17-waveform-performance-guide.md
    git commit -m "docs(tracker): plan waveform performance guide"

Apply documentation edits:

    $EDITOR docs/public/reference/waveform-performance.md
    $EDITOR docs/public/intro.md
    $EDITOR docs/public/reference/command-model.md
    $EDITOR docs/public/commands/overview.md
    $EDITOR tests/docs_cli.rs

Run targeted validation:

    cargo test -q docs_cli

Run the project pre-handoff gate:

    just check

Before final PR, remove this branch-local WIP plan unless the maintainer asks to keep it:

    git rm docs/tracker/wip/issue-17-waveform-performance-guide.md

Commit implementation and cleanup with a conventional commit message that references issue #17. Then push and create a PR:

    git push -u origin issue-17-waveform-performance-guide
    gh pr create --repo kleverhq/wavepeek --base main --head issue-17-waveform-performance-guide --title "docs: add waveform performance guide" --body "Closes #17"

### Validation and Acceptance

The change is accepted when `wavepeek docs topics` lists `reference/waveform-performance` in the reference section, `wavepeek docs show reference/waveform-performance` prints the new guide, and `wavepeek docs search performance` can find it. The targeted Rust docs tests should pass with `cargo test -q docs_cli`. The repository pre-handoff gate should pass with `just check`.

A representative successful transcript from implementation validation:

    $ cargo run -q -- docs show reference/waveform-performance --description
    Understand format-level performance expectations for VCD, FST, and FSDB waveform inspection.

    $ cargo run -q -- docs search performance
    reference/waveform-performance  Waveform Performance Guide — Understand format-level performance expectations for VCD, FST, and FSDB waveform inspection. [matched id prefix]

    $ cargo test -q --test docs_cli
    running 24 tests
    ........................
    test result: ok. 24 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out

    $ cargo test -q export_writes_manifest_and_topics_without_skill_file
    Test Results: 1 passed

    $ just check
    checked docs for wavepeek 1.0.1; prepared 3 root artifact(s) under /workspaces/wavepeek/tmp/docs-site/root-artifacts

### Idempotence and Recovery

All edits are ordinary text changes and can be repeated safely. If a docs test fails because topic order changed, update `TOPIC_IDS` in `tests/docs_cli.rs` to match the section ranking and lexicographic order used by `src/docs/mod.rs`. If the new topic fails to load, check that the front matter fields exist, the id matches the path, and the first body heading exactly equals the title. If `just check` fails due to missing container state, record the exact failure and run the strongest available targeted tests.

### Artifacts and Notes

The key source measurements from `tmp/fsdb-eval/performance-hypotheses.md` are:

    File sizes: FSDB 15 MiB, VCD 3.4 GiB, FST 35 MiB.
    Representative command times and peak memory:
    FSDB about 4.7-5.3 s and about 2.0 GiB RSS.
    VCD about 14.8-15.0 s and about 5.3 GiB RSS.
    FST about 0.2-0.7 s and about 100 MiB RSS.

The FSDB profile in `tmp/fsdb-eval/fsdb-profile.md` refined the hypothesis:

    Native FSDB open was about 0.12-0.13 s.
    Native scope tree read plus Rust hierarchy construction was about 3.5 s.
    Hierarchy drop was about 0.7-0.8 s.
    Six-signal session open was about 0.079 s.
    Actual six-signal sampling was about 0.0001 s.

Use these numbers only as a generic example. Do not present them as a guaranteed benchmark for all waveforms.

### Interfaces and Dependencies

No Rust interfaces change. The only runtime dependency involved is the existing embedded docs loader in `src/docs/mod.rs`, which automatically includes Markdown topics under `docs/public/`. The tests in `tests/docs_cli.rs` are the primary contract checks for docs topic discovery, JSON topic listing, and export manifests.
