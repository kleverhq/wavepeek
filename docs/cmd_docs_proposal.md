# Self-Documenting CLI Docs Proposal

## Status

- Status: proposed
- Scope: installed `wavepeek` CLI UX
- Primary audience: agents first, humans second
- Goal: make the installed binary the version-matched entry point for syntax help, reference help, workflow docs, and troubleshooting docs

## Authority and Lifecycle

This file is a product/design proposal, not a new permanent source of truth for the shipped CLI surface.

Current repository policy still says the exact CLI surface is code-first and authoritative in `src/cli/`, `wavepeek --help`, and `wavepeek <command> --help`, while prose docs stay focused on semantics and guidance. This proposal exists to define the desired target UX before implementation.

If accepted, its outcomes should be migrated into the canonical design corpus and implementation artifacts:

- durable semantics and ownership rules into `docs/design/`,
- exact command behavior into CLI code/help/tests,
- derived operator guidance into `docs/design/reference/cli.md`.

The user requested this temporary file path directly (`docs/cmd_docs_proposal.md`), so this document should be treated as a staging proposal rather than a new permanent contract layer.

## Relationship to Existing Help Contract

This proposal intentionally changes the prior help-direction that made `-h` and `--help` identical everywhere.

If adopted, it supersedes the layered-help assumptions documented in the completed historical plan [`docs/exec-plans/completed/2026-03-01-cli-help-self-descriptive/PLAN.md`](exec-plans/completed/2026-03-01-cli-help-self-descriptive/PLAN.md), specifically the expectation that short and long help are byte-identical across all command scopes.

Adoption therefore requires coordinated updates to:

- CLI parsing/help behavior,
- CLI contract tests,
- canonical design docs,
- any derived reference docs that still describe parity.

## Problem Statement

Today, detailed CLI knowledge is split across command help, repository docs, and external agent guidance. That makes the installed binary an incomplete source of truth and forces both humans and agents to leave the tool to answer routine questions.

The target UX is a self-documenting CLI with progressive disclosure:

1. quick syntax lookup without flooding context,
2. deep command reference when semantics matter,
3. narrative docs for workflows, concepts, examples, and troubleshooting,
4. offline access that always matches the installed version.

## Design Principles

1. **Installed binary is the entry point.** Users should not need web docs for normal usage.
2. **Progressive disclosure.** Short help, long help, and narrative docs must reveal information in layers.
3. **Version-matched documentation.** The docs exposed by the CLI must describe the installed build.
4. **Agent-friendly by default.** Output should be plain `stdout`, deterministic, and easy to read incrementally.
5. **Single source of truth.** Narrative docs should live as source Markdown in the repo and be embedded into the binary rather than duplicated as Rust string literals.
6. **Reference vs. guide split.** Command syntax/reference belongs in help; workflows/tutorials/troubleshooting belong in `wavepeek docs`.

## Non-Goals

This proposal does not yet standardize:

- pager integration,
- localization,
- fuzzy-search sophistication beyond deterministic substring/token search,
- topic authoring workflow beyond the required source format and metadata contract,
- pretty terminal rendering beyond a possible future plain-text mode.

## Information Layers

### Layer 1: short help

`-h` is a compact lookup surface.

It should answer only:

- what the command is for,
- the argument shape,
- the most important flags or positional arguments,
- where to go next for more detail.

This output should fit routine lookup use and avoid examples or long semantic caveats unless they are critical for safe use.

### Layer 2: long help

`--help` and `wavepeek help ...` are the command reference surface.

They should answer:

- full syntax,
- argument semantics,
- caveats and boundary rules,
- output-mode notes,
- 2-4 small examples,
- cross-links into `wavepeek docs` topics when narrative guidance exists.

### Layer 3: narrative docs

`wavepeek docs ...` is the long-form documentation surface.

It should cover:

- concepts,
- workflows,
- extended examples,
- troubleshooting,
- agent usage guidance.

`wavepeek docs` must not compete with command reference. It complements it.

## Top-Level Command Contracts

### `wavepeek`

- With no arguments, `wavepeek` is an alias for `wavepeek --help`.
- It must not silently switch to `-h`, `wavepeek docs`, or an argument error.

### `wavepeek -h`

- Prints compact top-level help.
- Must stay short enough for routine scanning.
- Must mention `wavepeek --help`, `wavepeek help <command>`, and `wavepeek docs` as next steps.

### `wavepeek --help`

- Prints detailed top-level reference help.
- Includes command-family descriptions, important conventions, and examples.
- Must mention `wavepeek docs topics`, `wavepeek docs show <topic>`, and `wavepeek docs skill`.

### `wavepeek help`

- Alias for top-level `wavepeek --help`.

### `wavepeek help <command-path...>`

- Alias for `<command-path> --help`.
- Examples:
  - `wavepeek help value`
  - `wavepeek help docs show`
- The help command must support nested command paths rather than a single token only.

## `wavepeek docs` Command Family

### `wavepeek docs`

- Prints a short docs index and orientation message.
- Purpose: serve as the entry point into the documentation tree.
- Must include:
  - what kinds of docs exist,
  - how to list topics,
  - how to read one topic,
  - how to search,
  - how to print the agent skill,
  - how to export the full docs tree.

Example shape:

```text
wavepeek local docs

Start here when you need more than command syntax.

Try:
  wavepeek docs topics
  wavepeek docs show intro
  wavepeek docs search transitions
  wavepeek docs skill
  wavepeek docs export /tmp/wavepeek-docs
```

### `wavepeek docs topics`

- Lists all available documentation topics.
- Topics use stable slash-separated IDs without file extensions.
- Results must be ordered lexicographically by topic ID in both text and JSON output.
- Example IDs:
  - `intro`
  - `concepts/time`
  - `concepts/selectors`
  - `commands/change`
  - `workflows/find-first-change`
  - `troubleshooting/empty-results`

Default text output must include, per topic:

- topic ID,
- short title,
- one-sentence summary.

Options:

- `--summary`: allowed, but remains list-oriented; it should print only topic IDs plus summary text in a compact form.
- `--json`: prints the machine-readable JSON form.

Flag interaction rules:

- `--json` is mutually exclusive with `--summary`.
- When `--json` is selected, the command always returns the full machine-readable topic metadata set defined by the schema.

`--json` output should expose stable machine-readable fields:

- `id`
- `title`
- `summary`
- `section`
- `see_also` (optional)

JSON envelope contract:

- `docs topics --json` is part of the normal machine-oriented CLI contract rather than an ad hoc debug convenience.
- It uses the standard `--json` success envelope defined by the machine-output contract rather than a docs-specific standalone wrapper.
- The base shape is:

```json
{
  "$schema": "https://raw.githubusercontent.com/kleverhq/wavepeek/v<version>/schema/wavepeek.json",
  "command": "docs topics",
  "data": {
    "topics": [
      {
        "id": "concepts/time",
        "title": "Time semantics",
        "summary": "How wavepeek normalizes and interprets time values.",
        "section": "concepts"
      }
    ]
  },
  "warnings": []
}
```

- The JSON output must include the standard `$schema` field, `command`, `data`, and `warnings` fields like other stable machine-oriented outputs.
- The `$schema` field must follow the same version-resolved schema-linkage policy as the rest of the CLI's `--json` surface.
- The authoritative schema for this output should live in `schema/wavepeek.json` or its future replacement alongside other stable machine contracts.

### `wavepeek docs show <topic>`

- Prints the selected topic to `stdout`.
- Default output is raw Markdown, not pretty terminal rendering.
- The topic argument is the stable topic ID, not a filesystem path with `.md`.

Options:

- `--summary`: prints the summary for the selected topic instead of the full body.
- `docs show --summary` always prints the stored summary text only.

Contract details:

- Without `--summary`, `show` prints the Markdown body exactly as stored for that topic, excluding YAML front matter.
- `show --summary` is topic-local: it summarizes only the requested topic.
- If the topic is unknown, the command must return a non-zero exit code and suggest close matches when available.

### `wavepeek docs search <query>`

- Searches the documentation tree and returns all matching topics in deterministic ranked order.
- `search` is required even if `export` exists; users should not need to dump docs to disk for routine discovery.

Default search scope:

- topic ID,
- title,
- summary,
- Markdown headings.

Optional expanded search scope:

- `--full-text`: also search the full Markdown body.

Default output should be concise and optimized for progressive disclosure. Each result should include:

- topic ID,
- title,
- summary,
- a brief match reason when practical (for example `matched heading`, `matched title`, `matched body`).

Options:

- `--full-text`
- `--json`: prints the machine-readable JSON form

Result cardinality rules:

- The base contract returns all matches in ranked order rather than an implementation-defined shortlist.
- If a future implementation adds result limiting, it must do so with an explicit user-facing control such as `--limit` or `--all`.

Ranking should be deterministic and use this total order:

1. exact whole-query topic ID match,
2. otherwise, topics matching more distinct query tokens rank ahead of topics matching fewer,
3. within the same token-count bucket, prefer stronger structural matches over weaker ones using this precedence:

   1. topic ID prefix match,
   2. exact title match,
   3. title or summary substring match,
   4. heading match,
   5. full body match,
4. within the same structural bucket, results are ordered lexicographically by topic ID.

Query normalization rules:

- Matching is case-insensitive.
- Leading and trailing ASCII whitespace is ignored.
- The initial implementation may use simple whitespace tokenization rather than stemming or fuzzy matching.
- Multi-token queries use OR semantics in the base contract: a topic matches if any token matches within the active search scope.
- Topics matching more distinct query tokens rank ahead of topics matching fewer query tokens.
- If normalization produces zero query tokens, the command must fail as an argument error with a non-zero exit code.
- If no topics match, the command must succeed with an empty result set rather than fail.

`--json` output should return a stable list of matches with at least:

- `id`
- `title`
- `summary`
- `match_kind`

`match_kind` represents the strongest structural match bucket that contributed to the result's ranking. Allowed values are:

- `id_exact`
- `id_prefix`
- `title_exact`
- `title_or_summary`
- `heading`
- `body`

JSON envelope contract:

- `docs search --json` is part of the normal machine-oriented CLI contract rather than an ad hoc debug convenience.
- It uses the standard `--json` success envelope defined by the machine-output contract rather than a docs-specific standalone wrapper.
- The base shape is:

```json
{
  "$schema": "https://raw.githubusercontent.com/kleverhq/wavepeek/v<version>/schema/wavepeek.json",
  "command": "docs search",
  "data": {
    "query": "ready valid",
    "full_text": true,
    "matches": [
      {
        "id": "workflows/trace-valid-ready",
        "title": "Trace valid/ready handshakes",
        "summary": "Inspect common ready/valid interaction patterns.",
        "match_kind": "body"
      }
    ]
  },
  "warnings": []
}
```

- The JSON output must include the standard `$schema` field, `command`, `data`, and `warnings` fields like other stable machine-oriented outputs.
- The `$schema` field must follow the same version-resolved schema-linkage policy as the rest of the CLI's `--json` surface.
- The authoritative schema for this output should live in `schema/wavepeek.json` or its future replacement alongside other stable machine contracts.

### `wavepeek docs export <out-dir>`

- Exports the full installed documentation tree to the target directory.
- Purpose: provide an escape hatch for users or agents that want to inspect the entire corpus with external tools.
- This is an advanced command, not the primary discovery path.

Behavior:

- Create `<out-dir>` if it does not exist.
- Export one Markdown file per topic using the stable topic path, for example `concepts/time.md`.
- Preserve topic IDs in the on-disk layout.
- Write a root `manifest.json` file describing the export.

Required exported manifest fields:

- `kind`: exact string `wavepeek-docs-export`
- `export_format_version`: integer export format version
- `cli_name`: exact string `wavepeek`
- `cli_version`: installed CLI version string
- `topics`: array with `id`, `title`, and `summary`

Manifest interoperability contract:

- The manifest filename is exactly `manifest.json` at the export root.
- The manifest format is UTF-8 encoded JSON.
- Top-level keys must be emitted in deterministic order.
- The exported topic list inside the manifest must be ordered lexicographically by topic ID.
- Exported Markdown files must be UTF-8 encoded.

Export root ownership rules:

- The export target is a dedicated wavepeek-managed directory root.
- A successful export must write a root `manifest.json` file whose `kind` field is exactly `wavepeek-docs-export`.
- `--force` is allowed only when the target directory is empty or already identified by that exact manifest marker as a prior wavepeek export root.
- If `manifest.json` contains an unrecognized `export_format_version`, `--force` must refuse rather than guess overwrite compatibility.
- If the target directory is non-empty and lacks the sentinel, the command must refuse rather than guess what may be deleted.
- When `--force` is used on a prior wavepeek-managed export root, the implementation may replace the full contents of that managed root.
- Users who need to keep unrelated files must export into a dedicated subdirectory.

Overwrite behavior:

- If `<out-dir>` exists and is non-empty, the command must fail by default.
- Add `--force` to allow overwriting an existing target tree.
- With `--force`, the command must replace the previously exported wavepeek-managed tree under `<out-dir>` so removed or renamed topics do not leave stale exported files behind.
- `--force` must not silently merge new docs into stale wavepeek-exported files.
- If practical on the host platform, the implementation should prefer write-to-temp plus rename semantics to reduce partial-export risk.

Output expectations:

- Success prints a short summary to `stdout`, including target path and topic count.
- Failures go to `stderr` with a non-zero exit code.

### `wavepeek docs skill`

- Prints the raw Markdown content of the installed `wavepeek` agent skill.
- The purpose is to let agents or users inspect the exact workflow/router guidance that ships with the installed version.
- Default output should be raw Markdown.

This command exists because the skill is part of the product ergonomics, but it is not the primary source of truth for command or workflow semantics.

## `--json` Support Matrix for `docs`

`--json` support is intentionally limited.

- `wavepeek docs topics --json`: supported.
- `wavepeek docs search --json`: supported.
- `wavepeek docs show --json`: not supported.
- `wavepeek docs skill --json`: not supported.
- `wavepeek docs export --json`: not supported.

Rejection contract for unsupported cases:

- Unsupported `--json` combinations must fail as argument errors with a non-zero exit code.
- Implementations must not silently reinterpret `--json` as a human-readable output mode.

## Topic Model

The documentation tree should use topic-oriented Markdown source files stored in the repository and embedded into the binary at build time.

Recommended top-level topic families:

- `intro`
- `concepts/`
- `commands/`
- `workflows/`
- `troubleshooting/`

Each topic should have stable metadata:

- `id`
- `title`
- `summary`
- `section`
- `see_also` (optional)

Source format contract:

- Each topic is stored as one Markdown file with YAML front matter followed by the Markdown body.
- The YAML front matter is the canonical source for topic metadata.
- The Markdown body must begin with a visible top-level heading that matches the canonical `title` metadata.
- `wavepeek docs show` prints only the Markdown body, not the YAML front matter.
- `wavepeek docs export` writes the Markdown topic files with their YAML front matter intact, plus the export-root manifest used for discovery and overwrite safety.

The topic metadata should drive:

- `docs topics`
- `docs search`
- unknown-topic suggestions
- exported manifest generation

## Cross-Linking Contract

Help and docs must cross-link intentionally.

Examples:

- long help for `wavepeek change` should end with `See also:` references into `wavepeek docs show commands/change` and relevant workflow/concept topics,
- `wavepeek docs show commands/change` should point back to `wavepeek help change` for exact syntax.

This keeps reference help and narrative docs complementary rather than duplicative.

## Output and UX Rules

These rules apply to the full self-documenting surface.

- No pager by default.
- No interactive TUI required to read docs.
- Plain `stdout` text is the default success path.
- Errors go to `stderr` and use non-zero exit codes.
- Output must be deterministic for the same command, docs corpus, and version.
- Structured machine-oriented output must be explicit via `--json`.
- Raw Markdown output is preferred over decorative rendering for agent-facing paths.

For `docs export`, this determinism rule means the default exported tree and manifest should be byte-stable for the same CLI version and docs corpus. Runtime wall-clock timestamps are therefore out of scope for the base export contract unless added later behind an explicit opt-in flag.

## Single Source of Truth

The intended ownership model is:

- short and long help live in the CLI command definitions,
- narrative docs live as Markdown source files in the repository,
- the binary embeds the narrative docs and exposes them through `wavepeek docs`,
- the skill acts as a router/cookbook for agents and should point back into CLI help and docs instead of duplicating them.

This proposal explicitly rejects:

- skill-only documentation,
- man-page-only documentation,
- giant `--help` output as the only documentation layer,
- manually duplicated Markdown in Rust string literals.

### Ownership Matrix

The intended ownership split is:

| Concern | Primary owner |
| --- | --- |
| Exact command names, flags, aliases, defaults, requiredness, and rendered help output | CLI code, help text, and CLI contract tests |
| Narrative concepts, workflows, examples, troubleshooting, and topic metadata | Embedded Markdown topics with YAML front matter |
| Agent routing guidance | Installed `wavepeek` skill |
| Human-oriented repo summaries and operator guides | Derived docs under `docs/design/` |

This proposal describes the desired target structure for those owners. It does not replace them.

## Deferred / Optional Follow-Ups

The following may be added later, but are not required to satisfy this proposal:

- manpage generation from the CLI command model,
- online docs generated from the same Markdown source,
- richer text rendering for humans,
- related-topic discovery beyond static `see_also`,
- excerpt snippets in search results.

## Implementation Notes

- TDD is not applicable for this document-only change; implementation work that follows should add CLI contract tests before code changes.
- The likely storage model is embedded Markdown files with YAML front matter, plus the export-root manifest used by `docs export`.
- The likely command ownership split is:
  - `-h`: compact lookup,
  - `--help` / `help`: full command reference,
  - `docs`: long-form narrative documentation,
  - `docs search`: built-in discovery,
  - `docs export`: advanced escape hatch,
  - `docs skill`: installed agent-facing router guidance.

## Example Progressive-Disclosure Flow

One intended usage path is:

1. `wavepeek change -h` for a one-sentence reminder and argument shape.
2. `wavepeek help change` for command semantics, caveats, and examples.
3. `wavepeek docs show commands/change` for the narrative explanation of that command.
4. `wavepeek docs search ready valid --full-text` to discover related workflows or troubleshooting topics.
