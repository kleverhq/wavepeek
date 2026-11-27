set shell := ["bash", "-euo", "pipefail", "-c"]

export ONDAS_FIXTURES_DIR := `. ./.devcontainer/env_contract.sh; printf '%s\n' "$ONDAS_FIXTURES_DIR"`
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
require-xcelium: require-container
    @{{ python }} tools/xcelium/check_reader_env.py shm --require >/dev/null

[private]
require-questa: require-container
    @{{ python }} tools/wlf/check_wlf_env.py --require >/dev/null

[private]
run-if-questa recipe: require-container
    @set +e; \
    output="$({{ python }} tools/wlf/check_wlf_env.py 2>&1)"; status="$?"; \
    set -e; \
    if [ "$status" -eq 0 ]; then \
        printf '%s\n' "$output"; just "{{ recipe }}"; \
    elif [ "$status" -eq 77 ]; then \
        printf '%s\n' "$output"; \
    else \
        printf '%s\n' "$output" >&2; exit "$status"; \
    fi

# Lint optional WLF support
lint-wlf: require-questa
    CARGO_TARGET_DIR=target/wlf cargo clippy --locked --features wlf --all-targets -- -D warnings

# Check that the selected native library is linked and usable outside Cargo
check-wlf-build: require-questa
    @library="$({{ python }} tools/wlf/check_wlf_env.py --require --print-library)"; \
        CARGO_TARGET_DIR=target/wlf cargo build --locked --features wlf; \
        readelf -d target/wlf/debug/wavepeek | grep -F "Shared library: [$library]" >/dev/null; \
        target/wlf/debug/wavepeek --version

# Build the release binary with optional WLF support
build-release-wlf: require-questa
    CARGO_TARGET_DIR=target/wlf cargo build --locked --release --features wlf

# Prepare small WLF inputs for the regression and smoke suites
prepare-wlf-test-fixtures: require-questa prepare-waveform-fixtures
    {{ python }} tools/wlf/prepare_wlf_fixtures.py --tests-only

# Prepare WLF inputs for selected benchmarks
prepare-wlf-fixtures filter="": require-questa prepare-waveform-fixtures
    {{ python }} tools/wlf/prepare_wlf_fixtures.py --bench-only --filter '{{ filter }}'

# Run WLF CLI regressions
test-wlf: check-wlf-test-fixtures prepare-wlf-test-fixtures
    CARGO_TARGET_DIR=target/wlf cargo test --locked --features wlf --lib --test wlf_cli --test cli_contract --test change_query --test waveform_errors_cli

[private]
check-wlf-test-fixtures: require-questa
    @. ./.devcontainer/env_contract.sh; \
    for fixture in $WAVEPEEK_ONDAS_WLF_FIXTURES; do \
        for file in waveform.wlf fixture.json; do \
            if [ ! -f "${ONDAS_FIXTURES_DIR}/$fixture/$file" ]; then \
                printf '%s\n' "error: file: required fixture missing at ${ONDAS_FIXTURES_DIR}/$fixture/$file (rebuild the devcontainer image and run ./dev --recreate true)" >&2; \
                exit 1; \
            fi; \
        done; \
    done

# Run WLF E2E benchmarks
bench-e2e-wlf-run filter="": (prepare-wlf-fixtures filter) build-release-wlf
    @catalog="$(just generate-bench-e2e-catalog wlf)"; \
        {{ python }} bench/e2e/perf.py run --strict --binary wlf=./target/wlf/release/wavepeek --tests "$catalog" --filter '{{ filter }}'

# Smoke-test the WLF reader on a small RTL fixture
bench-e2e-wlf-smoke: prepare-wlf-test-fixtures build-release-wlf
    {{ python }} bench/e2e/smoke.py wlf ./target/wlf/release/wavepeek

[private]
run-if-xcelium recipe reader="shm": require-container
    @set +e; \
    output="$({{ python }} tools/xcelium/check_reader_env.py {{ reader }} 2>&1)"; status="$?"; \
    set -e; \
    if [ "$status" -eq 0 ]; then \
        printf '%s\n' "$output"; just "{{ recipe }}"; \
    elif [ "$status" -eq 77 ]; then \
        printf '%s\n' "$output"; \
    else \
        printf '%s\n' "$output" >&2; exit "$status"; \
    fi

# Check local Xcelium reader availability
check-shm-env: require-container
    @set +e; {{ python }} tools/xcelium/check_reader_env.py shm; status="$?"; \
    if [ "$status" -eq 77 ]; then exit 0; fi; exit "$status"

# Lint optional SHM support
lint-shm: require-xcelium
    CARGO_TARGET_DIR=target/shm bash tools/shm/with_shm_env.sh cargo clippy --features shm --all-targets -- -D warnings

# Build and launch outside Cargo to verify native reader loading
check-shm-build: require-xcelium
    CARGO_TARGET_DIR=target/shm bash tools/shm/with_shm_env.sh cargo build --features shm
    bash tools/shm/with_shm_env.sh target/shm/debug/wavepeek --version

# Build release binary with optional SHM support
build-release-shm: require-xcelium
    CARGO_TARGET_DIR=target/shm bash tools/shm/with_shm_env.sh cargo build --release --features shm

# Prepare small SHM regression inputs
prepare-shm-test-fixtures: require-xcelium prepare-waveform-fixtures
    {{ python }} tools/shm/prepare_shm_fixtures.py --tests-only

# Prepare SHM inputs for selected benchmark scenarios
prepare-shm-fixtures filter="": require-xcelium check-ondas-fixtures prepare-waveform-fixtures
    {{ python }} tools/shm/prepare_shm_fixtures.py --bench-only --filter '{{ filter }}'

# Run SHM regression tests with the selected native reader
test-shm: prepare-shm-test-fixtures
    CARGO_TARGET_DIR=target/shm bash tools/shm/with_shm_env.sh cargo test --features shm

# Run selected SHM benchmark scenarios
bench-e2e-shm-run filter="": (prepare-shm-fixtures filter) build-release-shm
    @catalog="$(just generate-bench-e2e-catalog shm)"; \
        bash tools/shm/with_shm_env.sh {{ python }} bench/e2e/perf.py run --strict --binary shm=./target/shm/release/wavepeek --tests "$catalog" --filter '{{ filter }}'

# Smoke-test the SHM reader on a small RTL fixture
bench-e2e-shm-smoke: prepare-shm-test-fixtures build-release-shm
    bash tools/shm/with_shm_env.sh {{ python }} bench/e2e/smoke.py shm ./target/shm/release/wavepeek

[private]
require-vwdb: require-container
    @{{ python }} tools/xcelium/check_reader_env.py vwdb --require >/dev/null

# Lint optional VWDB support
lint-vwdb: require-vwdb
    CARGO_TARGET_DIR=target/vwdb cargo clippy --locked --features vwdb --all-targets -- -D warnings

# Build and open a VWDB outside Cargo to verify runtime loading
check-vwdb-build: require-vwdb
    CARGO_TARGET_DIR=target/vwdb cargo build --locked --features vwdb
    target/vwdb/debug/wavepeek info --waves "$ONDAS_FIXTURES_DIR/vwdb/vwdb0046-opb-s/waveform.vwdb" --json

# Build the release binary with VWDB support
build-release-vwdb: require-vwdb
    CARGO_TARGET_DIR=target/vwdb cargo build --locked --release --features vwdb

# Run VWDB CLI regressions on native recordings
test-vwdb: require-vwdb prepare-waveform-fixtures
    @recording="$(bash tools/vwdb/record_fixture.sh m2_core extract_global_include)"; \
    WAVEPEEK_VWDB_TEST_DIR="$recording" CARGO_TARGET_DIR=target/vwdb \
        cargo test --locked --features vwdb --test vwdb_cli --test cli_contract

# Verify VWDB and embedded SHM in one build
test-vwdb-shm: require-vwdb require-xcelium
    CARGO_TARGET_DIR=target/vwdb-shm bash tools/shm/with_shm_env.sh cargo test --locked --features vwdb,shm \
        --test vwdb_cli systemc_digital_history_uses_the_selected_shm_directory -- --exact

# Smoke-test VWDB on a small RTL recording
bench-e2e-vwdb-smoke: build-release-vwdb
    @recording="$(bash tools/vwdb/record_fixture.sh m2_core)"; \
    {{ python }} bench/e2e/smoke.py vwdb ./target/vwdb/release/wavepeek --waves "$recording/m2_core/ida.db/ida.vwdb"

[private]
check-ondas-vwdb-fixtures: require-container
    @. ./.devcontainer/env_contract.sh; \
    for fixture in $WAVEPEEK_ONDAS_VWDB_FIXTURES; do \
        if [[ ! -s "$ONDAS_FIXTURES_DIR/$fixture/fixture.json" || ! -e "$ONDAS_FIXTURES_DIR/$fixture/waveform.${fixture%%/*}" ]]; then \
            printf 'error: fixture: missing installed %s\n' "$fixture" >&2; exit 1; \
        fi; \
    done

[private]
prepare-vwdb-bench-fixtures: require-vwdb check-ondas-vwdb-fixtures
    @bash tools/vwdb/record_fixture.sh extract_global_include

# Run the full VWDB benchmark suite
bench-e2e-vwdb-run filter="": build-release-vwdb
    @recording="$(just prepare-vwdb-bench-fixtures)"; \
    catalog="$(just generate-bench-e2e-catalog vwdb "$recording/extract_global_include/ida.db/ida.vwdb")"; \
    {{ python }} bench/e2e/perf.py run --strict --binary vwdb=./target/vwdb/release/wavepeek --tests "$catalog" --filter '{{ filter }}'

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
    done; \
    for fixture in $WAVEPEEK_ONDAS_FSDB_FIXTURES; do \
        for file in waveform.fsdb fixture.json; do \
            if [ ! -f "${ONDAS_FIXTURES_DIR}/$fixture/$file" ]; then \
                printf '%s\n' "error: file: required fixture missing at ${ONDAS_FIXTURES_DIR}/$fixture/$file (rebuild the devcontainer image and run ./dev --recreate true)" >&2; \
                exit 1; \
            fi; \
        done; \
    done

# Regenerate source-backed waveform fixtures under tests/fixtures/generated
prepare-waveform-fixtures: require-container
    {{ python }} tools/waveform/prepare_fixtures.py

# Lint GitHub Actions workflows
check-actions: require-container
    actionlint .github/workflows/*.yml

[private]
generate-bench-e2e-catalog format recording="": require-container
    @{{ python }} bench/e2e/generate_catalog.py "{{ format }}" --vwdb-recording "{{ recording }}"

# Keep converted RTL VCD files beside their FST sources for benchmark runs
prepare-vcd-ondas-fixtures: check-ondas-fixtures
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

[private]
lint-default: require-container
    cargo clippy --all-targets -- -D warnings

# Lint with clippy
lint: lint-default
    just run-if-verdi lint-fsdb
    just run-if-xcelium lint-shm
    just run-if-questa lint-wlf
    just run-if-xcelium lint-vwdb vwdb

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
    just run-if-xcelium test-shm
    just run-if-questa test-wlf
    just run-if-xcelium test-vwdb vwdb

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
prepare-fsdb-fixtures: require-verdi prepare-waveform-fixtures
    bash tools/fsdb/prepare_fsdb_fixtures.sh

# Prepare generated FSDB fixtures from VCD test fixtures only
prepare-fsdb-test-fixtures: require-verdi prepare-waveform-fixtures
    bash tools/fsdb/prepare_fsdb_fixtures.sh --test-vcd-only

# Verify FSDB benchmark artifacts exist next to required RTL FST fixtures
check-fsdb-ondas-fixtures: require-verdi check-ondas-fixtures
    @catalog="$(just generate-bench-e2e-catalog fsdb)"; \
        {{ python }} tools/fsdb/check_fsdb_bench_artifacts.py "$catalog"

# Prepare and verify FSDB benchmark artifacts in dependency order
prepare-and-check-fsdb-ondas-fixtures: require-verdi check-ondas-fixtures prepare-fsdb-fixtures check-fsdb-ondas-fixtures

# Build release binary with optional FSDB support
build-release-fsdb: require-verdi
    CARGO_TARGET_DIR=target/fsdb cargo build --release --features fsdb

# Build and smoke-test optional FSDB support
check-fsdb-build: require-verdi
    @fsdb_libdir="$({{ python }} tools/fsdb/check_fsdb_env.py --require --print-libdir)"; \
    export CARGO_TARGET_DIR=target/fsdb; \
    cargo check --features fsdb; \
    cargo build --features fsdb; \
    target/fsdb/debug/wavepeek --version; \
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
test-fsdb: check-ondas-fixtures check-fsdb-build prepare-fsdb-test-fixtures
    @export CARGO_TARGET_DIR=target/fsdb; \
    cargo test --features fsdb --lib && \
    cargo test --features fsdb --test fsdb_cli --test fsdb_output_cli --test waveform_errors_cli --test change_query

# Run auxiliary Python/unit test suites
test-aux: require-container
    {{ python }} -m unittest discover -s bench/e2e -p "test_*.py"
    {{ python }} -m unittest discover -s tools/bench -p "test_*.py"
    {{ python }} -m unittest discover -s tools/docs -p "test_*.py"
    {{ python }} -m unittest discover -s tools/release -p "test_*.py"
    {{ python }} -m unittest tools/coverage/test_check_coverage.py
    {{ python }} -m unittest discover -s tools/fsdb -p "test_*.py"
    {{ python }} -m unittest discover -s tools/shm -p "test_*.py"
    {{ python }} -m unittest discover -s tools/wlf -p "test_*.py"
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

# Require the full benchmark matrix for the selected release profile
bench-release-gate baseline_ref revised_ref="HEAD" profile="private": require-container
    {{ python }} tools/bench/gate.py --baseline-ref "{{ baseline_ref }}" --revised-ref "{{ revised_ref }}" --release-profile "{{ profile }}"

# Capture benchmark artifacts for one source ref
bench-capture ref="HEAD" fsdb="auto" vcd="never" shm="never" wlf="never": require-container
    {{ python }} tools/bench/capture.py --ref "{{ ref }}" --fsdb "{{ fsdb }}" --vcd "{{ vcd }}" --shm "{{ shm }}" --wlf "{{ wlf }}"

# Compare two benchmark capture directories
bench-compare golden_dir revised_dir: require-container
    {{ python }} tools/bench/compare.py --golden "{{ golden_dir }}" --revised "{{ revised_dir }}"

# Run the full FST benchmark suite
bench-e2e-run: check-ondas-fixtures prepare-waveform-fixtures build-release
    {{ python }} bench/e2e/perf.py run --strict --binary fst="{{ wavepeek_release_bin }}"

# Run the full VCD benchmark suite
bench-e2e-vcd-run: prepare-vcd-ondas-fixtures prepare-waveform-fixtures build-release
    @catalog="$(just generate-bench-e2e-catalog vcd)"; \
        {{ python }} bench/e2e/perf.py run --strict --binary vcd="{{ wavepeek_release_bin }}" --tests "$catalog"

# Run the full FSDB benchmark suite
bench-e2e-fsdb-run: prepare-and-check-fsdb-ondas-fixtures build-release-fsdb
    @catalog="$(just generate-bench-e2e-catalog fsdb)"; \
        {{ python }} bench/e2e/perf.py run --strict --binary fsdb="{{ wavepeek_fsdb_release_bin }}" --tests "$catalog"

# Smoke-test the public readers on a small RTL fixture
bench-e2e-smoke: prepare-waveform-fixtures build-release
    {{ python }} bench/e2e/smoke.py vcd "{{ wavepeek_release_bin }}"
    {{ python }} bench/e2e/smoke.py fst "{{ wavepeek_release_bin }}"

# Smoke-test the FSDB reader on a small RTL fixture
bench-e2e-fsdb-smoke: prepare-fsdb-test-fixtures build-release-fsdb
    {{ python }} bench/e2e/smoke.py fsdb "{{ wavepeek_fsdb_release_bin }}"

# Check everything
check: format-check lint check-actions check-build docs-site-check playground-test
    @just run-if-verdi check-fsdb-build
    @just run-if-xcelium check-shm-build
    @just run-if-questa check-wlf-build
    @just run-if-xcelium check-vwdb-build vwdb

# CI quality gate
ci: format-check lint-default check-actions check-ondas-vwdb-fixtures test-aux coverage-src-check check-build docs-site-check playground-test bench-e2e-smoke
    cargo test --doc
    @just ci-native

[private]
ci-fsdb: lint-fsdb test-fsdb bench-e2e-fsdb-smoke

[private]
ci-shm: lint-shm check-shm-build test-shm bench-e2e-shm-smoke

[private]
ci-wlf: lint-wlf check-wlf-build test-wlf bench-e2e-wlf-smoke

# Check the enabled VWDB reader and a native workload
ci-vwdb: lint-vwdb check-vwdb-build test-vwdb bench-e2e-vwdb-smoke
    @just run-if-xcelium test-vwdb-shm

# Run native lint, loader checks, CLI tests, and smoke for available readers
ci-native: require-container
    @just run-if-verdi ci-fsdb
    @just run-if-xcelium ci-shm
    @just run-if-questa ci-wlf
    @just run-if-xcelium ci-vwdb vwdb

# Fix everything
fix: format lint-fix

# Clean up
clean: require-container
    cargo clean

# Show recipes
help: require-container
    @just --list
