.DEFAULT_GOAL := help

RTL_ARTIFACTS_DIR ?= /opt/rtl-artifacts
REQUIRED_RTL_ARTIFACTS := picorv32_test_vcd.fst scr1_max_axi_coremark.fst
PICORV32_TEST_VCD_SHA256 := e01bbb94f3e934227ed371fdc1091a7b35868eeb1eb017567672e5d0e9621b25
SCR1_MAX_AXI_COREMARK_SHA256 := 1413002528f41c9e1dcbb5ff9f80e6e5edcb0e06093a18b8ada23da4ab283053

## Require containerized execution for selected targets
require-container:
	@if [ "$${WAVEPEEK_IN_CONTAINER:-0}" != "1" ]; then \
		printf '%s\n' "error: container: this target must run inside the wavepeek devcontainer/CI image (set WAVEPEEK_IN_CONTAINER=1)" >&2; \
		exit 1; \
	fi

## Verify external fixture payload is installed
check-rtl-artifacts:
	@for fixture in $(REQUIRED_RTL_ARTIFACTS); do \
		if [ ! -f "$(RTL_ARTIFACTS_DIR)/$$fixture" ]; then \
			printf '%s\n' "error: file: required fixture missing at $(RTL_ARTIFACTS_DIR)/$$fixture" >&2; \
			exit 1; \
		fi; \
	done
	@if [ ! -f "$(RTL_ARTIFACTS_DIR)/MANIFEST.json" ]; then \
		printf '%s\n' "error: file: fixture manifest missing at $(RTL_ARTIFACTS_DIR)/MANIFEST.json" >&2; \
		exit 1; \
	fi
	@if ! grep -F '"version"' "$(RTL_ARTIFACTS_DIR)/MANIFEST.json" >/dev/null; then \
		printf '%s\n' "error: file: fixture manifest is missing required key: version" >&2; \
		exit 1; \
	fi
	@if ! grep -E '"name"[[:space:]]*:[[:space:]]*"picorv32_test_vcd.fst"' "$(RTL_ARTIFACTS_DIR)/MANIFEST.json" >/dev/null; then \
		printf '%s\n' "error: file: fixture manifest missing picorv32_test_vcd.fst entry" >&2; \
		exit 1; \
	fi
	@if ! grep -E '"name"[[:space:]]*:[[:space:]]*"scr1_max_axi_coremark.fst"' "$(RTL_ARTIFACTS_DIR)/MANIFEST.json" >/dev/null; then \
		printf '%s\n' "error: file: fixture manifest missing scr1_max_axi_coremark.fst entry" >&2; \
		exit 1; \
	fi
	@if ! grep -zE '"name"[[:space:]]*:[[:space:]]*"picorv32_test_vcd.fst"[^}]*"sha256"[[:space:]]*:[[:space:]]*"$(PICORV32_TEST_VCD_SHA256)"' "$(RTL_ARTIFACTS_DIR)/MANIFEST.json" >/dev/null; then \
		printf '%s\n' "error: file: fixture manifest has unexpected (name, sha256) mapping for picorv32_test_vcd.fst" >&2; \
		exit 1; \
	fi
	@if ! grep -zE '"name"[[:space:]]*:[[:space:]]*"scr1_max_axi_coremark.fst"[^}]*"sha256"[[:space:]]*:[[:space:]]*"$(SCR1_MAX_AXI_COREMARK_SHA256)"' "$(RTL_ARTIFACTS_DIR)/MANIFEST.json" >/dev/null; then \
		printf '%s\n' "error: file: fixture manifest has unexpected (name, sha256) mapping for scr1_max_axi_coremark.fst" >&2; \
		exit 1; \
	fi

## Print fixture provenance manifest
print-rtl-manifest: check-rtl-artifacts
	@printf '%s\n' "rtl-artifacts manifest ($(RTL_ARTIFACTS_DIR)/MANIFEST.json):"
	@cat "$(RTL_ARTIFACTS_DIR)/MANIFEST.json"

## Bootstrap project env
bootstrap:
	rustup show >/dev/null
	cargo --version
	cargo fmt --version
	cargo clippy --version
	gtkwave --version
	surfer --version
	pre-commit install --hook-type commit-msg --hook-type pre-commit

## Format with rustfmt
format:
	cargo fmt

## Check formatting with rustfmt
format-check:
	cargo fmt -- --check

## Lint with clippy
lint:
	cargo clippy --all-targets --all-features -- -D warnings

## Fix linting with clippy
lint-fix:
	cargo clippy --all-targets --all-features --fix --allow-dirty --allow-staged -- -D warnings

## Type check with cargo
check-build:
	cargo check

## Run tests with cargo
test:
	cargo test

## Run pre-commit hooks on all files
pre-commit: require-container check-rtl-artifacts print-rtl-manifest
	pre-commit run --all-files

## Check commit messages
check-commit:
	cz check --commit-msg-file "$$(git rev-parse --git-path COMMIT_EDITMSG)"

## Check everything
check: format-check lint check-build check-commit

## CI quality gate (no commit-msg hook)
ci: require-container check-rtl-artifacts print-rtl-manifest format-check lint test check-build

## Fix everything
fix: format lint-fix

## Clean up
clean:
	cargo clean

## Show targets
help:
	@awk 'BEGIN{tabstop=8;targetcol=32} /^##/{desc=$$0;sub(/^##[ ]*/,"",desc);next} /^[a-zA-Z0-9_-]+:/{name=$$1;sub(/:.*/,"",name);col=length(name);pos=col;ntabs=0;while(pos<targetcol){ntabs++;pos=int(pos/tabstop+1)*tabstop}printf "%s",name;for(i=0;i<ntabs;i++)printf "\t";printf "%s\n",desc;desc=""}' Makefile
