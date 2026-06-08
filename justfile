set shell := ["bash", "-euo", "pipefail", "-c"]

export RTL_ARTIFACTS_DIR := `. ./.devcontainer/env_contract.sh; printf '%s\n' "$RTL_ARTIFACTS_DIR"`
schema_path := `python3 -B -c 'import pathlib, tomllib; version = tomllib.loads(pathlib.Path("Cargo.toml").read_text(encoding="utf-8"))["package"]["version"]; print("schema/wavepeek_v" + version.split(".")[0] + ".json")'`
bench_e2e_runs_dir := "bench/e2e/runs"
bench_e2e_baseline_dir := "bench/e2e/runs/baseline_fst"
bench_e2e_fsdb_tests := "bench/e2e/tests_fsdb.json"
bench_e2e_fsdb_baseline_dir := "bench/e2e/runs/baseline_fsdb"
bench_e2e_fsdb_smoke_filter := "^(info_picorv32_ez|scope_scr1_all_depth7_json|signal_scr1_top_recursive_depth2_json|value_scr1_signals_1|change_scr1_signals_1_window_2ns_trigger_any)$"
bench_e2e_fsdb_smoke_artifact_filter := "^(picorv32_test_ez_vcd|scr1_max_axi_riscv_compliance)[.]fst$"
bench_expr_runs_dir := "bench/expr/runs"
bench_expr_baseline_dir := "bench/expr/runs/baseline"
wavepeek_release_bin := "./target/release/wavepeek"
wavepeek_fsdb_release_bin := "./target/fsdb/release/wavepeek"
codex_setup_script := "tools/codex/codex_setup.sh"
codex_resume_script := "tools/codex/codex_resume.sh"
python := "python3 -B"
docs_site_dir := "tmp/docs-site"
docs_version := `python3 -B -c 'import pathlib, tomllib; print(tomllib.loads(pathlib.Path("Cargo.toml").read_text(encoding="utf-8"))["package"]["version"])'`
coverage_src_threshold := env_var_or_default("COVERAGE_SRC_THRESHOLD", "90")

[private]
default: help

[private]
print-coverage-src-threshold:
    @printf '%s\n' "{{ coverage_src_threshold }}"

[private]
require-container:
    @if [ "${WAVEPEEK_IN_CONTAINER:-0}" != "1" ]; then \
        printf '%s\n' "error: container: this target must run inside a wavepeek-managed container environment (set WAVEPEEK_IN_CONTAINER=1)" >&2; \
        exit 1; \
    fi

[private]
require-verdi: require-container
    @{{ python }} tools/fsdb/check_fsdb_env.py --require >/dev/null

[private]
run-if-verdi recipe: require-container
    @set +e; \
    output="$({{ python }} tools/fsdb/check_fsdb_env.py 2>&1)"; \
    status="$?"; \
    set -e; \
    if [ "$status" -eq 0 ]; then \
        printf '%s\n' "$output"; \
        just "{{ recipe }}"; \
    elif [ "$status" -eq 77 ]; then \
        printf '%s\n' "$output"; \
    else \
        printf '%s\n' "$output" >&2; \
        exit "$status"; \
    fi

[private]
check-rtl-artifacts: require-container
    @. ./.devcontainer/env_contract.sh; \
    for fixture in $WAVEPEEK_RTL_ARTIFACT_FILES; do \
        if [ ! -f "${RTL_ARTIFACTS_DIR}/$fixture" ]; then \
            printf '%s\n' "error: file: required fixture missing at ${RTL_ARTIFACTS_DIR}/$fixture" >&2; \
            exit 1; \
        fi; \
    done

# Regenerate canonical schema artifact from runtime output
update-schema: require-container
    @mkdir -p schema
    @tmp_file="$(mktemp)"; trap 'rm -f "$tmp_file"' EXIT; \
        cargo run --quiet -- schema > "$tmp_file"; \
        mv "$tmp_file" "{{ schema_path }}"

# Validate canonical schema freshness and JSON contract URL
check-schema: require-container
    @{{ python }} tools/schema/check_schema_contract.py "{{ schema_path }}"

# Lint GitHub Actions workflows
check-actions: require-container
    actionlint .github/workflows/*.yml

# Regenerate FSDB benchmark catalog from the FST benchmark catalog
update-bench-e2e-fsdb-catalog: require-container
    @{{ python }} tools/fsdb/generate_bench_catalog.py

# Validate FSDB benchmark catalog freshness
check-bench-e2e-fsdb-catalog: require-container
    @{{ python }} tools/fsdb/generate_bench_catalog.py --check

# Prepare local devcontainer environment and install git hooks
dev-setup: require-container
    rustup show >/dev/null
    cargo --version
    cargo fmt --version
    cargo clippy --version
    actionlint -version
    devcontainer --version
    gtkwave --version
    surfer --version
    mkdocs --version
    mike --version
    just --version
    pre-commit install --hook-type commit-msg --hook-type pre-commit

# Prepare Codex cloud environment for non-dev just recipes
codex-setup: require-container
    bash "{{ codex_setup_script }}"

# Repair Codex cloud environment after cache resume
codex-resume: require-container
    bash "{{ codex_resume_script }}"

# Format root justfile in place
format-justfile: require-container
    @just --unstable --fmt

# Check root justfile formatting
format-justfile-check: require-container
    @just --unstable --fmt --check

# Format with rustfmt and justfile formatter
format: require-container
    cargo fmt
    just format-justfile

# Check formatting with rustfmt and justfile formatter
format-check: require-container
    cargo fmt -- --check
    just format-justfile-check

# Lint with clippy
lint: require-container
    cargo clippy --all-targets -- -D warnings
    just run-if-verdi lint-fsdb

# Fix linting with clippy
lint-fix: require-container
    cargo clippy --all-targets --fix --allow-dirty --allow-staged -- -D warnings

# Type check with cargo
check-build: require-container
    cargo check

# Run tests with cargo
test: require-container check-rtl-artifacts
    cargo test -q
    just run-if-verdi test-fsdb

[private]
coverage-src-data: require-container check-rtl-artifacts
    @mkdir -p tmp/coverage
    cargo llvm-cov --workspace --summary-only --json --ignore-filename-regex '(/tests/|/target/|/\.cargo/registry/|/rustc/)' > tmp/coverage/coverage-src-summary.json

# Report source coverage for src/**/*.rs via cargo-llvm-cov
coverage-src: coverage-src-data
    {{ python }} tools/coverage/check_coverage.py \
        --summary-json tmp/coverage/coverage-src-summary.json \
        --min-regions 0 \
        --min-functions 0 \
        --min-lines 0

# Enforce minimum source coverage for src/**/*.rs
coverage-src-check: coverage-src-data
    {{ python }} tools/coverage/check_coverage.py \
        --summary-json tmp/coverage/coverage-src-summary.json \
        --min-regions {{ coverage_src_threshold }} \
        --min-functions {{ coverage_src_threshold }} \
        --min-lines {{ coverage_src_threshold }} \
        --markdown-output tmp/coverage/coverage-src-summary.md

# Check local FSDB Reader SDK availability
check-fsdb-env: require-container
    @set +e; \
    {{ python }} tools/fsdb/check_fsdb_env.py; \
    status="$?"; \
    if [ "$status" -eq 77 ]; then \
        exit 0; \
    fi; \
    exit "$status"

# Lint optional FSDB support
lint-fsdb: require-verdi
    CARGO_TARGET_DIR=target/fsdb cargo clippy --features fsdb --all-targets -- -D warnings

# Prepare generated FSDB fixtures from VCD fixtures and RTL FST artifacts
prepare-fsdb-fixtures: require-verdi check-bench-e2e-fsdb-catalog
    bash tools/fsdb/prepare_fsdb_fixtures.sh

# Prepare generated FSDB fixtures from hand-written VCD test fixtures only
prepare-fsdb-test-fixtures: require-verdi
    bash tools/fsdb/prepare_fsdb_fixtures.sh --hand-only

# Verify FSDB benchmark artifacts exist next to required RTL FST fixtures
check-fsdb-rtl-artifacts: require-verdi check-rtl-artifacts
    {{ python }} tools/fsdb/check_fsdb_bench_artifacts.py "{{ bench_e2e_fsdb_tests }}"

# Prepare and verify FSDB benchmark artifacts in dependency order
prepare-and-check-fsdb-rtl-artifacts: require-verdi
    just check-rtl-artifacts
    just prepare-fsdb-fixtures
    {{ python }} tools/fsdb/check_fsdb_bench_artifacts.py "{{ bench_e2e_fsdb_tests }}"

# Prepare and verify only FSDB RTL artifacts required by the pre-commit smoke
prepare-and-check-fsdb-smoke-rtl-artifacts: require-verdi
    just check-rtl-artifacts
    just check-bench-e2e-fsdb-catalog
    bash tools/fsdb/prepare_fsdb_fixtures.sh --rtl-only --rtl-filter '{{ bench_e2e_fsdb_smoke_artifact_filter }}'
    {{ python }} tools/fsdb/check_fsdb_bench_artifacts.py "{{ bench_e2e_fsdb_tests }}" --filter '{{ bench_e2e_fsdb_smoke_filter }}'

# Build release binary with optional FSDB support
build-release-fsdb: require-verdi
    CARGO_TARGET_DIR=target/fsdb cargo build --release --features fsdb

# Build and smoke-test optional FSDB support
check-fsdb-build: require-verdi
    @fsdb_libdir="$({{ python }} tools/fsdb/check_fsdb_env.py --require --print-libdir)"; \
    export CARGO_TARGET_DIR=target/fsdb; \
    cargo check --features fsdb; \
    cargo build --features fsdb; \
    readelf_output="$(readelf -d target/fsdb/debug/wavepeek)"; \
    if printf '%s\n' "$readelf_output" | grep -Eq '\(NEEDED\).*Shared library: \[/'; then \
        printf '%s\n' "error: fsdb: built binary must not contain an absolute DT_NEEDED path" >&2; \
        exit 1; \
    fi; \
    if ! printf '%s\n' "$readelf_output" | grep -E '\((RPATH|RUNPATH)\)' | grep -F -- "$fsdb_libdir" >/dev/null; then \
        printf '%s\n' "error: fsdb: built binary must contain an ELF RPATH/RUNPATH for $fsdb_libdir" >&2; \
        exit 1; \
    fi; \
    cargo test --features fsdb --lib fsdb_reader_metadata_smoke -- --nocapture; \
    cargo test --features fsdb --lib fsdb_reader_hierarchy_smoke -- --nocapture

# Run optional FSDB build smoke tests
test-fsdb: check-fsdb-build prepare-fsdb-test-fixtures
    @export CARGO_TARGET_DIR=target/fsdb; \
    cargo test --features fsdb --lib fsdb_expr_event_occurred_rejects_non_event_signal -- --nocapture && \
    cargo test --features fsdb --test fsdb_cli

# Run auxiliary Python/unit test suites
test-aux: require-container
    @just check-bench-e2e-fsdb-catalog
    {{ python }} -m unittest discover -s bench/e2e -p "test_*.py"
    {{ python }} -m unittest discover -s bench/expr -p "test_*.py"
    {{ python }} -m unittest discover -s tools/docs -p "test_*.py"
    {{ python }} -m unittest discover -s tools/release -p "test_*.py"
    {{ python }} -m unittest tools/coverage/test_check_coverage.py
    {{ python }} -m unittest discover -s tools/fsdb -p "test_*.py"
    {{ python }} -m unittest discover -s tools/repo -p "test_*.py"

# Build the generated MkDocs site from current embedded docs
docs-site-build: require-container
    cargo run --quiet -- docs export "{{ docs_site_dir }}/export" --force
    {{ python }} tools/docs/prepare_mkdocs.py "{{ docs_site_dir }}/export" \
        --output "{{ docs_site_dir }}/mkdocs-src" \
        --config-output "{{ docs_site_dir }}/mkdocs.yml" \
        --version "{{ docs_version }}" \
        --force
    mkdocs build --strict --config-file "{{ docs_site_dir }}/mkdocs.yml"

# Serve the generated docs site locally
docs-site-serve: docs-site-build
    mkdocs serve --config-file "{{ docs_site_dir }}/mkdocs.yml"

# Check docs site generation and root Pages artifacts without touching gh-pages
docs-site-check: require-container
    {{ python }} tools/docs/publish_docs.py check \
        --version "{{ docs_version }}" \
        --source-root . \
        --work-dir "{{ docs_site_dir }}"

# Stage a local gh-pages update for a release tag without pushing
docs-site-stage-deploy version=docs_version source_ref=("v" + docs_version) repair="0": require-container
    @repair_arg=""; \
        if [ "{{ repair }}" = "1" ] || [ "{{ repair }}" = "true" ]; then \
            repair_arg="--repair-existing-version"; \
        fi; \
        {{ python }} tools/docs/publish_docs.py stage-deploy \
            --version "{{ version }}" \
            --source-ref "{{ source_ref }}" \
            --work-dir "{{ docs_site_dir }}" \
            $repair_arg

# Verify and push the staged gh-pages bundle
docs-site-push-staged version=docs_version repair="0": require-container
    @repair_arg=""; \
        if [ "{{ repair }}" = "1" ] || [ "{{ repair }}" = "true" ]; then \
            repair_arg="--repair-existing-version"; \
        fi; \
        {{ python }} tools/docs/publish_docs.py push-staged \
            --version "{{ version }}" \
            --work-dir "{{ docs_site_dir }}" \
            $repair_arg

# Stage and push a release docs update
docs-site-deploy version=docs_version source_ref=("v" + docs_version) repair="0": require-container
    just docs-site-stage-deploy "{{ version }}" "{{ source_ref }}" "{{ repair }}"
    just docs-site-push-staged "{{ version }}" "{{ repair }}"

# Manually dispatch the remote docs publication workflow
docs-site-dispatch version=docs_version source_ref=("v" + docs_version) repair="false" ref="main": require-container
    gh workflow run docs.yml \
        --ref "{{ ref }}" \
        -f version="{{ version }}" \
        -f source_ref="{{ source_ref }}" \
        -F repair_existing_version="{{ repair }}"

# Build release binary
build-release: require-container
    cargo build --release

# Refresh benchmark e2e baseline artifacts
bench-e2e-update-baseline: check-rtl-artifacts build-release
    @mkdir -p "{{ bench_e2e_runs_dir }}"; tmp_parent="$(mktemp -d "{{ bench_e2e_runs_dir }}/baseline_fst.tmp.XXXXXX")"; tmp_baseline="$tmp_parent/baseline"; trap 'rm -rf "$tmp_parent"' EXIT; \
        WAVEPEEK_BIN="{{ wavepeek_release_bin }}" {{ python }} bench/e2e/perf.py run --run-dir "$tmp_baseline" && \
        rm -rf "{{ bench_e2e_baseline_dir }}" && \
        mv "$tmp_baseline" "{{ bench_e2e_baseline_dir }}"

# Run benchmark e2e suite with baseline compare
bench-e2e-run: check-rtl-artifacts build-release
    WAVEPEEK_BIN="{{ wavepeek_release_bin }}" {{ python }} bench/e2e/perf.py run --compare "{{ bench_e2e_baseline_dir }}"

# Refresh FSDB benchmark e2e baseline artifacts
bench-e2e-fsdb-update-baseline: prepare-and-check-fsdb-rtl-artifacts build-release-fsdb
    @mkdir -p "{{ bench_e2e_runs_dir }}"; tmp_parent="$(mktemp -d "{{ bench_e2e_runs_dir }}/baseline_fsdb.tmp.XXXXXX")"; tmp_baseline="$tmp_parent/baseline"; trap 'rm -rf "$tmp_parent"' EXIT; \
        WAVEPEEK_BIN="{{ wavepeek_fsdb_release_bin }}" {{ python }} bench/e2e/perf.py run --tests "{{ bench_e2e_fsdb_tests }}" --run-dir "$tmp_baseline" && \
        rm -rf "{{ bench_e2e_fsdb_baseline_dir }}" && \
        mv "$tmp_baseline" "{{ bench_e2e_fsdb_baseline_dir }}"

# Run FSDB benchmark e2e suite with FSDB baseline compare
bench-e2e-fsdb-run: prepare-and-check-fsdb-rtl-artifacts build-release-fsdb
    WAVEPEEK_BIN="{{ wavepeek_fsdb_release_bin }}" {{ python }} bench/e2e/perf.py run --tests "{{ bench_e2e_fsdb_tests }}" --compare "{{ bench_e2e_fsdb_baseline_dir }}"

# Refresh expression benchmark baseline artifacts
bench-expr-update-baseline: require-container
    @tmp_parent="$(mktemp -d)"; tmp_baseline="$tmp_parent/baseline"; trap 'rm -rf "$tmp_parent"' EXIT; \
        {{ python }} bench/expr/perf.py run --run-dir "$tmp_baseline" && \
        rm -rf "{{ bench_expr_baseline_dir }}" && \
        mv "$tmp_baseline" "{{ bench_expr_baseline_dir }}"

# Run expression benchmark suite with baseline compare
bench-expr-run: require-container
    @tmp_revised="$(mktemp -d)"; trap 'rm -rf "$tmp_revised"' EXIT; \
        {{ python }} bench/expr/perf.py run --run-dir "$tmp_revised" --compare "{{ bench_expr_baseline_dir }}" && \
        {{ python }} bench/expr/perf.py compare --revised "$tmp_revised" --golden "{{ bench_expr_baseline_dir }}" --max-negative-delta-pct 15 --require-matching-metadata cargo_version rustc_version criterion_version environment_note

# Run lightweight benchmark e2e smoke for pre-commit
bench-e2e-smoke-commit: check-rtl-artifacts build-release
    @tmp_revised="$(mktemp -d)"; trap 'rm -rf "$tmp_revised"' EXIT; \
        WAVEPEEK_BIN="{{ wavepeek_release_bin }}" {{ python }} bench/e2e/perf.py run --tests bench/e2e/tests_commit.json --run-dir "$tmp_revised" && \
        WAVEPEEK_BIN="{{ wavepeek_release_bin }}" {{ python }} bench/e2e/perf.py compare --revised "$tmp_revised" --golden "{{ bench_e2e_baseline_dir }}" --max-negative-delta-pct 100
    @just run-if-verdi bench-e2e-fsdb-smoke-commit

# Run lightweight FSDB benchmark e2e smoke for pre-commit
bench-e2e-fsdb-smoke-commit: prepare-and-check-fsdb-smoke-rtl-artifacts build-release-fsdb
    @tmp_revised="$(mktemp -d)"; trap 'rm -rf "$tmp_revised"' EXIT; \
        WAVEPEEK_BIN="{{ wavepeek_fsdb_release_bin }}" {{ python }} bench/e2e/perf.py run --tests "{{ bench_e2e_fsdb_tests }}" --run-dir "$tmp_revised" --filter '{{ bench_e2e_fsdb_smoke_filter }}' && \
        WAVEPEEK_BIN="{{ wavepeek_fsdb_release_bin }}" {{ python }} bench/e2e/perf.py compare --functional-only --allow-golden-extra --revised "$tmp_revised" --golden "{{ bench_e2e_fsdb_baseline_dir }}"

# Run pre-commit hooks on all files
pre-commit: require-container check-rtl-artifacts
    pre-commit run --all-files

# Check commit messages
check-commit: require-container
    cz check --commit-msg-file "$(git rev-parse --git-path COMMIT_EDITMSG)"

# Check everything
check: format-check lint check-schema check-actions check-bench-e2e-fsdb-catalog check-build docs-site-check check-commit
    @just run-if-verdi check-fsdb-build

# CI quality gate (no commit-msg hook)
ci: format-check lint check-schema check-actions test-aux coverage-src-check check-build docs-site-check
    @just run-if-verdi test-fsdb

# Fix everything
fix: format lint-fix update-schema

# Clean up
clean: require-container
    cargo clean

# Show recipes
help: require-container
    @just --list
