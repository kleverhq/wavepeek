# ---- RTL Fixture Layer --------------------------------------------------------
FROM ubuntu:24.04 AS rtl_artifacts

ENV DEBIAN_FRONTEND=noninteractive
COPY .devcontainer/env_contract.sh /tmp/wavepeek-env-contract.sh
RUN apt-get update && \
    apt-get install -y --no-install-recommends ca-certificates curl && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/*

RUN . /tmp/wavepeek-env-contract.sh && \
    mkdir -p "${RTL_ARTIFACTS_DIR}" && \
    for artifact in ${WAVEPEEK_RTL_ARTIFACT_FILES}; do \
      curl --retry 5 --retry-delay 5 --retry-all-errors --connect-timeout 30 -fsSL -o "${RTL_ARTIFACTS_DIR}/${artifact}" \
        "https://github.com/kleverhq/rtl-artifacts/releases/download/${WAVEPEEK_RTL_ARTIFACTS_VERSION}/${artifact}"; \
    done

# ---- Tool Image ---------------------------------------------------------------
FROM ubuntu:24.04

ARG TARGETARCH
ENV DEBIAN_FRONTEND=noninteractive
COPY .devcontainer/env_contract.sh /tmp/wavepeek-env-contract.sh

RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    ca-certificates curl git libatomic1 g++ \
    python3 python3-venv \
    iverilog gtkwave \
    libxt6t64 libxmu6 libnuma1 zlib1g-dev \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*

COPY requirements-docs.txt /tmp/wavepeek-requirements-docs.txt
ENV PLAYWRIGHT_BROWSERS_PATH=/opt/playwright-browsers
RUN . /tmp/wavepeek-env-contract.sh && \
    python3 -m venv /opt/wavepeek-python && \
    /opt/wavepeek-python/bin/pip install --no-cache-dir \
        -r /tmp/wavepeek-requirements-docs.txt \
        "pre-commit==${WAVEPEEK_PRECOMMIT_VERSION}" \
        "commitizen==${WAVEPEEK_COMMITIZEN_VERSION}" \
        "playwright==${WAVEPEEK_PLAYWRIGHT_VERSION}" && \
    rm -f /tmp/wavepeek-requirements-docs.txt
RUN /opt/wavepeek-python/bin/playwright install --with-deps --only-shell chromium && \
    chmod -R a+rX "${PLAYWRIGHT_BROWSERS_PATH}" && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/*

# Install GitHub Actions workflow linter for hooks and CI checks.
RUN . /tmp/wavepeek-env-contract.sh && \
    arch="${TARGETARCH:-amd64}" && \
    case "$arch" in \
        amd64|arm64) actionlint_arch="$arch" ;; \
        *) printf '%s\n' "unsupported actionlint architecture: $arch" >&2; exit 1 ;; \
    esac && \
    tmp_dir="$(mktemp -d)" && \
    curl --retry 5 --retry-delay 5 --retry-all-errors --connect-timeout 30 -fsSL -o "$tmp_dir/actionlint.tar.gz" \
        "https://github.com/rhysd/actionlint/releases/download/v${WAVEPEEK_ACTIONLINT_VERSION}/actionlint_${WAVEPEEK_ACTIONLINT_VERSION}_linux_${actionlint_arch}.tar.gz" && \
    tar -xzf "$tmp_dir/actionlint.tar.gz" -C "$tmp_dir" actionlint && \
    install -m 0755 "$tmp_dir/actionlint" /usr/local/bin/actionlint && \
    rm -rf "$tmp_dir"

# Install the pinned Hyperfine release for benchmark tooling.
RUN . /tmp/wavepeek-env-contract.sh && \
    arch="${TARGETARCH:-amd64}" && \
    case "$arch" in \
        amd64) hyperfine_arch="x86_64-unknown-linux-gnu" ;; \
        arm64) hyperfine_arch="aarch64-unknown-linux-gnu" ;; \
        *) printf '%s\n' "unsupported hyperfine architecture: $arch" >&2; exit 1 ;; \
    esac && \
    tmp_dir="$(mktemp -d)" && \
    curl --retry 5 --retry-delay 5 --retry-all-errors --connect-timeout 30 -fsSL -o "$tmp_dir/hyperfine.tar.gz" \
        "https://github.com/sharkdp/hyperfine/releases/download/v${WAVEPEEK_HYPERFINE_VERSION}/hyperfine-v${WAVEPEEK_HYPERFINE_VERSION}-${hyperfine_arch}.tar.gz" && \
    tar -xzf "$tmp_dir/hyperfine.tar.gz" -C "$tmp_dir" && \
    install -m 0755 "$tmp_dir"/*/hyperfine /usr/local/bin/hyperfine && \
    rm -rf "$tmp_dir"

# Install pinned GitHub CLI for repository recipes.
RUN . /tmp/wavepeek-env-contract.sh && \
    arch="${TARGETARCH:-amd64}" && \
    case "$arch" in \
        amd64|arm64) gh_arch="$arch" ;; \
        *) printf '%s\n' "unsupported GitHub CLI architecture: $arch" >&2; exit 1 ;; \
    esac && \
    tmp_dir="$(mktemp -d)" && \
    curl --retry 5 --retry-delay 5 --retry-all-errors --connect-timeout 30 -fsSL -o "$tmp_dir/gh.tar.gz" \
        "https://github.com/cli/cli/releases/download/v${WAVEPEEK_GH_VERSION}/gh_${WAVEPEEK_GH_VERSION}_linux_${gh_arch}.tar.gz" && \
    tar -xzf "$tmp_dir/gh.tar.gz" -C "$tmp_dir" && \
    install -m 0755 "$tmp_dir/gh_${WAVEPEEK_GH_VERSION}_linux_${gh_arch}/bin/gh" /usr/local/bin/gh && \
    rm -rf "$tmp_dir"

# Expose selected command-line Verdi FSDB utilities when /opt/verdi is mounted.
COPY --chmod=0755 .devcontainer/verdi-tool-wrapper.sh /usr/local/bin/wavepeek-verdi-tool
RUN for tool in \
        vcd2fsdb fsdb2vcd fsdbdebug fsdbreport fsdbextract fsdbmerge \
        fsdbjoin fsdbrecover fsdbsanitycheck fsdbqry fsdbdir fsdbedit \
        fsdbmangle fsdb2saif fsdb2ns txt2fsdb merge_multi_swp_fsdb \
        waveutils parallelWaveExpand; \
    do \
        ln -sf wavepeek-verdi-tool "/usr/local/bin/$tool"; \
    done

ENV LANG=C.UTF-8
ENV LC_ALL=C.UTF-8
ENV RTL_ARTIFACTS_DIR=/opt/rtl-artifacts
ENV VERDI_HOME=/opt/verdi
ENV PATH="/opt/wavepeek-python/bin:/home/ubuntu/.local/bin:/home/ubuntu/.cargo/bin:${PATH}"

RUN install -d -m 0755 -o ubuntu -g ubuntu /home/ubuntu/.local /home/ubuntu/.local/bin /workspaces

COPY --from=rtl_artifacts --chown=ubuntu:ubuntu /opt/rtl-artifacts ${RTL_ARTIFACTS_DIR}

USER ubuntu
ENV HOME=/home/ubuntu
ENV RUSTUP_HOME=/home/ubuntu/.rustup
ENV CARGO_HOME=/home/ubuntu/.cargo
RUN . /tmp/wavepeek-env-contract.sh && \
    curl https://sh.rustup.rs -sSf | sh -s -- -y --profile minimal && \
    rustup toolchain install ${WAVEPEEK_RUST_VERSION} --profile minimal && \
    rustup default ${WAVEPEEK_RUST_VERSION} && \
    rustup component add rustfmt clippy llvm-tools-preview --toolchain ${WAVEPEEK_RUST_VERSION} && \
    rustup target add wasm32-unknown-unknown --toolchain ${WAVEPEEK_RUST_VERSION} && \
    cargo install --locked cargo-llvm-cov --version ${WAVEPEEK_CARGO_LLVM_COV_VERSION} && \
    cargo install --locked wasm-bindgen-cli --version ${WAVEPEEK_WASM_BINDGEN_VERSION} && \
    cargo install --locked just --version ${WAVEPEEK_JUST_VERSION} --root /home/ubuntu/.local
