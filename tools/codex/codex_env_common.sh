#!/usr/bin/env bash
set -euo pipefail

# Codex cloud setup is maintained manually as a derivative of the repository's
# devcontainer contract. Keep `.devcontainer/` as the primary source of truth,
# especially `.devcontainer/env_contract.sh`, and update these tools whenever
# the devcontainer image contents or environment guarantees change.

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
# shellcheck source=../../.devcontainer/env_contract.sh
source "${REPO_ROOT}/.devcontainer/env_contract.sh"

readonly WAVEPEEK_CODEX_BIN_DIR="${HOME}/.local/bin"
readonly WAVEPEEK_CODEX_RTL_ARTIFACTS_PATH="$RTL_ARTIFACTS_DIR"
export RTL_ARTIFACTS_DIR="$WAVEPEEK_CODEX_RTL_ARTIFACTS_PATH"

log() {
    printf '%s\n' "$*"
}

error() {
    printf 'error: setup: %s\n' "$*" >&2
    exit 1
}

ensure_safe_directory() {
    local repo_root
    repo_root="$(git rev-parse --show-toplevel)"

    if git config --global --get-all safe.directory | grep -Fx "$repo_root" >/dev/null 2>&1; then
        return
    fi

    git config --global --add safe.directory "$repo_root"
}

ensure_local_bin_dir() {
    mkdir -p "$WAVEPEEK_CODEX_BIN_DIR"
    case ":$PATH:" in
        *":$WAVEPEEK_CODEX_BIN_DIR:"*) ;;
        *) export PATH="$WAVEPEEK_CODEX_BIN_DIR:$PATH" ;;
    esac
}

ensure_bashrc_line() {
    local line="$1"
    local bashrc="${HOME}/.bashrc"

    touch "$bashrc"

    if grep -Fqx "$line" "$bashrc"; then
        return
    fi

    printf '%s\n' "$line" >> "$bashrc"
}

persist_shell_env() {
    ensure_bashrc_line 'export PATH="$HOME/.local/bin:$HOME/.cargo/bin:$PATH"'
    ensure_bashrc_line "export RTL_ARTIFACTS_DIR=\"$WAVEPEEK_CODEX_RTL_ARTIFACTS_PATH\""
}

ensure_rust_toolchain() {
    local current_version
    current_version="$(rustc --version | awk '{print $2}')"

    if [ "$current_version" != "$WAVEPEEK_RUST_VERSION" ]; then
        log "Selecting Rust ${WAVEPEEK_RUST_VERSION}"
        rustup toolchain install "$WAVEPEEK_RUST_VERSION" --profile minimal
        rustup default "$WAVEPEEK_RUST_VERSION"
    fi

    rustup component add rustfmt clippy llvm-tools-preview --toolchain "$WAVEPEEK_RUST_VERSION"
}

ensure_cargo_llvm_cov() {
    local current_version=""

    if command -v cargo-llvm-cov >/dev/null 2>&1; then
        current_version="$(cargo llvm-cov --version | awk '{print $2}')"
    fi

    if [ "$current_version" = "$WAVEPEEK_CARGO_LLVM_COV_VERSION" ]; then
        return
    fi

    log "Installing cargo-llvm-cov ${WAVEPEEK_CARGO_LLVM_COV_VERSION}"
    cargo install --locked cargo-llvm-cov --version "$WAVEPEEK_CARGO_LLVM_COV_VERSION"
}

ensure_just() {
    local current_version=""

    if command -v just >/dev/null 2>&1; then
        current_version="$(just --version | awk '{print $2}')"
    fi

    if [ "$current_version" = "$WAVEPEEK_JUST_VERSION" ]; then
        return
    fi

    log "Installing just ${WAVEPEEK_JUST_VERSION}"
    cargo install --locked just --version "$WAVEPEEK_JUST_VERSION" --root "${HOME}/.local"
}

install_actionlint() {
    local arch
    local status
    local tmp_dir

    arch="$(uname -m)"
    case "$arch" in
        x86_64)
            arch="amd64"
            ;;
        aarch64|arm64)
            arch="arm64"
            ;;
        *)
            error "unsupported actionlint architecture: $arch"
            ;;
    esac

    tmp_dir="$(mktemp -d)"
    if {
        curl -fsSL -o "$tmp_dir/actionlint.tar.gz" \
            "https://github.com/rhysd/actionlint/releases/download/v${WAVEPEEK_ACTIONLINT_VERSION}/actionlint_${WAVEPEEK_ACTIONLINT_VERSION}_linux_${arch}.tar.gz"
        tar -xzf "$tmp_dir/actionlint.tar.gz" -C "$tmp_dir" actionlint
        install -m 0755 "$tmp_dir/actionlint" "$WAVEPEEK_CODEX_BIN_DIR/actionlint"
    }; then
        status=0
    else
        status=$?
    fi

    rm -rf "$tmp_dir"
    return "$status"
}

ensure_actionlint() {
    local current_version=""

    if command -v actionlint >/dev/null 2>&1; then
        current_version="$(actionlint -version | awk 'NR==1 {print $1}')"
    fi

    if [ "$current_version" = "$WAVEPEEK_ACTIONLINT_VERSION" ]; then
        return
    fi

    log "Installing actionlint ${WAVEPEEK_ACTIONLINT_VERSION}"
    install_actionlint
}

install_gh() {
    local arch
    local status
    local tmp_dir

    arch="$(uname -m)"
    case "$arch" in
        x86_64)
            arch="amd64"
            ;;
        aarch64|arm64)
            arch="arm64"
            ;;
        *)
            error "unsupported GitHub CLI architecture: $arch"
            ;;
    esac

    tmp_dir="$(mktemp -d)"
    if {
        curl --retry 5 --retry-delay 5 --retry-all-errors --connect-timeout 30 -fsSL -o "$tmp_dir/gh.tar.gz" \
            "https://github.com/cli/cli/releases/download/v${WAVEPEEK_GH_VERSION}/gh_${WAVEPEEK_GH_VERSION}_linux_${arch}.tar.gz"
        tar -xzf "$tmp_dir/gh.tar.gz" -C "$tmp_dir"
        install -m 0755 "$tmp_dir/gh_${WAVEPEEK_GH_VERSION}_linux_${arch}/bin/gh" "$WAVEPEEK_CODEX_BIN_DIR/gh"
    }; then
        status=0
    else
        status=$?
    fi

    rm -rf "$tmp_dir"
    return "$status"
}

ensure_gh() {
    local current_version=""

    if command -v gh >/dev/null 2>&1; then
        current_version="$(gh --version | awk 'NR==1 {print $3}')"
    fi

    if [ "$current_version" = "$WAVEPEEK_GH_VERSION" ]; then
        return
    fi

    log "Installing GitHub CLI ${WAVEPEEK_GH_VERSION}"
    install_gh
}

install_hyperfine() {
    local arch
    local asset_name
    local status
    local tmp_dir

    arch="$(uname -m)"
    case "$arch" in
        x86_64)
            asset_name="hyperfine-v${WAVEPEEK_HYPERFINE_VERSION}-x86_64-unknown-linux-gnu.tar.gz"
            ;;
        aarch64|arm64)
            asset_name="hyperfine-v${WAVEPEEK_HYPERFINE_VERSION}-aarch64-unknown-linux-gnu.tar.gz"
            ;;
        *)
            error "unsupported hyperfine architecture: $arch"
            ;;
    esac

    tmp_dir="$(mktemp -d)"
    if {
        curl -fsSL -o "$tmp_dir/hyperfine.tar.gz" \
            "https://github.com/sharkdp/hyperfine/releases/download/v${WAVEPEEK_HYPERFINE_VERSION}/${asset_name}"
        tar -xzf "$tmp_dir/hyperfine.tar.gz" -C "$tmp_dir"
        install -m 0755 "$tmp_dir"/*/hyperfine "$WAVEPEEK_CODEX_BIN_DIR/hyperfine"
    }; then
        status=0
    else
        status=$?
    fi

    rm -rf "$tmp_dir"
    return "$status"
}

ensure_hyperfine() {
    local current_version=""

    if command -v hyperfine >/dev/null 2>&1; then
        current_version="$(hyperfine --version | awk '{print $2}')"
    fi

    if [ "$current_version" = "$WAVEPEEK_HYPERFINE_VERSION" ]; then
        return
    fi

    log "Installing hyperfine ${WAVEPEEK_HYPERFINE_VERSION}"
    install_hyperfine
}

install_iverilog_from_deb() {
    local apt_root
    local install_root
    local download_dir
    local deb
    local ivl_dir

    command -v apt-get >/dev/null 2>&1 || error "apt-get is required to install Icarus Verilog in Codex setup"
    command -v dpkg-deb >/dev/null 2>&1 || error "dpkg-deb is required to install Icarus Verilog in Codex setup"

    apt_root="${HOME}/.cache/wavepeek-codex/apt-iverilog"
    install_root="${HOME}/.local/opt/wavepeek-iverilog"
    download_dir="$apt_root/download"
    rm -rf "$apt_root"
    mkdir -p "$apt_root/lists/partial" "$apt_root/cache/archives/partial" "$download_dir"

    apt-get \
        -o Dir::State::Lists="$apt_root/lists" \
        -o Dir::Cache="$apt_root/cache" \
        -o Dir::State::status=/var/lib/dpkg/status \
        update >/dev/null

    (
        cd "$download_dir"
        apt-get \
            -o Dir::State::Lists="$apt_root/lists" \
            -o Dir::Cache="$apt_root/cache" \
            -o Dir::State::status=/var/lib/dpkg/status \
            download iverilog >/dev/null
    )

    deb="$(find "$download_dir" -maxdepth 1 -type f -name 'iverilog_*.deb' | sort | head -n 1)"
    [ -n "$deb" ] || error "failed to download Icarus Verilog package"

    rm -rf "$install_root"
    mkdir -p "$install_root"
    dpkg-deb -x "$deb" "$install_root"
    ivl_dir="$(find "$install_root/usr/lib" -type d -name ivl | sort | head -n 1)"
    [ -n "$ivl_dir" ] || error "installed Icarus Verilog package did not contain an ivl directory"

    cat >"$WAVEPEEK_CODEX_BIN_DIR/iverilog" <<EOF
#!/usr/bin/env bash
exec "$install_root/usr/bin/iverilog" -B "$ivl_dir" "\$@"
EOF
    cat >"$WAVEPEEK_CODEX_BIN_DIR/vvp" <<EOF
#!/usr/bin/env bash
exec "$install_root/usr/bin/vvp" -M "$ivl_dir" "\$@"
EOF
    chmod 0755 "$WAVEPEEK_CODEX_BIN_DIR/iverilog" "$WAVEPEEK_CODEX_BIN_DIR/vvp"
}

ensure_iverilog() {
    if command -v iverilog >/dev/null 2>&1 && command -v vvp >/dev/null 2>&1; then
        return
    fi

    log "Installing Icarus Verilog"
    install_iverilog_from_deb
}

ensure_waveform_tools() {
    ensure_iverilog
    command -v vcd2fst >/dev/null 2>&1 || error "vcd2fst is required for waveform fixture generation; use the devcontainer or install GTKWave tools"
    command -v fst2vcd >/dev/null 2>&1 || error "fst2vcd is required for FSDB fixture preparation; use the devcontainer or install GTKWave tools"
}

ensure_pipx_package() {
    local package_name="$1"
    local package_version="$2"
    local command_name="$3"
    local version_args="$4"
    local current_version=""

    if command -v "$command_name" >/dev/null 2>&1; then
        current_version="$($command_name $version_args | awk 'NR==1 {print $NF}')"
    fi

    if [ "$current_version" = "$package_version" ]; then
        return
    fi

    log "Installing ${package_name} ${package_version}"
    pipx install --force "${package_name}==${package_version}"
}

ensure_rtl_artifacts_dir() {
    mkdir -p "$WAVEPEEK_CODEX_RTL_ARTIFACTS_PATH"
}

ensure_rtl_artifacts() {
    local artifact
    local tmp_file

    ensure_rtl_artifacts_dir

    for artifact in ${WAVEPEEK_RTL_ARTIFACT_FILES}; do
        if [ -f "$WAVEPEEK_CODEX_RTL_ARTIFACTS_PATH/$artifact" ]; then
            continue
        fi

        log "Downloading RTL artifact: $artifact"
        tmp_file="$(mktemp)"
        curl -fsSL -o "$tmp_file" \
            "https://github.com/kleverhq/rtl-artifacts/releases/download/${WAVEPEEK_RTL_ARTIFACTS_VERSION}/${artifact}"
        install -m 0644 "$tmp_file" "$WAVEPEEK_CODEX_RTL_ARTIFACTS_PATH/$artifact"
        rm -f "$tmp_file"
    done
}

ensure_cargo_fetch() {
    log "Fetching Cargo dependencies"
    cargo fetch --locked
}

ensure_codex_tooling() {
    ensure_safe_directory
    ensure_local_bin_dir
    persist_shell_env
    ensure_rust_toolchain
    ensure_just
    ensure_cargo_llvm_cov
    ensure_actionlint
    ensure_gh
    ensure_hyperfine
    ensure_waveform_tools
    ensure_pipx_package pre-commit "$WAVEPEEK_PRECOMMIT_VERSION" pre-commit --version
    ensure_pipx_package commitizen "$WAVEPEEK_COMMITIZEN_VERSION" cz version
    ensure_rtl_artifacts
}
