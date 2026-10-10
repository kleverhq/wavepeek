# Devcontainer Guidance

## Scope

This directory owns the public tool-only container definition, fixture provisioning, and environment-contract helpers used by local development and automation.

## Source of Truth

- Container workflow: `../docs/environment.md`
- Quality gates: `../docs/quality.md`
- Container config and provisioning: `Dockerfile`, `devcontainer.json`, `env_contract.sh`

## Local Guidance

- `Dockerfile` has one final image for local development, CI, release checks, and docs staging.
- Root `../dev` starts one container per worktree and configuration, adding the linked-worktree Git common-directory mount.
- Keep proprietary tools, mounts, and environment in ignored `.devcontainer.local/` profiles selected by `WAVEPEEK_DEV_CONFIG`. Public profiles must not discover or mount host SDKs.
- Keep the container credentialless. Do not mount agent state, host GitHub state, token files, or broad host directories.
- `verdi-tool-wrapper.sh` exposes selected command-line Verdi FSDB utilities and invokes their launchers with Bash for compatibility.
- Host networking is intentional for VPN-heavy environments.
- Container lifecycle commands must not install or rewrite host Git hooks. Hook activation is explicit through host `../dev --install-hooks`.

## Safety

Do not store credentials in repository files, `.git/config`, breadcrumbs, logs, or shell history. Selected waveform fixtures are baked into the image by the `ondas_fixtures` stage. Runtime tests should not download them from the network. When changing `WAVEPEEK_ONDAS_FIXTURES_REV` or `WAVEPEEK_ONDAS_FIXTURES`, rebuild the container and run `./dev just ci` plus `./dev just pre-commit`.
