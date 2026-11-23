# Architecture

This file holds the internal engineering view of wavepeek: non-functional requirements, module boundaries, dependencies, execution strategy, and testing strategy. It does not restate the exact CLI flag surface. For command semantics and machine-output guarantees, use `skills/wavepeek/references/commands.md` and `skills/wavepeek/references/machine-output.md`.

## Non-Functional Requirements

### Performance

Performance is the highest implementation priority. Rust is used specifically to keep waveform parsing and query execution fast on large dumps.

Benchmarks are maintained through `bench/e2e/perf.py` for end-to-end CLI scenarios.

### Compatibility

The default VCD/FST tool is intended to stay OS-agnostic across Linux, macOS, and Windows. Optional FSDB and SHM support are Linux x86_64-only and use local Verdi or Xcelium readers; see `fsdb.md` and `shm.md`.

### Output Stability

Identical inputs must produce deterministic output.

### LLM Agent Integration

The repository ships agent-facing workflow assets plus deterministic `--json` and waveform `--jsonl` contracts so LLM clients can consume output without ad hoc parsing.

## Technical Architecture

### Technology Stack

| Component | Choice | Rationale |
|-----------|--------|-----------|
| Language | Rust stable (MSRV 1.93) | Performance, memory safety, and predictable resource use on large dumps |
| CLI framework | `clap` derive API | Self-documenting command definitions with compile-time validation |
| Waveform parsing | `ondas` | Unified VCD/FST access and optional SDK-backed FSDB/SHM access |
| Serialization | `serde` + `serde_json` | Standard JSON and JSONL rendering for machine output |
| Pattern matching | `regex` | Shared filtering surface for hierarchy and signal discovery |
| Error handling | `thiserror` | Typed error enums without runtime boxing |
| Browser adapter | `wasm-bindgen` + Web Worker | Reuses the Rust CLI and isolates cancellable waveform work without a frontend framework |
| Build automation | Cargo + just | Cargo owns compilation; the root `justfile` exposes repository quality gates |

### High-Level Execution Layers

wavepeek is organized as three execution layers plus two shared support modules. Data flows top-down: the CLI parses arguments, the engine executes command logic, waveform commands query the waveform layer, the skill helper extracts embedded package assets, and the output module renders results.

1. **CLI layer** (`src/cli/`) parses arguments, owns help text, normalizes clap errors, and dispatches typed command structs.
2. **Engine layer** (`src/engine/`) implements command behavior, shared time handling, shared value formatting, expression-runtime helpers, and command dispatch.
3. **Waveform layer** (`src/waveform/`) adapts Ondas file opening, metadata, exact hierarchy, point/batch samples, and activity queries. Ondas supplies VCD/FST access in default builds and SDK-backed FSDB/SHM access through the `fsdb` and `shm` features. Native reader setup lives in `fsdb.md` and `shm.md`.
4. **Embedded skill runtime** (`src/skill.rs`) extracts the packaged agent skill from repository assets.
5. **Output module** (`src/output.rs`) owns stdout rendering for human mode, strict JSON result and fatal values, and JSONL stream records.
6. **Browser adapter** (`src/browser.rs`) supplies explicit argv, output buffers, and one invocation-scoped VCD/FST byte source to the same CLI and waveform layers. It rejects FSDB, SHM, `skill`, and extraction `--source` before dispatch.

Key architectural consequences:

- Execution is stateless. Every command opens the dump and runs once; the native wrapper exits, while the browser worker returns stdout, stderr, and status. The plain-JavaScript terminal keeps only in-tab command navigation and a bounded newest-first visible transcript.
- Local preview composes the separately generated current Playground and documentation under `/wavepeek/` and `/wavepeek/latest/`; production uses the same paths while Mike retains historical documentation versions.
- Shared `src/logic.rs` defines four-state interpretation for expressions, edges and bit-output sampling. Renderers consume normalized samples; raw expression samples and enum keys retain their original states.
- Ondas owns format-specific reading and optional native reader access behind the waveform facade. The range engines share the expression query adapter.
- Ondas owns hierarchy identity and selector resolution. The adapter traverses public hierarchy views for listing and retains resolved Signal handles for reading. Selected declaration metadata and explicit projections define expression types; canonical paths are formatted for output. VCD split-vector normalization belongs to Ondas.
- `value` submits all selected signals to one Ondas point query for a single requested time, or one `samples_at` query for multiple times. Both paths preserve request order and duplicates. Range consumers read through the shared expression query adapter without saved histories or candidate grids.
- `property` binds drivers, guards and evaluation operands into one ordered Ondas selection. It evaluates candidates during `Selection::query`; a callback host reads only completed `t` and absolute `t - 1`. Transition captures first sample the window start once to establish their baseline, including quiet windows.
- `extract generic` and its APB/ATB/AXI/AXI-Stream adapters bind all sources into one ordered Ondas selection and query. Drivers produce candidates; each group checks its completed-tick event and `iff`, then reads `when` and payload at absolute `t - 1`. Rows retain source declaration order, aliases, projections, and repeated payload fields. A complete matching row beyond the limit establishes truncation.
- The skill helper keeps its source of truth in packaged files instead of duplicated Rust string tables.
- JSON and JSONL contracts are covered by direct serialization and command-runtime tests.

### Module Structure

```text
src/
├── lib.rs               # Crate entrypoint (`run_cli`) + module ownership
├── main.rs              # Thin binary wrapper returning `wavepeek::main_exit_code()`
├── browser.rs           # WASM adapter for argv, output buffers, status, and waveform bytes
├── cli/                 # CLI layer: argument definitions, help text, dispatch
│   ├── mod.rs           # Top-level CLI, early output selection, failure reporting, output handoff
│   ├── limits.rs        # Shared bounded-output flag parsing (`--max`, `--max-depth`)
│   ├── info.rs          # `info` command args + clap help
│   ├── scope.rs         # `scope` command args + clap help
│   ├── signal.rs        # `signal` command args + clap help
│   ├── value.rs         # `value` command args + clap help
│   ├── change.rs        # `change` command args + clap help
│   ├── property.rs      # `property` command args + clap help
│   ├── extract.rs       # `extract` command namespace and subcommand args + clap help
│   └── skill.rs         # `skill` helper command args + clap help
├── engine/              # Business logic per command
│   ├── mod.rs           # Command dispatch + shared result types
│   ├── info.rs          # Dump metadata extraction
│   ├── scope.rs         # Hierarchy traversal with depth/filter
│   ├── signal.rs        # Signal listing within scope
│   ├── value.rs         # Value extraction at time point
│   ├── change.rs        # Value-change tracking through the shared query
│   ├── expr_runtime.rs  # Shared typed-expression binding/evaluation helpers
│   ├── time.rs          # Shared time token parsing/validation/alignment helpers
│   ├── value_format.rs  # Shared Verilog literal formatting helpers
│   ├── property.rs      # Property runtime entrypoint and capture-mode execution
│   ├── extract.rs       # Generic event-row extraction runtime
│   ├── ahb.rs           # Stateful AHB address/data pipeline extraction
│   ├── apb.rs           # Stateless APB profile mapping and event adaptation
│   ├── atb.rs           # Stateless ATB profile mapping and event adaptation
│   ├── axi.rs           # Stateless AXI-family profile mapping and transfer adaptation
│   ├── axistream.rs     # AXI-Stream profile adapter over generic extraction
│   ├── signal_mapping.rs # Protocol-neutral standard-name matching for adapters
│   └── skill.rs         # Packaged agent skill extraction runtime
├── skill.rs             # Embedded package asset traversal and materialization
├── contract/            # Runtime JSON and JSONL data transfer objects
│   ├── common.rs        # Shared diagnostics, paths, times, values, and kind aliases
│   ├── output.rs        # JSON envelope and command payload structures
│   └── stream.rs        # JSONL begin/data/diagnostic/end structures
├── expr/                # Expression engine shared by `change`, `property`, and `extract`
│   ├── mod.rs           # Public typed facade for parsing/binding/evaluation
│   ├── ast.rs           # Spanned expression AST types
│   ├── diagnostic.rs    # Parse/semantic/runtime diagnostic contract
│   ├── lexer.rs         # Spanned tokenizer for event/logical parsing
│   ├── parser.rs        # Strict typed parser
│   ├── host.rs          # Host trait + signal/type/value bridge types
│   ├── sema.rs          # Typed binders for event and logical expressions
│   └── eval.rs          # Typed event matcher and logical evaluator
├── waveform/            # Backend-neutral waveform facade plus concrete backends
│   ├── mod.rs           # Public facade, backend dispatch, and query helpers
│   ├── types.rs         # Shared waveform metadata, signal, sample, and backend-facing types
│   ├── ondas_backend.rs # Exact hierarchy, logical handles, point/batch samples, and queries
│   ├── ondas_tests.rs   # Waveform facade regression tests
│   ├── ondas_fsdb_tests.rs # Feature-gated SDK-backed regression tests
│   ├── ondas_error.rs   # Typed reader errors mapped to CLI categories
│   ├── expr_host.rs     # Waveform-backed binding and point-expression host
│   └── expr_query.rs    # Selection mapping and expression values for one query tick
├── output.rs            # Shared output formatting (human, JSON envelope, JSONL)
└── error.rs             # `WavepeekError` enum and exit mapping
```

### Separation of Concerns

| Module | Knows about | Does not know about |
|--------|-------------|---------------------|
| `cli/` | clap, dispatch, help text | waveform parsing internals, output serialization details |
| `engine/` | domain logic, waveform API, shared semantics helpers | clap parsing flow |
| `expr/` | expression AST, types, evaluation | CLI, output formatting, `ondas` |
| `waveform/` | Ondas access, exact hierarchy paths, query caching | CLI behavior, output formatting |
| `output` | JSON and human rendering | waveform access, clap parsing |
| `error` | all stable error variants | everything else |

### Key Dependencies

| Crate | Version | Purpose | Notes |
|-------|---------|---------|-------|
| `ondas` | 2.2.0 | VCD/FST parsing and optional FSDB/SHM access | Native features forward to `ondas/fsdb-lib` and `ondas/shm-lib`; VCD/FST support byte input for WASM |
| `clap` | ~4 | CLI argument parsing | Derive API for declarative CLI definitions |
| `serde` | ~1 | Serialization | Used for machine-readable output structures |
| `serde_json` | ~1 | JSON output | Envelope and JSONL record rendering |
| `regex` | ~1 | Pattern matching | Shared filter support |
| `thiserror` | ~2 | Error derivation | Typed errors with explicit exit mapping |

Development dependencies include `assert_cmd`, `predicates`, `tempfile`, and `insta` to cover integration tests, fixture handling, and snapshots.

## Expression Engine Architecture

The `change`, `property`, and `extract` commands share a typed expression stack in `src/expr/`. The language contract itself lives in `skills/wavepeek/references/boolean-expressions.md`; this section describes how the implementation is arranged.

The pipeline is:

input string → lexer → parser → AST → typed binding → runtime evaluation against sampled waveform values

The main components are:

- **Lexer and parser** for event and logical expression text.
- **AST and semantic types** that give the rest of the system a stable internal representation.
- **Typed binder and evaluator** that resolve names against waveform metadata and compute runtime results.
- **Waveform-backed host bridge** in `src/waveform/expr_host.rs` that exposes dump metadata and sampled values to the typed runtime without widening the public facade unnecessarily. The bridge retains typed read failures until the engine completes that expression evaluation; the expression module remains independent of CLI error categories.
- **Query host** in `src/waveform/expr_query.rs` that uses frozen binding metadata and selection positions. Its value cache lasts for one callback and contains only `t` and `t - 1`. Nested read failures and application errors stop the outer query through a typed `Break` payload; reader errors use the existing waveform error mapping. No callback reenters `Waveform` for point reads.

The current implementation status is:

- typed standalone event and logical runtimes are implemented under `src/expr/`,
- rich metadata is bridged into those runtimes through the waveform host adapter,
- production `change`, `property`, and `extract` execution reuses the same typed parser, binder, and evaluator path,
- `extract axi` and `extract axistream` build protocol-specific mappings and plans, then delegate waveform traversal, event matching, pre-edge evaluation, limits, and diagnostics to `src/engine/extract.rs`, and
- the older transitional compatibility parser has been retired.

## Error Handling Strategy

### Principles

- **Fail fast.** The first error stops execution.
- **Stable failures and diagnostics.** Human process failures follow `fatal: <category>: <message>`; JSON and JSONL failures use the typed fatal records defined in `skills/wavepeek/references/machine-output.md`. Successful command diagnostics use typed JSON objects and coded human lines such as `warning[WPK-W0002]: <message>`.
- **No panics in production paths.** Recoverable failures use `Result<T, WavepeekError>`.

### Exit Behavior

`src/error.rs` owns the process-level mapping from error variants to categories and exit codes.

- Exit code `0` means success.
- Exit code `1` means user-facing errors such as bad arguments, missing signals, or invalid expressions.
- Exit code `2` means file-level failures such as open or parse errors.

Non-fatal diagnostics do not change the exit code.

## Waveform Access

Metadata and hierarchy commands use Ondas metadata and exact hierarchy components. `value` sends all selected handles
to one `samples` call for a single time, or one `samples_at` call for multiple
times. Request order and duplicates are preserved.

Range engines use the shared expression query adapter. AHB queries
from dump start through the upper bound to preserve pipeline warmup before the
output range. Protocol state belongs to its walker; sampled values belong to one
callback. APB, ATB, AXI, and AXI-Stream use generic extraction. Browser byte input
enters the same facade and command engines through `ondas::open_bytes`.

CLI list and single-entry normalization share one lexical walk. It preserves
selector bytes and required escaped-identifier terminators while trimming outer
padding. `SignalSelector::parse` parses the selector. `Hierarchy::resolve_selector` resolves it with its component scope prefix.
The result retains the selected declaration, Signal, and explicit normalized
projection, including a full-width slice. WavePeek applies expression type policy
and formats reusable selectors from that result. Listing omits ambiguous paths according to
Ondas exact lookup. VCD split declarations are normalized by Ondas before hierarchy
publication; other formats retain the hierarchy supplied by the library.

Binding caches map operand names to expression handles and selected declaration
types. Query values are cached only for the current callback.

The application stops callbacks through `Break`, including typed sink and
expression failures. Exact result completeness can require a matching row beyond
the output limit. Reader setup and initial state may read a prefix; callback-local
state does not bound reader or SDK memory. Owned JSON is all-or-error; JSONL bytes
already written remain visible if a later failure ends the stream with `fatal`.

## `change` Command Execution Architecture

`change` uses one ordered Ondas selection and activity query for native and
pre-edge sampling. Drivers include explicit event operands and, for wildcard
triggers, the selected logical signals. Guard and payload operands share the
selection without adding candidate times. Event evaluation filters candidates
against completed `t` and absolute `t - 1` states through the shared query host.

Sparse rows compare against samples at the range start, then against emitted
samples. Delta columns follow the same selected values. Only emitted rows update
comparison state and counters. After the row limit, matching lookahead finds one
more eligible row or reaches the end of the query.

Payload conversion preserves Missing, maps logic through WavePeek's four-state
policy, and represents raw events as zero-width values. Real and string payloads
retain the existing unsupported-value errors. Typed application and sink errors
stop the outer query through the shared adapter.

For `--jsonl`, `change` emits snapshots through a sink during the query. Human and
`--json` output use the same sink interface with a collector. `property` uses the
same output pattern for captured rows.

## Protocol Extraction Architecture

`src/engine/axi.rs` maps supported AXI-family profiles and ready/valid channels into the protocol-neutral runtime in `src/engine/extract.rs`. `src/engine/ahb.rs` is a dedicated stateful walker because an accepted AHB address phase completes on a later edge, can remain pending across wait states, and requires warm-up before a lower time bound. The stateless adapters use the generic extraction query; AHB binds the clock and mapped fields into one query. Clock candidates must pass a rising-edge check on completed `t` and absolute `t - 1`; mapped fields are sampled at `t - 1`. Its walker warms up from dump start, preserves pending phases and reset state across callbacks, and records the initial data phase before in-range edges. Quiet windows retain that context without synthetic edges. Both engines use the shared waveform facade, pre-edge sampling model, time/limit helpers, contract DTOs, and output sinks; they do not share a speculative protocol framework.

## Testing Strategy

### Test Levels

| Level | What | How | Fixtures |
|-------|------|-----|----------|
| Unit tests | Individual helpers and modules in `engine/`, `expr/`, and `waveform/` | `#[cfg(test)]` plus `cargo test` | Hand-crafted inline or small `.vcd` fixtures |
| Integration tests | Full CLI invocations | `assert_cmd` suites under `tests/` | Hand fixtures plus container-provisioned Ondas fixtures under `ONDAS_FIXTURES_DIR` |
| Expression tests | Parser, binder, and evaluator behavior | Unit tests in `src/expr/` plus integration-style suites in `tests/` | Pure string cases and structured expression fixtures |

### Fixture Strategy

wavepeek uses two fixture sources:

1. **Hand-crafted VCD fixtures** for edge cases, tiny examples, and direct unit coverage.
2. **Container-provisioned representative fixtures** for realistic integration and performance scenarios.

Runtime test execution does not fetch those larger fixtures dynamically; they are provisioned by the shared devcontainer image.

### What Integration Tests Must Assert

- deterministic stdout behavior,
- exit codes,
- stderr formatting for error cases,
- direct `--json` envelope and payload shape assertions,
- direct `--jsonl` record-shape and stream-order invariants,
- human-output stability when a command-level contract explicitly fixes that formatting, and
- VCD/FST parity where equivalent queries should return the same result.

## Practical Ownership Boundaries

The architectural split matters for docs maintenance:

- `src/cli/`, `wavepeek --help`, and `wavepeek <command> --help` are the exact CLI surface authority.
- `skills/wavepeek/references/machine-output.md` and direct runtime tests define machine-output behavior.
- Flat Markdown files under `skills/wavepeek/references/` document user-visible semantics, while `references/docs.json` defines website navigation.
- this file documents internals that help contributors change implementation safely without regrowing a monolithic design doc.
