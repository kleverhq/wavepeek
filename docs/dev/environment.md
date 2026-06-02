# Development Environment

`wavepeek` is developed container-first. The devcontainer and CI image keep Rust, Python helpers, actionlint, fixtures, and git hooks aligned so contributors do not debug two subtly different universes, which is traditionally where the gremlins live.

## Containers

Local interactive work uses `.devcontainer/devcontainer.json`, which targets the `dev` stage in `.devcontainer/Dockerfile`. CI and release automation use `.devcontainer/devcontainer.ci.json`, which targets the leaner `ci` stage from the same Dockerfile.

Recipes in `justfile` require `WAVEPEEK_IN_CONTAINER=1`. Set it only inside a wavepeek-managed devcontainer or CI image; outside the container, install or enter the proper environment instead of bypassing the guard.

Run `just dev-setup` after opening or rebuilding the devcontainer. It verifies tool availability and installs the pre-commit and commit-msg hooks.

The devcontainer may prepare host-side mount sources under `~/.cache/wavepeek`, including the Verdi mount source at `~/.cache/wavepeek/verdi`. Treat that directory as wavepeek-managed cache state: do not place manual installs or durable files there.

## Codex Cloud Setup

For first-time Codex bootstrap, run `bash tools/codex/codex_setup.sh`. This direct script path exists because the first bootstrap may need to install or repair tools before `just` recipes are safe to assume. After the environment has `just`, use `just codex-resume` for maintenance after cache resume.

Codex setup may populate the cache fallback for RTL fixtures, then `just` propagates the resolved path to runtime and test processes as `WAVEPEEK_RTL_ARTIFACTS_DIR`.

## Fixture Location Resolution

Large RTL fixtures are baked into the devcontainer and CI image under `/opt/rtl-artifacts`. Runtime path resolution prefers `WAVEPEEK_RTL_ARTIFACTS_DIR`, then `RTL_ARTIFACTS_DIR`, then `/opt/rtl-artifacts`, and finally `~/.cache/wavepeek/rtl-artifacts`.

The shared environment contract lives in `.devcontainer/env_contract.sh` and `.devcontainer/resolve_rtl_artifacts_dir.sh`. Update those files, container provisioning, and Codex helper behavior together when fixture versions or layout change.

## Verdi / FSDB Development

FSDB work is optional and local-only unless a task explicitly says otherwise. The devcontainer sets `VERDI_HOME=/opt/verdi`, which is normally a host-mounted cache location. `.devcontainer/resolve_verdi_home.sh` and `tools/codex/check_fsdb_env.py` verify that the mount contains a usable Synopsys Verdi FSDB Reader SDK before any FSDB build, test, fixture, or benchmark recipe runs.

Feature-enabled binaries link to local Verdi shared libraries, but `build.rs` does not embed the absolute Reader library directory as an ELF rpath by default. The FSDB `just` recipes prepend the selected Reader library directory to `LD_LIBRARY_PATH` when they run tests or binaries. Direct `cargo test --features fsdb`, `cargo run --features fsdb`, or release-binary invocations need the same runtime library path; resolve it with `python3 -B tools/codex/check_fsdb_env.py --require --print-libdir`. `WAVEPEEK_FSDB_EMBED_RPATH=1` is a local opt-in for throwaway builds that deliberately accept a non-relocatable binary and machine-specific path leakage; use it sparingly.

Do not commit Verdi headers, libraries, installed documentation, generated bindings that copy proprietary declarations, converter logs, or `.fsdb` waveform files. Generated FSDB fixtures belong in ignored paths such as `tests/fixtures/fsdb/`, neighboring ignored RTL artifact copies, or repository-root `tmp/`. The devcontainer provides wrapper commands such as `vcd2fsdb`, `fst2vcd`, `fsdb2vcd`, `fsdbdebug`, and `fsdbextract`; call those wrappers from `PATH` instead of hard-coding `$VERDI_HOME/bin/...`.

## Debug Mode

`DEBUG=1` enables maintainer-only internal diagnostics and hidden controls. Hidden controls are unstable implementation details and are not part of the public CLI contract, even when debug mode exposes them.

## Temporary Files

Use repository-root `tmp/` for scratch files, ad hoc logs, temporary benchmark captures, and other disposable working artifacts. It is ignored by git and may be created freely.

Never globally clean `tmp/` or delete arbitrary existing files there. Other agents or the user may own them. If a temporary artifact needs review or must survive across sessions, move it intentionally into a tracked location such as `docs/tracker/wip/` and explain why.
