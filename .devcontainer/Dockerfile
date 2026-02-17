# ---- Verilator Builder -------------------------------------------------------
FROM ubuntu:24.04 AS verilator_builder

# Setup environment variables
ENV VERILATOR_VERSION=v5.042 \
    VERILATOR_INSTALL=/opt/verilator

# Avoid prompts from apt
ENV DEBIAN_FRONTEND=noninteractive

# Install dependencies
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    ca-certificates git \
    help2man perl python3 make autoconf g++ mold flex bison ccache libfl2 libfl-dev

# Build Verilator
RUN git clone --branch ${VERILATOR_VERSION} --depth 1 https://github.com/verilator/verilator.git
RUN cd verilator && \
    autoconf && \
    ./configure --prefix ${VERILATOR_INSTALL} && \
    make -j$(nproc) && \
    make install

# ---- slang-server Builder -------------------------------------------------------
FROM ubuntu:24.04 AS slang_server_builder

# Setup environment variables
ENV SLANG_SERVER_VERSION=v0.2.1 \
    SLANG_SERVER_INSTALL=/opt/slang-server

# Avoid prompts from apt
ENV DEBIAN_FRONTEND=noninteractive

# Install build dependencies
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    ca-certificates git cmake g++ ninja-build python3 python3-dev

# Clone the repository and build
RUN git clone --branch ${SLANG_SERVER_VERSION} --depth 1 https://github.com/hudson-trading/slang-server.git && \
    cd slang-server && \
    git submodule update --init --recursive --depth 1 && \
    cmake -B build -DCMAKE_BUILD_TYPE=Release -G Ninja && \
    cmake --build build --target slang_server && \
    mkdir -p ${SLANG_SERVER_INSTALL}/bin && \
    cp build/bin/slang-server ${SLANG_SERVER_INSTALL}/bin/

# ---- Surfer Builder ------------------------------------------------------------
FROM ubuntu:24.04 AS surfer_builder

# Avoid prompts from apt
ENV DEBIAN_FRONTEND=noninteractive

# Setup environment variables
ENV SURFER_INSTALL=/opt/surfer \
    RUSTUP_HOME=/opt/rustup \
    CARGO_HOME=/opt/cargo \
    RUST_VERSION=1.93.0

# Install build dependencies
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    ca-certificates curl git pkg-config libssl-dev build-essential && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/*

# Install Rust toolchain
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y --profile minimal && \
    ${CARGO_HOME}/bin/rustup toolchain install ${RUST_VERSION} --profile minimal && \
    ${CARGO_HOME}/bin/rustup default ${RUST_VERSION}

# Build Surfer
ARG SURFER_VERSION=v0.5.0
RUN ${CARGO_HOME}/bin/cargo install --locked \
    --git https://gitlab.com/surfer-project/surfer.git \
    --tag ${SURFER_VERSION} \
    --root ${SURFER_INSTALL} \
    surfer

# ---- Base Image --------------------------------------------------------------
FROM ubuntu:24.04 AS base

# Install necessary system dependencies
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    ca-certificates curl git make libatomic1 \
    openssl libssl-dev pkg-config \
    g++ mold ccache \
    locales \
    && locale-gen en_US.UTF-8 \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*

ENV LANG=en_US.UTF-8
ENV LC_ALL=en_US.UTF-8

# Ensure the devcontainer user exists
RUN if ! id -u ubuntu >/dev/null 2>&1; then useradd -m -s /bin/bash ubuntu; fi

# Ensure /workspaces exists and is writable
RUN mkdir -p /workspaces && chown ubuntu:ubuntu /workspaces && chmod 775 /workspaces

# Install Verilator
ENV VERILATOR_INSTALL=/opt/verilator
COPY --from=verilator_builder ${VERILATOR_INSTALL} ${VERILATOR_INSTALL}

# Switch to non-root user
USER ubuntu

# Install Rust toolchain
ENV RUSTUP_HOME=/home/ubuntu/.rustup
ENV CARGO_HOME=/home/ubuntu/.cargo
ENV RUST_VERSION=1.93.0
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y --profile minimal && \
    /home/ubuntu/.cargo/bin/rustup toolchain install ${RUST_VERSION} --profile minimal && \
    /home/ubuntu/.cargo/bin/rustup default ${RUST_VERSION} && \
    /home/ubuntu/.cargo/bin/rustup component add rustfmt clippy --toolchain ${RUST_VERSION}

# ---- CI Image ----------------------------------------------------------------
FROM base AS ci

# Update PATH
ENV PATH="${HOME}/.local/bin:${CARGO_HOME}/bin:${VERILATOR_INSTALL}/bin:${PATH}"

# ---- Dev Image ---------------------------------------------------------------
FROM base AS dev

USER root

# Install development-only dependencies
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    nano openssh-client bash-completion \
    libx11-xcb1 libxkbcommon-x11-0 libegl1 libgl1 libglx-mesa0 libgl1-mesa-dri \
    jq ripgrep gh \
    perl-doc z3 gtkwave \
    python3 pipx \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*

USER ubuntu

# Install slang-server
ENV SLANG_SERVER_INSTALL=/opt/slang-server
COPY --from=slang_server_builder --chown=ubuntu:ubuntu ${SLANG_SERVER_INSTALL} ${SLANG_SERVER_INSTALL}

# Install surfer
ENV SURFER_INSTALL=/opt/surfer
COPY --from=surfer_builder --chown=ubuntu:ubuntu ${SURFER_INSTALL} ${SURFER_INSTALL}

# Install OpenCode
ENV OPENCODE_VERSION=1.1.53
RUN curl -fsSL https://opencode.ai/install | bash -s -- --version ${OPENCODE_VERSION}

# Prefer X11 backend in devcontainer GUI sessions.
ENV WINIT_UNIX_BACKEND=x11

# Install Python tooling for hooks
ENV PRECOMMIT_VERSION=4.5.1
ENV COMMITIZEN_VERSION=4.12.1
RUN pipx install pre-commit==${PRECOMMIT_VERSION} commitizen==${COMMITIZEN_VERSION}

# Update PATH
ENV PATH="${HOME}/.local/bin:${CARGO_HOME}/bin:${VERILATOR_INSTALL}/bin:${SLANG_SERVER_INSTALL}/bin:${SURFER_INSTALL}/bin:${PATH}"
