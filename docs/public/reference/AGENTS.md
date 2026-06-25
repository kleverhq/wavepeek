# Public Reference Topic Guidance

## Source of Truth

- Cross-cutting command semantics: `command-model.md`
- Machine output, diagnostics, and exit behavior: `machine-output.md`
- Expression language syntax and semantics: `expression-language.md`
- Exact JSON schema: the current artifact such as `../../../schema/wavepeek_v2.0.json` and `wavepeek schema`
- Topic metadata and docs style rules: `../../dev/style.md`
- Exact command reference: `../../../src/cli/`, `wavepeek --help`, `wavepeek help <command-path...>`, and `wavepeek docs --help`

## Local Guidance

Keep these topics focused on stable behavior that code, generated help, or schema alone do not explain clearly enough. Avoid release planning, maintainer process, or speculative future syntax here.
