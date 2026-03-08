# CLI E2E Bench Run: baseline

- Generated at (UTC): 2026-03-08T11:18:05Z
- Hyperfine JSON files: 140
- Wavepeek JSON files: 140

## change

| test | mean_s | meta |
| --- | --- | --- |
| change_chipyard_dualrocketconfig_dhrystone_signals_1_window_32us_trigger_posedge_clk | 0.592428 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M window_size=32us signal_count=1 trigger=posedge TOP.TestDriver.clock |
| change_chipyard_clusteredrocketconfig_dhrystone_signals_1_window_32us_trigger_posedge_clk | 0.532377 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M window_size=32us signal_count=1 trigger=posedge TOP.TestDriver.clock |
| change_picorv32_signals_100_window_32us_trigger_signal | 0.441279 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M window_size=32us signal_count=100 trigger=testbench.top.mem_axi_arvalid |
| change_picorv32_signals_100_window_8us_trigger_signal | 0.431210 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M window_size=8us signal_count=100 trigger=testbench.top.mem_axi_arvalid |
| change_picorv32_signals_100_window_2us_trigger_signal | 0.430502 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M window_size=2us signal_count=100 trigger=testbench.top.mem_axi_arvalid |
| change_picorv32_signals_100_window_32us_trigger_any | 0.410785 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M window_size=32us signal_count=100 trigger=* |
| change_picorv32_signals_100_window_2us_trigger_posedge_clk | 0.410258 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M window_size=2us signal_count=100 trigger=posedge testbench.clk |
| change_picorv32_signals_100_window_8us_trigger_any | 0.410109 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M window_size=8us signal_count=100 trigger=* |
| change_picorv32_signals_100_window_2us_trigger_any | 0.390575 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M window_size=2us signal_count=100 trigger=* |
| change_picorv32_signals_100_window_32us_trigger_posedge_clk | 0.390165 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M window_size=32us signal_count=100 trigger=posedge testbench.clk |
| change_picorv32_signals_100_window_8us_trigger_posedge_clk | 0.389151 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M window_size=8us signal_count=100 trigger=posedge testbench.clk |
| change_chipyard_clusteredrocketconfig_dhrystone_signals_100_window_32us_trigger_posedge_clk | 0.342847 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M window_size=32us signal_count=100 trigger=posedge TOP.TestDriver.clock |
| change_chipyard_clusteredrocketconfig_dhrystone_signals_100_window_32us_trigger_signal | 0.301084 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M window_size=32us signal_count=100 trigger=TOP.TestDriver.testHarness.chiptop0.axi4_mem_0_bits_ar_valid |
| change_chipyard_clusteredrocketconfig_dhrystone_signals_100_window_2us_trigger_signal | 0.300549 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M window_size=2us signal_count=100 trigger=TOP.TestDriver.testHarness.chiptop0.axi4_mem_0_bits_ar_valid |
| change_chipyard_clusteredrocketconfig_dhrystone_signals_1_window_8us_trigger_posedge_clk | 0.291658 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M window_size=8us signal_count=1 trigger=posedge TOP.TestDriver.clock |
| change_chipyard_clusteredrocketconfig_dhrystone_signals_100_window_2us_trigger_posedge_clk | 0.291340 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M window_size=2us signal_count=100 trigger=posedge TOP.TestDriver.clock |
| change_chipyard_clusteredrocketconfig_dhrystone_signals_10_window_32us_trigger_signal | 0.290744 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M window_size=32us signal_count=10 trigger=TOP.TestDriver.testHarness.chiptop0.axi4_mem_0_bits_ar_valid |
| change_chipyard_clusteredrocketconfig_dhrystone_signals_100_window_8us_trigger_signal | 0.290386 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M window_size=8us signal_count=100 trigger=TOP.TestDriver.testHarness.chiptop0.axi4_mem_0_bits_ar_valid |
| change_chipyard_clusteredrocketconfig_dhrystone_signals_100_window_32us_trigger_any | 0.261759 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M window_size=32us signal_count=100 trigger=* |
| change_chipyard_clusteredrocketconfig_dhrystone_signals_100_window_2us_trigger_any | 0.260487 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M window_size=2us signal_count=100 trigger=* |
| change_chipyard_dualrocketconfig_dhrystone_signals_1_window_8us_trigger_posedge_clk | 0.259274 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M window_size=8us signal_count=1 trigger=posedge TOP.TestDriver.clock |
| change_chipyard_dualrocketconfig_dhrystone_signals_1_window_2us_trigger_posedge_clk | 0.250594 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M window_size=2us signal_count=1 trigger=posedge TOP.TestDriver.clock |
| change_chipyard_dualrocketconfig_dhrystone_signals_100_window_32us_trigger_posedge_clk | 0.249795 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M window_size=32us signal_count=100 trigger=posedge TOP.TestDriver.clock |
| change_chipyard_clusteredrocketconfig_dhrystone_signals_1_window_2us_trigger_posedge_clk | 0.249664 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M window_size=2us signal_count=1 trigger=posedge TOP.TestDriver.clock |
| change_chipyard_clusteredrocketconfig_dhrystone_signals_100_window_8us_trigger_posedge_clk | 0.249534 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M window_size=8us signal_count=100 trigger=posedge TOP.TestDriver.clock |
| change_chipyard_clusteredrocketconfig_dhrystone_signals_100_window_8us_trigger_any | 0.248974 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M window_size=8us signal_count=100 trigger=* |
| change_chipyard_dualrocketconfig_dhrystone_signals_10_window_8us_trigger_posedge_clk | 0.242279 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M window_size=8us signal_count=10 trigger=posedge TOP.TestDriver.clock |
| change_chipyard_dualrocketconfig_dhrystone_signals_10_window_2us_trigger_posedge_clk | 0.241840 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M window_size=2us signal_count=10 trigger=posedge TOP.TestDriver.clock |
| change_chipyard_clusteredrocketconfig_dhrystone_signals_10_window_8us_trigger_any | 0.241782 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M window_size=8us signal_count=10 trigger=* |
| change_chipyard_dualrocketconfig_dhrystone_signals_100_window_2us_trigger_signal | 0.241525 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M window_size=2us signal_count=100 trigger=TOP.TestDriver.testHarness.chiptop0.axi4_mem_0_bits_ar_valid |
| change_chipyard_dualrocketconfig_dhrystone_signals_10_window_2us_trigger_signal | 0.241502 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M window_size=2us signal_count=10 trigger=TOP.TestDriver.testHarness.chiptop0.axi4_mem_0_bits_ar_valid |
| change_chipyard_clusteredrocketconfig_dhrystone_signals_10_window_32us_trigger_posedge_clk | 0.241270 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M window_size=32us signal_count=10 trigger=posedge TOP.TestDriver.clock |
| change_chipyard_dualrocketconfig_dhrystone_signals_100_window_32us_trigger_signal | 0.241223 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M window_size=32us signal_count=100 trigger=TOP.TestDriver.testHarness.chiptop0.axi4_mem_0_bits_ar_valid |
| change_chipyard_dualrocketconfig_dhrystone_signals_100_window_8us_trigger_signal | 0.241029 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M window_size=8us signal_count=100 trigger=TOP.TestDriver.testHarness.chiptop0.axi4_mem_0_bits_ar_valid |
| change_chipyard_dualrocketconfig_dhrystone_signals_10_window_8us_trigger_signal | 0.240955 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M window_size=8us signal_count=10 trigger=TOP.TestDriver.testHarness.chiptop0.axi4_mem_0_bits_ar_valid |
| change_chipyard_dualrocketconfig_dhrystone_signals_100_window_2us_trigger_posedge_clk | 0.240178 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M window_size=2us signal_count=100 trigger=posedge TOP.TestDriver.clock |
| change_chipyard_clusteredrocketconfig_dhrystone_signals_10_window_2us_trigger_posedge_clk | 0.239640 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M window_size=2us signal_count=10 trigger=posedge TOP.TestDriver.clock |
| change_chipyard_clusteredrocketconfig_dhrystone_signals_10_window_8us_trigger_posedge_clk | 0.239371 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M window_size=8us signal_count=10 trigger=posedge TOP.TestDriver.clock |
| change_chipyard_dualrocketconfig_dhrystone_signals_10_window_32us_trigger_signal | 0.239304 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M window_size=32us signal_count=10 trigger=TOP.TestDriver.testHarness.chiptop0.axi4_mem_0_bits_ar_valid |
| change_chipyard_clusteredrocketconfig_dhrystone_signals_10_window_8us_trigger_signal | 0.238583 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M window_size=8us signal_count=10 trigger=TOP.TestDriver.testHarness.chiptop0.axi4_mem_0_bits_ar_valid |
| change_chipyard_clusteredrocketconfig_dhrystone_signals_10_window_2us_trigger_signal | 0.238425 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M window_size=2us signal_count=10 trigger=TOP.TestDriver.testHarness.chiptop0.axi4_mem_0_bits_ar_valid |
| change_chipyard_clusteredrocketconfig_dhrystone_signals_10_window_2us_trigger_any | 0.210143 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M window_size=2us signal_count=10 trigger=* |
| change_chipyard_dualrocketconfig_dhrystone_signals_10_window_2us_trigger_any | 0.200681 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M window_size=2us signal_count=10 trigger=* |
| change_chipyard_clusteredrocketconfig_dhrystone_signals_1_window_32us_trigger_signal | 0.199989 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M window_size=32us signal_count=1 trigger=TOP.TestDriver.testHarness.chiptop0.axi4_mem_0_bits_ar_valid |
| change_chipyard_dualrocketconfig_dhrystone_signals_100_window_32us_trigger_any | 0.199056 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M window_size=32us signal_count=100 trigger=* |
| change_chipyard_dualrocketconfig_dhrystone_signals_100_window_8us_trigger_posedge_clk | 0.199001 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M window_size=8us signal_count=100 trigger=posedge TOP.TestDriver.clock |
| change_chipyard_clusteredrocketconfig_dhrystone_signals_10_window_32us_trigger_any | 0.198665 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M window_size=32us signal_count=10 trigger=* |
| change_scr1_signals_100_window_8ns_trigger_any | 0.193229 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=3.5M window_size=8ns signal_count=100 trigger=* |
| change_scr1_signals_100_window_4ns_trigger_posedge_clk | 0.192303 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=3.5M window_size=4ns signal_count=100 trigger=posedge TOP.clk |
| change_scr1_signals_100_window_2ns_trigger_signal | 0.192192 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=3.5M window_size=2ns signal_count=100 trigger=TOP.scr1_top_tb_axi.io_axi_dmem_araddr |
| change_scr1_signals_100_window_8ns_trigger_signal | 0.191592 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=3.5M window_size=8ns signal_count=100 trigger=TOP.scr1_top_tb_axi.io_axi_dmem_araddr |
| change_scr1_signals_100_pos_50_window_2ns_trigger_any | 0.191352 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=3.5M window_size=2ns signal_count=100 trigger=* |
| change_chipyard_clusteredrocketconfig_dhrystone_signals_1_window_2us_trigger_signal | 0.191290 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M window_size=2us signal_count=1 trigger=TOP.TestDriver.testHarness.chiptop0.axi4_mem_0_bits_ar_valid |
| change_scr1_signals_100_window_4ns_trigger_any | 0.191096 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=3.5M window_size=4ns signal_count=100 trigger=* |
| change_chipyard_dualrocketconfig_dhrystone_signals_1_window_32us_trigger_signal | 0.191008 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M window_size=32us signal_count=1 trigger=TOP.TestDriver.testHarness.chiptop0.axi4_mem_0_bits_ar_valid |
| change_scr1_signals_100_window_2ns_trigger_posedge_clk | 0.190770 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=3.5M window_size=2ns signal_count=100 trigger=posedge TOP.clk |
| change_chipyard_clusteredrocketconfig_dhrystone_signals_1_window_32us_trigger_any | 0.190569 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M window_size=32us signal_count=1 trigger=* |
| change_chipyard_clusteredrocketconfig_dhrystone_signals_1_window_8us_trigger_signal | 0.190556 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M window_size=8us signal_count=1 trigger=TOP.TestDriver.testHarness.chiptop0.axi4_mem_0_bits_ar_valid |
| change_scr1_signals_100_window_4ns_trigger_signal | 0.190290 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=3.5M window_size=4ns signal_count=100 trigger=TOP.scr1_top_tb_axi.io_axi_dmem_araddr |
| change_scr1_signals_100_window_8ns_trigger_posedge_clk | 0.190176 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=3.5M window_size=8ns signal_count=100 trigger=posedge TOP.clk |
| change_chipyard_dualrocketconfig_dhrystone_signals_10_window_32us_trigger_posedge_clk | 0.189977 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M window_size=32us signal_count=10 trigger=posedge TOP.TestDriver.clock |
| change_chipyard_dualrocketconfig_dhrystone_signals_10_window_32us_trigger_any | 0.189749 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M window_size=32us signal_count=10 trigger=* |
| change_chipyard_dualrocketconfig_dhrystone_signals_100_window_2us_trigger_any | 0.189612 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M window_size=2us signal_count=100 trigger=* |
| change_chipyard_dualrocketconfig_dhrystone_signals_100_window_8us_trigger_any | 0.189461 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M window_size=8us signal_count=100 trigger=* |
| change_chipyard_dualrocketconfig_dhrystone_signals_10_window_8us_trigger_any | 0.189158 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M window_size=8us signal_count=10 trigger=* |
| change_chipyard_dualrocketconfig_dhrystone_signals_1_window_32us_trigger_any | 0.159627 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M window_size=32us signal_count=1 trigger=* |
| change_chipyard_dualrocketconfig_dhrystone_signals_1_window_8us_trigger_signal | 0.159447 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M window_size=8us signal_count=1 trigger=TOP.TestDriver.testHarness.chiptop0.axi4_mem_0_bits_ar_valid |
| change_picorv32_signals_10_window_32us_trigger_any | 0.142236 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M window_size=32us signal_count=10 trigger=* |
| change_picorv32_signals_1_window_8us_trigger_posedge_clk | 0.141721 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M window_size=8us signal_count=1 trigger=posedge testbench.clk |
| change_picorv32_signals_10_window_2us_trigger_any | 0.141449 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M window_size=2us signal_count=10 trigger=* |
| change_picorv32_signals_10_window_2us_trigger_signal | 0.140832 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M window_size=2us signal_count=10 trigger=testbench.top.mem_axi_arvalid |
| change_picorv32_signals_10_window_8us_trigger_posedge_clk | 0.140818 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M window_size=8us signal_count=10 trigger=posedge testbench.clk |
| change_chipyard_dualrocketconfig_dhrystone_signals_1_window_8us_trigger_any | 0.140755 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M window_size=8us signal_count=1 trigger=* |
| change_picorv32_signals_10_window_8us_trigger_any | 0.140738 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M window_size=8us signal_count=10 trigger=* |
| change_chipyard_clusteredrocketconfig_dhrystone_signals_1_window_2us_trigger_any | 0.140645 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M window_size=2us signal_count=1 trigger=* |
| change_chipyard_dualrocketconfig_dhrystone_signals_1_window_2us_trigger_any | 0.140617 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M window_size=2us signal_count=1 trigger=* |
| change_picorv32_signals_10_window_32us_trigger_signal | 0.140496 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M window_size=32us signal_count=10 trigger=testbench.top.mem_axi_arvalid |
| change_picorv32_signals_10_window_32us_trigger_posedge_clk | 0.140048 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M window_size=32us signal_count=10 trigger=posedge testbench.clk |
| change_picorv32_signals_10_window_2us_trigger_posedge_clk | 0.140037 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M window_size=2us signal_count=10 trigger=posedge testbench.clk |
| change_picorv32_signals_10_window_8us_trigger_signal | 0.139952 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M window_size=8us signal_count=10 trigger=testbench.top.mem_axi_arvalid |
| change_chipyard_clusteredrocketconfig_dhrystone_signals_1_window_8us_trigger_any | 0.139668 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M window_size=8us signal_count=1 trigger=* |
| change_chipyard_dualrocketconfig_dhrystone_signals_1_window_2us_trigger_signal | 0.138329 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M window_size=2us signal_count=1 trigger=TOP.TestDriver.testHarness.chiptop0.axi4_mem_0_bits_ar_valid |
| change_picorv32_signals_1_window_2us_trigger_posedge_clk | 0.130544 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M window_size=2us signal_count=1 trigger=posedge testbench.clk |
| change_picorv32_signals_1_window_32us_trigger_posedge_clk | 0.129750 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M window_size=32us signal_count=1 trigger=posedge testbench.clk |
| change_picorv32_signals_1_window_8us_trigger_any | 0.091773 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M window_size=8us signal_count=1 trigger=* |
| change_scr1_coremark_imem_axi_2sig_to_1000ps | 0.091253 | waves=/opt/rtl-artifacts/scr1_max_axi_coremark.fst scope=TOP.scr1_top_tb_axi.i_top.i_imem_axi signal_count=2 window_to=1000ps |
| change_picorv32_signals_1_window_32us_trigger_any | 0.090952 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M window_size=32us signal_count=1 trigger=* |
| change_picorv32_signals_1_window_32us_trigger_signal | 0.090728 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M window_size=32us signal_count=1 trigger=testbench.top.mem_axi_arvalid |
| change_picorv32_signals_1_window_2us_trigger_any | 0.090438 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M window_size=2us signal_count=1 trigger=* |
| change_picorv32_signals_1_window_8us_trigger_signal | 0.089884 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M window_size=8us signal_count=1 trigger=testbench.top.mem_axi_arvalid |
| change_picorv32_signals_1_window_2us_trigger_signal | 0.088758 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M window_size=2us signal_count=1 trigger=testbench.top.mem_axi_arvalid |
| change_scr1_coremark_imem_axi_1sig_to_1000ps | 0.080017 | waves=/opt/rtl-artifacts/scr1_max_axi_coremark.fst scope=TOP.scr1_top_tb_axi.i_top.i_imem_axi signal_count=1 window_to=1000ps |
| change_scr1_signals_1_window_2ns_trigger_posedge_clk | 0.057639 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=3.5M window_size=2ns signal_count=1 trigger=posedge TOP.clk |
| change_scr1_signals_10_window_4ns_trigger_posedge_clk | 0.057364 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=3.5M window_size=4ns signal_count=10 trigger=posedge TOP.clk |
| change_scr1_signals_1_window_4ns_trigger_any | 0.057079 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=3.5M window_size=4ns signal_count=1 trigger=* |
| change_scr1_signals_1_window_4ns_trigger_posedge_clk | 0.057031 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=3.5M window_size=4ns signal_count=1 trigger=posedge TOP.clk |
| change_scr1_signals_10_window_8ns_trigger_signal | 0.056983 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=3.5M window_size=8ns signal_count=10 trigger=TOP.scr1_top_tb_axi.io_axi_dmem_araddr |
| change_scr1_signals_1_window_2ns_trigger_any | 0.056855 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=3.5M window_size=2ns signal_count=1 trigger=* |
| change_scr1_signals_1_window_8ns_trigger_any | 0.056658 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=3.5M window_size=8ns signal_count=1 trigger=* |
| change_scr1_signals_1_window_8ns_trigger_posedge_clk | 0.056652 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=3.5M window_size=8ns signal_count=1 trigger=posedge TOP.clk |
| change_scr1_signals_10_window_4ns_trigger_any | 0.056594 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=3.5M window_size=4ns signal_count=10 trigger=* |
| change_scr1_signals_1_window_8ns_trigger_signal | 0.056451 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=3.5M window_size=8ns signal_count=1 trigger=TOP.scr1_top_tb_axi.io_axi_dmem_araddr |
| change_scr1_signals_10_window_2ns_trigger_signal | 0.056445 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=3.5M window_size=2ns signal_count=10 trigger=TOP.scr1_top_tb_axi.io_axi_dmem_araddr |
| change_scr1_signals_1_window_4ns_trigger_signal | 0.056280 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=3.5M window_size=4ns signal_count=1 trigger=TOP.scr1_top_tb_axi.io_axi_dmem_araddr |
| change_scr1_signals_10_window_2ns_trigger_any | 0.056234 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=3.5M window_size=2ns signal_count=10 trigger=* |
| change_scr1_signals_10_window_2ns_trigger_posedge_clk | 0.056149 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=3.5M window_size=2ns signal_count=10 trigger=posedge TOP.clk |
| change_scr1_signals_10_window_4ns_trigger_signal | 0.056091 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=3.5M window_size=4ns signal_count=10 trigger=TOP.scr1_top_tb_axi.io_axi_dmem_araddr |
| change_scr1_signals_1_window_2ns_trigger_signal | 0.055885 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=3.5M window_size=2ns signal_count=1 trigger=TOP.scr1_top_tb_axi.io_axi_dmem_araddr |
| change_scr1_signals_10_window_8ns_trigger_posedge_clk | 0.055751 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=3.5M window_size=8ns signal_count=10 trigger=posedge TOP.clk |
| change_scr1_signals_10_window_8ns_trigger_any | 0.055535 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=3.5M window_size=8ns signal_count=10 trigger=* |

## info

| test | mean_s | meta |
| --- | --- | --- |
| info_chipyard_clustered_mt_memcpy | 0.190767 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_mt-memcpy.fst size=411M |
| info_chipyard_clustered_dhrystone | 0.091649 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M |
| info_chipyard_dualrocket_dhrystone | 0.090400 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M |
| info_scr1 | 0.046953 | waves=/opt/rtl-artifacts/scr1_max_axi_coremark.fst size=21M |
| info_picorv32 | 0.041040 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M |
| info_scr1_riscv_compliance | 0.031923 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=4M |
| info_scr1_isr_sample | 0.027301 | waves=/opt/rtl-artifacts/scr1_max_axi_isr_sample.fst size=69K |
| info_picorv32_ez | 0.027272 | waves=/opt/rtl-artifacts/picorv32_test_ez_vcd.fst size=17K |

## scope

| test | mean_s | meta |
| --- | --- | --- |
| scope_clustered_all_depth13_json | 0.189225 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_mt-memcpy.fst size=411M filter=.* max_depth=13 scope_count=4625 |
| scope_dualrocket_filter_frontend_depth12_json | 0.089034 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M filter=.*frontend.* max_depth=12 scope_count=118 |
| scope_scr1_all_depth7_json | 0.033829 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=4M filter=.* max_depth=7 scope_count=136 |

## signal

| test | mean_s | meta |
| --- | --- | --- |
| signal_scr1_top_recursive_all_json | 0.039328 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst scope=TOP filter=.* recursive=True max_depth=default |
| signal_scr1_top_recursive_filter_valid_json | 0.035870 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst scope=TOP filter=(?i).*valid.* recursive=True max_depth=default |
| signal_scr1_top_recursive_depth2_json | 0.031902 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst scope=TOP filter=.* recursive=True max_depth=2 |

## value

| test | mean_s | meta |
| --- | --- | --- |
| value_picorv32_signals_1000 | 2.062278 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M signal_count=1000 |
| value_chipyard_dualrocketconfig_dhrystone_signals_1000 | 0.909483 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M signal_count=1000 |
| value_chipyard_clusteredrocketconfig_dhrystone_signals_1000 | 0.900737 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M signal_count=1000 |
| value_chipyard_clusteredrocketconfig_dhrystone_signals_100 | 0.541917 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M signal_count=100 |
| value_chipyard_dualrocketconfig_dhrystone_signals_100 | 0.504892 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M signal_count=100 |
| value_picorv32_signals_100 | 0.441514 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M signal_count=100 |
| value_scr1_signals_1000 | 0.391262 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=4M signal_count=1000 |
| value_chipyard_clusteredrocketconfig_dhrystone_signals_1 | 0.218019 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M signal_count=1 |
| value_chipyard_clusteredrocketconfig_dhrystone_signals_10 | 0.209793 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M signal_count=10 |
| value_chipyard_dualrocketconfig_dhrystone_signals_10 | 0.190461 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M signal_count=10 |
| value_picorv32_signals_10 | 0.190113 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M signal_count=10 |
| value_chipyard_dualrocketconfig_dhrystone_signals_1 | 0.189745 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M signal_count=1 |
| value_picorv32_signals_1 | 0.090022 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M signal_count=1 |
| value_scr1_signals_10 | 0.056950 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=4M signal_count=10 |
| value_scr1_signals_1 | 0.056891 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=4M signal_count=1 |
| value_scr1_signals_100 | 0.056887 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=4M signal_count=100 |
