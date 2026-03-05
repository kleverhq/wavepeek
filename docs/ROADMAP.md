# Roadmap

This document captures planned milestone scope and expected feature delivery.
It is intentionally forward-looking and may change as priorities evolve.

For factual release outcomes (what actually shipped), use `CHANGELOG.md`.

### Project Init (→ v0.1.0)

- Rust project init (Cargo.toml, dependencies per §5.4, module structure per §5.3)
- Empty module files with `todo!()` / placeholder stubs
- CLI entry point: `clap` with subcommand skeleton, `--help` only
- Makefile: `make format`, `make format-check`, `make lint`, `make test`, `make check`
- Pre-commit hooks (rustfmt, clippy, cargo check, cargo test)
- CI pipeline (GitHub Actions: format check, clippy, test, build)
- Release pipeline (tag-triggered: build binaries, GitHub Release)

### Core CLI (→ v0.2.0)

- `info` command (§3.2.1)
- `scope` command (§3.2.2)
- `signal` command (§3.2.3)
- VCD + FST format support
- Human default output for all commands with explicit `--json` contract mode
- `tree` and `modules` command surfaces replaced by `scope` (no aliases)
- `schema` command for canonical JSON schema export
- Canonical schema artifact at `schema/wavepeek.json` with `make update-schema`
- JSON envelope metadata migrated from `schema_version` to versioned `$schema` URL
- Error handling: WavepeekError enum, exit codes, stderr format (§5.6)
- Hand-crafted VCD test fixtures
- Integration tests with `assert_cmd`

### Value Extraction (→ v0.3.0)

- `value` command (§3.2.4)
- `change` command — unified `--when` trigger model (default `*`, edge/signal expressions) (§3.2.5)
- Time parsing with mandatory units (`--from`, `--to`, `--at`)
- Expanded container-provisioned fixtures for value-extraction scenarios

### Query Engine (→ v0.4.0)

- `when` command (§3.2.6)
- Expression engine: lexer, parser (Pratt/recursive descent), evaluator (§5.5)
- MVP operators: `!`, `<`, `>`, `<=`, `>=`, `==`, `!=`, `&&`, `||`
- Literals: hex, binary, decimal
- Parentheses grouping, truthy semantics
- SystemVerilog-like 4-state evaluation with short-circuiting; unknown `x` casts to false for matching

### Agent-Ready (→ v0.5.0)

- LLM agent skill definition for CLI agents
- Performance benchmarks
- Post-MVP expression extensions: bit select/slice, bitwise ops, arithmetic, shift

### MCP Server (→ v0.6.0)

- MCP server for LLM agent integration

### Unmapped

- Proprietary formats (FSDB, VPD, WLF)
- Signed comparison in expression engine
