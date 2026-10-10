#!/usr/bin/env sh

# Shared environment contract for the wavepeek container image stages. Keep
# versions, fixture locations, and externally fetched artifact identities here.

WAVEPEEK_RUST_VERSION="1.93.0"
WAVEPEEK_CARGO_LLVM_COV_VERSION="0.8.7"
WAVEPEEK_JUST_VERSION="1.21.0"
WAVEPEEK_ACTIONLINT_VERSION="1.7.12"
WAVEPEEK_GH_VERSION="2.94.0"
WAVEPEEK_HYPERFINE_VERSION="1.18.0"
WAVEPEEK_PRECOMMIT_VERSION="4.5.1"
WAVEPEEK_COMMITIZEN_VERSION="4.12.1"
WAVEPEEK_WASM_BINDGEN_VERSION="0.2.127"
WAVEPEEK_PLAYWRIGHT_VERSION="1.62.0"
WAVEPEEK_ONDAS_FIXTURES_REV="aadb6d00597265fdef1a8227825465dbffecee43"
ONDAS_FIXTURES_DIR="/opt/ondas-fixtures"
WAVEPEEK_ONDAS_FIXTURES="fst/fst0013-picorv32-test-vcd fst/fst0022-scr1-max-axi-coremark fst/fst0012-picorv32-test-ez-vcd fst/fst0025-scr1-max-axi-isr-sample fst/fst0027-scr1-max-axi-riscv-compliance fst/fst0006-chipyard-dualrocketconfig-dhrystone fst/fst0000-chipyard-clusteredrocketconfig-dhrystone fst/fst0002-chipyard-clusteredrocketconfig-mt-memcpy"
