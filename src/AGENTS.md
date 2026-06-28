# Source Code Guidance

## Source of Truth

- Rust style and CLI constraints: `../docs/dev/style.md`
- Internal architecture: `../docs/dev/architecture.md`
- Public command and output contracts: `../docs/public/reference/command-model.md`, `../docs/public/reference/machine-output.md`
- Expression semantics for `change`, `property`, and `extract generic`: `../docs/public/reference/expression-language.md`

## Embedded Docs Runtime

- Runtime loader and helpers for `wavepeek docs` live under `docs/` in this source tree.
- Packaged Markdown topic source lives at `../docs/public/`.
- Packaged skill source lives at `../docs/skills/wavepeek.md`.
- Keep metadata sourced from embedded Markdown files rather than duplicated as hand-maintained Rust literals.

## Local Guidance

Keep `../docs/dev/architecture.md` consistent when module boundaries, execution layers, or ownership responsibilities change. Public behavior changes must update the relevant public reference docs and tests in the same slice.
