# Design Document

## 1. Overview

### 1.1 Vision
wavepeek is a command-line tool for RTL waveform inspection. It provides deterministic, machine-friendly output and a minimal set of primitives that compose into repeatable debug recipes.

### 1.2 Problem Statement

**The waveform access gap for LLM agents**

In RTL design and verification, waveforms are the primary debugging artifact. Unlike software where logs, stack traces, and debugger output are mostly textual and sequential, RTL debugging requires analyzing dense temporal data across hierarchical module structures. Waveforms are a high-bandwidth information channel — humans process them visually, leveraging the eye's capacity for pattern recognition.

**Why waveforms matter:**
- Bug reports come with attached dumps (reproducing from scratch can take days)
- Engineers ask questions about waveforms and want to delegate analysis
- Waveforms are the de-facto artifact for RTL debugging — without them, little can be done

**The gap:** There is no tool to "grep" waveforms — to extract signal information, values, and temporal relationships via short, composable commands. GUI viewers (GTKWave) work for humans but cannot be automated. This leaves LLM agents blind to simulation results.

VCD is text and therefore natively readable by LLM agents, but real-world dumps are typically very large. Directly exploring raw VCD content is difficult and inefficient: it quickly consumes context window budget and makes iterative analysis expensive. Agents need a structured, deterministic, LLM-friendly derivative view of waveform data rather than raw dump text.

### 1.3 Target Users

**Primary: LLM agents**
- Agentic systems for RTL debugging and verification
- Need structured, deterministic output for reasoning
- Require composable primitives for tool chaining

**Secondary: CI/CD and automation**
- Automated regression pipelines
- Post-simulation analysis scripts
- Waveform-based assertions in CI

**Non-target: Interactive human debugging**
- Humans are not the primary audience
- The tool can be used for scripting and quick queries
- Interactive waveform exploration will still be done in GUI viewers

### 1.4 Design Principles

1. **LLM-first** — Output formats, command structure, and error messages are designed for machine consumption
2. **Self-documenting I/O** — Commands read as unambiguous descriptions of what they do. Human-readable output is the default UX, while strict machine output is explicit via `--json`.
3. **Composable commands** — Unix philosophy: do one thing well, combine via pipes. Command names are unambiguous first, short second
4. **Deterministic output** — Same input always produces same output (no timestamps, random IDs, etc.)
5. **Stable formats** — JSON output uses an explicit versioned schema URL (`$schema`) when `--json` is requested. Human-readable output remains intentionally flexible and is the default for waveform commands.
6. **Minimal footprint** — Fast startup, low memory, no background processes

---

## 2. Scope

### 2.1 In Scope
- VCD/FST dump file support
- Signal discovery: list, search, hierarchy navigation
- Value extraction over time ranges
- Event search (find time when condition holds)
- Stateless CLI (no sessions, no caching, no background processes)

### 2.2 Out of Scope
- GUI/TUI waveform viewer
- Real-time waveform streaming
- Live simulator connections
- Waveform diffing/comparison

### 2.3 Future Considerations
- MCP server for LLM agent integration
- Other formats (FSDB, VPD, WLF)

---

## 3. Functional Requirements

### 3.1 Supported File Formats

- VCD (Value Change Dump)
- FST (Fast Signal Trace)

### 3.2 CLI Commands

#### General conventions

- **No positional arguments.** All arguments are named flags for self-documenting commands.
  All commands that operate on a waveform dump require `--waves <file>`. Commands that do not
  read a dump (e.g., `schema`) do not accept `--waves`.
- **Standalone help contract.** Help text is a first-class CLI contract. `wavepeek` (no args), `wavepeek -h`, and `wavepeek --help` are byte-identical top-level entry points, and each shipped subcommand makes `-h` byte-identical to `--help` while documenting command semantics, defaults/requiredness, boundary rules, error-category guidance, and output shape.
- **Bounded output.** All commands have bounded output by default (to avoid flooding LLM context).
  Boundedness is achieved via one or more of: `--max`, `--first`/`--last`, input size (e.g., number
  of `--signals`), or inherently finite output (e.g., `schema`). When list output is truncated due
  to `--max`, a warning is emitted.
- **Bounded recursion.** Recursive commands have `--max-depth` with a default of 5.
- **Output format.** Default output is human-readable for waveform commands.
  Strict machine output is enabled explicitly with `--json`. The `schema` command is a special case and always emits one JSON Schema document to stdout.
- **JSON envelope (`--json` mode).** On success, JSON output is a single object:

  ```json
  {
    "$schema": "https://raw.githubusercontent.com/kleverhq/wavepeek/v<version>/schema/wavepeek.json",
    "command": "<command>",
    "data": {},
    "warnings": []
  }
  ```

  Notes:
  - `$schema` is always serialized literally as `$schema` and points to the canonical schema artifact for the current tool version.
  - `command` is the subcommand name and can be used to discriminate the shape of `data`.
  - `data` is an object for scalar outputs (e.g., `info`) and an array for list-like outputs.
  - `warnings` is an array of free-form strings. In human mode, warnings are printed to stderr.
  - On error, stdout is empty; stderr contains `error: <category>: <message>` (see §5.6).
- **Time format.** All time values require explicit units: `zs`, `as`, `fs`, `ps`, `ns`, `us`, `ms`, `s`.
  The numeric part must be an integer (e.g., `2000ps`, `15ns`).
  Bare numbers without units are rejected.
- **Time normalization.** All parsed times are converted to the dump's time unit.
  Output timestamps are printed as normalized integer counts in dump `time_unit` units (e.g., `2000ps`).
  If a provided time cannot be represented exactly in dump precision, it is an error.
- **Time ranges.** Commands that operate on time ranges use `--from` and `--to`.
  Both are optional: `--from` + `--to` defines a window, only `--from` means
  from that point to end of dump, only `--to` means from start to that point,
  neither means the entire dump.
  Range boundaries are inclusive.

#### 3.2.0 `schema` — Canonical JSON schema export

Outputs the canonical JSON schema document for wavepeek machine output contracts.

```
wavepeek schema
```

**Behavior:**
- Accepts no command-specific flags or positional arguments.
- Writes exactly one JSON Schema document to stdout (no envelope wrapping).
- Output is deterministic and byte-stable.
- Output bytes match the canonical repository artifact at `schema/wavepeek.json`.

#### 3.2.1 `info` — Dump metadata

Outputs basic metadata about the waveform dump.

```
wavepeek info --waves <file> [--json]
```

**Parameters:**
| Parameter | Default | Description |
|-----------|---------|-------------|
| `--waves <file>` | required | Path to VCD/FST file |
| `--json` | off | Strict JSON envelope output |

**Behavior:**
- Default output: human-readable metadata summary.
- Summary includes: time unit, start and end time of the dump
- `--json` prints strict JSON envelope output.

**Examples:**
```bash
# Default human-readable output
wavepeek info --waves dump.vcd

# Strict JSON envelope
wavepeek info --waves dump.vcd --json
```

#### 3.2.2 `scope` — Hierarchy exploration

Outputs a flat list of hierarchy scopes, recursively traversing the hierarchy.

```
wavepeek scope --waves <file> [--max <n|unlimited>] [--max-depth <n|unlimited>] [--filter <regex>] [--tree] [--json]
```

**Parameters:**
| Parameter | Default | Description |
|-----------|---------|-------------|
| `--waves <file>` | required | Path to VCD/FST file |
| `--max <n\|unlimited>` | 50 | Maximum number of entries in output; `unlimited` disables truncation |
| `--max-depth <n\|unlimited>` | 5 | Maximum traversal depth; `unlimited` disables depth truncation |
| `--filter <regex>` | `.*` | Filter by full path (regex) |
| `--tree` | off | Render hierarchy as an indented tree in human mode |
| `--json` | off | Strict JSON envelope output |

**Behavior:**
- Outputs flat list of scope paths with metadata
- Includes all scope kinds available in hierarchy data (not only modules)
- Ordering: pre-order depth-first traversal; children at each scope are visited in lexicographic order
- Default output: human-readable list mode.
- `--tree` switches human output to visual hierarchy rendering.
- `--json` returns strict JSON envelope with a flat `data` array.
- Invalid regex is an `args` error.
- `--max 0` is an `args` error.
- `--max unlimited` disables count truncation and emits warning `limit disabled: --max=unlimited`.
- `--max-depth unlimited` disables depth truncation and emits warning `limit disabled: --max-depth=unlimited`.
- If both limits are `unlimited`, warnings are emitted in deterministic order: `--max` then `--max-depth`.
- Each item has:
  - `path`: full scope path (string)
  - `depth`: integer depth (root = 0)
  - `kind`: parser-native scope kind alias (string)
- If results exceed `--max`, output is truncated and a warning is emitted.
- Legacy `tree` command name is not supported.

**Examples:**
```bash
# List all scopes (default human output)
wavepeek scope --waves dump.vcd

# Find ALU-related scopes
wavepeek scope --waves dump.vcd --filter ".*alu.*"

# Explore 2 levels deep with visual tree rendering
wavepeek scope --waves dump.vcd --max-depth 2 --tree

# Get all scopes
wavepeek scope --waves dump.vcd --max 1000

# Disable both count and depth limits explicitly
wavepeek scope --waves dump.vcd --max unlimited --max-depth unlimited
```

#### 3.2.3 `signal` — Signal listing

Lists signals within a specific scope with their metadata.

```
wavepeek signal --waves <file> --scope <path> [--recursive] [--max-depth <n|unlimited>] [--max <n|unlimited>] [--filter <regex>] [--abs] [--json]
```

**Parameters:**
| Parameter | Default | Description |
|-----------|---------|-------------|
| `--waves <file>` | required | Path to VCD/FST file |
| `--scope <path>` | required | Exact scope path (e.g., `top.cpu`) |
| `--recursive` | off | Include nested child scopes under `--scope` |
| `--max-depth <n\|unlimited>` | 5 (when recursive) | Maximum recursion depth below `--scope` (requires `--recursive`); `unlimited` disables depth truncation |
| `--max <n\|unlimited>` | 50 | Maximum number of entries in output; `unlimited` disables truncation |
| `--filter <regex>` | `.*` | Filter by signal name (regex) |
| `--abs` | off | Show full signal paths in human mode |
| `--json` | off | Strict JSON envelope output |

**Behavior:**
- Without `--recursive`, lists only signals directly in the specified scope.
- With `--recursive`, traverses child scopes depth-first in deterministic lexicographic scope order.
- `--max-depth` is valid only with `--recursive`; depth `0` means only the selected scope.
- Human mode defaults to short signal names in non-recursive mode.
- Human mode in recursive mode prints paths relative to `--scope` (for example, `cpu.valid`); `--abs` always shows canonical full paths.
- Output fields (JSON): `name`, `path`, `kind`, `width` (`path` stays full/canonical in JSON mode)
- Signal ordering is deterministic: per visited scope sort by `(name, path)`.
- Default output: human-readable listing.
- `--json` returns strict JSON envelope with `data` as an array of objects.
- Invalid regex is an `args` error.
- `--max-depth` without `--recursive` is an `args` error.
- `--max 0` is an `args` error.
- `--max unlimited` disables count truncation and emits warning `limit disabled: --max=unlimited`.
- `--max-depth unlimited` (with `--recursive`) disables depth truncation and emits warning `limit disabled: --max-depth=unlimited`.
- If unlimited-limit warnings and legacy truncation warnings both apply, unlimited warnings are emitted first.
- If results exceed `--max`, output is truncated and a warning is emitted.
- Legacy `signals` command name is not supported.

**Examples:**
```bash
# List signals in top.cpu (default human, short names)
wavepeek signal --waves dump.vcd --scope top.cpu

# Find clock signals
wavepeek signal --waves dump.vcd --scope top.cpu --filter ".*clk.*"

# Human output with full paths
wavepeek signal --waves dump.vcd --scope top.cpu --abs

# Recursively list nested signals under top (relative display paths in human mode)
wavepeek signal --waves dump.vcd --scope top --recursive --max-depth 2

# Disable recursion-depth limit while keeping count bound
wavepeek signal --waves dump.vcd --scope top --recursive --max 100 --max-depth unlimited
```

#### 3.2.4 `at` — Value extraction at time point

Gets signal values at a specific time point.

```
wavepeek at --waves <file> --time <time> [--scope <path>] --signals <names> [--abs] [--json]
```

**Parameters:**
| Parameter | Default | Description |
|-----------|---------|-------------|
| `--waves <file>` | required | Path to VCD/FST file |
| `--time <time>` | required | Time point with units (e.g., `1337ns`, `10us`) |
| `--scope <path>` | — | Scope for short signal names |
| `--signals <names>` | required | Comma-separated signal names |
| `--abs` | off | Show canonical paths in human mode |
| `--json` | off | Strict JSON envelope output |

**Modes:**
- `--signals clk,data` (no scope) — names are full paths
- `--scope top.cpu --signals clk,data` — names are short and resolved relative to scope

**Behavior:**
- Outputs signal values at specified time
- Default output: human-readable value summary.
- Human mode shows compact lines: `@<time>` and `<display> <value>` per signal.
- Human mode uses exact `--signals` tokens as display names by default; `--abs` switches display names to canonical paths.
- `--json` outputs JSON envelope with `data` as an object:
  - `time`: normalized time string in dump `time_unit`
  - `signals`: array of `{ path, value }` in the same order as `--signals` (`path` is always canonical absolute path)
- Values are emitted as Verilog literals: `<width>'h<digits>` (including `x`/`z`).
- Fail fast: error if any signal not found

**Examples:**
```bash
# Get values by full paths
wavepeek at --waves dump.vcd --time 100ns --signals top.cpu.clk,top.cpu.data

# Get values relative to scope (shorter)
wavepeek at --waves dump.vcd --time 100ns --scope top.cpu --signals clk,data,valid

# Strict JSON envelope
wavepeek at --waves dump.vcd --time 100ns --scope top.cpu --signals clk --json
```

#### 3.2.5 `change` — Value changes over time range

Outputs delta snapshots of signal values over a time range using one event-trigger model.

```
wavepeek change --waves <file> [--from <time>] [--to <time>] [--scope <path>] --signals <names> [--when <event_expr>] [--max <n|unlimited>] [--abs] [--json]
```

**Parameters:**
| Parameter | Default | Description |
|-----------|---------|-------------|
| `--waves <file>` | required | Path to VCD/FST file |
| `--from <time>` | start of dump | Start time with units (e.g., `1us`) |
| `--to <time>` | end of dump | End time with units (e.g., `2us`) |
| `--scope <path>` | — | Scope for short signal/trigger names |
| `--signals <names>` | required | Comma-separated signal names |
| `--when <event_expr>` | `*` | Event expression (`*`, named, edge, union) |
| `--max <n\|unlimited>` | 50 | Maximum number of snapshot rows; `unlimited` disables truncation |
| `--abs` | off | Canonical paths in human output |
| `--json` | off | Strict JSON envelope output |

**Supported `--when` forms:**
- Non-edge: `*` (any change in resolved `--signals`) and `<name>` (any change of that signal)
- Edge: `posedge <name>`, `negedge <name>`, `edge <name>`
- Union: `event or event`, `event, event` (exact synonyms)
- Optional staged clause: `event iff logical_expr`

Current delivery parses `iff` and preserves its binding (`iff` applies to the immediately preceding event term),
but runtime evaluation of `logical_expr` is intentionally deferred and returns
`error: args: iff logical expressions are not implemented yet`.

**Behavior:**
- Name resolution follows `at`: without `--scope`, tokens are canonical paths; with `--scope`, tokens are short names relative to that scope.
- Scoped mode rejects canonical full-path tokens for both `--signals` and `--when` names.
- Time tokens require explicit units and exact alignment to dump precision.
- `--from` defines a baseline checkpoint: values are sampled/initialized at `--from` (or dump start when omitted), and no row is emitted at that exact timestamp.
- Candidate timestamps come from `--when` inside inclusive `[--from, --to]`; rows can only be emitted for timestamps strictly greater than the baseline checkpoint.
- Candidate evaluation is internally reduced to event-relevant timestamps (tracked/event signal change points) instead of scanning every dump timestamp, while preserving observable behavior.
- Each emitted row is delta-only: emit only when sampled `--signals` changed relative to the last known sampled state strictly before that timestamp.
- Delta comparison always uses sampled state strictly before the candidate timestamp in the underlying dump order (not merely before the previous emitted candidate).
- If an event fires but sampled `--signals` did not change, no row is emitted.
- Per-signal baseline initialization is lazy: missing prior sampled state is initialized at the timestamp and excluded from delta decision for that timestamp.
- Edge detection uses previous value strictly before timestamp and SystemVerilog-style classification on LSB only, with nine-state values (`h/u/w/l/-`) normalized to `x`.
- For FST dumps, implementation may route candidate-time discovery through `wellen::stream` filter pushdown (`start`, `end`, `signals`) for heavy windows while preserving stateless single-process execution and fallback behavior.
- Default output: human-readable snapshot list.
- Human rows are one line per snapshot: `@<time> <display_1>=<value_1> <display_2>=<value_2> ...`.
  `--abs` switches `<display_i>` from requested token to canonical path.
- JSON rows are `{ "time": <normalized>, "signals": [{"path": ..., "value": ...}, ...] }`.
- Values are Verilog literals: `<width>'h<digits>` with lowercase hex and `x`/`z` support.
- If no rows are emitted, `data` is empty and warning is exactly `no signal changes found in selected time range`.
- `--max 0` is an `args` error.
- `--max unlimited` disables row truncation and emits warning `limit disabled: --max=unlimited`.
- If `--max unlimited` is used together with any existing warning (for example empty-result warning), the unlimited warning is emitted first.
- If results exceed `--max`, output is truncated and a warning is emitted.
- Legacy `changes` command name is not supported.

**Examples:**
```bash
# Default trigger (`*`): any sampled signal change
wavepeek change --waves dump.vcd --from 1us --to 2us --signals top.cpu.clk,top.cpu.data

# Named non-edge trigger with scope-relative names
wavepeek change --waves dump.vcd --from 1us --to 2us --scope top.cpu --signals data,valid --when "data"

# Union trigger with comma/or synonyms
wavepeek change --waves dump.vcd --from 1us --to 2us --scope top.cpu --signals clk1 --when "posedge clk1, posedge clk2"

# Strict JSON envelope
wavepeek change --waves dump.vcd --from 1us --to 2us --scope top.cpu --signals clk --when "edge clk" --json

# Disable row truncation explicitly
wavepeek change --waves dump.vcd --signals top.sig --max unlimited
```

#### 3.2.6 `when` — Event search

Status: planned, not implemented in the current release. Current runtime behavior is
`error: unimplemented: when command execution is not implemented yet`.

Finds clock cycles where a boolean expression evaluates to true.
Expression is evaluated on every posedge of the specified clock.

```
wavepeek when --waves <file> --clk <name> [--from <time>] [--to <time>] [--scope <path>] --cond <expr> [--first [<n>]] [--last [<n>]] [--max <n|unlimited>] [--json]
```

**Parameters:**
| Parameter | Default | Description |
|-----------|---------|-------------|
| `--waves <file>` | required | Path to VCD/FST file |
| `--clk <name>` | required | Clock signal for posedge sampling |
| `--from <time>` | start of dump | Start of time range |
| `--to <time>` | end of dump | End of time range |
| `--scope <path>` | — | Scope for short signal/clock names in expression |
| `--cond <expr>` | required | Boolean expression in expression language |
| `--first [<n>]` | 1 if flag present | Return first N matches |
| `--last [<n>]` | 1 if flag present | Return last N matches |
| `--max <n\|unlimited>` | 50 | Maximum matches (when neither --first nor --last); `unlimited` disables truncation |
| `--json` | off | Strict JSON envelope output |

**Behavior:**
- Expression is evaluated at every posedge (clean `0 -> 1` transition) of `--clk`.
  Transitions involving `x`/`z` do not count as posedge.
- Outputs timestamps where the expression is true after 2-state casting (unknown `x` counts as false).
- Default output: human-readable timestamp list.
- `--json` outputs JSON envelope with `data` as an array of objects: `{ "time": "<normalized>" }`.
- `--clk` follows same scope convention as `--signals`: short name with `--scope`, full path without
- **No qualifier:** return all matches up to `--max`
- **`--first`:** return first N matches (default N=1 if no value given)
- **`--last`:** return last N matches (default N=1 if no value given)
- `--first` and `--last` are mutually exclusive — error if both specified
- `--max` is not allowed together with `--first` or `--last`
- `--max unlimited` is accepted by CLI parsing; runtime remains unimplemented.
- Fail fast: error if any signal in expression not found
- If no matches, `data` is empty and a warning is emitted.
- If matches exceed `--max` (when neither `--first` nor `--last` is used), output is truncated and a warning is emitted.

**Expression language (MVP):**

Operands:
- Signal names (scope-aware): `valid`, `data`, `top.cpu.data`
- Hex literal: `0xff`
- Binary literal: `0b1010`
- Decimal literal: `42`
- All values are unsigned

Operators (by precedence, highest first):

| Precedence | Operator | Description |
|------------|----------|-------------|
| 1 | `!` | Logical NOT |
| 2 | `<`, `>`, `<=`, `>=` | Comparison (unsigned) |
| 3 | `==`, `!=` | Equality |
| 4 | `&&` | Logical AND |
| 5 | `\|\|` | Logical OR |

Grouping: `(`, `)`

Truthy semantics: signal used without comparison operator is truthy if non-zero
(e.g., `valid` means `valid != 0`).

4-state semantics (MVP):
- Signal values are treated as 4-state bit vectors (`0`/`1`/`x`/`z`). Operators follow SystemVerilog-like
  4-state semantics and may produce an unknown boolean result `x`.
- `&&` and `||` use short-circuit evaluation: the RHS is evaluated only when needed to determine the result.
- For match decisions, the final expression result is cast to 2-state: only `1` is true; `0` and `x` are false.

Post-MVP additions:
- Bit select/slice: `data[7:0]`, `data[3]`
- Bitwise operators: `&`, `|`, `^`, `~`
- Arithmetic: `+`, `-`
- Shift: `<<`, `>>`
- Signed comparison

**Examples:**
```bash
# Find all clock cycles when data equals 0xff (up to 50)
wavepeek when --waves dump.vcd --clk clk --cond "data == 0xff" --scope top.cpu

# First cycle where valid handshake occurs
wavepeek when --waves dump.vcd --clk clk --cond "valid && ready" --scope top.cpu --first

# Last 3 cycles where error was asserted
wavepeek when --waves dump.vcd --clk clk --cond "err == 1" --scope top.cpu --last 3

# Search within time window
wavepeek when --waves dump.vcd --clk top.cpu.clk --from 1us --to 2us --cond "top.cpu.data == 0xff"

# Complex expression
wavepeek when --waves dump.vcd --clk clk --scope top.cpu --cond "(a || b) && !reset"
```

---

## 4. Non-Functional Requirements

### 4.1 Performance
- Performance is the highest priority
- Rust is chosen specifically for this reason
- Benchmarks will be added as the project matures

### 4.2 Compatibility
- OS agnostic: Linux, macOS, Windows

### 4.3 Output Stability
- Deterministic output for identical input data

### 4.4 LLM Agent Integration
- Ready-made skill definition for LLM CLI agents (OpenCode, Codex CLI, Claude Code)
- Skill shipped in repo with setup instructions
- Deterministic `--json` output with stable `$schema` URL contracts for machine consumers

---

## 5. Technical Architecture

### 5.1 Technology Stack

| Component | Choice | Rationale |
|-----------|--------|-----------|
| **Language** | Rust stable (MSRV TBD) | Performance, memory safety, zero-cost abstractions. Ideal for parsing large binary/text dump files without GC pauses. |
| **CLI framework** | `clap` (derive API) | De-facto standard for Rust CLIs. Derive API provides self-documenting argument definitions with compile-time validation. |
| **Waveform parsing** | `wellen` | Unified interface for VCD and FST formats. Battle-tested in the [Surfer](https://surfer-project.org/) waveform viewer. Multi-threaded VCD parsing via `rayon`. Optimized for on-demand signal access (loads hierarchy first, signal data lazily). |
| **Serialization** | `serde` + `serde_json` | Standard Rust serialization. Used for strict `--json` envelope output and JSON schema export. |
| **Pattern matching** | `regex` | For `--filter` flag in `scope` and `signal` commands. |
| **Error handling** | `thiserror` | Typed error enums with `#[derive(Error)]`. All error variants are known at compile time. No runtime error boxing (no `anyhow`). |
| **Build automation** | Cargo + Make | Cargo for compilation. Makefile provides shorthand targets: `make format`, `make format-check`, `make lint`, `make test`, `make check`. |

### 5.2 High-Level Architecture

The system is organized in three layers. Data flows top-down: CLI parses arguments,
delegates to the engine, which reads waveform data and returns structured results.
The CLI layer formats results for output.

**Layers (top to bottom):**

1. **CLI Layer** (`clap`) — Argument parsing, validation, output formatting.
   Human-readable default output for all commands, strict JSON via `--json`, errors to stderr.
   Passes typed command structs down to the engine.

2. **Engine Layer** — Business logic per command: `info`, `scope`, `signal`,
   `at`, `change`, `when`, `schema`. Operates on waveform abstractions, returns structured
   results. Contains expression evaluator (for `when`) and the `change` multi-engine dispatcher
   described in [5.7 Change Command Execution Architecture](#57-change-command-execution-architecture).

3. **Waveform Layer** (`wellen`) — Thin adapter over wellen. Handles file opening,
   format detection (VCD/FST), hierarchy traversal, and signal value queries.
   Exposes: Hierarchy, Signal, TimeTable.

**Key architectural decisions:**

- **Stable output schema.** JSON output in `--json` mode uses a versioned `$schema` URL.
  The canonical schema artifact is tracked at `schema/wavepeek.json`; implementation may
  introduce internal data structures to stabilize the output contract independently of
  upstream dependency APIs.

- **Stateless execution.** Each CLI invocation opens the file, executes one command,
  and exits. No caching, no sessions, no background processes. This simplifies the
  architecture and matches the design principle of minimal footprint.

- **Format-agnostic engine.** The engine layer does not know whether it is working
  with VCD or FST. Format detection and loading is handled entirely by
  the waveform layer (wellen auto-detects format from file content).

### 5.3 Module Structure

Single crate with internal module boundaries. This is sufficient for a focused CLI tool.
If a library target is needed later (e.g., for an MCP server), the
engine and waveform modules can be extracted into a workspace crate with minimal effort.

```
src/
├── main.rs              # Entry point, top-level error handling
├── cli/                 # CLI layer: argument definitions and output formatting
│   ├── mod.rs           # Top-level CLI struct, subcommand dispatch
│   ├── info.rs          # `info` command args + output
│   ├── scope.rs         # `scope` command args + output
│   ├── signal.rs        # `signal` command args + output
│   ├── at.rs            # `at` command args + output
│   ├── change.rs        # `change` command args + output
│   ├── when.rs          # `when` command args + output
│   └── schema.rs        # `schema` command args + output
├── engine/              # Business logic per command
│   ├── mod.rs           # Shared types (result structs, time parsing)
│   ├── info.rs          # Dump metadata extraction
│   ├── scope.rs         # Hierarchy traversal with depth/filter
│   ├── signal.rs        # Signal listing within scope
│   ├── at.rs            # Value extraction at time point
│   ├── change.rs        # Value change tracking (`--when` event triggers, see 5.7)
│   ├── when.rs          # Condition evaluation over clock cycles
│   └── schema.rs        # JSON schema export
├── expr/                # Expression engine (for `when` command)
│   ├── mod.rs           # Public API: parse() + eval()
│   ├── lexer.rs         # Tokenizer
│   ├── parser.rs        # Expression parser → AST
│   └── eval.rs          # AST evaluator against signal values
├── waveform/            # Thin adapter over wellen
│   └── mod.rs           # File loading, format detection, query helpers
├── output.rs            # Shared output formatting (JSON + human)
└── error.rs             # Error enum (WavepeekError)
```

**Separation of concerns:**

| Module | Knows about | Does not know about |
|--------|-------------|---------------------|
| `cli/` | clap, output formats (JSON/human) | wellen, file I/O |
| `engine/` | domain logic, waveform layer API | clap, output formats |
| `expr/` | expression AST, signal values | wellen, CLI, output |
| `waveform/` | wellen API | CLI, engine logic |
| `output` | serde_json, JSON/human formatting | domain logic |
| `error` | all error variants | nothing else |

### 5.4 Key Dependencies

| Crate | Version | Purpose | Notes |
|-------|---------|---------|-------|
| `wellen` | ~0.20 | VCD, FST parsing | Core dependency. BSD-3-Clause. Brings in `rayon`, `fst-reader`, `memmap2`, `lz4_flex` transitively. |
| `clap` | ~4 | CLI argument parsing | With `derive` feature for declarative argument definitions. |
| `serde` | ~1 | Serialization framework | With `derive` feature. |
| `serde_json` | ~1 | JSON output | Default output serialization and schema export. |
| `regex` | ~1 | Pattern matching | For `--filter` in `scope` and `signal`. |
| `thiserror` | ~2 | Error type derivation | `#[derive(Error)]` for typed error enums. |

**Dev dependencies:**

| Crate | Purpose |
|-------|---------|
| `assert_cmd` | CLI integration testing (run binary, assert stdout/stderr/exit code) |
| `predicates` | Assertion helpers for `assert_cmd` |
| `tempfile` | Temporary file creation for tests |

### 5.5 Expression Engine

The `when` command requires evaluating boolean expressions against signal values
at each clock posedge. This needs a small expression language (defined in 3.2.6).

**Pipeline:** Input string → Lexer → Token stream → Parser → AST → Evaluator (+ signal values at current time point)
→ 4-state boolean (`0`/`1`/`x`) → 2-state match (`x` counts as false)

**Components:**

- **Lexer** — Converts input string into tokens: identifiers (signal names),
  literals (hex `0xff`, binary `0b1010`, decimal `42`), operators (`==`, `!=`,
  `&&`, `||`, `!`, `<`, `>`, `<=`, `>=`), and parentheses.

- **Parser** — Pratt parser or recursive descent. Produces an AST respecting
  operator precedence (see 3.2.6). All signal names in the AST are strings
  that get resolved against the waveform hierarchy at evaluation time.

- **AST** — Enum-based tree: `BinaryOp(op, lhs, rhs)`, `UnaryOp(op, expr)`,
  `Signal(name)`, `Literal(value)`. Minimal, no optimization passes.

- **Evaluator** — Walks the AST, resolves signal names to current values
  (provided by the engine), and computes a 4-state boolean result using SystemVerilog-like semantics.
  `&&` and `||` use short-circuit evaluation. For `when` match decisions, unknown `x` is cast to false.

**Implementation status:** Event-expression parsing for `change --when` is now
implemented with a hand-written parser (`parse_event_expr` in
`src/expr/parser.rs`, types in `src/expr/mod.rs`). For logical expressions,
current `--cond` handling is still staged (`parse` stores validated source text),
and full parser/evaluator runtime for the planned `property` flow remains
deferred to post-MVP implementation work.

### 5.6 Error Handling Strategy

**Principles:**
- **Fail fast.** On the first error, print a message to stderr and exit with a
  non-zero code. Stdout is empty on errors (no JSON envelope).
- **LLM-parseable errors.** Error messages follow a fixed format so agents can
  detect and interpret them programmatically.
- **No panics in production paths.** All recoverable errors use `Result<T, WavepeekError>`.
  `unwrap()` / `expect()` only in cases that indicate programmer bugs.

**Error format (stderr):**

```
error: <category>: <message>
```

Examples:
```
error: file: cannot open 'dump.vcd': No such file or directory
error: args: --time requires units (e.g., '100ns'), got '100'
error: signal: signal 'top.cpu.foo' not found in dump
error: expr: parse error in condition: unexpected token ')' at position 12
```

**Exit codes:**

| Code | Meaning |
|------|---------|
| `0` | Success |
| `1` | User error (bad arguments, signal not found, invalid expression) |
| `2` | File error (cannot open, cannot parse, unsupported format) |

**Warnings:**

- Warnings (e.g., output truncation due to `--max`, no matches in a query) do not change exit code (still `0`).
- In `--json` mode, warnings are appended to the `warnings` array in the JSON envelope.
- In human mode, warnings are printed to stderr as free-form text.

**Error enum:**

A single `WavepeekError` enum in `src/error.rs` with variants covering all failure
modes. Each variant maps to an exit code and a formatted message.
The CLI layer converts `WavepeekError` into stderr output and exit code.

### 5.7 Change Command Execution Architecture

The `change` command uses a multi-engine execution model that keeps one user
contract while allowing different internal execution strategies for different
workloads.

Execution engines:

- **Baseline engine** — Conservative baseline path used for narrow or low-work
  windows. It prioritizes predictable behavior and low fixed overhead.
- **Fused engine** — Optimized for broad candidate sets and any-tracked windows.
  It reduces repeated per-signal work by combining more work in shared passes.
- **Edge-fast engine** — Specialized path for dense edge-trigger workloads. It
  applies edge-first gating and lightweight prefiltering before full delta
  materialization.

Dispatcher heuristics:

- Default mode routes requests automatically from simple workload estimates
  (window size, candidate density, requested signal count, and trigger shape).
- Edge-only trigger profiles can be routed to fused or edge-fast depending on
  estimated work, while sparse profiles remain on baseline.
- This policy is intentionally internal and may evolve without changing user
  payload semantics.

Why this complexity exists:

- A single engine could not deliver consistent low latency for both tiny and
  large-window scenarios.
- The dispatcher keeps externally visible behavior contract-equivalent while
  choosing faster internals for high-work inputs.
- The design target is stable output parity with practical latency improvements
  on large dumps/windows where optimized paths materially reduce runtime.

### 5.8 Testing Strategy

**Levels:**

| Level | What | How | Fixtures |
|-------|------|-----|----------|
| **Unit tests** | Individual functions in `engine/`, `expr/`, `waveform/` | `#[cfg(test)]` modules, `cargo test` | Hand-crafted VCD strings (inline or small `.vcd` files) |
| **Integration tests** | Full CLI invocations end-to-end | `assert_cmd` in `tests/` directory | Hand fixtures + container-provisioned fixtures at `/opt/rtl-artifacts` |
| **Expression tests** | Lexer, parser, evaluator independently | Unit tests in `expr/` submodules | None (pure logic, string inputs) |

**Test fixture strategy (two sources):**

1. **Hand-crafted VCD files** — For unit tests and edge cases. VCD is a simple
   text format, easy to write manually. Covers: empty files, single-bit signals,
   multi-bit signals, X/Z values, multiple scopes, deep hierarchy, time edge cases
   (zero duration, single timestamp). Stored in `tests/fixtures/hand/`.

2. **Container-provisioned representative fixtures** — For integration tests.
   Required large fixtures are downloaded during devcontainer/CI image build from
   a pinned release version and installed under `/opt/rtl-artifacts`.
   Runtime test execution does not download fixtures.

**What to assert in integration tests:**

- Exact stdout output (deterministic output is a design principle)
- Exit code
- Stderr content for error cases
- `--json` output validates against expected JSON structure and `$schema` URL contract
- Human output is generally not asserted for exact formatting unless a command-level contract explicitly fixes it (currently `at`, including `--abs` display behavior)
- Consistency: same query on VCD and FST of the same design produces identical output

---

## 7. Open Questions

1. **Scope/path canonicalization.** What is the canonical path syntax and escaping rules for VCD escaped identifiers and unusual names across formats?
2. **Warnings (codes vs free text).** Do we add stable warning codes for promote/suppress, or keep warnings as free-form strings only?
3. **Value radix options.** Do we add `--radix` (hex/bin/dec/auto) and what is the default policy beyond the Verilog-literal representation?
4. **Schema evolution policy.** Do we keep a single canonical schema file indefinitely, or split into per-command schemas in future milestones?
5. **Signal metadata schema.** Exact JSON fields for `signal` output (`kind`, `width`, and other metadata) and how they map across formats.
6. **GHW support scope.** Should GHW be added after MVP, and if yes, what acceptance criteria and priority should gate its introduction?

---

## 8. References
- [GTKWave](http://gtkwave.sourceforge.net/) — reference waveform viewer
- [Surfer](https://surfer-project.org/) — modern waveform viewer; reference implementation context for the parsing stack
- [VCD Format Specification](https://en.wikipedia.org/wiki/Value_change_dump)
- [FST Format](https://gtkwave.sourceforge.net/gtkwave.pdf)
- [wellen (GitHub)](https://github.com/ekiwi/wellen) — Rust waveform parsing library used by wavepeek
- [wellen (docs.rs)](https://docs.rs/wellen/0.20.2) — API documentation
