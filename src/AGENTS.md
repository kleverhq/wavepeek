# Source Code Guidance

## Source of Truth

- Rust style and CLI constraints: `../docs/style.md`
- Internal architecture: `../docs/architecture.md`
- Public command and output contracts: `../skills/wavepeek/references/commands.md`, `../skills/wavepeek/references/machine-output.md`
- Expression semantics for `change`, `property`, and `extract` family: `../skills/wavepeek/references/boolean-expressions.md`

## Embedded Skill Runtime

- Packaged skill extraction lives in `skill.rs` and `engine/skill.rs`.
- Canonical package source lives at `../skills/wavepeek/`.
- Keep package metadata sourced from embedded package files rather than duplicated as hand-maintained Rust literals.

## Local Guidance

Keep `../docs/architecture.md` consistent when module boundaries, execution layers, or ownership responsibilities change. Public behavior changes must update the relevant packaged references and tests in the same slice.
