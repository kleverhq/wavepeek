# Documentation Surface Contract

This document is normative for layered help, the visible `help` and `docs`
command surfaces, embedded documentation topics, packaged skill ownership, docs
search semantics, and docs export behavior. It intentionally avoids repeating
exact flag spellings and help examples. For the precise command-line surface,
follow `src/cli/`, `wavepeek -h`, `wavepeek --help`, `wavepeek help
<command-path...>`, and `wavepeek docs --help`.

## 1. Layered Help Model

`wavepeek` uses progressive disclosure instead of one help style everywhere.

- `wavepeek` with no arguments is an alias for top-level `wavepeek --help`.
- `wavepeek -h` is the compact lookup layer. It should stay short enough for
  routine scanning and point readers to the deeper layers.
- `wavepeek --help` is the detailed command-reference layer.
- `wavepeek help` is an alias for top-level `wavepeek --help`.
- `wavepeek help <command-path...>` is an alias for `<command-path...> --help`
  and must support nested paths such as `wavepeek help docs show`.

Short help is allowed to omit long-form conventions, caveats, and examples.
Long help is expected to include those details, plus cross-links into
`wavepeek docs` when narrative guidance exists.

## 2. `docs` Command Family

`wavepeek docs` is the installed narrative-documentation surface. It is not a
help alias.

- `wavepeek docs` prints a short orientation index.
- `wavepeek docs topics` lists packaged topic metadata in lexicographic topic-ID
  order.
- `wavepeek docs show <topic>` prints one topic body as raw Markdown with YAML
  front matter removed.
- `wavepeek docs show <topic> --summary` prints only the stored summary text.
- `wavepeek docs search <query>` returns matching topics in deterministic ranked
  order.
- `wavepeek docs export <out-dir>` writes the authored Markdown topic corpus to a
  managed export root.
- `wavepeek docs skill` prints the packaged skill Markdown.

`--json` support is intentionally narrow:

- `docs topics --json` is stable and uses the standard JSON success envelope.
- `docs search --json` is stable and uses the standard JSON success envelope.
- `docs show`, `docs export`, and `docs skill` do not support `--json`; those
  combinations fail as argument errors.

## 3. Topic Source and Metadata

The canonical authored source for embedded narrative docs lives under
`docs/cli/topics/`. Topic files are Markdown documents with YAML front matter.
The required front-matter keys are:

- `id`
- `title`
- `summary`
- `section`

`see_also` is optional and, when present, is an ordered list of stable topic
IDs.

Each topic body must begin with an H1 heading that matches `title` exactly. The
stable user-facing topic ID is the slash-separated `id`, not a filesystem path
with a `.md` suffix.

The `commands/*` topic family is narrative-only. Those files may explain
concepts, workflows, caveats, or longer examples, but they must not become a
second authoritative command-reference surface. Exact syntax, defaults,
requiredness, and flag tables remain code-first in `src/cli/` and generated
help. Narrative command topics should point readers back to `wavepeek help
<command-path...>` for exact syntax.

The canonical packaged skill source lives at `docs/cli/wavepeek-skill.md`.
`.opencode/skills/wavepeek/SKILL.md` is a derived runtime copy that must stay in
sync with the packaged source.

## 4. Docs Search Semantics

Docs search is deterministic and case-insensitive. Query normalization trims
leading and trailing ASCII whitespace and tokenizes on internal whitespace.

- If normalization yields zero tokens, the command fails as an argument error.
- Matching uses OR semantics: a topic matches if any normalized token matches in
  the active search scope.
- Default search scope is topic ID, title, summary, and Markdown headings.
- `--full-text` expands the scope to the full Markdown body.
- Topics matching more distinct query tokens rank ahead of topics matching fewer.

Within the same matched-token bucket, structural ranking is:

1. exact whole-query topic-ID match,
2. topic-ID prefix match,
3. exact whole-query title match,
4. title or summary substring match,
5. heading match,
6. full body match.

Remaining ties are ordered lexicographically by topic ID. `docs search --json`
reports the normalized query, the `full_text` mode, each matching topic, the
strongest `match_kind`, and the number of matched query tokens.

## 5. Docs Export Contract

`wavepeek docs export <out-dir>` writes one Markdown file per topic using the
stable topic-path layout and preserves the authored Markdown bytes exactly,
including YAML front matter formatting. It also writes a deterministic
`manifest.json` with these required fields:

- `kind = "wavepeek-docs-export"`
- `export_format_version = 1`
- `cli_name`
- `cli_version`
- `topics`

`topics` is ordered lexicographically by topic ID and uses the same metadata
shape as `docs topics --json`.

`docs export` does not export the packaged skill Markdown. That asset remains
available only through `wavepeek docs skill`.

Export safety rules are stable:

- a missing target directory is created,
- an empty target directory may be populated,
- a non-empty unmanaged target is rejected,
- `--force` is allowed only for an empty target or a previously managed export
  root with a recognized `export_format_version`, and
- managed roots with unrecognized manifest versions are rejected instead of
  guessed.

## 6. Ownership Split

The documentation surface intentionally uses multiple authorities with distinct
roles:

- `src/cli/`, generated help, and the installed binary are authoritative for the
  exact visible CLI surface.
- this document is authoritative for layered-help semantics, docs-topic rules,
  search behavior, export behavior, and ownership boundaries.
- `docs/design/contracts/machine_output.md` and `schema/wavepeek.json` are
  authoritative for stable JSON envelope behavior and exact JSON shapes.
- `docs/cli/` is authoritative for packaged narrative Markdown and the packaged
  skill source.
