# Style and Contract Conventions

This document covers maintainer conventions for Rust code, CLI behavior, deterministic output, and packaged docs. Public user-visible semantics live under `docs/public/reference/`; do not duplicate that reference surface here.

## Rust Style

Use rustfmt. Avoid manual alignment or stylistic churn that fights the formatter. Prefer explicit control flow over clever one-liners when clarity or error handling matters.

Use explicit imports instead of glob imports except in `mod.rs` where it materially reduces noise. Keep imports in the usual Rust order: standard library, external crates, then crate modules. Clippy runs with warnings denied, so unused imports are gate failures, not decorative lint confetti.

Use `snake_case` for modules, functions, and locals; `PascalCase` for types, traits, and enums; and `SCREAMING_SNAKE_CASE` for constants. CLI flags should be long, self-documenting, and kebab-case through clap.

Prefer borrowing at API boundaries. Use owned `String` and `Vec` when ownership is required, not because cloning felt easier. Avoid `Box<dyn Error>` in core paths; prefer typed errors with `thiserror`.

## Error Handling

No panics in production paths. Avoid `unwrap()` and `expect()` except for true programmer bugs that are unreachable in normal operation. Errors go to stderr; stdout is reserved for command output.

Preserve the stable process-level failure shape:

    fatal: <category>: <message>

Successful commands may emit non-fatal diagnostics. Human diagnostics use `warning[WPK-W####]: <message>` or `error[WPK-E####]: <message>`, and JSON diagnostics use typed objects in the success envelope.

Also preserve exit-code behavior. Exit code `0` is success, `1` is user-facing argument or query failures, and `2` is file open/parse failure.

## Deterministic Output

Identical inputs must produce identical outputs. Sort user-facing collections deterministically, avoid timestamps and random IDs, and never rely on hash-map iteration order. Default result sets must stay bounded with flags such as `--max` and `--max-depth`.

## CLI Design Constraints

Waveform-inspection commands use named flags for primary inputs. The waveform file flag is always `--waves`. Default output is human-readable; `--json` enables the strict JSON envelope documented in `docs/public/reference/machine-output.md`. Time values require explicit units; reject bare numbers.

Help must remain layered and standalone: `wavepeek` with no args aliases compact help, `-h` stays compact, `--help` stays detailed, `wavepeek help <command-path...>` aliases long nested help, `wavepeek docs` serves packaged narrative docs, and `wavepeek skill` prints the packaged agent skill.

## Public Docs Maintenance

The packaged `wavepeek docs` corpus lives under `docs/public/`. Topic files use YAML front matter with `id`, `title`, `description`, and `section`. `see_also` is optional but must reference existing topic IDs. Each topic body starts with an H1 that exactly matches `title`.

Topic IDs are stable slash-separated user-facing names, and file paths under `docs/public/` match the ID plus `.md`. Keep `docs/public/commands/help.md` and `docs/public/commands/docs.md` as the user-facing homes for layered help and docs command behavior.

The packaged skill source lives at `docs/skills/wavepeek.md` and is emitted verbatim by `wavepeek skill`. `wavepeek docs export` exports public topics only and intentionally excludes packaged skills.
