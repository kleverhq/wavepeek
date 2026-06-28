# Public Command Topic Guidance

## Source of Truth

- Command overview: `overview.md`
- Help/docs/skill behavior topics: `help.md`, `docs.md`, `skill.md`
- Stable command semantics: `../reference/command-model.md`
- Machine output behavior: `../reference/machine-output.md`
- Expression syntax for `change`, `property`, and `extract generic`: `../reference/expression-language.md`
- Topic metadata and docs style rules: `../../dev/style.md`
- Exact command reference: `../../../src/cli/`, `wavepeek --help`, `wavepeek help <command-path...>`, and `wavepeek docs --help`

## Local Guidance

Command topics may explain intent, workflows, examples, and troubleshooting. Keep generated help as the exact flag authority; copying it here creates two places for entropy to breed.
