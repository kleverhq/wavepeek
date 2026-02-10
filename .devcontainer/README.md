# Devcontainer Overview

Custom Ubuntu 24.04 devcontainer with ready-to-use tooling for RTL and Rust work.

## Workspace and mounts

- Mounts the repo parent so Git worktrees under the same parent work.
  - `workspaceMount`: `source=${localWorkspaceFolder}/..,target=/workspaces,type=bind`
  - `workspaceFolder`: `/workspaces/${localWorkspaceFolderBasename}`
- Persists CLI agent auth and sessions by mounting host state:
  - OpenCode: `~/.config/opencode`, `~/.local/share/opencode`, `~/.cache/opencode`
- `initializeCommand` creates these paths so first-run mounts succeed.

## Image and tooling

- Base image: Ubuntu 24.04.
- Dockerfile defines three targets:
  - `base`: shared toolchain and dependencies.
  - `ci`: lean CI/runtime target (no OpenCode, no Surfer, no slang-server, no GUI stack).
  - `dev`: full local-development target (adds OpenCode, Surfer, slang-server, X11/Mesa GUI runtime, and hook tooling).
- `devcontainer.json` builds `target: dev` for local VS Code workflows.
- `devcontainer.ci.json` builds `target: ci` for GitHub Actions via `devcontainers/ci`.
- Shared tooling (in `base`) includes `git`, `curl`, `make`, `g++`, `mold`, `ccache`, `gcc-riscv64-unknown-elf`, `binutils-riscv64-unknown-elf`, `picolibc-riscv64-unknown-elf`, Rust toolchain (`rustup` + `cargo`, with `rustfmt`/`clippy`), and Verilator at `/opt/verilator`.
- Dev-only tooling (in `dev`) includes `slang-server` at `/opt/slang-server`, `surfer` at `/opt/surfer`, OpenCode, `z3`, `gtkwave`, `jq`, `ripgrep`, `gh`, and Python-based `pre-commit`/`commitizen`.
- `dev` extends `PATH` with cargo, Verilator, slang-server, and surfer binaries.
- `dev` sets `WINIT_UNIX_BACKEND=x11` so Surfer uses X11 in VS Code devcontainer sessions.

## Container behavior

- Uses host networking (`--network=host`) so host VPN routing works.
- Runs as `ubuntu` and keeps UID/GID aligned with the host.
- Marks the workspace as a safe Git directory on create.
- Runs `make bootstrap` on start to install Rust tooling and set pre-commit hooks.

## VS Code extensions

Recommends extensions for Rust Analyzer, TOML, YAML, Markdown, Slang, and OpenCode.
