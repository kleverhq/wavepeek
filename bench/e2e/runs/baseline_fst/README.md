# CLI E2E Bench Run: baseline_fst

- Generated at (UTC): 2026-05-30T19:27:30Z
- Hyperfine JSON files: 142
- Wavepeek JSON files: 142

## change

| test | mean_s | meta |
| --- | --- | --- |
| change_picorv32_signals_100_window_8us_trigger_signal | 0.442932 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M window_size=8us signal_count=100 trigger=testbench.top.mem_axi_arvalid |
| change_picorv32_signals_100_window_32us_trigger_signal | 0.441964 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M window_size=32us signal_count=100 trigger=testbench.top.mem_axi_arvalid |
| change_picorv32_signals_100_window_2us_trigger_posedge_clk | 0.441713 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M window_size=2us signal_count=100 trigger=posedge testbench.clk |
| change_picorv32_signals_100_window_32us_trigger_posedge_clk | 0.441423 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M window_size=32us signal_count=100 trigger=posedge testbench.clk |
| change_picorv32_signals_100_window_2us_trigger_any | 0.441288 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M window_size=2us signal_count=100 trigger=* |
| change_picorv32_signals_100_window_8us_trigger_posedge_clk | 0.441283 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M window_size=8us signal_count=100 trigger=posedge testbench.clk |
| change_picorv32_signals_100_window_32us_trigger_any | 0.441063 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M window_size=32us signal_count=100 trigger=* |
| change_picorv32_signals_100_window_2us_trigger_signal | 0.440865 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M window_size=2us signal_count=100 trigger=testbench.top.mem_axi_arvalid |
| change_picorv32_signals_100_window_8us_trigger_any | 0.440542 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M window_size=8us signal_count=100 trigger=* |
| change_chipyard_dualrocketconfig_dhrystone_signals_1_window_32us_trigger_posedge_clk | 0.390433 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M window_size=32us signal_count=1 trigger=posedge TOP.TestDriver.clock |
| change_chipyard_clusteredrocketconfig_dhrystone_signals_1_window_32us_trigger_posedge_clk | 0.390407 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M window_size=32us signal_count=1 trigger=posedge TOP.TestDriver.clock |
| change_chipyard_clusteredrocketconfig_dhrystone_signals_100_window_32us_trigger_posedge_clk | 0.342171 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M window_size=32us signal_count=100 trigger=posedge TOP.TestDriver.clock |
| change_chipyard_clusteredrocketconfig_dhrystone_signals_100_window_32us_trigger_signal | 0.302388 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M window_size=32us signal_count=100 trigger=TOP.TestDriver.testHarness.chiptop0.axi4_mem_0_bits_ar_valid |
| change_chipyard_clusteredrocketconfig_dhrystone_signals_100_window_32us_trigger_any | 0.290875 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M window_size=32us signal_count=100 trigger=* |
| change_chipyard_clusteredrocketconfig_dhrystone_signals_100_window_8us_trigger_signal | 0.290566 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M window_size=8us signal_count=100 trigger=TOP.TestDriver.testHarness.chiptop0.axi4_mem_0_bits_ar_valid |
| change_chipyard_clusteredrocketconfig_dhrystone_signals_100_window_2us_trigger_posedge_clk | 0.290320 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M window_size=2us signal_count=100 trigger=posedge TOP.TestDriver.clock |
| change_chipyard_clusteredrocketconfig_dhrystone_signals_100_window_2us_trigger_signal | 0.290044 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M window_size=2us signal_count=100 trigger=TOP.TestDriver.testHarness.chiptop0.axi4_mem_0_bits_ar_valid |
| change_chipyard_clusteredrocketconfig_dhrystone_signals_10_window_32us_trigger_signal | 0.289624 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M window_size=32us signal_count=10 trigger=TOP.TestDriver.testHarness.chiptop0.axi4_mem_0_bits_ar_valid |
| change_chipyard_clusteredrocketconfig_dhrystone_signals_10_window_8us_trigger_signal | 0.261913 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M window_size=8us signal_count=10 trigger=TOP.TestDriver.testHarness.chiptop0.axi4_mem_0_bits_ar_valid |
| change_chipyard_clusteredrocketconfig_dhrystone_signals_1_window_8us_trigger_posedge_clk | 0.260702 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M window_size=8us signal_count=1 trigger=posedge TOP.TestDriver.clock |
| change_chipyard_clusteredrocketconfig_dhrystone_signals_10_window_2us_trigger_posedge_clk | 0.251796 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M window_size=2us signal_count=10 trigger=posedge TOP.TestDriver.clock |
| change_chipyard_clusteredrocketconfig_dhrystone_signals_1_window_2us_trigger_posedge_clk | 0.250649 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M window_size=2us signal_count=1 trigger=posedge TOP.TestDriver.clock |
| change_chipyard_dualrocketconfig_dhrystone_signals_10_window_32us_trigger_signal | 0.250173 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M window_size=32us signal_count=10 trigger=TOP.TestDriver.testHarness.chiptop0.axi4_mem_0_bits_ar_valid |
| change_chipyard_dualrocketconfig_dhrystone_signals_100_window_2us_trigger_signal | 0.243205 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M window_size=2us signal_count=100 trigger=TOP.TestDriver.testHarness.chiptop0.axi4_mem_0_bits_ar_valid |
| change_chipyard_dualrocketconfig_dhrystone_signals_100_window_32us_trigger_signal | 0.242506 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M window_size=32us signal_count=100 trigger=TOP.TestDriver.testHarness.chiptop0.axi4_mem_0_bits_ar_valid |
| change_chipyard_clusteredrocketconfig_dhrystone_signals_100_window_8us_trigger_posedge_clk | 0.242055 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M window_size=8us signal_count=100 trigger=posedge TOP.TestDriver.clock |
| change_chipyard_dualrocketconfig_dhrystone_signals_10_window_2us_trigger_posedge_clk | 0.241765 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M window_size=2us signal_count=10 trigger=posedge TOP.TestDriver.clock |
| change_chipyard_clusteredrocketconfig_dhrystone_signals_10_window_2us_trigger_any | 0.241367 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M window_size=2us signal_count=10 trigger=* |
| change_chipyard_clusteredrocketconfig_dhrystone_signals_100_window_2us_trigger_any | 0.241309 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M window_size=2us signal_count=100 trigger=* |
| change_chipyard_dualrocketconfig_dhrystone_signals_10_window_8us_trigger_signal | 0.241165 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M window_size=8us signal_count=10 trigger=TOP.TestDriver.testHarness.chiptop0.axi4_mem_0_bits_ar_valid |
| change_chipyard_clusteredrocketconfig_dhrystone_signals_10_window_8us_trigger_any | 0.241149 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M window_size=8us signal_count=10 trigger=* |
| change_chipyard_clusteredrocketconfig_dhrystone_signals_100_window_8us_trigger_any | 0.241021 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M window_size=8us signal_count=100 trigger=* |
| change_chipyard_clusteredrocketconfig_dhrystone_signals_10_window_2us_trigger_signal | 0.240811 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M window_size=2us signal_count=10 trigger=TOP.TestDriver.testHarness.chiptop0.axi4_mem_0_bits_ar_valid |
| change_chipyard_dualrocketconfig_dhrystone_signals_100_window_2us_trigger_posedge_clk | 0.240808 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M window_size=2us signal_count=100 trigger=posedge TOP.TestDriver.clock |
| change_chipyard_dualrocketconfig_dhrystone_signals_100_window_32us_trigger_posedge_clk | 0.240563 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M window_size=32us signal_count=100 trigger=posedge TOP.TestDriver.clock |
| change_chipyard_clusteredrocketconfig_dhrystone_signals_10_window_8us_trigger_posedge_clk | 0.240545 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M window_size=8us signal_count=10 trigger=posedge TOP.TestDriver.clock |
| change_chipyard_dualrocketconfig_dhrystone_signals_10_window_2us_trigger_signal | 0.240531 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M window_size=2us signal_count=10 trigger=TOP.TestDriver.testHarness.chiptop0.axi4_mem_0_bits_ar_valid |
| change_chipyard_dualrocketconfig_dhrystone_signals_1_window_8us_trigger_posedge_clk | 0.240478 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M window_size=8us signal_count=1 trigger=posedge TOP.TestDriver.clock |
| change_chipyard_dualrocketconfig_dhrystone_signals_100_window_8us_trigger_signal | 0.239544 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M window_size=8us signal_count=100 trigger=TOP.TestDriver.testHarness.chiptop0.axi4_mem_0_bits_ar_valid |
| change_chipyard_clusteredrocketconfig_dhrystone_signals_1_window_32us_trigger_signal | 0.239417 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M window_size=32us signal_count=1 trigger=TOP.TestDriver.testHarness.chiptop0.axi4_mem_0_bits_ar_valid |
| change_chipyard_dualrocketconfig_dhrystone_signals_1_window_2us_trigger_posedge_clk | 0.239388 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M window_size=2us signal_count=1 trigger=posedge TOP.TestDriver.clock |
| change_chipyard_dualrocketconfig_dhrystone_signals_10_window_8us_trigger_posedge_clk | 0.238844 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M window_size=8us signal_count=10 trigger=posedge TOP.TestDriver.clock |
| change_chipyard_clusteredrocketconfig_dhrystone_signals_10_window_32us_trigger_posedge_clk | 0.230414 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M window_size=32us signal_count=10 trigger=posedge TOP.TestDriver.clock |
| change_chipyard_clusteredrocketconfig_dhrystone_signals_10_window_32us_trigger_any | 0.229406 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M window_size=32us signal_count=10 trigger=* |
| change_chipyard_dualrocketconfig_dhrystone_signals_10_window_32us_trigger_posedge_clk | 0.200471 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M window_size=32us signal_count=10 trigger=posedge TOP.TestDriver.clock |
| change_chipyard_dualrocketconfig_dhrystone_signals_10_window_8us_trigger_any | 0.199549 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M window_size=8us signal_count=10 trigger=* |
| change_chipyard_dualrocketconfig_dhrystone_signals_100_window_8us_trigger_any | 0.199249 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M window_size=8us signal_count=100 trigger=* |
| change_chipyard_clusteredrocketconfig_dhrystone_signals_1_window_8us_trigger_signal | 0.192190 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M window_size=8us signal_count=1 trigger=TOP.TestDriver.testHarness.chiptop0.axi4_mem_0_bits_ar_valid |
| change_scr1_signals_100_window_4ns_trigger_any | 0.191774 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=3.5M window_size=4ns signal_count=100 trigger=* |
| change_scr1_signals_100_pos_50_window_2ns_trigger_any | 0.191603 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=3.5M window_size=2ns signal_count=100 trigger=* |
| change_scr1_signals_100_window_8ns_trigger_posedge_clk | 0.191445 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=3.5M window_size=8ns signal_count=100 trigger=posedge TOP.clk |
| change_chipyard_clusteredrocketconfig_dhrystone_signals_1_window_32us_trigger_any | 0.191443 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M window_size=32us signal_count=1 trigger=* |
| change_scr1_signals_100_window_2ns_trigger_posedge_clk | 0.191397 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=3.5M window_size=2ns signal_count=100 trigger=posedge TOP.clk |
| change_scr1_signals_100_window_4ns_trigger_posedge_clk | 0.191295 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=3.5M window_size=4ns signal_count=100 trigger=posedge TOP.clk |
| change_chipyard_dualrocketconfig_dhrystone_signals_1_window_32us_trigger_signal | 0.191072 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M window_size=32us signal_count=1 trigger=TOP.TestDriver.testHarness.chiptop0.axi4_mem_0_bits_ar_valid |
| change_scr1_signals_100_window_8ns_trigger_signal | 0.190725 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=3.5M window_size=8ns signal_count=100 trigger=TOP.scr1_top_tb_axi.io_axi_dmem_araddr |
| change_scr1_signals_100_window_8ns_trigger_any | 0.190699 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=3.5M window_size=8ns signal_count=100 trigger=* |
| change_chipyard_dualrocketconfig_dhrystone_signals_100_window_32us_trigger_any | 0.190378 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M window_size=32us signal_count=100 trigger=* |
| change_chipyard_dualrocketconfig_dhrystone_signals_10_window_2us_trigger_any | 0.190333 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M window_size=2us signal_count=10 trigger=* |
| change_scr1_signals_100_window_2ns_trigger_signal | 0.190119 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=3.5M window_size=2ns signal_count=100 trigger=TOP.scr1_top_tb_axi.io_axi_dmem_araddr |
| change_chipyard_clusteredrocketconfig_dhrystone_signals_1_window_2us_trigger_signal | 0.190070 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M window_size=2us signal_count=1 trigger=TOP.TestDriver.testHarness.chiptop0.axi4_mem_0_bits_ar_valid |
| change_chipyard_dualrocketconfig_dhrystone_signals_100_window_2us_trigger_any | 0.189944 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M window_size=2us signal_count=100 trigger=* |
| change_scr1_signals_100_window_4ns_trigger_signal | 0.189748 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=3.5M window_size=4ns signal_count=100 trigger=TOP.scr1_top_tb_axi.io_axi_dmem_araddr |
| change_chipyard_dualrocketconfig_dhrystone_signals_10_window_32us_trigger_any | 0.189553 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M window_size=32us signal_count=10 trigger=* |
| change_chipyard_dualrocketconfig_dhrystone_signals_100_window_8us_trigger_posedge_clk | 0.188538 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M window_size=8us signal_count=100 trigger=posedge TOP.TestDriver.clock |
| change_chipyard_dualrocketconfig_dhrystone_signals_1_window_2us_trigger_signal | 0.160309 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M window_size=2us signal_count=1 trigger=TOP.TestDriver.testHarness.chiptop0.axi4_mem_0_bits_ar_valid |
| change_chipyard_dualrocketconfig_dhrystone_signals_1_window_8us_trigger_signal | 0.152071 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M window_size=8us signal_count=1 trigger=TOP.TestDriver.testHarness.chiptop0.axi4_mem_0_bits_ar_valid |
| change_picorv32_signals_10_window_2us_trigger_posedge_clk | 0.151093 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M window_size=2us signal_count=10 trigger=posedge testbench.clk |
| change_chipyard_dualrocketconfig_dhrystone_signals_1_window_32us_trigger_any | 0.150584 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M window_size=32us signal_count=1 trigger=* |
| change_picorv32_signals_1_window_8us_trigger_posedge_clk | 0.142203 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M window_size=8us signal_count=1 trigger=posedge testbench.clk |
| change_chipyard_clusteredrocketconfig_dhrystone_signals_1_window_8us_trigger_any | 0.141457 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M window_size=8us signal_count=1 trigger=* |
| change_chipyard_dualrocketconfig_dhrystone_signals_1_window_2us_trigger_any | 0.141451 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M window_size=2us signal_count=1 trigger=* |
| change_picorv32_signals_10_window_2us_trigger_any | 0.141200 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M window_size=2us signal_count=10 trigger=* |
| change_picorv32_signals_10_window_2us_trigger_signal | 0.140980 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M window_size=2us signal_count=10 trigger=testbench.top.mem_axi_arvalid |
| change_picorv32_signals_1_window_2us_trigger_posedge_clk | 0.140779 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M window_size=2us signal_count=1 trigger=posedge testbench.clk |
| change_chipyard_dualrocketconfig_dhrystone_signals_1_window_8us_trigger_any | 0.140724 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M window_size=8us signal_count=1 trigger=* |
| change_picorv32_signals_1_window_32us_trigger_posedge_clk | 0.140642 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M window_size=32us signal_count=1 trigger=posedge testbench.clk |
| change_picorv32_signals_10_window_32us_trigger_signal | 0.140550 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M window_size=32us signal_count=10 trigger=testbench.top.mem_axi_arvalid |
| change_picorv32_signals_10_window_8us_trigger_any | 0.140209 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M window_size=8us signal_count=10 trigger=* |
| change_picorv32_signals_10_window_8us_trigger_posedge_clk | 0.140171 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M window_size=8us signal_count=10 trigger=posedge testbench.clk |
| change_chipyard_clusteredrocketconfig_dhrystone_signals_1_window_2us_trigger_any | 0.140045 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M window_size=2us signal_count=1 trigger=* |
| change_picorv32_signals_10_window_8us_trigger_signal | 0.139958 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M window_size=8us signal_count=10 trigger=testbench.top.mem_axi_arvalid |
| change_picorv32_signals_10_window_32us_trigger_any | 0.139822 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M window_size=32us signal_count=10 trigger=* |
| change_picorv32_signals_10_window_32us_trigger_posedge_clk | 0.139299 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M window_size=32us signal_count=10 trigger=posedge testbench.clk |
| change_picorv32_signals_1_window_32us_trigger_any | 0.090805 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M window_size=32us signal_count=1 trigger=* |
| change_picorv32_signals_1_window_8us_trigger_signal | 0.090677 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M window_size=8us signal_count=1 trigger=testbench.top.mem_axi_arvalid |
| change_scr1_coremark_imem_axi_2sig_to_1000ps | 0.090393 | waves=/opt/rtl-artifacts/scr1_max_axi_coremark.fst scope=TOP.scr1_top_tb_axi.i_top.i_imem_axi signal_count=2 window_to=1000ps |
| change_picorv32_signals_1_window_8us_trigger_any | 0.090387 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M window_size=8us signal_count=1 trigger=* |
| change_picorv32_signals_1_window_32us_trigger_signal | 0.090201 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M window_size=32us signal_count=1 trigger=testbench.top.mem_axi_arvalid |
| change_picorv32_signals_1_window_2us_trigger_any | 0.090176 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M window_size=2us signal_count=1 trigger=* |
| change_scr1_coremark_imem_axi_1sig_to_1000ps | 0.089911 | waves=/opt/rtl-artifacts/scr1_max_axi_coremark.fst scope=TOP.scr1_top_tb_axi.i_top.i_imem_axi signal_count=1 window_to=1000ps |
| change_picorv32_signals_1_window_2us_trigger_signal | 0.089387 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M window_size=2us signal_count=1 trigger=testbench.top.mem_axi_arvalid |
| change_scr1_signals_1_window_8ns_trigger_signal | 0.063761 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=3.5M window_size=8ns signal_count=1 trigger=TOP.scr1_top_tb_axi.io_axi_dmem_araddr |
| change_scr1_signals_10_window_4ns_trigger_signal | 0.061908 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=3.5M window_size=4ns signal_count=10 trigger=TOP.scr1_top_tb_axi.io_axi_dmem_araddr |
| change_scr1_signals_1_window_8ns_trigger_any | 0.058983 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=3.5M window_size=8ns signal_count=1 trigger=* |
| change_scr1_signals_10_window_2ns_trigger_posedge_clk | 0.058509 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=3.5M window_size=2ns signal_count=10 trigger=posedge TOP.clk |
| change_scr1_signals_1_window_2ns_trigger_posedge_clk | 0.058395 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=3.5M window_size=2ns signal_count=1 trigger=posedge TOP.clk |
| change_scr1_signals_10_window_8ns_trigger_signal | 0.058248 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=3.5M window_size=8ns signal_count=10 trigger=TOP.scr1_top_tb_axi.io_axi_dmem_araddr |
| change_scr1_signals_10_window_4ns_trigger_any | 0.058219 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=3.5M window_size=4ns signal_count=10 trigger=* |
| change_scr1_signals_1_window_2ns_trigger_any | 0.058033 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=3.5M window_size=2ns signal_count=1 trigger=* |
| change_scr1_signals_10_window_2ns_trigger_any | 0.057909 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=3.5M window_size=2ns signal_count=10 trigger=* |
| change_scr1_signals_1_window_8ns_trigger_posedge_clk | 0.057776 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=3.5M window_size=8ns signal_count=1 trigger=posedge TOP.clk |
| change_scr1_signals_1_window_2ns_trigger_signal | 0.057743 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=3.5M window_size=2ns signal_count=1 trigger=TOP.scr1_top_tb_axi.io_axi_dmem_araddr |
| change_scr1_signals_1_window_4ns_trigger_any | 0.057660 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=3.5M window_size=4ns signal_count=1 trigger=* |
| change_scr1_signals_10_window_8ns_trigger_posedge_clk | 0.057578 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=3.5M window_size=8ns signal_count=10 trigger=posedge TOP.clk |
| change_scr1_signals_10_window_8ns_trigger_any | 0.057515 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=3.5M window_size=8ns signal_count=10 trigger=* |
| change_scr1_signals_1_window_4ns_trigger_posedge_clk | 0.057450 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=3.5M window_size=4ns signal_count=1 trigger=posedge TOP.clk |
| change_scr1_signals_1_window_4ns_trigger_signal | 0.057347 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=3.5M window_size=4ns signal_count=1 trigger=TOP.scr1_top_tb_axi.io_axi_dmem_araddr |
| change_scr1_signals_10_window_2ns_trigger_signal | 0.057236 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=3.5M window_size=2ns signal_count=10 trigger=TOP.scr1_top_tb_axi.io_axi_dmem_araddr |
| change_scr1_signals_10_window_4ns_trigger_posedge_clk | 0.057214 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=3.5M window_size=4ns signal_count=10 trigger=posedge TOP.clk |

## info

| test | mean_s | meta |
| --- | --- | --- |
| info_chipyard_clustered_mt_memcpy | 0.189722 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_mt-memcpy.fst size=411M |
| info_chipyard_clustered_dhrystone | 0.089672 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M |
| info_chipyard_dualrocket_dhrystone | 0.089616 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M |
| info_scr1 | 0.048253 | waves=/opt/rtl-artifacts/scr1_max_axi_coremark.fst size=21M |
| info_picorv32 | 0.046456 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M |
| info_scr1_riscv_compliance | 0.035040 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=4M |
| info_picorv32_ez | 0.029265 | waves=/opt/rtl-artifacts/picorv32_test_ez_vcd.fst size=17K |
| info_scr1_isr_sample | 0.029206 | waves=/opt/rtl-artifacts/scr1_max_axi_isr_sample.fst size=69K |

## property

| test | mean_s | meta |
| --- | --- | --- |
| property_chipyard_clusteredrocketconfig_dhrystone_window_2us_match_posedge_clk | 0.250291 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M window_size=2us capture=match trigger=posedge TOP.TestDriver.clock eval=TOP.TestDriver.testHarness.io_success == 1'b1 |
| property_chipyard_clusteredrocketconfig_dhrystone_window_2us_switch_wildcard | 0.140222 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M window_size=2us capture=switch trigger=* eval=TOP.TestDriver.testHarness.chiptop0.axi4_mem_0_bits_ar_valid == 1'b1 |

## scope

| test | mean_s | meta |
| --- | --- | --- |
| scope_clustered_all_depth13_json | 0.189426 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_mt-memcpy.fst size=411M filter=.* max_depth=13 scope_count=4625 |
| scope_dualrocket_filter_frontend_depth12_json | 0.089881 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M filter=.*frontend.* max_depth=12 scope_count=118 |
| scope_scr1_all_depth7_json | 0.036133 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=4M filter=.* max_depth=7 scope_count=136 |

## signal

| test | mean_s | meta |
| --- | --- | --- |
| signal_scr1_top_recursive_filter_valid_json | 0.042099 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst scope=TOP filter=(?i).*valid.* recursive=True max_depth=default |
| signal_scr1_top_recursive_all_json | 0.041601 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst scope=TOP filter=.* recursive=True max_depth=default |
| signal_scr1_top_recursive_depth2_json | 0.040130 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst scope=TOP filter=.* recursive=True max_depth=2 |

## value

| test | mean_s | meta |
| --- | --- | --- |
| value_picorv32_signals_1000 | 2.171648 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M signal_count=1000 |
| value_chipyard_clusteredrocketconfig_dhrystone_signals_1000 | 0.943722 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M signal_count=1000 |
| value_chipyard_dualrocketconfig_dhrystone_signals_1000 | 0.943274 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M signal_count=1000 |
| value_scr1_signals_1000 | 0.650508 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=4M signal_count=1000 |
| value_chipyard_dualrocketconfig_dhrystone_signals_100 | 0.543092 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M signal_count=100 |
| value_chipyard_clusteredrocketconfig_dhrystone_signals_100 | 0.540210 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M signal_count=100 |
| value_picorv32_signals_100 | 0.446576 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M signal_count=100 |
| value_chipyard_clusteredrocketconfig_dhrystone_signals_1 | 0.238395 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M signal_count=1 |
| value_chipyard_clusteredrocketconfig_dhrystone_signals_10 | 0.230457 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M signal_count=10 |
| value_chipyard_dualrocketconfig_dhrystone_signals_1 | 0.190644 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M signal_count=1 |
| value_picorv32_signals_10 | 0.190540 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M signal_count=10 |
| value_chipyard_dualrocketconfig_dhrystone_signals_10 | 0.189524 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M signal_count=10 |
| value_scr1_signals_100 | 0.141133 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=4M signal_count=100 |
| value_picorv32_signals_1 | 0.090150 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M signal_count=1 |
| value_scr1_signals_1 | 0.058151 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=4M signal_count=1 |
| value_scr1_signals_10 | 0.057573 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=4M signal_count=10 |
