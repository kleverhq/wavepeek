# ---- Shared Environment Contract -----------------------------------------------
FROM scratch AS env_contract
COPY .devcontainer/env_contract.sh /wavepeek-env-contract.sh

# ---- Surfer Builder ------------------------------------------------------------
FROM ubuntu:24.04 AS surfer_builder

# Avoid prompts from apt
ENV DEBIAN_FRONTEND=noninteractive

# Setup environment variables
COPY --from=env_contract /wavepeek-env-contract.sh /tmp/wavepeek-env-contract.sh
ENV SURFER_INSTALL=/opt/surfer \
    RUSTUP_HOME=/opt/rustup \
    CARGO_HOME=/opt/cargo

# Install build dependencies
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    ca-certificates curl git pkg-config libssl-dev build-essential && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/*

# Install Rust toolchain
RUN . /tmp/wavepeek-env-contract.sh && \
    curl https://sh.rustup.rs -sSf | sh -s -- -y --profile minimal && \
    ${CARGO_HOME}/bin/rustup toolchain install ${WAVEPEEK_RUST_VERSION} --profile minimal && \
    ${CARGO_HOME}/bin/rustup default ${WAVEPEEK_RUST_VERSION}

# Build Surfer
RUN . /tmp/wavepeek-env-contract.sh && \
    ${CARGO_HOME}/bin/cargo install --locked \
    --git https://gitlab.com/surfer-project/surfer.git \
    --tag ${WAVEPEEK_SURFER_VERSION} \
    --root ${SURFER_INSTALL} \
    surfer

# ---- RTL Fixture Layer --------------------------------------------------------
FROM ubuntu:24.04 AS rtl_artifacts

ENV DEBIAN_FRONTEND=noninteractive
COPY --from=env_contract /wavepeek-env-contract.sh /tmp/wavepeek-env-contract.sh
ENV RTL_ARTIFACTS_DIR=/opt/rtl-artifacts

RUN apt-get update && \
    apt-get install -y --no-install-recommends ca-certificates curl && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/*

RUN . /tmp/wavepeek-env-contract.sh && \
    mkdir -p ${RTL_ARTIFACTS_DIR} && \
    for artifact in ${WAVEPEEK_RTL_ARTIFACT_FILES}; do \
      curl -fsSL -o "${RTL_ARTIFACTS_DIR}/${artifact}" \
        "https://github.com/kleverhq/rtl-artifacts/releases/download/${WAVEPEEK_RTL_ARTIFACTS_VERSION}/${artifact}"; \
    done

# ---- Node Toolchain ----------------------------------------------------------
FROM node:24-bookworm AS node_toolchain

# ---- Base Image --------------------------------------------------------------
FROM ubuntu:24.04 AS base

ARG TARGETARCH
COPY --from=env_contract /wavepeek-env-contract.sh /tmp/wavepeek-env-contract.sh

# Install necessary system dependencies
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    ca-certificates curl git make libatomic1 \
    openssl libssl-dev pkg-config \
    g++ mold ccache \
    python3 \
    hyperfine \
    locales \
    && locale-gen en_US.UTF-8 \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*

# Install GitHub Actions workflow linter for hooks and CI checks.
RUN . /tmp/wavepeek-env-contract.sh && \
    arch="${TARGETARCH:-amd64}" && \
    case "$arch" in \
        amd64|arm64) actionlint_arch="$arch" ;; \
        *) printf '%s\n' "unsupported actionlint architecture: $arch" >&2; exit 1 ;; \
    esac && \
    tmp_dir="$(mktemp -d)" && \
    curl -fsSL -o "$tmp_dir/actionlint.tar.gz" \
        "https://github.com/rhysd/actionlint/releases/download/v${WAVEPEEK_ACTIONLINT_VERSION}/actionlint_${WAVEPEEK_ACTIONLINT_VERSION}_linux_${actionlint_arch}.tar.gz" && \
    tar -xzf "$tmp_dir/actionlint.tar.gz" -C "$tmp_dir" actionlint && \
    install -m 0755 "$tmp_dir/actionlint" /usr/local/bin/actionlint && \
    rm -rf "$tmp_dir"

ENV LANG=en_US.UTF-8
ENV LC_ALL=en_US.UTF-8

# Ensure the devcontainer user exists
RUN if ! id -u ubuntu >/dev/null 2>&1; then useradd -m -s /bin/bash ubuntu; fi

# Ensure user config directories exist and are writable
RUN install -d -m 0755 -o ubuntu -g ubuntu \
    /home/ubuntu/.config \
    /home/ubuntu/.cache \
    /home/ubuntu/.local \
    /home/ubuntu/.local/bin \
    /home/ubuntu/.local/share \
    /home/ubuntu/.local/state

# Ensure /workspaces exists and is writable
RUN mkdir -p /workspaces && chown ubuntu:ubuntu /workspaces && chmod 775 /workspaces

# Provision pinned RTL fixtures for tests
COPY --from=rtl_artifacts /opt/rtl-artifacts /opt/rtl-artifacts

# Switch to non-root user
USER ubuntu

# Install Rust toolchain
ENV HOME=/home/ubuntu
ENV RUSTUP_HOME=/home/ubuntu/.rustup
ENV CARGO_HOME=/home/ubuntu/.cargo
RUN . /tmp/wavepeek-env-contract.sh && \
    curl https://sh.rustup.rs -sSf | sh -s -- -y --profile minimal && \
    /home/ubuntu/.cargo/bin/rustup toolchain install ${WAVEPEEK_RUST_VERSION} --profile minimal && \
    /home/ubuntu/.cargo/bin/rustup default ${WAVEPEEK_RUST_VERSION} && \
    /home/ubuntu/.cargo/bin/rustup component add rustfmt clippy llvm-tools-preview --toolchain ${WAVEPEEK_RUST_VERSION} && \
    /home/ubuntu/.cargo/bin/cargo install --locked cargo-llvm-cov --version ${WAVEPEEK_CARGO_LLVM_COV_VERSION}

# ---- CI Image ----------------------------------------------------------------
FROM base AS ci

# Update PATH
ENV PATH="/home/ubuntu/.local/bin:${CARGO_HOME}/bin:${PATH}"

# ---- Dev Image ---------------------------------------------------------------
FROM base AS dev

USER root

COPY --from=node_toolchain /usr/local/ /usr/local/

# Install development-only dependencies
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    nano tmux openssh-client bash-completion \
    libx11-xcb1 libxkbcommon-x11-0 libegl1 libgl1 libglx-mesa0 libgl1-mesa-dri \
    jq ripgrep ugrep pdfgrep poppler-utils fd-find tree gh \
    perl-doc z3 gtkwave \
    python3 pipx \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*

USER ubuntu

# Install surfer
ENV SURFER_INSTALL=/opt/surfer
COPY --from=surfer_builder --chown=ubuntu:ubuntu ${SURFER_INSTALL} ${SURFER_INSTALL}

# Install latest npm-based coding agents
ENV NPM_CONFIG_PREFIX=/home/ubuntu/.local
RUN . /tmp/wavepeek-env-contract.sh && \
    npm install -g \
    @devcontainers/cli@${WAVEPEEK_DEVCONTAINERS_CLI_VERSION} \
    @openai/codex \
    @mariozechner/pi-coding-agent

# Install latest OpenCode
RUN curl -fsSL https://opencode.ai/install | bash && \
    ln -s /home/ubuntu/.opencode/bin/opencode /home/ubuntu/.local/bin/opencode

# Install latest Claude Code
RUN curl -fsSL https://claude.ai/install.sh | bash

# Prefer X11 backend in devcontainer GUI sessions.
ENV WINIT_UNIX_BACKEND=x11

# Install Python tooling for hooks
RUN . /tmp/wavepeek-env-contract.sh && \
    pipx install pre-commit==${WAVEPEEK_PRECOMMIT_VERSION} && \
    pipx install commitizen==${WAVEPEEK_COMMITIZEN_VERSION}

# Update PATH
ENV PATH="/home/ubuntu/.local/bin:${CARGO_HOME}/bin:${SURFER_INSTALL}/bin:${PATH}"
