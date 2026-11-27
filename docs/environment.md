# Development Environment

Development, CI, and release checks use the public image defined in `.devcontainer/`. The public profile uses host networking during build and run so it can use the host's VPN routes. Keep the container credentialless. The public profile rejects extra bind mounts. Linked worktrees need the Git common directory mounted at the same path as on the host.

`./dev` gives each worktree and configuration its own container. A change to the configuration JSON or required workspace mounts invalidates the selected container. Use `./dev --recreate true` after a change to the Dockerfile, image, or expanded host variables. Store persistent data in the checkout or a named volume. Use `./dev just dev-setup` to verify a new container.

## Private profiles

Set `WAVEPEEK_DEV_CONFIG` to a worktree-relative configuration path. To keep the selection across commands, set it in the ignored root `.env`. The launcher sources this file as trusted Bash. An explicit host setting takes precedence.

Copy `.devcontainer/devcontainer.json` to `.devcontainer.local/devcontainer.json` and set its `build.dockerfile` to `../.devcontainer/Dockerfile`. Keep the workspace and user settings and `WAVEPEEK_IN_CONTAINER=1`. Mount the required installation read-only. Set `VERDI_HOME`, `XCELIUM_HOME`, `QUESTA_HOME`, or `XILINX_VIVADO` to the applicable container path. The WDB CLI tests also use XSim from that installation. Configure tool licenses privately. Keep all vendor payloads, license material, and credentials out of Git and public build contexts.

For a prebuilt private image, replace `build` with `image`. For a derived image, set `build.context` to `.` and `build.dockerfile` to `Dockerfile`. Keep the Dockerfile and its `.dockerignore` in `.devcontainer.local/`.

## Fixture provisioning

The image installs selected, pinned Ondas fixtures before tests run. Tests use them offline. `ONDAS_FIXTURES_DIR` is writable so local conversions can live next to their inputs. When changing the corpus version, selection, or layout in `.devcontainer/env_contract.sh`, update any affected provisioning, rebuild the container, and run `./dev just ci`.

GitHub Actions uses its job token for fixture downloads. Docker mounts it as a BuildKit secret for the fixture installation step and removes the mount when the step ends.

`DEBUG=1` enables maintainer diagnostics outside the public CLI contract.
