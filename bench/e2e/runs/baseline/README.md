# CLI E2E Bench Run: baseline

- Generated at (UTC): 2026-03-28T21:58:35Z
- Hyperfine JSON files: 142
- Wavepeek JSON files: 142

## change

| test | mean_s | meta |
| --- | --- | --- |
| change_picorv32_signals_100_window_32us_trigger_posedge_clk | 0.442463 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M window_size=32us signal_count=100 trigger=posedge testbench.clk |
| change_picorv32_signals_100_window_8us_trigger_posedge_clk | 0.442351 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M window_size=8us signal_count=100 trigger=posedge testbench.clk |
| change_picorv32_signals_100_window_2us_trigger_signal | 0.441809 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M window_size=2us signal_count=100 trigger=testbench.top.mem_axi_arvalid |
| change_picorv32_signals_100_window_2us_trigger_posedge_clk | 0.441688 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M window_size=2us signal_count=100 trigger=posedge testbench.clk |
| change_picorv32_signals_100_window_2us_trigger_any | 0.441532 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M window_size=2us signal_count=100 trigger=* |
| change_picorv32_signals_100_window_32us_trigger_any | 0.441352 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M window_size=32us signal_count=100 trigger=* |
| change_picorv32_signals_100_window_32us_trigger_signal | 0.441315 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M window_size=32us signal_count=100 trigger=testbench.top.mem_axi_arvalid |
| change_picorv32_signals_100_window_8us_trigger_any | 0.440915 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M window_size=8us signal_count=100 trigger=* |
| change_picorv32_signals_100_window_8us_trigger_signal | 0.440256 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M window_size=8us signal_count=100 trigger=testbench.top.mem_axi_arvalid |
| change_chipyard_dualrocketconfig_dhrystone_signals_1_window_32us_trigger_posedge_clk | 0.401296 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M window_size=32us signal_count=1 trigger=posedge TOP.TestDriver.clock |
| change_chipyard_clusteredrocketconfig_dhrystone_signals_1_window_32us_trigger_posedge_clk | 0.390514 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M window_size=32us signal_count=1 trigger=posedge TOP.TestDriver.clock |
| change_chipyard_clusteredrocketconfig_dhrystone_signals_100_window_32us_trigger_posedge_clk | 0.341397 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M window_size=32us signal_count=100 trigger=posedge TOP.TestDriver.clock |
| change_chipyard_clusteredrocketconfig_dhrystone_signals_100_window_2us_trigger_signal | 0.309785 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M window_size=2us signal_count=100 trigger=TOP.TestDriver.testHarness.chiptop0.axi4_mem_0_bits_ar_valid |
| change_chipyard_clusteredrocketconfig_dhrystone_signals_100_window_32us_trigger_signal | 0.300984 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M window_size=32us signal_count=100 trigger=TOP.TestDriver.testHarness.chiptop0.axi4_mem_0_bits_ar_valid |
| change_chipyard_clusteredrocketconfig_dhrystone_signals_100_window_2us_trigger_posedge_clk | 0.300239 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M window_size=2us signal_count=100 trigger=posedge TOP.TestDriver.clock |
| change_chipyard_clusteredrocketconfig_dhrystone_signals_10_window_32us_trigger_signal | 0.291067 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M window_size=32us signal_count=10 trigger=TOP.TestDriver.testHarness.chiptop0.axi4_mem_0_bits_ar_valid |
| change_chipyard_clusteredrocketconfig_dhrystone_signals_100_window_8us_trigger_signal | 0.290033 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M window_size=8us signal_count=100 trigger=TOP.TestDriver.testHarness.chiptop0.axi4_mem_0_bits_ar_valid |
| change_chipyard_clusteredrocketconfig_dhrystone_signals_100_window_8us_trigger_posedge_clk | 0.282665 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M window_size=8us signal_count=100 trigger=posedge TOP.TestDriver.clock |
| change_chipyard_clusteredrocketconfig_dhrystone_signals_100_window_8us_trigger_any | 0.281290 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M window_size=8us signal_count=100 trigger=* |
| change_chipyard_clusteredrocketconfig_dhrystone_signals_1_window_8us_trigger_posedge_clk | 0.280958 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M window_size=8us signal_count=1 trigger=posedge TOP.TestDriver.clock |
| change_chipyard_clusteredrocketconfig_dhrystone_signals_100_window_32us_trigger_any | 0.280742 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M window_size=32us signal_count=100 trigger=* |
| change_chipyard_clusteredrocketconfig_dhrystone_signals_10_window_8us_trigger_posedge_clk | 0.280522 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M window_size=8us signal_count=10 trigger=posedge TOP.TestDriver.clock |
| change_chipyard_clusteredrocketconfig_dhrystone_signals_100_window_2us_trigger_any | 0.269632 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M window_size=2us signal_count=100 trigger=* |
| change_chipyard_clusteredrocketconfig_dhrystone_signals_10_window_2us_trigger_posedge_clk | 0.261076 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M window_size=2us signal_count=10 trigger=posedge TOP.TestDriver.clock |
| change_chipyard_clusteredrocketconfig_dhrystone_signals_10_window_8us_trigger_signal | 0.251984 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M window_size=8us signal_count=10 trigger=TOP.TestDriver.testHarness.chiptop0.axi4_mem_0_bits_ar_valid |
| change_chipyard_dualrocketconfig_dhrystone_signals_10_window_32us_trigger_signal | 0.250600 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M window_size=32us signal_count=10 trigger=TOP.TestDriver.testHarness.chiptop0.axi4_mem_0_bits_ar_valid |
| change_chipyard_clusteredrocketconfig_dhrystone_signals_1_window_2us_trigger_posedge_clk | 0.249761 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M window_size=2us signal_count=1 trigger=posedge TOP.TestDriver.clock |
| change_chipyard_clusteredrocketconfig_dhrystone_signals_10_window_2us_trigger_signal | 0.249095 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M window_size=2us signal_count=10 trigger=TOP.TestDriver.testHarness.chiptop0.axi4_mem_0_bits_ar_valid |
| change_chipyard_dualrocketconfig_dhrystone_signals_10_window_2us_trigger_posedge_clk | 0.241362 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M window_size=2us signal_count=10 trigger=posedge TOP.TestDriver.clock |
| change_chipyard_dualrocketconfig_dhrystone_signals_10_window_8us_trigger_signal | 0.241260 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M window_size=8us signal_count=10 trigger=TOP.TestDriver.testHarness.chiptop0.axi4_mem_0_bits_ar_valid |
| change_chipyard_clusteredrocketconfig_dhrystone_signals_10_window_8us_trigger_any | 0.240726 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M window_size=8us signal_count=10 trigger=* |
| change_chipyard_dualrocketconfig_dhrystone_signals_100_window_2us_trigger_signal | 0.240649 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M window_size=2us signal_count=100 trigger=TOP.TestDriver.testHarness.chiptop0.axi4_mem_0_bits_ar_valid |
| change_chipyard_dualrocketconfig_dhrystone_signals_1_window_2us_trigger_posedge_clk | 0.240637 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M window_size=2us signal_count=1 trigger=posedge TOP.TestDriver.clock |
| change_chipyard_dualrocketconfig_dhrystone_signals_10_window_2us_trigger_signal | 0.240522 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M window_size=2us signal_count=10 trigger=TOP.TestDriver.testHarness.chiptop0.axi4_mem_0_bits_ar_valid |
| change_chipyard_clusteredrocketconfig_dhrystone_signals_10_window_2us_trigger_any | 0.240408 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M window_size=2us signal_count=10 trigger=* |
| change_chipyard_dualrocketconfig_dhrystone_signals_10_window_8us_trigger_posedge_clk | 0.240245 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M window_size=8us signal_count=10 trigger=posedge TOP.TestDriver.clock |
| change_chipyard_dualrocketconfig_dhrystone_signals_100_window_32us_trigger_posedge_clk | 0.239918 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M window_size=32us signal_count=100 trigger=posedge TOP.TestDriver.clock |
| change_chipyard_dualrocketconfig_dhrystone_signals_100_window_2us_trigger_posedge_clk | 0.239682 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M window_size=2us signal_count=100 trigger=posedge TOP.TestDriver.clock |
| change_chipyard_dualrocketconfig_dhrystone_signals_100_window_8us_trigger_signal | 0.239627 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M window_size=8us signal_count=100 trigger=TOP.TestDriver.testHarness.chiptop0.axi4_mem_0_bits_ar_valid |
| change_chipyard_dualrocketconfig_dhrystone_signals_100_window_32us_trigger_signal | 0.239552 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M window_size=32us signal_count=100 trigger=TOP.TestDriver.testHarness.chiptop0.axi4_mem_0_bits_ar_valid |
| change_chipyard_dualrocketconfig_dhrystone_signals_1_window_8us_trigger_posedge_clk | 0.239147 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M window_size=8us signal_count=1 trigger=posedge TOP.TestDriver.clock |
| change_chipyard_clusteredrocketconfig_dhrystone_signals_10_window_32us_trigger_any | 0.230428 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M window_size=32us signal_count=10 trigger=* |
| change_chipyard_clusteredrocketconfig_dhrystone_signals_1_window_32us_trigger_signal | 0.220340 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M window_size=32us signal_count=1 trigger=TOP.TestDriver.testHarness.chiptop0.axi4_mem_0_bits_ar_valid |
| change_chipyard_clusteredrocketconfig_dhrystone_signals_10_window_32us_trigger_posedge_clk | 0.219240 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M window_size=32us signal_count=10 trigger=posedge TOP.TestDriver.clock |
| change_chipyard_dualrocketconfig_dhrystone_signals_100_window_8us_trigger_any | 0.199646 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M window_size=8us signal_count=100 trigger=* |
| change_chipyard_dualrocketconfig_dhrystone_signals_100_window_8us_trigger_posedge_clk | 0.199441 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M window_size=8us signal_count=100 trigger=posedge TOP.TestDriver.clock |
| change_chipyard_dualrocketconfig_dhrystone_signals_100_window_32us_trigger_any | 0.199400 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M window_size=32us signal_count=100 trigger=* |
| change_scr1_signals_100_window_8ns_trigger_any | 0.191704 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=3.5M window_size=8ns signal_count=100 trigger=* |
| change_chipyard_dualrocketconfig_dhrystone_signals_1_window_32us_trigger_signal | 0.191563 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M window_size=32us signal_count=1 trigger=TOP.TestDriver.testHarness.chiptop0.axi4_mem_0_bits_ar_valid |
| change_chipyard_dualrocketconfig_dhrystone_signals_10_window_8us_trigger_any | 0.190928 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M window_size=8us signal_count=10 trigger=* |
| change_scr1_signals_100_window_8ns_trigger_signal | 0.190835 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=3.5M window_size=8ns signal_count=100 trigger=TOP.scr1_top_tb_axi.io_axi_dmem_araddr |
| change_chipyard_clusteredrocketconfig_dhrystone_signals_1_window_2us_trigger_signal | 0.190793 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M window_size=2us signal_count=1 trigger=TOP.TestDriver.testHarness.chiptop0.axi4_mem_0_bits_ar_valid |
| change_scr1_signals_100_window_4ns_trigger_posedge_clk | 0.190484 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=3.5M window_size=4ns signal_count=100 trigger=posedge TOP.clk |
| change_scr1_signals_100_window_2ns_trigger_posedge_clk | 0.190419 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=3.5M window_size=2ns signal_count=100 trigger=posedge TOP.clk |
| change_scr1_signals_100_pos_50_window_2ns_trigger_any | 0.190289 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=3.5M window_size=2ns signal_count=100 trigger=* |
| change_scr1_signals_100_window_2ns_trigger_signal | 0.190196 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=3.5M window_size=2ns signal_count=100 trigger=TOP.scr1_top_tb_axi.io_axi_dmem_araddr |
| change_chipyard_clusteredrocketconfig_dhrystone_signals_1_window_8us_trigger_signal | 0.190169 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M window_size=8us signal_count=1 trigger=TOP.TestDriver.testHarness.chiptop0.axi4_mem_0_bits_ar_valid |
| change_scr1_signals_100_window_8ns_trigger_posedge_clk | 0.190012 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=3.5M window_size=8ns signal_count=100 trigger=posedge TOP.clk |
| change_chipyard_clusteredrocketconfig_dhrystone_signals_1_window_32us_trigger_any | 0.189881 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M window_size=32us signal_count=1 trigger=* |
| change_scr1_signals_100_window_4ns_trigger_any | 0.189872 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=3.5M window_size=4ns signal_count=100 trigger=* |
| change_chipyard_dualrocketconfig_dhrystone_signals_10_window_32us_trigger_posedge_clk | 0.189871 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M window_size=32us signal_count=10 trigger=posedge TOP.TestDriver.clock |
| change_chipyard_dualrocketconfig_dhrystone_signals_10_window_32us_trigger_any | 0.189729 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M window_size=32us signal_count=10 trigger=* |
| change_scr1_signals_100_window_4ns_trigger_signal | 0.189656 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=3.5M window_size=4ns signal_count=100 trigger=TOP.scr1_top_tb_axi.io_axi_dmem_araddr |
| change_chipyard_dualrocketconfig_dhrystone_signals_100_window_2us_trigger_any | 0.189386 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M window_size=2us signal_count=100 trigger=* |
| change_chipyard_dualrocketconfig_dhrystone_signals_10_window_2us_trigger_any | 0.188501 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M window_size=2us signal_count=10 trigger=* |
| change_chipyard_dualrocketconfig_dhrystone_signals_1_window_8us_trigger_signal | 0.160311 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M window_size=8us signal_count=1 trigger=TOP.TestDriver.testHarness.chiptop0.axi4_mem_0_bits_ar_valid |
| change_picorv32_signals_10_window_8us_trigger_signal | 0.150212 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M window_size=8us signal_count=10 trigger=testbench.top.mem_axi_arvalid |
| change_chipyard_dualrocketconfig_dhrystone_signals_1_window_32us_trigger_any | 0.149533 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M window_size=32us signal_count=1 trigger=* |
| change_chipyard_dualrocketconfig_dhrystone_signals_1_window_8us_trigger_any | 0.142452 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M window_size=8us signal_count=1 trigger=* |
| change_picorv32_signals_1_window_2us_trigger_posedge_clk | 0.141047 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M window_size=2us signal_count=1 trigger=posedge testbench.clk |
| change_picorv32_signals_10_window_32us_trigger_any | 0.140577 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M window_size=32us signal_count=10 trigger=* |
| change_picorv32_signals_10_window_8us_trigger_any | 0.140499 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M window_size=8us signal_count=10 trigger=* |
| change_picorv32_signals_10_window_8us_trigger_posedge_clk | 0.140452 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M window_size=8us signal_count=10 trigger=posedge testbench.clk |
| change_picorv32_signals_10_window_2us_trigger_any | 0.140443 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M window_size=2us signal_count=10 trigger=* |
| change_chipyard_clusteredrocketconfig_dhrystone_signals_1_window_8us_trigger_any | 0.140199 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M window_size=8us signal_count=1 trigger=* |
| change_picorv32_signals_1_window_32us_trigger_posedge_clk | 0.140142 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M window_size=32us signal_count=1 trigger=posedge testbench.clk |
| change_picorv32_signals_1_window_8us_trigger_posedge_clk | 0.140061 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M window_size=8us signal_count=1 trigger=posedge testbench.clk |
| change_chipyard_dualrocketconfig_dhrystone_signals_1_window_2us_trigger_any | 0.139871 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M window_size=2us signal_count=1 trigger=* |
| change_picorv32_signals_10_window_2us_trigger_posedge_clk | 0.139752 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M window_size=2us signal_count=10 trigger=posedge testbench.clk |
| change_chipyard_clusteredrocketconfig_dhrystone_signals_1_window_2us_trigger_any | 0.139725 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M window_size=2us signal_count=1 trigger=* |
| change_picorv32_signals_10_window_2us_trigger_signal | 0.139488 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M window_size=2us signal_count=10 trigger=testbench.top.mem_axi_arvalid |
| change_picorv32_signals_10_window_32us_trigger_posedge_clk | 0.139155 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M window_size=32us signal_count=10 trigger=posedge testbench.clk |
| change_picorv32_signals_10_window_32us_trigger_signal | 0.138897 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M window_size=32us signal_count=10 trigger=testbench.top.mem_axi_arvalid |
| change_chipyard_dualrocketconfig_dhrystone_signals_1_window_2us_trigger_signal | 0.138455 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M window_size=2us signal_count=1 trigger=TOP.TestDriver.testHarness.chiptop0.axi4_mem_0_bits_ar_valid |
| change_scr1_coremark_imem_axi_1sig_to_1000ps | 0.090762 | waves=/opt/rtl-artifacts/scr1_max_axi_coremark.fst scope=TOP.scr1_top_tb_axi.i_top.i_imem_axi signal_count=1 window_to=1000ps |
| change_scr1_coremark_imem_axi_2sig_to_1000ps | 0.090471 | waves=/opt/rtl-artifacts/scr1_max_axi_coremark.fst scope=TOP.scr1_top_tb_axi.i_top.i_imem_axi signal_count=2 window_to=1000ps |
| change_picorv32_signals_1_window_8us_trigger_any | 0.090256 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M window_size=8us signal_count=1 trigger=* |
| change_picorv32_signals_1_window_2us_trigger_any | 0.089719 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M window_size=2us signal_count=1 trigger=* |
| change_picorv32_signals_1_window_32us_trigger_any | 0.089656 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M window_size=32us signal_count=1 trigger=* |
| change_picorv32_signals_1_window_8us_trigger_signal | 0.089571 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M window_size=8us signal_count=1 trigger=testbench.top.mem_axi_arvalid |
| change_picorv32_signals_1_window_32us_trigger_signal | 0.089517 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M window_size=32us signal_count=1 trigger=testbench.top.mem_axi_arvalid |
| change_picorv32_signals_1_window_2us_trigger_signal | 0.088872 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M window_size=2us signal_count=1 trigger=testbench.top.mem_axi_arvalid |
| change_scr1_signals_10_window_8ns_trigger_signal | 0.065424 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=3.5M window_size=8ns signal_count=10 trigger=TOP.scr1_top_tb_axi.io_axi_dmem_araddr |
| change_scr1_signals_10_window_2ns_trigger_signal | 0.061501 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=3.5M window_size=2ns signal_count=10 trigger=TOP.scr1_top_tb_axi.io_axi_dmem_araddr |
| change_scr1_signals_10_window_4ns_trigger_signal | 0.061402 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=3.5M window_size=4ns signal_count=10 trigger=TOP.scr1_top_tb_axi.io_axi_dmem_araddr |
| change_scr1_signals_10_window_4ns_trigger_posedge_clk | 0.060755 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=3.5M window_size=4ns signal_count=10 trigger=posedge TOP.clk |
| change_scr1_signals_1_window_8ns_trigger_signal | 0.060530 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=3.5M window_size=8ns signal_count=1 trigger=TOP.scr1_top_tb_axi.io_axi_dmem_araddr |
| change_scr1_signals_1_window_2ns_trigger_any | 0.057981 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=3.5M window_size=2ns signal_count=1 trigger=* |
| change_scr1_signals_1_window_8ns_trigger_posedge_clk | 0.057559 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=3.5M window_size=8ns signal_count=1 trigger=posedge TOP.clk |
| change_scr1_signals_10_window_8ns_trigger_posedge_clk | 0.057534 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=3.5M window_size=8ns signal_count=10 trigger=posedge TOP.clk |
| change_scr1_signals_10_window_2ns_trigger_posedge_clk | 0.057444 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=3.5M window_size=2ns signal_count=10 trigger=posedge TOP.clk |
| change_scr1_signals_1_window_4ns_trigger_signal | 0.057348 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=3.5M window_size=4ns signal_count=1 trigger=TOP.scr1_top_tb_axi.io_axi_dmem_araddr |
| change_scr1_signals_1_window_4ns_trigger_any | 0.057289 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=3.5M window_size=4ns signal_count=1 trigger=* |
| change_scr1_signals_10_window_8ns_trigger_any | 0.057214 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=3.5M window_size=8ns signal_count=10 trigger=* |
| change_scr1_signals_1_window_2ns_trigger_signal | 0.057143 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=3.5M window_size=2ns signal_count=1 trigger=TOP.scr1_top_tb_axi.io_axi_dmem_araddr |
| change_scr1_signals_1_window_2ns_trigger_posedge_clk | 0.057124 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=3.5M window_size=2ns signal_count=1 trigger=posedge TOP.clk |
| change_scr1_signals_10_window_4ns_trigger_any | 0.057004 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=3.5M window_size=4ns signal_count=10 trigger=* |
| change_scr1_signals_10_window_2ns_trigger_any | 0.056997 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=3.5M window_size=2ns signal_count=10 trigger=* |
| change_scr1_signals_1_window_8ns_trigger_any | 0.056957 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=3.5M window_size=8ns signal_count=1 trigger=* |
| change_scr1_signals_1_window_4ns_trigger_posedge_clk | 0.056671 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=3.5M window_size=4ns signal_count=1 trigger=posedge TOP.clk |

## info

| test | mean_s | meta |
| --- | --- | --- |
| info_chipyard_clustered_mt_memcpy | 0.189625 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_mt-memcpy.fst size=411M |
| info_chipyard_dualrocket_dhrystone | 0.089537 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M |
| info_chipyard_clustered_dhrystone | 0.088999 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M |
| info_scr1 | 0.049564 | waves=/opt/rtl-artifacts/scr1_max_axi_coremark.fst size=21M |
| info_picorv32 | 0.044254 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M |
| info_scr1_riscv_compliance | 0.034748 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=4M |
| info_scr1_isr_sample | 0.029103 | waves=/opt/rtl-artifacts/scr1_max_axi_isr_sample.fst size=69K |
| info_picorv32_ez | 0.028711 | waves=/opt/rtl-artifacts/picorv32_test_ez_vcd.fst size=17K |

## property

| test | mean_s | meta |
| --- | --- | --- |
| property_chipyard_clusteredrocketconfig_dhrystone_window_2us_match_posedge_clk | 0.249484 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M window_size=2us capture=match trigger=posedge TOP.TestDriver.clock eval=TOP.TestDriver.testHarness.io_success == 1'b1 |
| property_chipyard_clusteredrocketconfig_dhrystone_window_2us_switch_wildcard | 0.139589 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M window_size=2us capture=switch trigger=* eval=TOP.TestDriver.testHarness.chiptop0.axi4_mem_0_bits_ar_valid == 1'b1 |

## scope

| test | mean_s | meta |
| --- | --- | --- |
| scope_clustered_all_depth13_json | 0.190657 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_mt-memcpy.fst size=411M filter=.* max_depth=13 scope_count=4625 |
| scope_dualrocket_filter_frontend_depth12_json | 0.089101 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M filter=.*frontend.* max_depth=12 scope_count=118 |
| scope_scr1_all_depth7_json | 0.039227 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=4M filter=.* max_depth=7 scope_count=136 |

## signal

| test | mean_s | meta |
| --- | --- | --- |
| signal_scr1_top_recursive_all_json | 0.041754 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst scope=TOP filter=.* recursive=True max_depth=default |
| signal_scr1_top_recursive_filter_valid_json | 0.039730 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst scope=TOP filter=(?i).*valid.* recursive=True max_depth=default |
| signal_scr1_top_recursive_depth2_json | 0.037059 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst scope=TOP filter=.* recursive=True max_depth=2 |

## value

| test | mean_s | meta |
| --- | --- | --- |
| value_picorv32_signals_1000 | 2.195246 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M signal_count=1000 |
| value_chipyard_clusteredrocketconfig_dhrystone_signals_1000 | 0.959873 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M signal_count=1000 |
| value_chipyard_dualrocketconfig_dhrystone_signals_1000 | 0.942282 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M signal_count=1000 |
| value_chipyard_clusteredrocketconfig_dhrystone_signals_100 | 0.551040 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M signal_count=100 |
| value_chipyard_dualrocketconfig_dhrystone_signals_100 | 0.542050 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M signal_count=100 |
| value_picorv32_signals_100 | 0.440168 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M signal_count=100 |
| value_scr1_signals_1000 | 0.399200 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=4M signal_count=1000 |
| value_chipyard_clusteredrocketconfig_dhrystone_signals_10 | 0.240559 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M signal_count=10 |
| value_chipyard_clusteredrocketconfig_dhrystone_signals_1 | 0.223092 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M signal_count=1 |
| value_chipyard_dualrocketconfig_dhrystone_signals_10 | 0.189885 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M signal_count=10 |
| value_picorv32_signals_10 | 0.189698 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M signal_count=10 |
| value_chipyard_dualrocketconfig_dhrystone_signals_1 | 0.188840 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M signal_count=1 |
| value_picorv32_signals_1 | 0.089329 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M signal_count=1 |
| value_scr1_signals_100 | 0.057485 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=4M signal_count=100 |
| value_scr1_signals_1 | 0.056792 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=4M signal_count=1 |
| value_scr1_signals_10 | 0.056514 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=4M signal_count=10 |
