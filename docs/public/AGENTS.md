# Public Docs Guidance

## Source of Truth

- Public introduction and topic map: `intro.md`
- Command-family guidance: `commands/`
- Stable semantics references: `reference/`
- Topic metadata and docs style rules: `../dev/style.md`
- Exact command reference: `../../src/cli/`, `wavepeek --help`, `wavepeek help <command-path...>`, and `wavepeek docs --help`

## Local Guidance

- Keep topic IDs stable, slash-separated, and user-facing.
- Do not duplicate exact flag tables from generated help; explain intent, workflows, edge cases, and contracts that help cannot express cleanly.
- `wavepeek docs export` includes public topics only, not packaged skills.
