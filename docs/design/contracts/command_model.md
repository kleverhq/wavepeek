# Command Model Contract

This document is normative for the cross-cutting semantics shared across the shipped waveform-inspection commands. It intentionally avoids repeating exact flag lists and defaults. Layered help, the visible `help` subcommand, the `docs` command family, embedded topic rules, and docs export/search behavior are governed by `documentation_surface.md` instead. For the precise command-line surface, follow `src/cli/`, `wavepeek --help`, and `wavepeek <command> --help`.

## 1. Waveform Input Model

wavepeek is a stateless CLI. Each invocation opens one waveform dump when needed, executes one command, writes its result, and exits.

All waveform-inspection commands require `--waves <FILE>` and operate on a single dump per invocation. Non-waveform surfaces such as `schema`, `help`, and `docs` are outside this document's scope and follow `documentation_surface.md` plus the exact CLI/help surface.

The supported dump formats are VCD (Value Change Dump) and FST (Fast Signal Trace).

## 2. Time Tokens and Normalization

Every explicit time token requires an integer magnitude plus a unit suffix. The accepted suffixes are `zs`, `as`, `fs`, `ps`, `ns`, `us`, `ms`, and `s`. Bare numbers such as `100` are invalid.

When wavepeek parses a time token, it converts that value into the dump's native `time_unit`. All observable timestamps are then rendered back as normalized integer counts in that dump unit. If a requested time cannot be represented exactly at dump precision, the command fails instead of silently rounding.

These rules apply to point sampling (`--at`) and to window boundaries (`--from`, `--to`).

## 3. Time Windows and Inclusive Boundaries

Commands that accept `--from` and `--to` interpret them as an inclusive time window.

- `--from` plus `--to` means the closed interval from the start token through the end token.
- `--from` without `--to` means from that timestamp through the end of the dump.
- `--to` without `--from` means from the start of the dump through that timestamp.
- Omitting both means the entire dump.

Commands without time-window flags do not participate in this model. `value` uses the same time-token rules but samples one exact timestamp through `--at`.

## 4. Naming, Scopes, and Resolution

wavepeek uses canonical dump-derived paths as the stable naming model. Without `--scope`, signal-like names are interpreted as canonical full paths.

Commands that support `--scope` allow shorter names relative to the selected scope. In those scoped modes, name resolution happens inside the declared scope rather than against the full hierarchy root. Human-readable output may render short or relative names for compactness, but machine-readable output keeps canonical paths where the contract defines them.

The commands that depend on this model are:

- `signal`, which requires an exact scope path and can optionally traverse child scopes.
- `value`, which accepts either canonical paths or scope-relative signal names depending on whether `--scope` is set.
- `change` and `property`, which apply the same scope-relative resolution model to sampled signals, trigger names, and expression references.

Unresolved names are errors. In scoped `change` and `property` mode, canonical full-path tokens are rejected in places where the command contract expects names to stay relative to the selected scope, preventing mixed-resolution queries.

## 5. Human-Readable and Machine-Readable Modes

Waveform commands default to human-readable output. Machine-readable output is enabled explicitly with `--json`.

Human-readable output is optimized for compact operator use and may vary when formatting improvements are made. Machine-readable output is strict and versioned through the JSON schema contract described in `machine_output.md` and exposed by `wavepeek schema`.

`schema` is a special case: it always prints one JSON Schema document to stdout and never wraps that payload in the normal command envelope. The non-waveform `docs` command family has its own help and narrative-doc semantics in `documentation_surface.md`; only `docs topics --json` and `docs search --json` participate in the stable JSON envelope.

## 6. Bounded Output and Warning Semantics

wavepeek is designed to avoid flooding terminals and LLM context windows. Commands therefore keep output bounded by default through one or more of these mechanisms:

- explicit count limits such as `--max`,
- depth limits such as `--max-depth`,
- the finite size of the requested input set, or
- an inherently finite command shape such as `schema`.

When a command truncates output because of an active limit, it emits a warning. When a command supports disabling a limit explicitly, that opt-out also emits a warning so automation can tell the boundedness contract changed on purpose.

## 7. Deterministic Ordering

Deterministic output is a repository-wide design requirement. Given identical input data and identical command arguments, wavepeek must emit results in a stable order.

The main ordering rules are:

- `scope` traverses hierarchy in pre-order depth-first order with lexicographic child ordering.
- Recursive `signal` queries walk scopes in that same stable order and sort signals deterministically within each visited scope.
- `value` preserves the request order from `--signals`.
- `change` and `property` emit rows in ascending normalized timestamp order.
- When multiple warnings apply, their order is deterministic for a given command contract.

These ordering guarantees are part of the command model because automation depends on predictable, replayable output.
