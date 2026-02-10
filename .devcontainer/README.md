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

- Base image: Ubuntu 24.04 (chosen as a reasonable default).
- Installs system tools: `git`, `curl`, `make`, `g++`, `mold`, `ccache`, `z3`, `gtkwave`, `gcc-riscv64-unknown-elf`, `binutils-riscv64-unknown-elf`, `picolibc-riscv64-unknown-elf`, `npm`, `openssh-client`, `bash-completion`.
- Installs Verilator at `/opt/verilator` for RTL workflows.
- Installs `slang-server` at `/opt/slang-server` for SystemVerilog language services.
- Installs `surfer` (Rust-based waveform viewer).
- Installs Mesa/X11 runtime libs required by `surfer` in containerized GUI sessions.
- Installs CLI agent (OpenCode) for out-of-box access.
- Installs Rust toolchain (rustup + cargo, with rustfmt/clippy).
- Installs Python-based `pre-commit` and `commitizen`.
- Extends `PATH` with cargo, Verilator, and slang-server.
- Sets `WINIT_UNIX_BACKEND=x11` so Surfer uses X11 in VS Code devcontainer environments.

## Container behavior

- Uses host networking (`--network=host`) so host VPN routing works.
- Runs as `ubuntu` and keeps UID/GID aligned with the host.
- Marks the workspace as a safe Git directory on create.
- Runs `make bootstrap` on start to install Rust tooling and set pre-commit hooks.

## VS Code extensions

Recommends extensions for Rust Analyzer, TOML, YAML, Markdown, Slang, and OpenCode.
