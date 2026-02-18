.DEFAULT_GOAL := help

RTL_ARTIFACTS_DIR ?= /opt/rtl-artifacts
REQUIRED_RTL_ARTIFACTS := picorv32_test_vcd.fst scr1_max_axi_coremark.fst

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
	cargo test

## Run pre-commit hooks on all files
pre-commit: require-container check-rtl-artifacts
	pre-commit run --all-files

## Check commit messages
check-commit: require-container
	cz check --commit-msg-file "$$(git rev-parse --git-path COMMIT_EDITMSG)"

## Check everything
check: format-check lint check-build check-commit

## CI quality gate (no commit-msg hook)
ci: format-check lint test check-build

## Fix everything
fix: format lint-fix

## Clean up
clean: require-container
	cargo clean

## Show targets
help: require-container
	@awk 'BEGIN{tabstop=8;targetcol=32} /^##/{desc=$$0;sub(/^##[ ]*/,"",desc);next} /^[a-zA-Z0-9_-]+:/{name=$$1;sub(/:.*/,"",name);col=length(name);pos=col;ntabs=0;while(pos<targetcol){ntabs++;pos=int(pos/tabstop+1)*tabstop}printf "%s",name;for(i=0;i<ntabs;i++)printf "\t";printf "%s\n",desc;desc=""}' Makefile
