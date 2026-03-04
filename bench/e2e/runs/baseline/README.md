# CLI E2E Bench Run: baseline

- Generated at (UTC): 2026-03-04T10:14:21Z
- Run directory: `bench/e2e/runs/baseline`
- Hyperfine JSON files: 137
- Wavepeek JSON files: 137

## at

| test | mean_s | meta |
| --- | --- | --- |
| at_picorv32_signals_1000 | 2.111695 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M signal_count=1000 |
| at_chipyard_clusteredrocketconfig_dhrystone_signals_1000 | 0.943218 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M signal_count=1000 |
| at_chipyard_dualrocketconfig_dhrystone_signals_1000 | 0.909162 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M signal_count=1000 |
| at_chipyard_clusteredrocketconfig_dhrystone_signals_100 | 0.541489 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M signal_count=100 |
| at_chipyard_dualrocketconfig_dhrystone_signals_100 | 0.537441 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M signal_count=100 |
| at_picorv32_signals_100 | 0.442084 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M signal_count=100 |
| at_scr1_signals_1000 | 0.391028 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=4M signal_count=1000 |
| at_chipyard_clusteredrocketconfig_dhrystone_signals_1 | 0.236177 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M signal_count=1 |
| at_chipyard_clusteredrocketconfig_dhrystone_signals_10 | 0.234630 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M signal_count=10 |
| at_chipyard_dualrocketconfig_dhrystone_signals_1 | 0.192960 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M signal_count=1 |
| at_picorv32_signals_10 | 0.190834 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M signal_count=10 |
| at_chipyard_dualrocketconfig_dhrystone_signals_10 | 0.189862 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M signal_count=10 |
| at_picorv32_signals_1 | 0.090509 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M signal_count=1 |
| at_scr1_signals_1 | 0.057242 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=4M signal_count=1 |
| at_scr1_signals_10 | 0.057124 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=4M signal_count=10 |
| at_scr1_signals_100 | 0.057084 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=4M signal_count=100 |

## change

| test | mean_s | meta |
| --- | --- | --- |
| change_chipyard_dualrocketconfig_dhrystone_signals_1_window_32us_trigger_posedge_clk | 0.591289 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M window_size=32us signal_count=1 trigger=posedge TOP.TestDriver.clock |
| change_chipyard_clusteredrocketconfig_dhrystone_signals_1_window_32us_trigger_posedge_clk | 0.531244 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M window_size=32us signal_count=1 trigger=posedge TOP.TestDriver.clock |
| change_picorv32_signals_100_window_32us_trigger_signal | 0.442625 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M window_size=32us signal_count=100 trigger=testbench.top.mem_axi_arvalid |
| change_picorv32_signals_100_window_2us_trigger_signal | 0.442112 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M window_size=2us signal_count=100 trigger=testbench.top.mem_axi_arvalid |
| change_picorv32_signals_100_window_8us_trigger_signal | 0.440949 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M window_size=8us signal_count=100 trigger=testbench.top.mem_axi_arvalid |
| change_picorv32_signals_100_window_32us_trigger_posedge_clk | 0.420937 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M window_size=32us signal_count=100 trigger=posedge testbench.clk |
| change_picorv32_signals_100_window_2us_trigger_posedge_clk | 0.411228 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M window_size=2us signal_count=100 trigger=posedge testbench.clk |
| change_picorv32_signals_100_window_8us_trigger_posedge_clk | 0.410178 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M window_size=8us signal_count=100 trigger=posedge testbench.clk |
| change_picorv32_signals_100_window_32us_trigger_any | 0.389907 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M window_size=32us signal_count=100 trigger=* |
| change_picorv32_signals_100_window_8us_trigger_any | 0.389881 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M window_size=8us signal_count=100 trigger=* |
| change_picorv32_signals_100_window_2us_trigger_any | 0.388895 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M window_size=2us signal_count=100 trigger=* |
| change_chipyard_clusteredrocketconfig_dhrystone_signals_100_window_32us_trigger_posedge_clk | 0.330920 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M window_size=32us signal_count=100 trigger=posedge TOP.TestDriver.clock |
| change_chipyard_clusteredrocketconfig_dhrystone_signals_10_window_32us_trigger_signal | 0.291834 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M window_size=32us signal_count=10 trigger=TOP.TestDriver.testHarness.chiptop0.axi4_mem_0_bits_ar_valid |
| change_chipyard_clusteredrocketconfig_dhrystone_signals_100_window_2us_trigger_signal | 0.291741 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M window_size=2us signal_count=100 trigger=TOP.TestDriver.testHarness.chiptop0.axi4_mem_0_bits_ar_valid |
| change_chipyard_clusteredrocketconfig_dhrystone_signals_1_window_8us_trigger_posedge_clk | 0.291410 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M window_size=8us signal_count=1 trigger=posedge TOP.TestDriver.clock |
| change_chipyard_clusteredrocketconfig_dhrystone_signals_100_window_8us_trigger_signal | 0.290809 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M window_size=8us signal_count=100 trigger=TOP.TestDriver.testHarness.chiptop0.axi4_mem_0_bits_ar_valid |
| change_chipyard_clusteredrocketconfig_dhrystone_signals_100_window_32us_trigger_signal | 0.290497 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M window_size=32us signal_count=100 trigger=TOP.TestDriver.testHarness.chiptop0.axi4_mem_0_bits_ar_valid |
| change_chipyard_clusteredrocketconfig_dhrystone_signals_100_window_2us_trigger_posedge_clk | 0.290457 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M window_size=2us signal_count=100 trigger=posedge TOP.TestDriver.clock |
| change_chipyard_dualrocketconfig_dhrystone_signals_100_window_32us_trigger_posedge_clk | 0.279866 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M window_size=32us signal_count=100 trigger=posedge TOP.TestDriver.clock |
| change_chipyard_clusteredrocketconfig_dhrystone_signals_100_window_8us_trigger_any | 0.271525 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M window_size=8us signal_count=100 trigger=* |
| change_chipyard_clusteredrocketconfig_dhrystone_signals_100_window_2us_trigger_any | 0.260162 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M window_size=2us signal_count=100 trigger=* |
| change_chipyard_dualrocketconfig_dhrystone_signals_10_window_32us_trigger_signal | 0.250710 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M window_size=32us signal_count=10 trigger=TOP.TestDriver.testHarness.chiptop0.axi4_mem_0_bits_ar_valid |
| change_chipyard_clusteredrocketconfig_dhrystone_signals_100_window_32us_trigger_any | 0.249907 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M window_size=32us signal_count=100 trigger=* |
| change_chipyard_clusteredrocketconfig_dhrystone_signals_10_window_2us_trigger_signal | 0.249779 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M window_size=2us signal_count=10 trigger=TOP.TestDriver.testHarness.chiptop0.axi4_mem_0_bits_ar_valid |
| change_chipyard_dualrocketconfig_dhrystone_signals_10_window_2us_trigger_posedge_clk | 0.242058 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M window_size=2us signal_count=10 trigger=posedge TOP.TestDriver.clock |
| change_chipyard_clusteredrocketconfig_dhrystone_signals_10_window_8us_trigger_any | 0.241967 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M window_size=8us signal_count=10 trigger=* |
| change_chipyard_dualrocketconfig_dhrystone_signals_100_window_2us_trigger_signal | 0.241540 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M window_size=2us signal_count=100 trigger=TOP.TestDriver.testHarness.chiptop0.axi4_mem_0_bits_ar_valid |
| change_chipyard_clusteredrocketconfig_dhrystone_signals_10_window_32us_trigger_posedge_clk | 0.241471 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M window_size=32us signal_count=10 trigger=posedge TOP.TestDriver.clock |
| change_chipyard_dualrocketconfig_dhrystone_signals_1_window_2us_trigger_posedge_clk | 0.241464 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M window_size=2us signal_count=1 trigger=posedge TOP.TestDriver.clock |
| change_chipyard_dualrocketconfig_dhrystone_signals_10_window_8us_trigger_signal | 0.241460 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M window_size=8us signal_count=10 trigger=TOP.TestDriver.testHarness.chiptop0.axi4_mem_0_bits_ar_valid |
| change_chipyard_dualrocketconfig_dhrystone_signals_10_window_2us_trigger_signal | 0.241238 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M window_size=2us signal_count=10 trigger=TOP.TestDriver.testHarness.chiptop0.axi4_mem_0_bits_ar_valid |
| change_chipyard_clusteredrocketconfig_dhrystone_signals_10_window_2us_trigger_any | 0.240987 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M window_size=2us signal_count=10 trigger=* |
| change_chipyard_dualrocketconfig_dhrystone_signals_10_window_8us_trigger_posedge_clk | 0.240973 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M window_size=8us signal_count=10 trigger=posedge TOP.TestDriver.clock |
| change_chipyard_dualrocketconfig_dhrystone_signals_100_window_2us_trigger_posedge_clk | 0.240720 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M window_size=2us signal_count=100 trigger=posedge TOP.TestDriver.clock |
| change_chipyard_clusteredrocketconfig_dhrystone_signals_10_window_32us_trigger_any | 0.240459 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M window_size=32us signal_count=10 trigger=* |
| change_chipyard_clusteredrocketconfig_dhrystone_signals_10_window_8us_trigger_posedge_clk | 0.240008 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M window_size=8us signal_count=10 trigger=posedge TOP.TestDriver.clock |
| change_chipyard_clusteredrocketconfig_dhrystone_signals_100_window_8us_trigger_posedge_clk | 0.239739 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M window_size=8us signal_count=100 trigger=posedge TOP.TestDriver.clock |
| change_chipyard_dualrocketconfig_dhrystone_signals_100_window_8us_trigger_signal | 0.239579 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M window_size=8us signal_count=100 trigger=TOP.TestDriver.testHarness.chiptop0.axi4_mem_0_bits_ar_valid |
| change_chipyard_dualrocketconfig_dhrystone_signals_100_window_32us_trigger_signal | 0.239486 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M window_size=32us signal_count=100 trigger=TOP.TestDriver.testHarness.chiptop0.axi4_mem_0_bits_ar_valid |
| change_chipyard_dualrocketconfig_dhrystone_signals_1_window_8us_trigger_posedge_clk | 0.239450 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M window_size=8us signal_count=1 trigger=posedge TOP.TestDriver.clock |
| change_chipyard_clusteredrocketconfig_dhrystone_signals_10_window_2us_trigger_posedge_clk | 0.239321 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M window_size=2us signal_count=10 trigger=posedge TOP.TestDriver.clock |
| change_chipyard_clusteredrocketconfig_dhrystone_signals_10_window_8us_trigger_signal | 0.239234 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M window_size=8us signal_count=10 trigger=TOP.TestDriver.testHarness.chiptop0.axi4_mem_0_bits_ar_valid |
| change_chipyard_clusteredrocketconfig_dhrystone_signals_1_window_2us_trigger_posedge_clk | 0.239112 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M window_size=2us signal_count=1 trigger=posedge TOP.TestDriver.clock |
| change_scr1_signals_100_window_4ns_trigger_posedge_clk | 0.192789 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=3.5M window_size=4ns signal_count=100 trigger=posedge TOP.clk |
| change_chipyard_clusteredrocketconfig_dhrystone_signals_1_window_32us_trigger_any | 0.192653 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M window_size=32us signal_count=1 trigger=* |
| change_scr1_signals_100_window_8ns_trigger_any | 0.191771 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=3.5M window_size=8ns signal_count=100 trigger=* |
| change_scr1_signals_100_window_2ns_trigger_posedge_clk | 0.191622 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=3.5M window_size=2ns signal_count=100 trigger=posedge TOP.clk |
| change_scr1_signals_100_window_8ns_trigger_signal | 0.191373 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=3.5M window_size=8ns signal_count=100 trigger=TOP.scr1_top_tb_axi.io_axi_dmem_araddr |
| change_scr1_signals_100_pos_50_window_2ns_trigger_any | 0.191126 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=3.5M window_size=2ns signal_count=100 trigger=* |
| change_chipyard_dualrocketconfig_dhrystone_signals_1_window_32us_trigger_signal | 0.190959 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M window_size=32us signal_count=1 trigger=TOP.TestDriver.testHarness.chiptop0.axi4_mem_0_bits_ar_valid |
| change_chipyard_dualrocketconfig_dhrystone_signals_10_window_32us_trigger_posedge_clk | 0.190711 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M window_size=32us signal_count=10 trigger=posedge TOP.TestDriver.clock |
| change_scr1_signals_100_window_4ns_trigger_signal | 0.190632 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=3.5M window_size=4ns signal_count=100 trigger=TOP.scr1_top_tb_axi.io_axi_dmem_araddr |
| change_chipyard_clusteredrocketconfig_dhrystone_signals_1_window_2us_trigger_signal | 0.190502 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M window_size=2us signal_count=1 trigger=TOP.TestDriver.testHarness.chiptop0.axi4_mem_0_bits_ar_valid |
| change_scr1_signals_100_window_8ns_trigger_posedge_clk | 0.190433 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=3.5M window_size=8ns signal_count=100 trigger=posedge TOP.clk |
| change_scr1_signals_100_window_4ns_trigger_any | 0.190421 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=3.5M window_size=4ns signal_count=100 trigger=* |
| change_scr1_signals_100_window_2ns_trigger_signal | 0.190255 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=3.5M window_size=2ns signal_count=100 trigger=TOP.scr1_top_tb_axi.io_axi_dmem_araddr |
| change_chipyard_clusteredrocketconfig_dhrystone_signals_1_window_8us_trigger_signal | 0.190146 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M window_size=8us signal_count=1 trigger=TOP.TestDriver.testHarness.chiptop0.axi4_mem_0_bits_ar_valid |
| change_chipyard_dualrocketconfig_dhrystone_signals_10_window_8us_trigger_any | 0.189765 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M window_size=8us signal_count=10 trigger=* |
| change_chipyard_dualrocketconfig_dhrystone_signals_10_window_2us_trigger_any | 0.189723 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M window_size=2us signal_count=10 trigger=* |
| change_chipyard_dualrocketconfig_dhrystone_signals_100_window_32us_trigger_any | 0.189498 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M window_size=32us signal_count=100 trigger=* |
| change_chipyard_dualrocketconfig_dhrystone_signals_10_window_32us_trigger_any | 0.189380 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M window_size=32us signal_count=10 trigger=* |
| change_chipyard_dualrocketconfig_dhrystone_signals_100_window_8us_trigger_posedge_clk | 0.188903 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M window_size=8us signal_count=100 trigger=posedge TOP.TestDriver.clock |
| change_chipyard_dualrocketconfig_dhrystone_signals_100_window_2us_trigger_any | 0.188836 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M window_size=2us signal_count=100 trigger=* |
| change_chipyard_dualrocketconfig_dhrystone_signals_100_window_8us_trigger_any | 0.188463 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M window_size=8us signal_count=100 trigger=* |
| change_chipyard_clusteredrocketconfig_dhrystone_signals_1_window_32us_trigger_signal | 0.188001 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M window_size=32us signal_count=1 trigger=TOP.TestDriver.testHarness.chiptop0.axi4_mem_0_bits_ar_valid |
| change_chipyard_dualrocketconfig_dhrystone_signals_1_window_2us_trigger_signal | 0.148919 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M window_size=2us signal_count=1 trigger=TOP.TestDriver.testHarness.chiptop0.axi4_mem_0_bits_ar_valid |
| change_picorv32_signals_1_window_8us_trigger_posedge_clk | 0.142557 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M window_size=8us signal_count=1 trigger=posedge testbench.clk |
| change_picorv32_signals_10_window_8us_trigger_any | 0.141691 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M window_size=8us signal_count=10 trigger=* |
| change_chipyard_clusteredrocketconfig_dhrystone_signals_1_window_8us_trigger_any | 0.141519 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M window_size=8us signal_count=1 trigger=* |
| change_picorv32_signals_10_window_2us_trigger_signal | 0.141503 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M window_size=2us signal_count=10 trigger=testbench.top.mem_axi_arvalid |
| change_picorv32_signals_10_window_2us_trigger_any | 0.141426 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M window_size=2us signal_count=10 trigger=* |
| change_picorv32_signals_10_window_32us_trigger_any | 0.141401 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M window_size=32us signal_count=10 trigger=* |
| change_chipyard_dualrocketconfig_dhrystone_signals_1_window_8us_trigger_any | 0.141215 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M window_size=8us signal_count=1 trigger=* |
| change_picorv32_signals_10_window_8us_trigger_posedge_clk | 0.141153 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M window_size=8us signal_count=10 trigger=posedge testbench.clk |
| change_picorv32_signals_1_window_32us_trigger_posedge_clk | 0.141003 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M window_size=32us signal_count=1 trigger=posedge testbench.clk |
| change_picorv32_signals_10_window_32us_trigger_posedge_clk | 0.140947 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M window_size=32us signal_count=10 trigger=posedge testbench.clk |
| change_chipyard_clusteredrocketconfig_dhrystone_signals_1_window_2us_trigger_any | 0.140935 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M window_size=2us signal_count=1 trigger=* |
| change_picorv32_signals_10_window_2us_trigger_posedge_clk | 0.140875 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M window_size=2us signal_count=10 trigger=posedge testbench.clk |
| change_picorv32_signals_1_window_2us_trigger_posedge_clk | 0.140807 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M window_size=2us signal_count=1 trigger=posedge testbench.clk |
| change_picorv32_signals_10_window_32us_trigger_signal | 0.140479 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M window_size=32us signal_count=10 trigger=testbench.top.mem_axi_arvalid |
| change_picorv32_signals_10_window_8us_trigger_signal | 0.140455 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M window_size=8us signal_count=10 trigger=testbench.top.mem_axi_arvalid |
| change_chipyard_dualrocketconfig_dhrystone_signals_1_window_2us_trigger_any | 0.139487 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M window_size=2us signal_count=1 trigger=* |
| change_chipyard_dualrocketconfig_dhrystone_signals_1_window_32us_trigger_any | 0.139354 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M window_size=32us signal_count=1 trigger=* |
| change_chipyard_dualrocketconfig_dhrystone_signals_1_window_8us_trigger_signal | 0.138819 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M window_size=8us signal_count=1 trigger=TOP.TestDriver.testHarness.chiptop0.axi4_mem_0_bits_ar_valid |
| change_picorv32_signals_1_window_32us_trigger_any | 0.091208 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M window_size=32us signal_count=1 trigger=* |
| change_picorv32_signals_1_window_8us_trigger_any | 0.090597 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M window_size=8us signal_count=1 trigger=* |
| change_picorv32_signals_1_window_2us_trigger_any | 0.090484 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M window_size=2us signal_count=1 trigger=* |
| change_picorv32_signals_1_window_2us_trigger_signal | 0.089898 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M window_size=2us signal_count=1 trigger=testbench.top.mem_axi_arvalid |
| change_picorv32_signals_1_window_8us_trigger_signal | 0.089835 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M window_size=8us signal_count=1 trigger=testbench.top.mem_axi_arvalid |
| change_picorv32_signals_1_window_32us_trigger_signal | 0.089616 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M window_size=32us signal_count=1 trigger=testbench.top.mem_axi_arvalid |
| change_scr1_coremark_imem_axi_2sig_to_1000ps | 0.084229 | waves=/opt/rtl-artifacts/scr1_max_axi_coremark.fst scope=TOP.scr1_top_tb_axi.i_top.i_imem_axi signal_count=2 window_to=1000ps |
| change_scr1_coremark_imem_axi_1sig_to_1000ps | 0.083349 | waves=/opt/rtl-artifacts/scr1_max_axi_coremark.fst scope=TOP.scr1_top_tb_axi.i_top.i_imem_axi signal_count=1 window_to=1000ps |
| change_scr1_signals_10_window_4ns_trigger_signal | 0.064061 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=3.5M window_size=4ns signal_count=10 trigger=TOP.scr1_top_tb_axi.io_axi_dmem_araddr |
| change_scr1_signals_1_window_8ns_trigger_posedge_clk | 0.058040 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=3.5M window_size=8ns signal_count=1 trigger=posedge TOP.clk |
| change_scr1_signals_1_window_4ns_trigger_any | 0.057683 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=3.5M window_size=4ns signal_count=1 trigger=* |
| change_scr1_signals_1_window_8ns_trigger_any | 0.057563 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=3.5M window_size=8ns signal_count=1 trigger=* |
| change_scr1_signals_1_window_2ns_trigger_posedge_clk | 0.057411 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=3.5M window_size=2ns signal_count=1 trigger=posedge TOP.clk |
| change_scr1_signals_10_window_4ns_trigger_any | 0.057370 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=3.5M window_size=4ns signal_count=10 trigger=* |
| change_scr1_signals_1_window_4ns_trigger_posedge_clk | 0.057261 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=3.5M window_size=4ns signal_count=1 trigger=posedge TOP.clk |
| change_scr1_signals_10_window_4ns_trigger_posedge_clk | 0.057217 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=3.5M window_size=4ns signal_count=10 trigger=posedge TOP.clk |
| change_scr1_signals_1_window_2ns_trigger_any | 0.057174 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=3.5M window_size=2ns signal_count=1 trigger=* |
| change_scr1_signals_1_window_8ns_trigger_signal | 0.057144 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=3.5M window_size=8ns signal_count=1 trigger=TOP.scr1_top_tb_axi.io_axi_dmem_araddr |
| change_scr1_signals_1_window_4ns_trigger_signal | 0.057134 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=3.5M window_size=4ns signal_count=1 trigger=TOP.scr1_top_tb_axi.io_axi_dmem_araddr |
| change_scr1_signals_10_window_8ns_trigger_any | 0.057120 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=3.5M window_size=8ns signal_count=10 trigger=* |
| change_scr1_signals_10_window_8ns_trigger_posedge_clk | 0.057119 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=3.5M window_size=8ns signal_count=10 trigger=posedge TOP.clk |
| change_scr1_signals_10_window_2ns_trigger_posedge_clk | 0.056984 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=3.5M window_size=2ns signal_count=10 trigger=posedge TOP.clk |
| change_scr1_signals_10_window_2ns_trigger_signal | 0.056867 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=3.5M window_size=2ns signal_count=10 trigger=TOP.scr1_top_tb_axi.io_axi_dmem_araddr |
| change_scr1_signals_1_window_2ns_trigger_signal | 0.056575 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=3.5M window_size=2ns signal_count=1 trigger=TOP.scr1_top_tb_axi.io_axi_dmem_araddr |
| change_scr1_signals_10_window_2ns_trigger_any | 0.056396 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=3.5M window_size=2ns signal_count=10 trigger=* |
| change_scr1_signals_10_window_8ns_trigger_signal | 0.056221 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=3.5M window_size=8ns signal_count=10 trigger=TOP.scr1_top_tb_axi.io_axi_dmem_araddr |

## info

| test | mean_s | meta |
| --- | --- | --- |
| info_chipyard_clustered_mt_memcpy | 0.189627 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_mt-memcpy.fst size=411M |
| info_chipyard_dualrocket_dhrystone | 0.090886 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M |
| info_chipyard_clustered_dhrystone | 0.088897 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M |
| info_scr1 | 0.048256 | waves=/opt/rtl-artifacts/scr1_max_axi_coremark.fst size=21M |
| info_picorv32 | 0.046390 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M |
| info_scr1_riscv_compliance | 0.032886 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=4M |
| info_scr1_isr_sample | 0.028246 | waves=/opt/rtl-artifacts/scr1_max_axi_isr_sample.fst size=69K |
| info_picorv32_ez | 0.027236 | waves=/opt/rtl-artifacts/picorv32_test_ez_vcd.fst size=17K |

## signal

| test | mean_s | meta |
| --- | --- | --- |
| signal_scr1_top_recursive_all_json | 0.039850 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst scope=TOP filter=.* recursive=True max_depth=default |
| signal_scr1_top_recursive_filter_valid_json | 0.037561 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst scope=TOP filter=(?i).*valid.* recursive=True max_depth=default |
| signal_scr1_top_recursive_depth2_json | 0.037220 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst scope=TOP filter=.* recursive=True max_depth=2 |
