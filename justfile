set shell := ["bash", "-euo", "pipefail", "-c"]

export ONDAS_FIXTURES_DIR := `. ./.devcontainer/env_contract.sh; printf '%s\n' "$ONDAS_FIXTURES_DIR"`
bench_e2e_fsdb_tests := "bench/e2e/tests_fsdb.json"
bench_e2e_fsdb_smoke_filter := "^(info_picorv32_ez|scope_scr1_all_depth7_json|signal_scr1_top_recursive_depth2_json|value_scr1_signals_1|change_scr1_signals_1_window_2ns_trigger_any)$"
bench_e2e_fsdb_smoke_artifact_filter := "^(fst0012-picorv32-test-ez-vcd|fst0027-scr1-max-axi-riscv-compliance)$"
wavepeek_release_bin := "./target/release/wavepeek"
wavepeek_fsdb_release_bin := "./target/fsdb/release/wavepeek"
python := "python3 -B"
docs_site_dir := "tmp/docs-site"
playground_dir := "tmp/playground"
playground_preview_dir := playground_dir + "/preview"
docs_pages_url := "https://kleverhq.github.io/wavepeek"
docs_repository := env_var_or_default("DOCS_REPOSITORY", "")
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
check-ondas-fixtures: require-container
    @. ./.devcontainer/env_contract.sh; \
    for fixture in $WAVEPEEK_ONDAS_FIXTURES; do \
        if [ ! -f "${ONDAS_FIXTURES_DIR}/$fixture/waveform.fst" ]; then \
            printf '%s\n' "error: file: required fixture missing at ${ONDAS_FIXTURES_DIR}/$fixture/waveform.fst" >&2; \
            exit 1; \
        fi; \
    done

# Regenerate source-backed waveform fixtures under tests/fixtures/generated
prepare-waveform-fixtures: require-container
    {{ python }} tools/waveform/prepare_fixtures.py

# Lint GitHub Actions workflows
check-actions: require-container
    actionlint .github/workflows/*.yml

# Regenerate FSDB benchmark catalog from the FST benchmark catalog
update-bench-e2e-fsdb-catalog: require-container
    @{{ python }} tools/fsdb/generate_bench_catalog.py

# Validate FSDB benchmark catalog freshness
check-bench-e2e-fsdb-catalog: require-container
    @{{ python }} tools/fsdb/generate_bench_catalog.py --check

# Generate and validate the VCD benchmark catalog from the FST catalog
update-bench-e2e-vcd-catalog: require-container
    @{{ python }} tools/fsdb/generate_bench_catalog.py --target vcd

check-bench-e2e-vcd-catalog: require-container
    @{{ python }} tools/fsdb/generate_bench_catalog.py --target vcd --check

# Keep converted RTL VCD files beside their FST sources for benchmark runs
prepare-vcd-ondas-fixtures: check-ondas-fixtures check-bench-e2e-vcd-catalog
    @. ./.devcontainer/env_contract.sh; \
    for fixture in $WAVEPEEK_ONDAS_FIXTURES; do \
        source="${ONDAS_FIXTURES_DIR}/$fixture/waveform.fst"; output="${source%.fst}.vcd"; \
        if [ -s "$output" ] && [ "$output" -nt "$source" ]; then \
            printf '%s\n' "info: vcd fixture: up to date $output"; \
            continue; \
        fi; \
        temp="$(mktemp "${output}.tmp.XXXXXXXX")"; \
        if ! fst2vcd -f "$source" | cat > "$temp"; then rm -f "$temp"; exit 1; fi; \
        mv "$temp" "$output"; \
        printf '%s\n' "info: vcd fixture: converted $source -> $output"; \
    done

# Verify the local devcontainer environment
dev-setup: require-container
    rustup show >/dev/null
    cargo --version
    cargo fmt --version
    cargo clippy --version
    actionlint -version
    gh --version
    iverilog -V >/dev/null
    vvp -V >/dev/null
    vcd2fst --help >/dev/null
    fst2vcd --help >/dev/null
    mkdocs --version
    mike --version
    wasm-bindgen --version
    playwright --version
    just --version
    cz version
    pre-commit --version

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
test: require-container check-ondas-fixtures prepare-waveform-fixtures
    cargo test -q
    just run-if-verdi test-fsdb

[private]
coverage-src-data: require-container check-ondas-fixtures prepare-waveform-fixtures
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
prepare-fsdb-fixtures: require-verdi check-bench-e2e-fsdb-catalog prepare-waveform-fixtures
    bash tools/fsdb/prepare_fsdb_fixtures.sh

# Prepare generated FSDB fixtures from VCD test fixtures only
prepare-fsdb-test-fixtures: require-verdi prepare-waveform-fixtures
    bash tools/fsdb/prepare_fsdb_fixtures.sh --test-vcd-only

# Verify FSDB benchmark artifacts exist next to required RTL FST fixtures
check-fsdb-ondas-fixtures: require-verdi check-ondas-fixtures
    {{ python }} tools/fsdb/check_fsdb_bench_artifacts.py "{{ bench_e2e_fsdb_tests }}"

# Prepare and verify FSDB benchmark artifacts in dependency order
prepare-and-check-fsdb-ondas-fixtures: require-verdi
    just check-ondas-fixtures
    just prepare-fsdb-fixtures
    {{ python }} tools/fsdb/check_fsdb_bench_artifacts.py "{{ bench_e2e_fsdb_tests }}"

# Prepare and verify only FSDB RTL artifacts required by the pre-commit smoke
prepare-and-check-fsdb-smoke-ondas-fixtures: require-verdi
    just check-ondas-fixtures
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
    for library in libnffr.so libnsys.so; do \
        library_path="$(readlink -f "$fsdb_libdir/$library")"; \
        if ! printf '%s\n' "$readelf_output" | grep -F "Shared library: [$library_path]" >/dev/null; then \
            printf '%s\n' "error: fsdb: built binary must link the Ondas-selected SDK library $library_path" >&2; \
            exit 1; \
        fi; \
    done; \
    if ! printf '%s\n' "$readelf_output" | grep -Eq '\(NEEDED\).*Shared library: \[libz\.so(\.[^]]*)?\]'; then \
        printf '%s\n' "error: fsdb: built binary must contain a libz DT_NEEDED entry" >&2; \
        exit 1; \
    fi; \
    cargo test --features fsdb --lib fsdb_reader_metadata_smoke -- --nocapture; \
    cargo test --features fsdb --lib fsdb_reader_hierarchy_smoke -- --nocapture

# Run optional FSDB build smoke tests
test-fsdb: check-fsdb-build prepare-fsdb-test-fixtures
    @export CARGO_TARGET_DIR=target/fsdb; \
    cargo test --features fsdb --lib && \
    cargo test --features fsdb --test fsdb_cli

# Run auxiliary Python/unit test suites
test-aux: require-container
    @just check-bench-e2e-fsdb-catalog
    {{ python }} -m unittest discover -s bench/e2e -p "test_*.py"
    {{ python }} -m unittest discover -s tools/bench -p "test_*.py"
    {{ python }} -m unittest discover -s tools/docs -p "test_*.py"
    {{ python }} -m unittest discover -s tools/release -p "test_*.py"
    {{ python }} -m unittest tools/coverage/test_check_coverage.py
    {{ python }} -m unittest discover -s tools/fsdb -p "test_*.py"
    {{ python }} -m unittest discover -s tools/repo -p "test_*.py"
    {{ python }} -m unittest discover -s tools/skill -p "test_*.py"

# Build the current browser Playground
playground-build: require-container
    @rm -rf "{{ playground_dir }}"
    cargo build --locked --release --target wasm32-unknown-unknown --lib
    mkdir -p "{{ playground_dir }}/wasm"
    wasm-bindgen --target web --no-typescript \
        --out-dir "{{ playground_dir }}/wasm" \
        target/wasm32-unknown-unknown/release/wavepeek.wasm
    {{ python }} tools/docs/prepare_playground.py . \
        --wasm-dir "{{ playground_dir }}/wasm" \
        --output "{{ playground_dir }}/mkdocs-src" \
        --config-output "{{ playground_dir }}/mkdocs.yml" \
        --site-output "{{ playground_dir }}/site" \
        --version "{{ docs_version }}" \
        --force
    mkdocs build --strict --config-file "{{ playground_dir }}/mkdocs.yml"

# Compose the current Playground and documentation as one local Pages preview
playground-preview-build: playground-build docs-site-build
    rm -rf "{{ playground_preview_dir }}"
    mkdir -p "{{ playground_preview_dir }}/wavepeek/latest"
    cp -a "{{ playground_dir }}/site/." "{{ playground_preview_dir }}/wavepeek/"
    cp -a "{{ docs_site_dir }}/mkdocs-site/." "{{ playground_preview_dir }}/wavepeek/latest/"

# Test the composed browser Playground against native WavePeek
playground-test: playground-preview-build build-release
    {{ python }} tools/docs/check_playground.py \
        --site "{{ playground_preview_dir }}" \
        --native-bin "{{ wavepeek_release_bin }}"

# Serve the composed Playground and current documentation locally
playground-serve: playground-preview-build
    cd "{{ playground_preview_dir }}" && {{ python }} -m http.server 8000 --bind 0.0.0.0

# Regenerate the packaged CLI reference from clap help
update-cli-reference: require-container
    cargo build --quiet --locked
    {{ python }} tools/docs/generate_cli_reference.py \
        --binary target/debug/wavepeek \
        --output skills/wavepeek/references/cli-reference.md

# Verify the packaged CLI reference matches clap help
check-cli-reference: require-container
    cargo build --quiet --locked
    {{ python }} tools/docs/generate_cli_reference.py \
        --binary target/debug/wavepeek \
        --output skills/wavepeek/references/cli-reference.md \
        --check

# Build the generated MkDocs site from the bundled skill references
docs-site-build: require-container check-cli-reference
    @rm -rf "{{ docs_site_dir }}/skill"
    cargo run --quiet --locked -- skill "{{ docs_site_dir }}/skill"
    {{ python }} tools/docs/prepare_mkdocs.py "{{ docs_site_dir }}/skill" \
        --output "{{ docs_site_dir }}/mkdocs-src" \
        --config-output "{{ docs_site_dir }}/mkdocs.yml" \
        --version "{{ docs_version }}" \
        --force
    mkdocs build --strict --config-file "{{ docs_site_dir }}/mkdocs.yml"

# Serve current documentation inside the composed local Pages preview
docs-site-serve: playground-serve

# Check docs site generation and root Pages artifacts without touching gh-pages
docs-site-check: require-container check-cli-reference
    {{ python }} tools/docs/publish_docs.py check \
        --version "{{ docs_version }}" \
        --source-root . \
        --work-dir "{{ docs_site_dir }}"

# Stage a local gh-pages update for a release tag without pushing
docs-site-stage-deploy version=docs_version source_ref=("v" + docs_version) repair="0": require-container
    @mkdir -p "{{ docs_site_dir }}/release-assets"
    gh release download "{{ source_ref }}" --pattern wavepeek-installer.sh --pattern wavepeek-installer.ps1 --dir "{{ docs_site_dir }}/release-assets" --clobber
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

# Check deployed GitHub Pages docs for a release version
docs-site-check-deploy version=docs_version base_url=docs_pages_url repository=docs_repository: require-container
    @repo_arg=(); \
        if [ -n "{{ repository }}" ]; then \
            repo_arg=(--repository "{{ repository }}"); \
        fi; \
        {{ python }} tools/docs/check_deploy.py \
            --version "{{ version }}" \
            --base-url "{{ base_url }}" \
            "${repo_arg[@]}"

# Build release binary
build-release: require-container
    cargo build --release

# Run the manual performance gate for two source refs
bench-gate baseline_ref revised_ref="HEAD" fsdb="auto" vcd="never": require-container
    {{ python }} tools/bench/gate.py --baseline-ref "{{ baseline_ref }}" --revised-ref "{{ revised_ref }}" --fsdb "{{ fsdb }}" --vcd "{{ vcd }}"

# Capture benchmark artifacts for one source ref
bench-capture ref="HEAD" fsdb="auto" vcd="never": require-container
    {{ python }} tools/bench/capture.py --ref "{{ ref }}" --fsdb "{{ fsdb }}" --vcd "{{ vcd }}"

# Compare two benchmark capture directories
bench-compare golden_dir revised_dir: require-container
    {{ python }} tools/bench/compare.py --golden "{{ golden_dir }}" --revised "{{ revised_dir }}"

[private]
bench-e2e-run: check-ondas-fixtures prepare-waveform-fixtures build-release
    {{ python }} bench/e2e/perf.py run --binary subject="{{ wavepeek_release_bin }}"

[private]
bench-e2e-vcd-run: prepare-vcd-ondas-fixtures prepare-waveform-fixtures build-release
    {{ python }} bench/e2e/perf.py run --binary subject="{{ wavepeek_release_bin }}" --tests bench/e2e/tests_vcd.json

[private]
bench-e2e-fsdb-run: prepare-and-check-fsdb-ondas-fixtures build-release-fsdb
    {{ python }} bench/e2e/perf.py run --binary subject="{{ wavepeek_fsdb_release_bin }}" --tests "{{ bench_e2e_fsdb_tests }}"

# Run lightweight benchmark e2e smoke for pre-commit
[private]
bench-e2e-smoke-commit: check-ondas-fixtures build-release
    @tmp_revised="$(mktemp -d)"; trap 'rm -rf "$tmp_revised"' EXIT; \
        {{ python }} bench/e2e/perf.py run --binary subject="{{ wavepeek_release_bin }}" --tests bench/e2e/tests_commit.json --run-dir "$tmp_revised"
    @just run-if-verdi bench-e2e-fsdb-smoke-commit

# Run lightweight FSDB benchmark e2e smoke for pre-commit
[private]
bench-e2e-fsdb-smoke-commit: prepare-and-check-fsdb-smoke-ondas-fixtures build-release-fsdb
    @tmp_revised="$(mktemp -d)"; trap 'rm -rf "$tmp_revised"' EXIT; \
        {{ python }} bench/e2e/perf.py run --binary subject="{{ wavepeek_fsdb_release_bin }}" --tests "{{ bench_e2e_fsdb_tests }}" --run-dir "$tmp_revised" --filter '{{ bench_e2e_fsdb_smoke_filter }}'

# Run pre-commit hooks on all files
pre-commit: require-container check-ondas-fixtures prepare-waveform-fixtures
    pre-commit run --all-files

# Check a commit message, defaulting to Git's standard message file
check-commit message=`git rev-parse --git-path COMMIT_EDITMSG`: require-container
    cz check --commit-msg-file {{ quote(message) }}

# Check everything
check: format-check lint check-actions check-bench-e2e-fsdb-catalog check-bench-e2e-vcd-catalog check-build docs-site-check playground-test check-commit
    @just run-if-verdi check-fsdb-build

# CI quality gate (no commit-msg hook)
ci: format-check lint check-actions check-bench-e2e-vcd-catalog test-aux coverage-src-check check-build docs-site-check playground-test
    @just run-if-verdi test-fsdb

# Fix everything
fix: format lint-fix

# Clean up
clean: require-container
    cargo clean

# Show recipes
help: require-container
    @just --list
