#!/usr/bin/env sh

# Shared environment contract for the wavepeek container image stages. Keep
# versions, fixture locations, and externally fetched artifact identities here.

WAVEPEEK_RUST_VERSION="1.93.0"
WAVEPEEK_CARGO_LLVM_COV_VERSION="0.8.7"
WAVEPEEK_JUST_VERSION="1.21.0"
WAVEPEEK_ACTIONLINT_VERSION="1.7.12"
WAVEPEEK_GH_VERSION="2.94.0"
WAVEPEEK_HYPERFINE_VERSION="1.18.0"
WAVEPEEK_WASM_BINDGEN_VERSION="0.2.127"
WAVEPEEK_PLAYWRIGHT_VERSION="1.62.0"
WAVEPEEK_ONDAS_FIXTURES_REV="e3cd9208a7f41c29cb9a335f6e12c99589e7cbb4"
ONDAS_FIXTURES_DIR="/opt/ondas-fixtures"
WAVEPEEK_ONDAS_FSDB_FIXTURES="fsdb/fsdb0019-typed-values fsdb/fsdb0020-nine-state-ranges"
WAVEPEEK_ONDAS_WLF_FIXTURES="wlf/wlf0070-typed-deltas-aliases wlf/wlf0071-event-occurrences wlf/wlf0072-strings-and-names"
WAVEPEEK_ONDAS_WDB_FIXTURES="wdb/wdb0305-enum-core"
WAVEPEEK_ONDAS_FIXTURES="fst/fst0013-picorv32-test-vcd fst/fst0022-scr1-max-axi-coremark fst/fst0012-picorv32-test-ez-vcd fst/fst0025-scr1-max-axi-isr-sample fst/fst0027-scr1-max-axi-riscv-compliance fst/fst0006-chipyard-dualrocketconfig-dhrystone fst/fst0000-chipyard-clusteredrocketconfig-dhrystone fst/fst0002-chipyard-clusteredrocketconfig-mt-memcpy"
WAVEPEEK_ONDAS_VWDB_FIXTURES="vwdb/vwdb0000-enum-core vwdb/vwdb0046-opb-s fst/fst0092-opb-s vwdb/vwdb0047-chipyard-clusteredrocketconfig-dhrystone vwdb/vwdb0049-chipyard-clusteredrocketconfig-mt-memcpy vwdb/vwdb0053-chipyard-dualrocketconfig-dhrystone vwdb/vwdb0059-picorv32-test vwdb/vwdb0060-picorv32-test-ez vwdb/vwdb0069-scr1-axi-coremark vwdb/vwdb0072-scr1-axi-isr_sample vwdb/vwdb0074-scr1-axi-riscv_compliance vwdb/vwdb0085-systemc-lwtr4tlm2"
