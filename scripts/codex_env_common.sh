#!/usr/bin/env bash
set -euo pipefail

# Codex cloud setup is maintained manually as a derivative of the repository's
# devcontainer contract. Keep `.devcontainer/` as the primary source of truth,
# especially `.devcontainer/env_contract.sh`, and update this script whenever
# the devcontainer image contents or environment guarantees change.

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
# shellcheck source=../.devcontainer/env_contract.sh
source "${REPO_ROOT}/.devcontainer/env_contract.sh"

readonly WAVEPEEK_CODEX_BIN_DIR="${HOME}/.local/bin"
readonly WAVEPEEK_CODEX_RTL_ARTIFACTS_DIR="$("${REPO_ROOT}/.devcontainer/resolve_rtl_artifacts_dir.sh")"
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
    ensure_bashrc_line "export RTL_ARTIFACTS_DIR=\"$WAVEPEEK_CODEX_RTL_ARTIFACTS_DIR\""
    ensure_bashrc_line "export WAVEPEEK_RTL_ARTIFACTS_DIR=\"$WAVEPEEK_CODEX_RTL_ARTIFACTS_DIR\""
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

install_actionlint() {
    local arch
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
    trap 'rm -rf "$tmp_dir"' RETURN

    curl -fsSL -o "$tmp_dir/actionlint.tar.gz" \
        "https://github.com/rhysd/actionlint/releases/download/v${WAVEPEEK_ACTIONLINT_VERSION}/actionlint_${WAVEPEEK_ACTIONLINT_VERSION}_linux_${arch}.tar.gz"
    tar -xzf "$tmp_dir/actionlint.tar.gz" -C "$tmp_dir" actionlint
    install -m 0755 "$tmp_dir/actionlint" "$WAVEPEEK_CODEX_BIN_DIR/actionlint"
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

install_hyperfine() {
    local arch
    local asset_name
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
    trap 'rm -rf "$tmp_dir"' RETURN

    curl -fsSL -o "$tmp_dir/hyperfine.tar.gz" \
        "https://github.com/sharkdp/hyperfine/releases/download/v${WAVEPEEK_HYPERFINE_VERSION}/${asset_name}"
    tar -xzf "$tmp_dir/hyperfine.tar.gz" -C "$tmp_dir"
    install -m 0755 "$tmp_dir"/*/hyperfine "$WAVEPEEK_CODEX_BIN_DIR/hyperfine"
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
    mkdir -p "$WAVEPEEK_CODEX_RTL_ARTIFACTS_DIR"
}

ensure_rtl_artifacts() {
    local artifact
    local tmp_file

    ensure_rtl_artifacts_dir

    for artifact in ${WAVEPEEK_RTL_ARTIFACT_FILES}; do
        if [ -f "$WAVEPEEK_CODEX_RTL_ARTIFACTS_DIR/$artifact" ]; then
            continue
        fi

        log "Downloading RTL artifact: $artifact"
        tmp_file="$(mktemp)"
        curl -fsSL -o "$tmp_file" \
            "https://github.com/kleverhq/rtl-artifacts/releases/download/${WAVEPEEK_RTL_ARTIFACTS_VERSION}/${artifact}"
        install -m 0644 "$tmp_file" "$WAVEPEEK_CODEX_RTL_ARTIFACTS_DIR/$artifact"
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
    ensure_cargo_llvm_cov
    ensure_actionlint
    ensure_hyperfine
    ensure_pipx_package pre-commit "$WAVEPEEK_PRECOMMIT_VERSION" pre-commit --version
    ensure_pipx_package commitizen "$WAVEPEEK_COMMITIZEN_VERSION" cz version
    ensure_rtl_artifacts
}
