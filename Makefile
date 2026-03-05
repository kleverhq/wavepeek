.DEFAULT_GOAL := help

RTL_ARTIFACTS_DIR ?= /opt/rtl-artifacts
REQUIRED_RTL_ARTIFACTS := picorv32_test_vcd.fst scr1_max_axi_coremark.fst picorv32_test_ez_vcd.fst scr1_max_axi_isr_sample.fst scr1_max_axi_riscv_compliance.fst chipyard_DualRocketConfig_dhrystone.fst chipyard_ClusteredRocketConfig_dhrystone.fst chipyard_ClusteredRocketConfig_mt-memcpy.fst
SCHEMA_PATH := schema/wavepeek.json
BENCH_E2E_RUNS_DIR := bench/e2e/runs
BENCH_E2E_BASELINE_DIR := $(BENCH_E2E_RUNS_DIR)/baseline
WAVEPEEK_RELEASE_BIN := ./target/release/wavepeek

## Require containerized execution
require-container:
	@if [ "$${WAVEPEEK_IN_CONTAINER:-0}" != "1" ]; then \
		printf '%s\n' "error: container: this target must run inside the wavepeek devcontainer/CI image (set WAVEPEEK_IN_CONTAINER=1)" >&2; \
		exit 1; \
	fi

## Verify external fixture payload is installed
check-rtl-artifacts: require-container
	@for fixture in $(REQUIRED_RTL_ARTIFACTS); do \
		if [ ! -f "$(RTL_ARTIFACTS_DIR)/$$fixture" ]; then \
			printf '%s\n' "error: file: required fixture missing at $(RTL_ARTIFACTS_DIR)/$$fixture" >&2; \
			exit 1; \
		fi; \
	done

## Regenerate canonical schema artifact from runtime output
update-schema: require-container
	@mkdir -p schema
	@tmp_file="$$(mktemp)"; trap 'rm -f "$$tmp_file"' EXIT; \
		cargo run --quiet -- schema > "$$tmp_file"; \
		mv "$$tmp_file" "$(SCHEMA_PATH)"

## Validate canonical schema freshness and JSON contract URL
check-schema: require-container
	@python3 scripts/check_schema_contract.py "$(SCHEMA_PATH)"

## Bootstrap project env
bootstrap: require-container
	rustup show >/dev/null
	cargo --version
	cargo fmt --version
	cargo clippy --version
	gtkwave --version
	surfer --version
	pre-commit install --hook-type commit-msg --hook-type pre-commit

## Format with rustfmt
format: require-container
	cargo fmt

## Check formatting with rustfmt
format-check: require-container
	cargo fmt -- --check

## Lint with clippy
lint: require-container
	cargo clippy --all-targets --all-features -- -D warnings

## Fix linting with clippy
lint-fix: require-container
	cargo clippy --all-targets --all-features --fix --allow-dirty --allow-staged -- -D warnings

## Type check with cargo
check-build: require-container
	cargo check

## Run tests with cargo
test: require-container check-rtl-artifacts
	cargo test -q

## Run benchmark harness unit tests
test-bench-e2e: require-container
	python3 -m unittest discover -s bench/e2e -p "test_*.py"

## Build release binary
build-release: require-container
	cargo build --release

## Refresh benchmark e2e baseline artifacts
bench-e2e-update-baseline: check-rtl-artifacts build-release
	rm -rf "$(BENCH_E2E_BASELINE_DIR)"
	mkdir -p "$(BENCH_E2E_BASELINE_DIR)"
	WAVEPEEK_BIN="$(WAVEPEEK_RELEASE_BIN)" python3 bench/e2e/perf.py run --run-dir "$(BENCH_E2E_BASELINE_DIR)"

## Run benchmark e2e suite with baseline compare
bench-e2e-run: check-rtl-artifacts build-release
	WAVEPEEK_BIN="$(WAVEPEEK_RELEASE_BIN)" python3 bench/e2e/perf.py run --compare "$(BENCH_E2E_BASELINE_DIR)"

## Run lightweight benchmark e2e smoke for pre-commit
bench-e2e-smoke-commit: check-rtl-artifacts build-release
	@tmp_revised="$$(mktemp -d)"; trap 'rm -rf "$$tmp_revised"' EXIT; \
		WAVEPEEK_BIN="$(WAVEPEEK_RELEASE_BIN)" python3 bench/e2e/perf.py run --tests bench/e2e/tests_commit.json --run-dir "$$tmp_revised" && \
		WAVEPEEK_BIN="$(WAVEPEEK_RELEASE_BIN)" python3 bench/e2e/perf.py compare --revised "$$tmp_revised" --golden "$(BENCH_E2E_BASELINE_DIR)" --max-negative-delta-pct 100

## Run pre-commit hooks on all files
pre-commit: require-container check-rtl-artifacts
	pre-commit run --all-files

## Check commit messages
check-commit: require-container
	cz check --commit-msg-file "$$(git rev-parse --git-path COMMIT_EDITMSG)"

## Check everything
check: format-check lint check-schema check-build check-commit

## CI quality gate (no commit-msg hook)
ci: format-check lint check-schema test test-bench-e2e check-build

## Fix everything
fix: format lint-fix update-schema

## Clean up
clean: require-container
	cargo clean

## Show targets
help: require-container
	@awk 'BEGIN{tabstop=8;targetcol=32} /^##/{desc=$$0;sub(/^##[ ]*/,"",desc);next} /^[a-zA-Z0-9_-]+:/{name=$$1;sub(/:.*/,"",name);col=length(name);pos=col;ntabs=0;while(pos<targetcol){ntabs++;pos=int(pos/tabstop+1)*tabstop}printf "%s",name;for(i=0;i<ntabs;i++)printf "\t";printf "%s\n",desc;desc=""}' Makefile
