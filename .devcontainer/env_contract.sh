#!/usr/bin/env sh

# Shared environment contract for the wavepeek devcontainer image and the
# Codex cloud setup scripts. Keep versions and externally fetched artifact
# identities here so Dockerfile and Codex runtime setup do not drift.

WAVEPEEK_RUST_VERSION="1.93.0"
WAVEPEEK_CARGO_LLVM_COV_VERSION="0.8.7"
WAVEPEEK_ACTIONLINT_VERSION="1.7.12"
WAVEPEEK_HYPERFINE_VERSION="1.18.0"
WAVEPEEK_PRECOMMIT_VERSION="4.5.1"
WAVEPEEK_COMMITIZEN_VERSION="4.12.1"
WAVEPEEK_RTL_ARTIFACTS_VERSION="v1.0.0"
WAVEPEEK_SURFER_VERSION="v0.5.0"
WAVEPEEK_DEVCONTAINERS_CLI_VERSION="0.86.0"
WAVEPEEK_OPENCODE_VERSION="1.15.4"
WAVEPEEK_RTL_ARTIFACT_FILES="picorv32_test_vcd.fst scr1_max_axi_coremark.fst picorv32_test_ez_vcd.fst scr1_max_axi_isr_sample.fst scr1_max_axi_riscv_compliance.fst chipyard_DualRocketConfig_dhrystone.fst chipyard_ClusteredRocketConfig_dhrystone.fst chipyard_ClusteredRocketConfig_mt-memcpy.fst"
