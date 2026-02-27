# CLI E2E Bench Run: baseline

- Generated at (UTC): 2026-02-27T19:56:44Z
- Run directory: `/workspaces/feat-perf-harness-2/bench/e2e/runs/baseline`
- Hyperfine JSON files: 132

## at

| test | mean_s | meta |
| --- | --- | --- |
| at_picorv32_signals_1000 | 1.965522 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M signal_count=1000 |
| at_chipyard_clusteredrocketconfig_dhrystone_signals_1000 | 0.831917 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M signal_count=1000 |
| at_chipyard_dualrocketconfig_dhrystone_signals_1000 | 0.826907 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M signal_count=1000 |
| at_chipyard_clusteredrocketconfig_dhrystone_signals_100 | 0.457780 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M signal_count=100 |
| at_chipyard_dualrocketconfig_dhrystone_signals_100 | 0.453401 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M signal_count=100 |
| at_picorv32_signals_100 | 0.359129 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M signal_count=100 |
| at_scr1_signals_1000 | 0.319485 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=4M signal_count=1000 |
| at_chipyard_clusteredrocketconfig_dhrystone_signals_1 | 0.156700 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M signal_count=1 |
| at_chipyard_clusteredrocketconfig_dhrystone_signals_10 | 0.156350 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M signal_count=10 |
| at_chipyard_dualrocketconfig_dhrystone_signals_10 | 0.143679 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M signal_count=10 |
| at_chipyard_dualrocketconfig_dhrystone_signals_1 | 0.143534 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M signal_count=1 |
| at_picorv32_signals_10 | 0.135469 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M signal_count=10 |
| at_picorv32_signals_1 | 0.044486 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M signal_count=1 |
| at_scr1_signals_100 | 0.019116 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=4M signal_count=100 |
| at_scr1_signals_10 | 0.018932 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=4M signal_count=10 |
| at_scr1_signals_1 | 0.018932 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=4M signal_count=1 |

## change

| test | mean_s | meta |
| --- | --- | --- |
| change_scr1_signals_100_pos_50_window_8000_trigger_signal | 47.055737 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=3.5M window_size=8000 signal_count=100 trigger=TOP.scr1_top_tb_axi.io_axi_dmem_araddr |
| change_scr1_signals_100_pos_50_window_2000_trigger_signal | 34.181891 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=3.5M window_size=2000 signal_count=100 trigger=TOP.scr1_top_tb_axi.io_axi_dmem_araddr |
| change_chipyard_clusteredrocketconfig_dhrystone_signals_100_pos_50_window_8000_trigger_posedge_clk | 32.547362 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M window_size=8000 signal_count=100 trigger=posedge TOP.TestDriver.clock |
| change_scr1_signals_100_pos_50_window_4000_trigger_signal | 32.535255 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=3.5M window_size=4000 signal_count=100 trigger=TOP.scr1_top_tb_axi.io_axi_dmem_araddr |
| change_chipyard_clusteredrocketconfig_dhrystone_signals_100_pos_50_window_8000_trigger_signal | 32.222843 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M window_size=8000 signal_count=100 trigger=TOP.TestDriver.testHarness.chiptop0.axi4_mem_0_bits_ar_bits_addr |
| change_chipyard_clusteredrocketconfig_dhrystone_signals_100_pos_50_window_8000_trigger_any | 32.132266 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M window_size=8000 signal_count=100 trigger=* |
| change_chipyard_dualrocketconfig_dhrystone_signals_100_pos_50_window_8000_trigger_posedge_clk | 26.687438 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M window_size=8000 signal_count=100 trigger=posedge TOP.TestDriver.clock |
| change_chipyard_dualrocketconfig_dhrystone_signals_100_pos_50_window_8000_trigger_signal | 26.675447 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M window_size=8000 signal_count=100 trigger=TOP.TestDriver.testHarness.chiptop0.axi4_mem_0_bits_ar_bits_addr |
| change_chipyard_dualrocketconfig_dhrystone_signals_100_pos_50_window_8000_trigger_any | 26.475717 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M window_size=8000 signal_count=100 trigger=* |
| change_chipyard_clusteredrocketconfig_dhrystone_signals_100_pos_50_window_4000_trigger_any | 16.436100 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M window_size=4000 signal_count=100 trigger=* |
| change_chipyard_clusteredrocketconfig_dhrystone_signals_100_pos_50_window_4000_trigger_posedge_clk | 16.413961 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M window_size=4000 signal_count=100 trigger=posedge TOP.TestDriver.clock |
| change_chipyard_clusteredrocketconfig_dhrystone_signals_100_pos_50_window_4000_trigger_signal | 16.231053 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M window_size=4000 signal_count=100 trigger=TOP.TestDriver.testHarness.chiptop0.axi4_mem_0_bits_ar_bits_addr |
| change_chipyard_dualrocketconfig_dhrystone_signals_100_pos_50_window_4000_trigger_signal | 13.536282 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M window_size=4000 signal_count=100 trigger=TOP.TestDriver.testHarness.chiptop0.axi4_mem_0_bits_ar_bits_addr |
| change_chipyard_dualrocketconfig_dhrystone_signals_100_pos_50_window_4000_trigger_posedge_clk | 13.488849 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M window_size=4000 signal_count=100 trigger=posedge TOP.TestDriver.clock |
| change_chipyard_dualrocketconfig_dhrystone_signals_100_pos_50_window_4000_trigger_any | 13.453004 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M window_size=4000 signal_count=100 trigger=* |
| change_scr1_signals_100_pos_50_window_2000_trigger_posedge_clk | 11.861650 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=3.5M window_size=2000 signal_count=100 trigger=posedge TOP.clk |
| change_scr1_signals_100_pos_50_window_4000_trigger_posedge_clk | 11.699520 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=3.5M window_size=4000 signal_count=100 trigger=posedge TOP.clk |
| change_scr1_signals_100_pos_50_window_8000_trigger_posedge_clk | 11.674767 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=3.5M window_size=8000 signal_count=100 trigger=posedge TOP.clk |
| change_chipyard_clusteredrocketconfig_dhrystone_signals_100_pos_50_window_2000_trigger_posedge_clk | 9.413477 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M window_size=2000 signal_count=100 trigger=posedge TOP.TestDriver.clock |
| change_chipyard_clusteredrocketconfig_dhrystone_signals_100_pos_50_window_2000_trigger_any | 9.405727 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M window_size=2000 signal_count=100 trigger=* |
| change_chipyard_clusteredrocketconfig_dhrystone_signals_100_pos_50_window_2000_trigger_signal | 9.306654 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M window_size=2000 signal_count=100 trigger=TOP.TestDriver.testHarness.chiptop0.axi4_mem_0_bits_ar_bits_addr |
| change_chipyard_dualrocketconfig_dhrystone_signals_100_pos_50_window_2000_trigger_signal | 7.661702 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M window_size=2000 signal_count=100 trigger=TOP.TestDriver.testHarness.chiptop0.axi4_mem_0_bits_ar_bits_addr |
| change_chipyard_dualrocketconfig_dhrystone_signals_100_pos_50_window_2000_trigger_posedge_clk | 7.632622 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M window_size=2000 signal_count=100 trigger=posedge TOP.TestDriver.clock |
| change_chipyard_dualrocketconfig_dhrystone_signals_100_pos_50_window_2000_trigger_any | 7.607146 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M window_size=2000 signal_count=100 trigger=* |
| change_scr1_signals_100_pos_50_window_2000_trigger_any | 6.065045 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=3.5M window_size=2000 signal_count=100 trigger=* |
| change_scr1_signals_100_pos_50_window_8000_trigger_any | 6.008900 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=3.5M window_size=8000 signal_count=100 trigger=* |
| change_scr1_signals_100_pos_50_window_4000_trigger_any | 5.996367 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=3.5M window_size=4000 signal_count=100 trigger=* |
| change_scr1_signals_10_pos_50_window_8000_trigger_signal | 4.737597 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=3.5M window_size=8000 signal_count=10 trigger=TOP.scr1_top_tb_axi.io_axi_dmem_araddr |
| change_scr1_signals_10_pos_50_window_2000_trigger_signal | 3.435338 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=3.5M window_size=2000 signal_count=10 trigger=TOP.scr1_top_tb_axi.io_axi_dmem_araddr |
| change_chipyard_clusteredrocketconfig_dhrystone_signals_10_pos_50_window_8000_trigger_any | 3.413161 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M window_size=8000 signal_count=10 trigger=* |
| change_chipyard_clusteredrocketconfig_dhrystone_signals_10_pos_50_window_8000_trigger_posedge_clk | 3.402003 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M window_size=8000 signal_count=10 trigger=posedge TOP.TestDriver.clock |
| change_chipyard_clusteredrocketconfig_dhrystone_signals_10_pos_50_window_8000_trigger_signal | 3.397477 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M window_size=8000 signal_count=10 trigger=TOP.TestDriver.testHarness.chiptop0.axi4_mem_0_bits_ar_bits_addr |
| change_scr1_signals_10_pos_50_window_4000_trigger_signal | 3.243682 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=3.5M window_size=4000 signal_count=10 trigger=TOP.scr1_top_tb_axi.io_axi_dmem_araddr |
| change_chipyard_dualrocketconfig_dhrystone_signals_10_pos_50_window_8000_trigger_signal | 3.133370 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M window_size=8000 signal_count=10 trigger=TOP.TestDriver.testHarness.chiptop0.axi4_mem_0_bits_ar_bits_addr |
| change_chipyard_dualrocketconfig_dhrystone_signals_10_pos_50_window_8000_trigger_posedge_clk | 2.837489 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M window_size=8000 signal_count=10 trigger=posedge TOP.TestDriver.clock |
| change_chipyard_dualrocketconfig_dhrystone_signals_10_pos_50_window_8000_trigger_any | 2.831873 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M window_size=8000 signal_count=10 trigger=* |
| change_chipyard_clusteredrocketconfig_dhrystone_signals_10_pos_50_window_4000_trigger_signal | 1.795548 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M window_size=4000 signal_count=10 trigger=TOP.TestDriver.testHarness.chiptop0.axi4_mem_0_bits_ar_bits_addr |
| change_chipyard_clusteredrocketconfig_dhrystone_signals_10_pos_50_window_4000_trigger_any | 1.792207 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M window_size=4000 signal_count=10 trigger=* |
| change_chipyard_clusteredrocketconfig_dhrystone_signals_10_pos_50_window_4000_trigger_posedge_clk | 1.791754 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M window_size=4000 signal_count=10 trigger=posedge TOP.TestDriver.clock |
| change_chipyard_dualrocketconfig_dhrystone_signals_10_pos_50_window_4000_trigger_signal | 1.662550 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M window_size=4000 signal_count=10 trigger=TOP.TestDriver.testHarness.chiptop0.axi4_mem_0_bits_ar_bits_addr |
| change_chipyard_dualrocketconfig_dhrystone_signals_10_pos_50_window_4000_trigger_posedge_clk | 1.515693 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M window_size=4000 signal_count=10 trigger=posedge TOP.TestDriver.clock |
| change_chipyard_dualrocketconfig_dhrystone_signals_10_pos_50_window_4000_trigger_any | 1.511603 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M window_size=4000 signal_count=10 trigger=* |
| change_picorv32_signals_100_pos_50_window_8000_trigger_posedge_clk | 1.327240 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M window_size=8000 signal_count=100 trigger=posedge testbench.clk |
| change_picorv32_signals_100_pos_50_window_4000_trigger_posedge_clk | 1.321756 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M window_size=4000 signal_count=100 trigger=posedge testbench.clk |
| change_picorv32_signals_100_pos_50_window_2000_trigger_posedge_clk | 1.321137 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M window_size=2000 signal_count=100 trigger=posedge testbench.clk |
| change_picorv32_signals_100_pos_50_window_8000_trigger_signal | 1.320196 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M window_size=8000 signal_count=100 trigger=testbench.top.mem_axi_araddr |
| change_picorv32_signals_100_pos_50_window_8000_trigger_any | 1.320012 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M window_size=8000 signal_count=100 trigger=* |
| change_picorv32_signals_100_pos_50_window_4000_trigger_any | 1.319603 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M window_size=4000 signal_count=100 trigger=* |
| change_picorv32_signals_100_pos_50_window_4000_trigger_signal | 1.318948 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M window_size=4000 signal_count=100 trigger=testbench.top.mem_axi_araddr |
| change_picorv32_signals_100_pos_50_window_2000_trigger_signal | 1.310936 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M window_size=2000 signal_count=100 trigger=testbench.top.mem_axi_araddr |
| change_picorv32_signals_100_pos_50_window_2000_trigger_any | 1.302513 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M window_size=2000 signal_count=100 trigger=* |
| change_scr1_signals_10_pos_50_window_4000_trigger_posedge_clk | 1.187106 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=3.5M window_size=4000 signal_count=10 trigger=posedge TOP.clk |
| change_scr1_signals_10_pos_50_window_8000_trigger_posedge_clk | 1.185901 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=3.5M window_size=8000 signal_count=10 trigger=posedge TOP.clk |
| change_scr1_signals_10_pos_50_window_2000_trigger_posedge_clk | 1.185406 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=3.5M window_size=2000 signal_count=10 trigger=posedge TOP.clk |
| change_chipyard_clusteredrocketconfig_dhrystone_signals_10_pos_50_window_2000_trigger_signal | 1.092370 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M window_size=2000 signal_count=10 trigger=TOP.TestDriver.testHarness.chiptop0.axi4_mem_0_bits_ar_bits_addr |
| change_chipyard_clusteredrocketconfig_dhrystone_signals_10_pos_50_window_2000_trigger_any | 1.088238 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M window_size=2000 signal_count=10 trigger=* |
| change_chipyard_clusteredrocketconfig_dhrystone_signals_10_pos_50_window_2000_trigger_posedge_clk | 1.087801 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M window_size=2000 signal_count=10 trigger=posedge TOP.TestDriver.clock |
| change_chipyard_dualrocketconfig_dhrystone_signals_10_pos_50_window_2000_trigger_signal | 1.007926 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M window_size=2000 signal_count=10 trigger=TOP.TestDriver.testHarness.chiptop0.axi4_mem_0_bits_ar_bits_addr |
| change_scr1_signals_1_pos_50_window_8000_trigger_signal | 0.971275 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=3.5M window_size=8000 signal_count=1 trigger=TOP.scr1_top_tb_axi.io_axi_dmem_araddr |
| change_chipyard_dualrocketconfig_dhrystone_signals_10_pos_50_window_2000_trigger_any | 0.922631 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M window_size=2000 signal_count=10 trigger=* |
| change_chipyard_dualrocketconfig_dhrystone_signals_10_pos_50_window_2000_trigger_posedge_clk | 0.922599 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M window_size=2000 signal_count=10 trigger=posedge TOP.TestDriver.clock |
| change_chipyard_clusteredrocketconfig_dhrystone_signals_1_pos_50_window_8000_trigger_signal | 0.832189 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M window_size=8000 signal_count=1 trigger=TOP.TestDriver.testHarness.chiptop0.axi4_mem_0_bits_ar_bits_addr |
| change_scr1_signals_1_pos_50_window_2000_trigger_signal | 0.711462 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=3.5M window_size=2000 signal_count=1 trigger=TOP.scr1_top_tb_axi.io_axi_dmem_araddr |
| change_chipyard_dualrocketconfig_dhrystone_signals_1_pos_50_window_8000_trigger_signal | 0.711458 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M window_size=8000 signal_count=1 trigger=TOP.TestDriver.testHarness.chiptop0.axi4_mem_0_bits_ar_bits_addr |
| change_scr1_signals_1_pos_50_window_4000_trigger_signal | 0.676412 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=3.5M window_size=4000 signal_count=1 trigger=TOP.scr1_top_tb_axi.io_axi_dmem_araddr |
| change_scr1_signals_10_pos_50_window_8000_trigger_any | 0.614733 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=3.5M window_size=8000 signal_count=10 trigger=* |
| change_scr1_signals_10_pos_50_window_4000_trigger_any | 0.614337 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=3.5M window_size=4000 signal_count=10 trigger=* |
| change_scr1_signals_10_pos_50_window_2000_trigger_any | 0.611599 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=3.5M window_size=2000 signal_count=10 trigger=* |
| change_chipyard_clusteredrocketconfig_dhrystone_signals_1_pos_50_window_4000_trigger_signal | 0.505036 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M window_size=4000 signal_count=1 trigger=TOP.TestDriver.testHarness.chiptop0.axi4_mem_0_bits_ar_bits_addr |
| change_chipyard_clusteredrocketconfig_dhrystone_signals_1_pos_50_window_8000_trigger_any | 0.503895 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M window_size=8000 signal_count=1 trigger=* |
| change_chipyard_clusteredrocketconfig_dhrystone_signals_1_pos_50_window_8000_trigger_posedge_clk | 0.503748 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M window_size=8000 signal_count=1 trigger=posedge TOP.TestDriver.clock |
| change_chipyard_dualrocketconfig_dhrystone_signals_1_pos_50_window_4000_trigger_signal | 0.442211 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M window_size=4000 signal_count=1 trigger=TOP.TestDriver.testHarness.chiptop0.axi4_mem_0_bits_ar_bits_addr |
| change_chipyard_dualrocketconfig_dhrystone_signals_1_pos_50_window_8000_trigger_posedge_clk | 0.436438 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M window_size=8000 signal_count=1 trigger=posedge TOP.TestDriver.clock |
| change_chipyard_dualrocketconfig_dhrystone_signals_1_pos_50_window_8000_trigger_any | 0.432943 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M window_size=8000 signal_count=1 trigger=* |
| change_chipyard_clusteredrocketconfig_dhrystone_signals_1_pos_50_window_2000_trigger_signal | 0.363840 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M window_size=2000 signal_count=1 trigger=TOP.TestDriver.testHarness.chiptop0.axi4_mem_0_bits_ar_bits_addr |
| change_chipyard_clusteredrocketconfig_dhrystone_signals_1_pos_50_window_4000_trigger_posedge_clk | 0.344734 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M window_size=4000 signal_count=1 trigger=posedge TOP.TestDriver.clock |
| change_chipyard_clusteredrocketconfig_dhrystone_signals_1_pos_50_window_4000_trigger_any | 0.340732 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M window_size=4000 signal_count=1 trigger=* |
| change_chipyard_dualrocketconfig_dhrystone_signals_1_pos_50_window_2000_trigger_signal | 0.320828 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M window_size=2000 signal_count=1 trigger=TOP.TestDriver.testHarness.chiptop0.axi4_mem_0_bits_ar_bits_addr |
| change_chipyard_dualrocketconfig_dhrystone_signals_1_pos_50_window_4000_trigger_any | 0.298734 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M window_size=4000 signal_count=1 trigger=* |
| change_chipyard_dualrocketconfig_dhrystone_signals_1_pos_50_window_4000_trigger_posedge_clk | 0.298456 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M window_size=4000 signal_count=1 trigger=posedge TOP.TestDriver.clock |
| change_chipyard_clusteredrocketconfig_dhrystone_signals_1_pos_50_window_2000_trigger_posedge_clk | 0.273174 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M window_size=2000 signal_count=1 trigger=posedge TOP.TestDriver.clock |
| change_chipyard_clusteredrocketconfig_dhrystone_signals_1_pos_50_window_2000_trigger_any | 0.271691 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M window_size=2000 signal_count=1 trigger=* |
| change_chipyard_dualrocketconfig_dhrystone_signals_1_pos_50_window_2000_trigger_posedge_clk | 0.244053 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M window_size=2000 signal_count=1 trigger=posedge TOP.TestDriver.clock |
| change_chipyard_dualrocketconfig_dhrystone_signals_1_pos_50_window_2000_trigger_any | 0.243222 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M window_size=2000 signal_count=1 trigger=* |
| change_picorv32_signals_10_pos_50_window_2000_trigger_any | 0.194989 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M window_size=2000 signal_count=10 trigger=* |
| change_picorv32_signals_10_pos_50_window_4000_trigger_signal | 0.192853 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M window_size=4000 signal_count=10 trigger=testbench.top.mem_axi_araddr |
| change_picorv32_signals_10_pos_50_window_4000_trigger_any | 0.192298 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M window_size=4000 signal_count=10 trigger=* |
| change_picorv32_signals_10_pos_50_window_8000_trigger_signal | 0.191114 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M window_size=8000 signal_count=10 trigger=testbench.top.mem_axi_araddr |
| change_picorv32_signals_10_pos_50_window_2000_trigger_signal | 0.190999 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M window_size=2000 signal_count=10 trigger=testbench.top.mem_axi_araddr |
| change_picorv32_signals_10_pos_50_window_8000_trigger_any | 0.190788 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M window_size=8000 signal_count=10 trigger=* |
| change_picorv32_signals_10_pos_50_window_8000_trigger_posedge_clk | 0.189485 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M window_size=8000 signal_count=10 trigger=posedge testbench.clk |
| change_picorv32_signals_10_pos_50_window_4000_trigger_posedge_clk | 0.189411 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M window_size=4000 signal_count=10 trigger=posedge testbench.clk |
| change_picorv32_signals_10_pos_50_window_2000_trigger_posedge_clk | 0.188228 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M window_size=2000 signal_count=10 trigger=posedge testbench.clk |
| change_scr1_signals_1_pos_50_window_8000_trigger_posedge_clk | 0.140467 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=3.5M window_size=8000 signal_count=1 trigger=posedge TOP.clk |
| change_scr1_signals_1_pos_50_window_2000_trigger_posedge_clk | 0.139710 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=3.5M window_size=2000 signal_count=1 trigger=posedge TOP.clk |
| change_scr1_signals_1_pos_50_window_4000_trigger_posedge_clk | 0.138107 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=3.5M window_size=4000 signal_count=1 trigger=posedge TOP.clk |
| change_picorv32_signals_1_pos_50_window_4000_trigger_signal | 0.083387 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M window_size=4000 signal_count=1 trigger=testbench.top.mem_axi_araddr |
| change_picorv32_signals_1_pos_50_window_2000_trigger_signal | 0.082822 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M window_size=2000 signal_count=1 trigger=testbench.top.mem_axi_araddr |
| change_picorv32_signals_1_pos_50_window_8000_trigger_signal | 0.081520 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M window_size=8000 signal_count=1 trigger=testbench.top.mem_axi_araddr |
| change_scr1_signals_1_pos_50_window_8000_trigger_any | 0.079226 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=3.5M window_size=8000 signal_count=1 trigger=* |
| change_scr1_signals_1_pos_50_window_2000_trigger_any | 0.079153 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=3.5M window_size=2000 signal_count=1 trigger=* |
| change_scr1_signals_1_pos_50_window_4000_trigger_any | 0.079112 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=3.5M window_size=4000 signal_count=1 trigger=* |
| change_picorv32_signals_1_pos_50_window_4000_trigger_posedge_clk | 0.063169 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M window_size=4000 signal_count=1 trigger=posedge testbench.clk |
| change_picorv32_signals_1_pos_50_window_8000_trigger_any | 0.061697 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M window_size=8000 signal_count=1 trigger=* |
| change_picorv32_signals_1_pos_50_window_8000_trigger_posedge_clk | 0.061677 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M window_size=8000 signal_count=1 trigger=posedge testbench.clk |
| change_picorv32_signals_1_pos_50_window_4000_trigger_any | 0.061395 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M window_size=4000 signal_count=1 trigger=* |
| change_picorv32_signals_1_pos_50_window_2000_trigger_any | 0.061355 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M window_size=2000 signal_count=1 trigger=* |
| change_picorv32_signals_1_pos_50_window_2000_trigger_posedge_clk | 0.061343 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M window_size=2000 signal_count=1 trigger=posedge testbench.clk |

## info

| test | mean_s | meta |
| --- | --- | --- |
| info_chipyard_clustered_mt_memcpy | 0.138295 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_mt-memcpy.fst size=411M |
| info_chipyard_clustered_dhrystone | 0.053805 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M |
| info_chipyard_dualrocket_dhrystone | 0.041587 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M |
| info_scr1 | 0.014926 | waves=/opt/rtl-artifacts/scr1_max_axi_coremark.fst size=21M |
| info_picorv32 | 0.013600 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M |
| info_scr1_riscv_compliance | 0.006769 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=4M |
| info_scr1_isr_sample | 0.002232 | waves=/opt/rtl-artifacts/scr1_max_axi_isr_sample.fst size=69K |
| info_picorv32_ez | 0.001475 | waves=/opt/rtl-artifacts/picorv32_test_ez_vcd.fst size=17K |
