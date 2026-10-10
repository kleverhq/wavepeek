# Devcontainer Guidance

## Scope

This directory owns the public tool-only container definition, fixture provisioning, and environment-contract helpers used by local development and automation.

## Source of Truth

- Container workflow: `../docs/environment.md`
- Quality gates: `../docs/quality.md`
- Container config and provisioning: `Dockerfile`, `devcontainer.json`, `env_contract.sh`

## Local Guidance

- `Dockerfile` has one final image for local development, CI, release checks, and docs staging.
- The root `../dev` wrapper starts one container per absolute Git worktree and selected configuration, and adds only the linked-worktree Git common-directory mount.
- Proprietary tools, SDK mounts, and vendor environment belong to ignored `.devcontainer.local/` profiles selected through `WAVEPEEK_DEV_CONFIG`. The public configuration must not discover or mount host SDKs.
- Keep the container credentialless. Do not mount agent state, host GitHub state, token files, or broad host directories.
- `verdi-tool-wrapper.sh` exposes selected command-line Verdi FSDB utilities and invokes their launchers with Bash for compatibility.
- Host networking is intentional for VPN-heavy environments.
- Container lifecycle commands must not install or rewrite host Git hooks. Hook activation is explicit through host `../dev --install-hooks`.

## Safety

Do not store credentials in repository files, `.git/config`, breadcrumbs, logs, or shell history. Large waveform fixtures are baked into the image by the `rtl_artifacts` stage. Runtime tests should not download them from the network. When bumping `WAVEPEEK_RTL_ARTIFACTS_VERSION`, rebuild the container and run `./dev just ci` plus `./dev just pre-commit`.
