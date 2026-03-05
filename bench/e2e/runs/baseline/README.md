# CLI E2E Bench Run: new-baseline

- Generated at (UTC): 2026-03-05T10:20:59Z
- Hyperfine JSON files: 137
- Wavepeek JSON files: 137

## change

| test | mean_s | meta |
| --- | --- | --- |
| change_chipyard_dualrocketconfig_dhrystone_signals_1_window_32us_trigger_posedge_clk | 0.592914 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M window_size=32us signal_count=1 trigger=posedge TOP.TestDriver.clock |
| change_chipyard_clusteredrocketconfig_dhrystone_signals_1_window_32us_trigger_posedge_clk | 0.511009 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M window_size=32us signal_count=1 trigger=posedge TOP.TestDriver.clock |
| change_picorv32_signals_100_window_8us_trigger_signal | 0.420783 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M window_size=8us signal_count=100 trigger=testbench.top.mem_axi_arvalid |
| change_picorv32_signals_100_window_2us_trigger_signal | 0.411365 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M window_size=2us signal_count=100 trigger=testbench.top.mem_axi_arvalid |
| change_picorv32_signals_100_window_32us_trigger_signal | 0.400145 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M window_size=32us signal_count=100 trigger=testbench.top.mem_axi_arvalid |
| change_picorv32_signals_100_window_32us_trigger_any | 0.399455 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M window_size=32us signal_count=100 trigger=* |
| change_picorv32_signals_100_window_8us_trigger_posedge_clk | 0.399200 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M window_size=8us signal_count=100 trigger=posedge testbench.clk |
| change_picorv32_signals_100_window_2us_trigger_posedge_clk | 0.399162 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M window_size=2us signal_count=100 trigger=posedge testbench.clk |
| change_picorv32_signals_100_window_32us_trigger_posedge_clk | 0.390228 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M window_size=32us signal_count=100 trigger=posedge testbench.clk |
| change_picorv32_signals_100_window_2us_trigger_any | 0.390035 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M window_size=2us signal_count=100 trigger=* |
| change_picorv32_signals_100_window_8us_trigger_any | 0.389722 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M window_size=8us signal_count=100 trigger=* |
| change_chipyard_clusteredrocketconfig_dhrystone_signals_100_window_2us_trigger_signal | 0.291542 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M window_size=2us signal_count=100 trigger=TOP.TestDriver.testHarness.chiptop0.axi4_mem_0_bits_ar_valid |
| change_chipyard_clusteredrocketconfig_dhrystone_signals_10_window_32us_trigger_signal | 0.291456 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M window_size=32us signal_count=10 trigger=TOP.TestDriver.testHarness.chiptop0.axi4_mem_0_bits_ar_valid |
| change_chipyard_clusteredrocketconfig_dhrystone_signals_1_window_8us_trigger_posedge_clk | 0.290927 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M window_size=8us signal_count=1 trigger=posedge TOP.TestDriver.clock |
| change_chipyard_clusteredrocketconfig_dhrystone_signals_100_window_2us_trigger_posedge_clk | 0.290812 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M window_size=2us signal_count=100 trigger=posedge TOP.TestDriver.clock |
| change_chipyard_clusteredrocketconfig_dhrystone_signals_100_window_8us_trigger_signal | 0.290325 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M window_size=8us signal_count=100 trigger=TOP.TestDriver.testHarness.chiptop0.axi4_mem_0_bits_ar_valid |
| change_chipyard_clusteredrocketconfig_dhrystone_signals_100_window_32us_trigger_signal | 0.289657 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M window_size=32us signal_count=100 trigger=TOP.TestDriver.testHarness.chiptop0.axi4_mem_0_bits_ar_valid |
| change_chipyard_clusteredrocketconfig_dhrystone_signals_100_window_32us_trigger_posedge_clk | 0.288550 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M window_size=32us signal_count=100 trigger=posedge TOP.TestDriver.clock |
| change_chipyard_dualrocketconfig_dhrystone_signals_100_window_32us_trigger_posedge_clk | 0.249222 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M window_size=32us signal_count=100 trigger=posedge TOP.TestDriver.clock |
| change_chipyard_dualrocketconfig_dhrystone_signals_1_window_2us_trigger_posedge_clk | 0.241975 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M window_size=2us signal_count=1 trigger=posedge TOP.TestDriver.clock |
| change_chipyard_dualrocketconfig_dhrystone_signals_100_window_2us_trigger_signal | 0.241708 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M window_size=2us signal_count=100 trigger=TOP.TestDriver.testHarness.chiptop0.axi4_mem_0_bits_ar_valid |
| change_chipyard_dualrocketconfig_dhrystone_signals_10_window_8us_trigger_signal | 0.241696 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M window_size=8us signal_count=10 trigger=TOP.TestDriver.testHarness.chiptop0.axi4_mem_0_bits_ar_valid |
| change_chipyard_dualrocketconfig_dhrystone_signals_10_window_8us_trigger_posedge_clk | 0.241663 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M window_size=8us signal_count=10 trigger=posedge TOP.TestDriver.clock |
| change_chipyard_dualrocketconfig_dhrystone_signals_10_window_2us_trigger_signal | 0.241640 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M window_size=2us signal_count=10 trigger=TOP.TestDriver.testHarness.chiptop0.axi4_mem_0_bits_ar_valid |
| change_chipyard_dualrocketconfig_dhrystone_signals_100_window_32us_trigger_signal | 0.241116 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M window_size=32us signal_count=100 trigger=TOP.TestDriver.testHarness.chiptop0.axi4_mem_0_bits_ar_valid |
| change_chipyard_dualrocketconfig_dhrystone_signals_100_window_8us_trigger_signal | 0.240923 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M window_size=8us signal_count=100 trigger=TOP.TestDriver.testHarness.chiptop0.axi4_mem_0_bits_ar_valid |
| change_chipyard_clusteredrocketconfig_dhrystone_signals_10_window_2us_trigger_posedge_clk | 0.240320 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M window_size=2us signal_count=10 trigger=posedge TOP.TestDriver.clock |
| change_chipyard_dualrocketconfig_dhrystone_signals_10_window_2us_trigger_posedge_clk | 0.240303 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M window_size=2us signal_count=10 trigger=posedge TOP.TestDriver.clock |
| change_chipyard_clusteredrocketconfig_dhrystone_signals_100_window_2us_trigger_any | 0.239898 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M window_size=2us signal_count=100 trigger=* |
| change_chipyard_dualrocketconfig_dhrystone_signals_1_window_8us_trigger_posedge_clk | 0.239684 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M window_size=8us signal_count=1 trigger=posedge TOP.TestDriver.clock |
| change_chipyard_dualrocketconfig_dhrystone_signals_100_window_2us_trigger_posedge_clk | 0.239630 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M window_size=2us signal_count=100 trigger=posedge TOP.TestDriver.clock |
| change_chipyard_clusteredrocketconfig_dhrystone_signals_10_window_2us_trigger_signal | 0.239491 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M window_size=2us signal_count=10 trigger=TOP.TestDriver.testHarness.chiptop0.axi4_mem_0_bits_ar_valid |
| change_chipyard_clusteredrocketconfig_dhrystone_signals_100_window_32us_trigger_any | 0.239438 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M window_size=32us signal_count=100 trigger=* |
| change_chipyard_clusteredrocketconfig_dhrystone_signals_1_window_2us_trigger_posedge_clk | 0.239429 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M window_size=2us signal_count=1 trigger=posedge TOP.TestDriver.clock |
| change_chipyard_clusteredrocketconfig_dhrystone_signals_100_window_8us_trigger_posedge_clk | 0.239297 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M window_size=8us signal_count=100 trigger=posedge TOP.TestDriver.clock |
| change_chipyard_clusteredrocketconfig_dhrystone_signals_10_window_8us_trigger_posedge_clk | 0.239181 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M window_size=8us signal_count=10 trigger=posedge TOP.TestDriver.clock |
| change_chipyard_clusteredrocketconfig_dhrystone_signals_100_window_8us_trigger_any | 0.239136 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M window_size=8us signal_count=100 trigger=* |
| change_chipyard_dualrocketconfig_dhrystone_signals_10_window_32us_trigger_signal | 0.239009 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M window_size=32us signal_count=10 trigger=TOP.TestDriver.testHarness.chiptop0.axi4_mem_0_bits_ar_valid |
| change_chipyard_clusteredrocketconfig_dhrystone_signals_10_window_8us_trigger_signal | 0.238852 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M window_size=8us signal_count=10 trigger=TOP.TestDriver.testHarness.chiptop0.axi4_mem_0_bits_ar_valid |
| change_chipyard_clusteredrocketconfig_dhrystone_signals_10_window_32us_trigger_posedge_clk | 0.209143 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M window_size=32us signal_count=10 trigger=posedge TOP.TestDriver.clock |
| change_chipyard_dualrocketconfig_dhrystone_signals_10_window_2us_trigger_any | 0.199376 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M window_size=2us signal_count=10 trigger=* |
| change_chipyard_clusteredrocketconfig_dhrystone_signals_10_window_32us_trigger_any | 0.199099 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M window_size=32us signal_count=10 trigger=* |
| change_scr1_signals_100_window_2ns_trigger_posedge_clk | 0.192319 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=3.5M window_size=2ns signal_count=100 trigger=posedge TOP.clk |
| change_chipyard_clusteredrocketconfig_dhrystone_signals_1_window_8us_trigger_signal | 0.191626 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M window_size=8us signal_count=1 trigger=TOP.TestDriver.testHarness.chiptop0.axi4_mem_0_bits_ar_valid |
| change_scr1_signals_100_window_8ns_trigger_posedge_clk | 0.191609 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=3.5M window_size=8ns signal_count=100 trigger=posedge TOP.clk |
| change_scr1_signals_100_window_8ns_trigger_signal | 0.191586 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=3.5M window_size=8ns signal_count=100 trigger=TOP.scr1_top_tb_axi.io_axi_dmem_araddr |
| change_scr1_signals_100_pos_50_window_2ns_trigger_any | 0.191538 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=3.5M window_size=2ns signal_count=100 trigger=* |
| change_scr1_signals_100_window_8ns_trigger_any | 0.191485 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=3.5M window_size=8ns signal_count=100 trigger=* |
| change_scr1_signals_100_window_4ns_trigger_any | 0.191404 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=3.5M window_size=4ns signal_count=100 trigger=* |
| change_chipyard_clusteredrocketconfig_dhrystone_signals_1_window_2us_trigger_signal | 0.191344 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M window_size=2us signal_count=1 trigger=TOP.TestDriver.testHarness.chiptop0.axi4_mem_0_bits_ar_valid |
| change_scr1_signals_100_window_4ns_trigger_signal | 0.191105 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=3.5M window_size=4ns signal_count=100 trigger=TOP.scr1_top_tb_axi.io_axi_dmem_araddr |
| change_chipyard_dualrocketconfig_dhrystone_signals_1_window_32us_trigger_signal | 0.190442 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M window_size=32us signal_count=1 trigger=TOP.TestDriver.testHarness.chiptop0.axi4_mem_0_bits_ar_valid |
| change_chipyard_dualrocketconfig_dhrystone_signals_10_window_32us_trigger_posedge_clk | 0.190401 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M window_size=32us signal_count=10 trigger=posedge TOP.TestDriver.clock |
| change_chipyard_dualrocketconfig_dhrystone_signals_10_window_32us_trigger_any | 0.190204 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M window_size=32us signal_count=10 trigger=* |
| change_chipyard_dualrocketconfig_dhrystone_signals_10_window_8us_trigger_any | 0.190162 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M window_size=8us signal_count=10 trigger=* |
| change_scr1_signals_100_window_2ns_trigger_signal | 0.190093 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=3.5M window_size=2ns signal_count=100 trigger=TOP.scr1_top_tb_axi.io_axi_dmem_araddr |
| change_scr1_signals_100_window_4ns_trigger_posedge_clk | 0.189977 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=3.5M window_size=4ns signal_count=100 trigger=posedge TOP.clk |
| change_chipyard_dualrocketconfig_dhrystone_signals_100_window_8us_trigger_any | 0.189702 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M window_size=8us signal_count=100 trigger=* |
| change_chipyard_clusteredrocketconfig_dhrystone_signals_1_window_32us_trigger_any | 0.189622 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M window_size=32us signal_count=1 trigger=* |
| change_chipyard_dualrocketconfig_dhrystone_signals_100_window_8us_trigger_posedge_clk | 0.189071 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M window_size=8us signal_count=100 trigger=posedge TOP.TestDriver.clock |
| change_chipyard_dualrocketconfig_dhrystone_signals_100_window_32us_trigger_any | 0.188900 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M window_size=32us signal_count=100 trigger=* |
| change_chipyard_dualrocketconfig_dhrystone_signals_100_window_2us_trigger_any | 0.188867 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M window_size=2us signal_count=100 trigger=* |
| change_chipyard_clusteredrocketconfig_dhrystone_signals_10_window_2us_trigger_any | 0.188508 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M window_size=2us signal_count=10 trigger=* |
| change_chipyard_clusteredrocketconfig_dhrystone_signals_10_window_8us_trigger_any | 0.188486 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M window_size=8us signal_count=10 trigger=* |
| change_chipyard_clusteredrocketconfig_dhrystone_signals_1_window_32us_trigger_signal | 0.188092 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M window_size=32us signal_count=1 trigger=TOP.TestDriver.testHarness.chiptop0.axi4_mem_0_bits_ar_valid |
| change_picorv32_signals_10_window_2us_trigger_any | 0.141844 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M window_size=2us signal_count=10 trigger=* |
| change_picorv32_signals_10_window_32us_trigger_any | 0.141455 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M window_size=32us signal_count=10 trigger=* |
| change_picorv32_signals_10_window_2us_trigger_signal | 0.141376 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M window_size=2us signal_count=10 trigger=testbench.top.mem_axi_arvalid |
| change_picorv32_signals_10_window_32us_trigger_posedge_clk | 0.141283 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M window_size=32us signal_count=10 trigger=posedge testbench.clk |
| change_chipyard_clusteredrocketconfig_dhrystone_signals_1_window_2us_trigger_any | 0.141267 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M window_size=2us signal_count=1 trigger=* |
| change_picorv32_signals_10_window_8us_trigger_any | 0.141113 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M window_size=8us signal_count=10 trigger=* |
| change_picorv32_signals_10_window_32us_trigger_signal | 0.140929 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M window_size=32us signal_count=10 trigger=testbench.top.mem_axi_arvalid |
| change_picorv32_signals_10_window_2us_trigger_posedge_clk | 0.140905 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M window_size=2us signal_count=10 trigger=posedge testbench.clk |
| change_picorv32_signals_10_window_8us_trigger_signal | 0.140753 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M window_size=8us signal_count=10 trigger=testbench.top.mem_axi_arvalid |
| change_picorv32_signals_1_window_2us_trigger_posedge_clk | 0.140714 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M window_size=2us signal_count=1 trigger=posedge testbench.clk |
| change_picorv32_signals_1_window_32us_trigger_posedge_clk | 0.140561 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M window_size=32us signal_count=1 trigger=posedge testbench.clk |
| change_picorv32_signals_10_window_8us_trigger_posedge_clk | 0.140515 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M window_size=8us signal_count=10 trigger=posedge testbench.clk |
| change_chipyard_dualrocketconfig_dhrystone_signals_1_window_8us_trigger_any | 0.140340 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M window_size=8us signal_count=1 trigger=* |
| change_picorv32_signals_1_window_8us_trigger_posedge_clk | 0.140267 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M window_size=8us signal_count=1 trigger=posedge testbench.clk |
| change_chipyard_clusteredrocketconfig_dhrystone_signals_1_window_8us_trigger_any | 0.139816 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M window_size=8us signal_count=1 trigger=* |
| change_chipyard_dualrocketconfig_dhrystone_signals_1_window_2us_trigger_signal | 0.139651 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M window_size=2us signal_count=1 trigger=TOP.TestDriver.testHarness.chiptop0.axi4_mem_0_bits_ar_valid |
| change_chipyard_dualrocketconfig_dhrystone_signals_1_window_32us_trigger_any | 0.139520 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M window_size=32us signal_count=1 trigger=* |
| change_chipyard_dualrocketconfig_dhrystone_signals_1_window_2us_trigger_any | 0.139517 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M window_size=2us signal_count=1 trigger=* |
| change_chipyard_dualrocketconfig_dhrystone_signals_1_window_8us_trigger_signal | 0.138498 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M window_size=8us signal_count=1 trigger=TOP.TestDriver.testHarness.chiptop0.axi4_mem_0_bits_ar_valid |
| change_picorv32_signals_1_window_8us_trigger_any | 0.091204 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M window_size=8us signal_count=1 trigger=* |
| change_picorv32_signals_1_window_2us_trigger_any | 0.090250 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M window_size=2us signal_count=1 trigger=* |
| change_picorv32_signals_1_window_32us_trigger_any | 0.089798 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M window_size=32us signal_count=1 trigger=* |
| change_picorv32_signals_1_window_32us_trigger_signal | 0.089459 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M window_size=32us signal_count=1 trigger=testbench.top.mem_axi_arvalid |
| change_picorv32_signals_1_window_2us_trigger_signal | 0.089267 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M window_size=2us signal_count=1 trigger=testbench.top.mem_axi_arvalid |
| change_picorv32_signals_1_window_8us_trigger_signal | 0.088898 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M window_size=8us signal_count=1 trigger=testbench.top.mem_axi_arvalid |
| change_scr1_coremark_imem_axi_2sig_to_1000ps | 0.079189 | waves=/opt/rtl-artifacts/scr1_max_axi_coremark.fst scope=TOP.scr1_top_tb_axi.i_top.i_imem_axi signal_count=2 window_to=1000ps |
| change_scr1_coremark_imem_axi_1sig_to_1000ps | 0.074515 | waves=/opt/rtl-artifacts/scr1_max_axi_coremark.fst scope=TOP.scr1_top_tb_axi.i_top.i_imem_axi signal_count=1 window_to=1000ps |
| change_scr1_signals_10_window_8ns_trigger_signal | 0.060094 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=3.5M window_size=8ns signal_count=10 trigger=TOP.scr1_top_tb_axi.io_axi_dmem_araddr |
| change_scr1_signals_1_window_2ns_trigger_any | 0.057584 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=3.5M window_size=2ns signal_count=1 trigger=* |
| change_scr1_signals_1_window_8ns_trigger_any | 0.056991 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=3.5M window_size=8ns signal_count=1 trigger=* |
| change_scr1_signals_1_window_4ns_trigger_posedge_clk | 0.056811 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=3.5M window_size=4ns signal_count=1 trigger=posedge TOP.clk |
| change_scr1_signals_1_window_4ns_trigger_signal | 0.056520 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=3.5M window_size=4ns signal_count=1 trigger=TOP.scr1_top_tb_axi.io_axi_dmem_araddr |
| change_scr1_signals_1_window_4ns_trigger_any | 0.056513 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=3.5M window_size=4ns signal_count=1 trigger=* |
| change_scr1_signals_1_window_2ns_trigger_signal | 0.056470 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=3.5M window_size=2ns signal_count=1 trigger=TOP.scr1_top_tb_axi.io_axi_dmem_araddr |
| change_scr1_signals_1_window_8ns_trigger_signal | 0.056416 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=3.5M window_size=8ns signal_count=1 trigger=TOP.scr1_top_tb_axi.io_axi_dmem_araddr |
| change_scr1_signals_1_window_2ns_trigger_posedge_clk | 0.056408 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=3.5M window_size=2ns signal_count=1 trigger=posedge TOP.clk |
| change_scr1_signals_10_window_4ns_trigger_posedge_clk | 0.056398 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=3.5M window_size=4ns signal_count=10 trigger=posedge TOP.clk |
| change_scr1_signals_10_window_4ns_trigger_signal | 0.056387 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=3.5M window_size=4ns signal_count=10 trigger=TOP.scr1_top_tb_axi.io_axi_dmem_araddr |
| change_scr1_signals_1_window_8ns_trigger_posedge_clk | 0.056383 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=3.5M window_size=8ns signal_count=1 trigger=posedge TOP.clk |
| change_scr1_signals_10_window_2ns_trigger_any | 0.056237 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=3.5M window_size=2ns signal_count=10 trigger=* |
| change_scr1_signals_10_window_2ns_trigger_posedge_clk | 0.056183 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=3.5M window_size=2ns signal_count=10 trigger=posedge TOP.clk |
| change_scr1_signals_10_window_8ns_trigger_any | 0.056126 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=3.5M window_size=8ns signal_count=10 trigger=* |
| change_scr1_signals_10_window_8ns_trigger_posedge_clk | 0.055989 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=3.5M window_size=8ns signal_count=10 trigger=posedge TOP.clk |
| change_scr1_signals_10_window_2ns_trigger_signal | 0.055919 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=3.5M window_size=2ns signal_count=10 trigger=TOP.scr1_top_tb_axi.io_axi_dmem_araddr |
| change_scr1_signals_10_window_4ns_trigger_any | 0.055917 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=3.5M window_size=4ns signal_count=10 trigger=* |

## info

| test | mean_s | meta |
| --- | --- | --- |
| info_chipyard_clustered_mt_memcpy | 0.190553 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_mt-memcpy.fst size=411M |
| info_chipyard_dualrocket_dhrystone | 0.089622 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M |
| info_chipyard_clustered_dhrystone | 0.088547 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M |
| info_picorv32 | 0.043933 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M |
| info_scr1 | 0.043777 | waves=/opt/rtl-artifacts/scr1_max_axi_coremark.fst size=21M |
| info_scr1_riscv_compliance | 0.032128 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=4M |
| info_scr1_isr_sample | 0.027293 | waves=/opt/rtl-artifacts/scr1_max_axi_isr_sample.fst size=69K |
| info_picorv32_ez | 0.026992 | waves=/opt/rtl-artifacts/picorv32_test_ez_vcd.fst size=17K |

## signal

| test | mean_s | meta |
| --- | --- | --- |
| signal_scr1_top_recursive_all_json | 0.038757 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst scope=TOP filter=.* recursive=True max_depth=default |
| signal_scr1_top_recursive_filter_valid_json | 0.038684 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst scope=TOP filter=(?i).*valid.* recursive=True max_depth=default |
| signal_scr1_top_recursive_depth2_json | 0.038028 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst scope=TOP filter=.* recursive=True max_depth=2 |

## value

| test | mean_s | meta |
| --- | --- | --- |
| value_picorv32_signals_1000 | 2.043653 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M signal_count=1000 |
| value_chipyard_clusteredrocketconfig_dhrystone_signals_1000 | 0.900581 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M signal_count=1000 |
| value_chipyard_dualrocketconfig_dhrystone_signals_1000 | 0.894141 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M signal_count=1000 |
| value_chipyard_clusteredrocketconfig_dhrystone_signals_100 | 0.536545 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M signal_count=100 |
| value_chipyard_dualrocketconfig_dhrystone_signals_100 | 0.489581 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M signal_count=100 |
| value_picorv32_signals_100 | 0.441733 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M signal_count=100 |
| value_scr1_signals_1000 | 0.392405 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=4M signal_count=1000 |
| value_chipyard_clusteredrocketconfig_dhrystone_signals_1 | 0.204215 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M signal_count=1 |
| value_chipyard_clusteredrocketconfig_dhrystone_signals_10 | 0.198819 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M signal_count=10 |
| value_picorv32_signals_10 | 0.190148 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M signal_count=10 |
| value_chipyard_dualrocketconfig_dhrystone_signals_1 | 0.190023 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M signal_count=1 |
| value_chipyard_dualrocketconfig_dhrystone_signals_10 | 0.189573 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M signal_count=10 |
| value_picorv32_signals_1 | 0.090258 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M signal_count=1 |
| value_scr1_signals_100 | 0.058020 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=4M signal_count=100 |
| value_scr1_signals_1 | 0.057339 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=4M signal_count=1 |
| value_scr1_signals_10 | 0.057269 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=4M signal_count=10 |
