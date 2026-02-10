.DEFAULT_GOAL := help

## Bootstrap project env
bootstrap:
	rustup show >/dev/null
	cargo --version
	cargo fmt --version
	cargo clippy --version
	verilator --version
	riscv64-unknown-elf-gcc --version
	riscv64-unknown-elf-objcopy --version
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
pre-commit:
	pre-commit run --all-files

## Check commit messages
check-commit:
	cz check --commit-msg-file "$$(git rev-parse --git-path COMMIT_EDITMSG)"

## Check everything
check: format-check lint check-build check-commit

## CI quality gate (no commit-msg hook)
ci: format-check lint test check-build

## Fix everything
fix: format lint-fix

## Clean up
clean:
	cargo clean

## Show targets
help:
	@awk 'BEGIN{tabstop=8;targetcol=32} /^##/{desc=$$0;sub(/^##[ ]*/,"",desc);next} /^[a-zA-Z0-9_-]+:/{name=$$1;sub(/:.*/,"",name);col=length(name);pos=col;ntabs=0;while(pos<targetcol){ntabs++;pos=int(pos/tabstop+1)*tabstop}printf "%s",name;for(i=0;i<ntabs;i++)printf "\t";printf "%s\n",desc;desc=""}' Makefile
