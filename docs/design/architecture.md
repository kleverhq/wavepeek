# Architecture

This file holds the internal engineering view of wavepeek: non-functional requirements, module boundaries, dependencies, execution strategy, and testing strategy. It does not restate the exact CLI flag surface. For command semantics and machine-output guarantees, use `contracts/command_model.md` and `contracts/machine_output.md`.

## Non-Functional Requirements

### Performance

Performance is the highest implementation priority. Rust is used specifically to keep waveform parsing and query execution fast on large dumps.

Benchmarks are maintained through `bench/e2e/perf.py` for end-to-end CLI scenarios and `bench/expr/perf.py` for expression-engine microbenchmarks.

### Compatibility

The tool is intended to stay OS-agnostic across Linux, macOS, and Windows.

### Output Stability

Identical inputs must produce deterministic output.

### LLM Agent Integration

The repository ships agent-facing workflow assets and deterministic `--json` contracts so LLM clients can consume output without ad hoc parsing.

## Technical Architecture

### Technology Stack

| Component | Choice | Rationale |
|-----------|--------|-----------|
| Language | Rust stable (MSRV 1.93) | Performance, memory safety, and predictable resource use on large dumps |
| CLI framework | `clap` derive API | Self-documenting command definitions with compile-time validation |
| Waveform parsing | `wellen` | Unified VCD/FST interface used successfully by existing waveform tooling |
| Serialization | `serde` + `serde_json` | Standard JSON rendering for machine contracts and schema export |
| Pattern matching | `regex` | Shared filtering surface for hierarchy and signal discovery |
| Error handling | `thiserror` | Typed error enums without runtime boxing |
| Build automation | Cargo + Make | Cargo owns compilation; the Makefile exposes repository quality gates |

### High-Level Execution Layers

wavepeek is organized as three execution layers plus a shared output module. Data flows top-down: the CLI parses arguments, the engine executes command logic, the waveform layer answers dump queries, and the output module renders results.

1. **CLI layer** (`src/cli/`) parses arguments, owns help text, normalizes clap errors, and dispatches typed command structs.
2. **Engine layer** (`src/engine/`) implements command behavior, shared time handling, shared value formatting, expression-runtime helpers, and command dispatch.
3. **Waveform layer** (`src/waveform/`) is the thin adapter over `wellen` for file opening, format detection, hierarchy traversal, and sampled-value access.
4. **Output module** (`src/output.rs`) owns stdout rendering for human mode and strict JSON mode.

Key architectural consequences:

- Execution is stateless. Every command opens the dump, runs once, and exits.
- The engine is format-agnostic. VCD versus FST handling stays in the waveform layer.
- JSON contracts are stabilized through the checked-in schema artifact at `schema/wavepeek.json`.

### Module Structure

```text
src/
├── lib.rs               # Crate entrypoint (`run_cli`) + module ownership
├── main.rs              # Thin binary wrapper around `wavepeek::run_cli()`
├── cli/                 # CLI layer: argument definitions, help text, dispatch
│   ├── mod.rs           # Top-level CLI struct, parse-error normalization, output handoff
│   ├── limits.rs        # Shared bounded-output flag parsing (`--max`, `--max-depth`)
│   ├── info.rs          # `info` command args + clap help
│   ├── scope.rs         # `scope` command args + clap help
│   ├── signal.rs        # `signal` command args + clap help
│   ├── value.rs         # `value` command args + clap help
│   ├── change.rs        # `change` command args + clap help
│   ├── property.rs      # `property` command args + clap help
│   └── schema.rs        # `schema` command args + clap help
├── engine/              # Business logic per command
│   ├── mod.rs           # Command dispatch + shared result types
│   ├── info.rs          # Dump metadata extraction
│   ├── scope.rs         # Hierarchy traversal with depth/filter
│   ├── signal.rs        # Signal listing within scope
│   ├── value.rs         # Value extraction at time point
│   ├── change.rs        # Value-change tracking and engine dispatch
│   ├── expr_runtime.rs  # Shared typed-expression binding/evaluation helpers
│   ├── time.rs          # Shared time token parsing/validation/alignment helpers
│   ├── value_format.rs  # Shared Verilog literal formatting helpers
│   ├── property.rs      # Property runtime entrypoint and capture-mode execution
│   └── schema.rs        # JSON schema export
├── schema_contract.rs   # Canonical schema URL and embedded schema artifact
├── expr/                # Expression engine shared by `change` and `property`
│   ├── mod.rs           # Public typed facade for parsing/binding/evaluation
│   ├── ast.rs           # Spanned expression AST types
│   ├── diagnostic.rs    # Parse/semantic/runtime diagnostic contract
│   ├── lexer.rs         # Spanned tokenizer for event/logical parsing
│   ├── parser.rs        # Strict typed parser
│   ├── host.rs          # Host trait + signal/type/value bridge types
│   ├── sema.rs          # Typed binders for event and logical expressions
│   └── eval.rs          # Typed event matcher and logical evaluator
├── waveform/            # Thin adapter over `wellen`
│   ├── mod.rs           # File loading, format detection, query helpers
│   └── expr_host.rs     # Waveform-backed expression host bridge
├── output.rs            # Shared output formatting (JSON + human)
└── error.rs             # `WavepeekError` enum and exit mapping
```

### Separation of Concerns

| Module | Knows about | Does not know about |
|--------|-------------|---------------------|
| `cli/` | clap, dispatch, help text | waveform parsing internals, output serialization details |
| `engine/` | domain logic, waveform API, shared semantics helpers | clap parsing flow |
| `expr/` | expression AST, types, evaluation | CLI, output formatting, `wellen` |
| `waveform/` | `wellen` API and dump access | CLI behavior, output formatting |
| `output` | JSON and human rendering | waveform access, clap parsing |
| `error` | all stable error variants | everything else |

### Key Dependencies

| Crate | Version | Purpose | Notes |
|-------|---------|---------|-------|
| `wellen` | ~0.20 | VCD and FST parsing | Core waveform dependency |
| `clap` | ~4 | CLI argument parsing | Derive API for declarative CLI definitions |
| `serde` | ~1 | Serialization | Used for machine-readable output structures |
| `serde_json` | ~1 | JSON output | Envelope rendering and schema export |
| `regex` | ~1 | Pattern matching | Shared filter support |
| `thiserror` | ~2 | Error derivation | Typed errors with explicit exit mapping |

Development dependencies include `assert_cmd`, `predicates`, `tempfile`, `insta`, and `criterion` to cover integration tests, fixture handling, snapshots, and benchmarks.

## Expression Engine Architecture

The `change` and `property` commands share a typed expression stack in `src/expr/`. The language contract itself lives in `contracts/expression_lang.md`; this section describes how the implementation is arranged.

The pipeline is:

input string → lexer → parser → AST → typed binding → runtime evaluation against sampled waveform values

The main components are:

- **Lexer and parser** for event and logical expression text.
- **AST and semantic types** that give the rest of the system a stable internal representation.
- **Typed binder and evaluator** that resolve names against waveform metadata and compute runtime results.
- **Waveform-backed host bridge** in `src/waveform/expr_host.rs` that exposes dump metadata and sampled values to the typed runtime without widening the public facade unnecessarily.

The current implementation status is:

- typed standalone event and logical runtimes are implemented under `src/expr/`,
- rich metadata is bridged into those runtimes through the waveform host adapter,
- production `change` and `property` execution reuses the same typed parser, binder, and evaluator path, and
- the older transitional compatibility parser has been retired.

## Error Handling Strategy

### Principles

- **Fail fast.** The first error stops execution.
- **Machine-parseable diagnostics.** Errors follow a stable `error: <category>: <message>` shape.
- **No panics in production paths.** Recoverable failures use `Result<T, WavepeekError>`.

### Exit Behavior

`src/error.rs` owns the process-level mapping from error variants to categories and exit codes.

- Exit code `0` means success.
- Exit code `1` means user-facing errors such as bad arguments, missing signals, or invalid expressions.
- Exit code `2` means file-level failures such as open or parse errors.

Warnings do not change the exit code.

## `change` Command Execution Architecture

`change` keeps one user-visible contract while choosing among several internal execution strategies.

Execution engines:

- **Baseline engine** for conservative low-overhead execution on small or simple workloads.
- **Fused engine** for broader candidate sets where more work can be shared across signals.
- **Edge-fast engine** for dense edge-trigger workloads that benefit from trigger-focused filtering.

The dispatcher chooses between those engines from internal workload estimates such as window size, candidate density, requested signal count, and trigger shape. This policy is intentionally internal and may evolve without changing the user contract.

The reason for the multi-engine design is simple: a single internal strategy could not keep latency consistently low across both tiny and large-window scenarios.

## Testing Strategy

### Test Levels

| Level | What | How | Fixtures |
|-------|------|-----|----------|
| Unit tests | Individual helpers and modules in `engine/`, `expr/`, and `waveform/` | `#[cfg(test)]` plus `cargo test` | Hand-crafted inline or small `.vcd` fixtures |
| Integration tests | Full CLI invocations | `assert_cmd` suites under `tests/` | Hand fixtures plus container-provisioned artifacts under `/opt/rtl-artifacts` |
| Expression tests | Parser, binder, and evaluator behavior | Unit tests in `src/expr/` plus integration-style suites in `tests/` | Pure string cases and structured expression fixtures |

### Fixture Strategy

wavepeek uses two fixture sources:

1. **Hand-crafted VCD fixtures** for edge cases, tiny examples, and direct unit coverage.
2. **Container-provisioned representative fixtures** for realistic integration and performance scenarios.

Runtime test execution does not fetch those larger fixtures dynamically; they are provisioned by the devcontainer and CI image.

### What Integration Tests Must Assert

- deterministic stdout behavior,
- exit codes,
- stderr formatting for error cases,
- `--json` payload conformance to the schema contract,
- human-output stability when a command-level contract explicitly fixes that formatting, and
- VCD/FST parity where equivalent queries should return the same result.

## Practical Ownership Boundaries

The architectural split matters for docs maintenance:

- `src/cli/`, `wavepeek --help`, and `wavepeek schema` are the exact CLI surface authority.
- `contracts/` documents the semantics that code and schema alone do not explain well enough.
- this file documents internals that help contributors change implementation safely without regrowing the old monolithic design doc.
