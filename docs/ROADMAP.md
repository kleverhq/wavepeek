# Roadmap

This document captures planned milestone scope and expected feature delivery.
It is intentionally forward-looking and may change as priorities evolve.

For factual release outcomes (what actually shipped), use `CHANGELOG.md`.

### Project Setup (→ v0.1.0)

- Rust project init (Cargo.toml, dependencies per §5.4, module structure per §5.3)
- Stub module layout with `todo!()` placeholders
- CLI entry point via `clap` with subcommand skeleton and `--help` only
- Makefile targets: `make format`, `make format-check`, `make lint`, `make test`, `make check`
- Pre-commit hooks for rustfmt, clippy, cargo check, and cargo test
- GitHub Actions CI pipeline for format check, clippy, test, and build
- Tag-triggered release pipeline for binary builds and GitHub Releases

### Core CLI (→ v0.2.0)

- `info` command (§3.2.1)
- `scope` command (§3.2.2)
- `signal` command (§3.2.3)
- VCD + FST format support
- Human-readable default output for all commands with explicit `--json` contract mode
- `tree` and `modules` command surfaces replaced by `scope` (no aliases)
- `schema` command for canonical JSON Schema export
- Canonical schema artifact at `schema/wavepeek.json` with `make update-schema`
- JSON envelope metadata migrated from `schema_version` to versioned `$schema` URLs
- Structured error handling: `WavepeekError`, exit codes, stderr format (§5.6)
- Handcrafted VCD test fixtures
- Integration tests with `assert_cmd`

### Value Extraction (→ v0.3.0)

- `value` command (§3.2.4)
- `change` command — unified `--on` trigger model (default `*`, edge/signal expressions) (§3.2.5)
- Time parsing with mandatory units (`--from`, `--to`, `--at`)
- Expanded container-provisioned fixtures for value extraction scenarios
- CLI agent skill for Wavepeek workflows
- Performance benchmark suite

### Query Engine (→ v0.4.0)

- `property` command (§3.2.6)
- Expression engine: lexer, parser (Pratt/recursive descent), and evaluator (§5.5)
- MVP operator set: `!`, `<`, `>`, `<=`, `>=`, `==`, `!=`, `&&`, `||`
- Literal support: hex, binary, decimal
- Parentheses grouping and truthy semantics
- SystemVerilog-like 4-state evaluation with short-circuiting; unknown `x` values cast to false for matching

### Query Engine Enhancements (→ v0.5.0)

- Post-MVP expression extensions: bit selects/slices, bitwise ops, arithmetic, shifts

### MCP Server (→ v0.6.0)

- MCP server for agent integration

### Unmapped

- Proprietary dump formats (FSDB, VPD, WLF)
- Signed comparisons in the expression engine
