# CLI E2E Bench Run: 2026-03-01_20-48-49Z

- Generated at (UTC): 2026-03-02T05:29:39Z
- Run directory: `bench/e2e/runs/2026-03-01_20-48-49Z`
- Hyperfine JSON files: 135
- Wavepeek JSON files: 135

## at

| test | mean_s | meta |
| --- | --- | --- |
| at_picorv32_signals_1000 | 2.203291 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M signal_count=1000 |
| at_chipyard_dualrocketconfig_dhrystone_signals_1000 | 0.962917 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M signal_count=1000 |
| at_chipyard_clusteredrocketconfig_dhrystone_signals_1000 | 0.944206 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M signal_count=1000 |
| at_chipyard_dualrocketconfig_dhrystone_signals_100 | 0.546798 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M signal_count=100 |
| at_chipyard_clusteredrocketconfig_dhrystone_signals_100 | 0.541645 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M signal_count=100 |
| at_picorv32_signals_100 | 0.439924 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M signal_count=100 |
| at_scr1_signals_1000 | 0.391195 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=4M signal_count=1000 |
| at_chipyard_clusteredrocketconfig_dhrystone_signals_1 | 0.220745 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M signal_count=1 |
| at_chipyard_clusteredrocketconfig_dhrystone_signals_10 | 0.213956 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M signal_count=10 |
| at_chipyard_dualrocketconfig_dhrystone_signals_10 | 0.201597 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M signal_count=10 |
| at_chipyard_dualrocketconfig_dhrystone_signals_1 | 0.192773 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M signal_count=1 |
| at_picorv32_signals_10 | 0.190331 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M signal_count=10 |
| at_picorv32_signals_1 | 0.089940 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M signal_count=1 |
| at_scr1_signals_100 | 0.058173 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=4M signal_count=100 |
| at_scr1_signals_10 | 0.057697 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=4M signal_count=10 |
| at_scr1_signals_1 | 0.057397 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=4M signal_count=1 |

## change

| test | mean_s | meta |
| --- | --- | --- |
| change_chipyard_clusteredrocketconfig_dhrystone_signals_1_window_32000000_trigger_posedge_clk | 300.044549 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M window_size=32000000 signal_count=1 trigger=posedge TOP.TestDriver.clock |
| change_chipyard_clusteredrocketconfig_dhrystone_signals_100_window_32000000_trigger_signal | 300.040667 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M window_size=32000000 signal_count=100 trigger=TOP.TestDriver.testHarness.chiptop0.axi4_mem_0_bits_ar_valid |
| change_chipyard_clusteredrocketconfig_dhrystone_signals_1_window_8000000_trigger_posedge_clk | 300.040577 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M window_size=8000000 signal_count=1 trigger=posedge TOP.TestDriver.clock |
| change_chipyard_dualrocketconfig_dhrystone_signals_100_window_8000000_trigger_signal | 300.040109 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M window_size=8000000 signal_count=100 trigger=TOP.TestDriver.testHarness.chiptop0.axi4_mem_0_bits_ar_valid |
| change_chipyard_clusteredrocketconfig_dhrystone_signals_100_window_2000000_trigger_signal | 300.039697 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M window_size=2000000 signal_count=100 trigger=TOP.TestDriver.testHarness.chiptop0.axi4_mem_0_bits_ar_valid |
| change_chipyard_dualrocketconfig_dhrystone_signals_100_window_32000000_trigger_signal | 300.039441 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M window_size=32000000 signal_count=100 trigger=TOP.TestDriver.testHarness.chiptop0.axi4_mem_0_bits_ar_valid |
| change_chipyard_dualrocketconfig_dhrystone_signals_10_window_8000000_trigger_signal | 300.038815 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M window_size=8000000 signal_count=10 trigger=TOP.TestDriver.testHarness.chiptop0.axi4_mem_0_bits_ar_valid |
| change_chipyard_clusteredrocketconfig_dhrystone_signals_10_window_2000000_trigger_signal | 300.038781 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M window_size=2000000 signal_count=10 trigger=TOP.TestDriver.testHarness.chiptop0.axi4_mem_0_bits_ar_valid |
| change_chipyard_dualrocketconfig_dhrystone_signals_10_window_32000000_trigger_signal | 300.038715 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M window_size=32000000 signal_count=10 trigger=TOP.TestDriver.testHarness.chiptop0.axi4_mem_0_bits_ar_valid |
| change_chipyard_clusteredrocketconfig_dhrystone_signals_1_window_8000000_trigger_any | 300.038708 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M window_size=8000000 signal_count=1 trigger=* |
| change_chipyard_dualrocketconfig_dhrystone_signals_1_window_8000000_trigger_posedge_clk | 300.038384 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M window_size=8000000 signal_count=1 trigger=posedge TOP.TestDriver.clock |
| change_chipyard_clusteredrocketconfig_dhrystone_signals_10_window_8000000_trigger_signal | 300.038309 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M window_size=8000000 signal_count=10 trigger=TOP.TestDriver.testHarness.chiptop0.axi4_mem_0_bits_ar_valid |
| change_chipyard_dualrocketconfig_dhrystone_signals_1_window_32000000_trigger_posedge_clk | 300.038304 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M window_size=32000000 signal_count=1 trigger=posedge TOP.TestDriver.clock |
| change_chipyard_clusteredrocketconfig_dhrystone_signals_10_window_32000000_trigger_signal | 300.038274 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M window_size=32000000 signal_count=10 trigger=TOP.TestDriver.testHarness.chiptop0.axi4_mem_0_bits_ar_valid |
| change_chipyard_dualrocketconfig_dhrystone_signals_100_window_2000000_trigger_signal | 300.038216 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M window_size=2000000 signal_count=100 trigger=TOP.TestDriver.testHarness.chiptop0.axi4_mem_0_bits_ar_valid |
| change_chipyard_dualrocketconfig_dhrystone_signals_10_window_2000000_trigger_signal | 300.037900 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M window_size=2000000 signal_count=10 trigger=TOP.TestDriver.testHarness.chiptop0.axi4_mem_0_bits_ar_valid |
| change_chipyard_clusteredrocketconfig_dhrystone_signals_1_window_8000000_trigger_signal | 300.037601 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M window_size=8000000 signal_count=1 trigger=TOP.TestDriver.testHarness.chiptop0.axi4_mem_0_bits_ar_valid |
| change_chipyard_clusteredrocketconfig_dhrystone_signals_100_window_8000000_trigger_signal | 300.037564 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M window_size=8000000 signal_count=100 trigger=TOP.TestDriver.testHarness.chiptop0.axi4_mem_0_bits_ar_valid |
| change_chipyard_dualrocketconfig_dhrystone_signals_1_window_32000000_trigger_any | 300.037451 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M window_size=32000000 signal_count=1 trigger=* |
| change_chipyard_dualrocketconfig_dhrystone_signals_1_window_32000000_trigger_signal | 300.037094 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M window_size=32000000 signal_count=1 trigger=TOP.TestDriver.testHarness.chiptop0.axi4_mem_0_bits_ar_valid |
| change_chipyard_dualrocketconfig_dhrystone_signals_1_window_8000000_trigger_signal | 300.037083 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M window_size=8000000 signal_count=1 trigger=TOP.TestDriver.testHarness.chiptop0.axi4_mem_0_bits_ar_valid |
| change_chipyard_clusteredrocketconfig_dhrystone_signals_1_window_32000000_trigger_signal | 300.037041 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M window_size=32000000 signal_count=1 trigger=TOP.TestDriver.testHarness.chiptop0.axi4_mem_0_bits_ar_valid |
| change_chipyard_clusteredrocketconfig_dhrystone_signals_1_window_32000000_trigger_any | 300.036001 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M window_size=32000000 signal_count=1 trigger=* |
| change_chipyard_dualrocketconfig_dhrystone_signals_1_window_8000000_trigger_any | 249.658151 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M window_size=8000000 signal_count=1 trigger=* |
| change_chipyard_clusteredrocketconfig_dhrystone_signals_100_window_2000000_trigger_posedge_clk | 207.378912 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M window_size=2000000 signal_count=100 trigger=posedge TOP.TestDriver.clock |
| change_chipyard_clusteredrocketconfig_dhrystone_signals_100_window_32000000_trigger_posedge_clk | 192.018158 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M window_size=32000000 signal_count=100 trigger=posedge TOP.TestDriver.clock |
| change_chipyard_clusteredrocketconfig_dhrystone_signals_100_window_8000000_trigger_posedge_clk | 191.866044 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M window_size=8000000 signal_count=100 trigger=posedge TOP.TestDriver.clock |
| change_chipyard_dualrocketconfig_dhrystone_signals_100_window_32000000_trigger_posedge_clk | 161.946556 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M window_size=32000000 signal_count=100 trigger=posedge TOP.TestDriver.clock |
| change_chipyard_dualrocketconfig_dhrystone_signals_100_window_8000000_trigger_posedge_clk | 161.799936 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M window_size=8000000 signal_count=100 trigger=posedge TOP.TestDriver.clock |
| change_chipyard_dualrocketconfig_dhrystone_signals_100_window_2000000_trigger_posedge_clk | 161.789326 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M window_size=2000000 signal_count=100 trigger=posedge TOP.TestDriver.clock |
| change_chipyard_clusteredrocketconfig_dhrystone_signals_1_window_2000000_trigger_signal | 157.376866 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M window_size=2000000 signal_count=1 trigger=TOP.TestDriver.testHarness.chiptop0.axi4_mem_0_bits_ar_valid |
| change_chipyard_clusteredrocketconfig_dhrystone_signals_1_window_2000000_trigger_posedge_clk | 151.208062 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M window_size=2000000 signal_count=1 trigger=posedge TOP.TestDriver.clock |
| change_chipyard_dualrocketconfig_dhrystone_signals_1_window_2000000_trigger_signal | 124.704592 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M window_size=2000000 signal_count=1 trigger=TOP.TestDriver.testHarness.chiptop0.axi4_mem_0_bits_ar_valid |
| change_chipyard_dualrocketconfig_dhrystone_signals_1_window_2000000_trigger_posedge_clk | 124.347674 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M window_size=2000000 signal_count=1 trigger=posedge TOP.TestDriver.clock |
| change_chipyard_clusteredrocketconfig_dhrystone_signals_100_window_2000000_trigger_any | 108.173818 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M window_size=2000000 signal_count=100 trigger=* |
| change_chipyard_clusteredrocketconfig_dhrystone_signals_100_window_32000000_trigger_any | 99.587675 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M window_size=32000000 signal_count=100 trigger=* |
| change_chipyard_clusteredrocketconfig_dhrystone_signals_100_window_8000000_trigger_any | 98.734402 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M window_size=8000000 signal_count=100 trigger=* |
| change_picorv32_signals_100_window_8000000_trigger_signal | 95.689638 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M window_size=8000000 signal_count=100 trigger=testbench.top.mem_axi_arvalid |
| change_picorv32_signals_100_window_2000000_trigger_signal | 95.639759 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M window_size=2000000 signal_count=100 trigger=testbench.top.mem_axi_arvalid |
| change_picorv32_signals_100_window_32000000_trigger_signal | 95.437287 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M window_size=32000000 signal_count=100 trigger=testbench.top.mem_axi_arvalid |
| change_chipyard_dualrocketconfig_dhrystone_signals_100_window_2000000_trigger_any | 82.190813 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M window_size=2000000 signal_count=100 trigger=* |
| change_chipyard_dualrocketconfig_dhrystone_signals_100_window_8000000_trigger_any | 82.146188 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M window_size=8000000 signal_count=100 trigger=* |
| change_chipyard_dualrocketconfig_dhrystone_signals_100_window_32000000_trigger_any | 82.048525 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M window_size=32000000 signal_count=100 trigger=* |
| change_chipyard_clusteredrocketconfig_dhrystone_signals_1_window_2000000_trigger_any | 75.729878 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M window_size=2000000 signal_count=1 trigger=* |
| change_chipyard_dualrocketconfig_dhrystone_signals_1_window_2000000_trigger_any | 62.398077 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M window_size=2000000 signal_count=1 trigger=* |
| change_scr1_signals_100_window_8000_trigger_signal | 48.260976 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=3.5M window_size=8000 signal_count=100 trigger=TOP.scr1_top_tb_axi.io_axi_dmem_araddr |
| change_picorv32_signals_100_window_8000000_trigger_posedge_clk | 46.865943 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M window_size=8000000 signal_count=100 trigger=posedge testbench.clk |
| change_picorv32_signals_100_window_2000000_trigger_posedge_clk | 46.863675 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M window_size=2000000 signal_count=100 trigger=posedge testbench.clk |
| change_picorv32_signals_100_window_32000000_trigger_posedge_clk | 46.858943 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M window_size=32000000 signal_count=100 trigger=posedge testbench.clk |
| change_scr1_signals_100_window_2000_trigger_signal | 35.222091 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=3.5M window_size=2000 signal_count=100 trigger=TOP.scr1_top_tb_axi.io_axi_dmem_araddr |
| change_scr1_signals_100_window_4000_trigger_signal | 33.107902 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=3.5M window_size=4000 signal_count=100 trigger=TOP.scr1_top_tb_axi.io_axi_dmem_araddr |
| change_picorv32_signals_100_window_8000000_trigger_any | 23.902822 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M window_size=8000000 signal_count=100 trigger=* |
| change_picorv32_signals_100_window_2000000_trigger_any | 23.853849 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M window_size=2000000 signal_count=100 trigger=* |
| change_picorv32_signals_100_window_32000000_trigger_any | 23.801696 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M window_size=32000000 signal_count=100 trigger=* |
| change_chipyard_clusteredrocketconfig_dhrystone_signals_10_window_2000000_trigger_posedge_clk | 19.790339 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M window_size=2000000 signal_count=10 trigger=posedge TOP.TestDriver.clock |
| change_chipyard_clusteredrocketconfig_dhrystone_signals_10_window_8000000_trigger_posedge_clk | 19.788577 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M window_size=8000000 signal_count=10 trigger=posedge TOP.TestDriver.clock |
| change_chipyard_clusteredrocketconfig_dhrystone_signals_10_window_32000000_trigger_posedge_clk | 19.740444 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M window_size=32000000 signal_count=10 trigger=posedge TOP.TestDriver.clock |
| change_chipyard_dualrocketconfig_dhrystone_signals_10_window_32000000_trigger_posedge_clk | 17.686953 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M window_size=32000000 signal_count=10 trigger=posedge TOP.TestDriver.clock |
| change_chipyard_dualrocketconfig_dhrystone_signals_10_window_8000000_trigger_posedge_clk | 17.682676 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M window_size=8000000 signal_count=10 trigger=posedge TOP.TestDriver.clock |
| change_chipyard_dualrocketconfig_dhrystone_signals_10_window_2000000_trigger_posedge_clk | 17.635949 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M window_size=2000000 signal_count=10 trigger=posedge TOP.TestDriver.clock |
| change_scr1_signals_100_window_8000_trigger_posedge_clk | 12.041344 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=3.5M window_size=8000 signal_count=100 trigger=posedge TOP.clk |
| change_scr1_signals_100_window_2000_trigger_posedge_clk | 12.033052 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=3.5M window_size=2000 signal_count=100 trigger=posedge TOP.clk |
| change_scr1_signals_100_window_4000_trigger_posedge_clk | 12.024110 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=3.5M window_size=4000 signal_count=100 trigger=posedge TOP.clk |
| change_picorv32_signals_10_window_2000000_trigger_signal | 10.321452 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M window_size=2000000 signal_count=10 trigger=testbench.top.mem_axi_arvalid |
| change_picorv32_signals_10_window_8000000_trigger_signal | 10.218332 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M window_size=8000000 signal_count=10 trigger=testbench.top.mem_axi_arvalid |
| change_chipyard_clusteredrocketconfig_dhrystone_signals_10_window_2000000_trigger_any | 10.216961 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M window_size=2000000 signal_count=10 trigger=* |
| change_picorv32_signals_10_window_32000000_trigger_signal | 10.214628 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M window_size=32000000 signal_count=10 trigger=testbench.top.mem_axi_arvalid |
| change_chipyard_clusteredrocketconfig_dhrystone_signals_10_window_8000000_trigger_any | 10.116111 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M window_size=8000000 signal_count=10 trigger=* |
| change_chipyard_clusteredrocketconfig_dhrystone_signals_10_window_32000000_trigger_any | 10.114806 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M window_size=32000000 signal_count=10 trigger=* |
| change_chipyard_dualrocketconfig_dhrystone_signals_10_window_8000000_trigger_any | 8.465509 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M window_size=8000000 signal_count=10 trigger=* |
| change_chipyard_dualrocketconfig_dhrystone_signals_10_window_2000000_trigger_any | 8.461782 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M window_size=2000000 signal_count=10 trigger=* |
| change_chipyard_dualrocketconfig_dhrystone_signals_10_window_32000000_trigger_any | 8.413081 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M window_size=32000000 signal_count=10 trigger=* |
| change_scr1_signals_100_window_4000_trigger_any | 6.178330 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=3.5M window_size=4000 signal_count=100 trigger=* |
| change_scr1_signals_100_window_2000_trigger_any | 6.168816 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=3.5M window_size=2000 signal_count=100 trigger=* |
| change_scr1_signals_100_window_8000_trigger_any | 6.151855 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=3.5M window_size=8000 signal_count=100 trigger=* |
| change_picorv32_signals_10_window_32000000_trigger_posedge_clk | 5.105291 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M window_size=32000000 signal_count=10 trigger=posedge testbench.clk |
| change_picorv32_signals_10_window_2000000_trigger_posedge_clk | 5.103223 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M window_size=2000000 signal_count=10 trigger=posedge testbench.clk |
| change_picorv32_signals_10_window_8000000_trigger_posedge_clk | 5.102954 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M window_size=8000000 signal_count=10 trigger=posedge testbench.clk |
| change_scr1_signals_10_window_8000_trigger_signal | 4.887935 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=3.5M window_size=8000 signal_count=10 trigger=TOP.scr1_top_tb_axi.io_axi_dmem_araddr |
| change_picorv32_signals_1_window_32000000_trigger_signal | 4.252106 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M window_size=32000000 signal_count=1 trigger=testbench.top.mem_axi_arvalid |
| change_picorv32_signals_1_window_8000000_trigger_signal | 4.201289 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M window_size=8000000 signal_count=1 trigger=testbench.top.mem_axi_arvalid |
| change_picorv32_signals_1_window_2000000_trigger_signal | 3.952419 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M window_size=2000000 signal_count=1 trigger=testbench.top.mem_axi_arvalid |
| change_picorv32_signals_1_window_8000000_trigger_posedge_clk | 3.900054 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M window_size=8000000 signal_count=1 trigger=posedge testbench.clk |
| change_picorv32_signals_1_window_32000000_trigger_posedge_clk | 3.899941 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M window_size=32000000 signal_count=1 trigger=posedge testbench.clk |
| change_picorv32_signals_1_window_2000000_trigger_posedge_clk | 3.849599 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M window_size=2000000 signal_count=1 trigger=posedge testbench.clk |
| change_scr1_signals_10_window_2000_trigger_signal | 3.558537 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=3.5M window_size=2000 signal_count=10 trigger=TOP.scr1_top_tb_axi.io_axi_dmem_araddr |
| change_scr1_signals_10_window_4000_trigger_signal | 3.368545 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=3.5M window_size=4000 signal_count=10 trigger=TOP.scr1_top_tb_axi.io_axi_dmem_araddr |
| change_picorv32_signals_10_window_32000000_trigger_any | 2.700132 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M window_size=32000000 signal_count=10 trigger=* |
| change_picorv32_signals_10_window_8000000_trigger_any | 2.699849 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M window_size=8000000 signal_count=10 trigger=* |
| change_picorv32_signals_10_window_2000000_trigger_any | 2.699401 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M window_size=2000000 signal_count=10 trigger=* |
| change_picorv32_signals_1_window_8000000_trigger_any | 1.996672 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M window_size=8000000 signal_count=1 trigger=* |
| change_picorv32_signals_1_window_2000000_trigger_any | 1.996247 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M window_size=2000000 signal_count=1 trigger=* |
| change_picorv32_signals_1_window_32000000_trigger_any | 1.994588 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M window_size=32000000 signal_count=1 trigger=* |
| change_scr1_signals_10_window_4000_trigger_posedge_clk | 1.258895 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=3.5M window_size=4000 signal_count=10 trigger=posedge TOP.clk |
| change_scr1_signals_10_window_8000_trigger_posedge_clk | 1.258190 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=3.5M window_size=8000 signal_count=10 trigger=posedge TOP.clk |
| change_scr1_signals_10_window_2000_trigger_posedge_clk | 1.257351 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=3.5M window_size=2000 signal_count=10 trigger=posedge TOP.clk |
| change_scr1_signals_1_window_8000_trigger_signal | 1.042323 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=3.5M window_size=8000 signal_count=1 trigger=TOP.scr1_top_tb_axi.io_axi_dmem_araddr |
| change_scr1_signals_1_window_2000_trigger_signal | 0.786899 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=3.5M window_size=2000 signal_count=1 trigger=TOP.scr1_top_tb_axi.io_axi_dmem_araddr |
| change_scr1_signals_1_window_4000_trigger_signal | 0.746201 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=3.5M window_size=4000 signal_count=1 trigger=TOP.scr1_top_tb_axi.io_axi_dmem_araddr |
| change_scr1_signals_10_window_8000_trigger_any | 0.691854 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=3.5M window_size=8000 signal_count=10 trigger=* |
| change_scr1_signals_10_window_2000_trigger_any | 0.691234 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=3.5M window_size=2000 signal_count=10 trigger=* |
| change_scr1_signals_10_window_4000_trigger_any | 0.679738 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=3.5M window_size=4000 signal_count=10 trigger=* |
| change_scr1_signals_1_window_8000_trigger_posedge_clk | 0.190442 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=3.5M window_size=8000 signal_count=1 trigger=posedge TOP.clk |
| change_scr1_signals_1_window_2000_trigger_posedge_clk | 0.190384 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=3.5M window_size=2000 signal_count=1 trigger=posedge TOP.clk |
| change_scr1_signals_1_window_4000_trigger_posedge_clk | 0.190171 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=3.5M window_size=4000 signal_count=1 trigger=posedge TOP.clk |
| change_scr1_signals_1_window_8000_trigger_any | 0.141326 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=3.5M window_size=8000 signal_count=1 trigger=* |
| change_scr1_signals_1_window_2000_trigger_any | 0.140640 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=3.5M window_size=2000 signal_count=1 trigger=* |
| change_scr1_signals_1_window_4000_trigger_any | 0.139841 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=3.5M window_size=4000 signal_count=1 trigger=* |

## info

| test | mean_s | meta |
| --- | --- | --- |
| info_chipyard_clustered_mt_memcpy | 0.189855 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_mt-memcpy.fst size=411M |
| info_chipyard_dualrocket_dhrystone | 0.089335 | waves=/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst size=76M |
| info_chipyard_clustered_dhrystone | 0.088096 | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_dhrystone.fst size=165M |
| info_scr1 | 0.043507 | waves=/opt/rtl-artifacts/scr1_max_axi_coremark.fst size=21M |
| info_picorv32 | 0.040749 | waves=/opt/rtl-artifacts/picorv32_test_vcd.fst size=13M |
| info_scr1_riscv_compliance | 0.032491 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst size=4M |
| info_scr1_isr_sample | 0.026487 | waves=/opt/rtl-artifacts/scr1_max_axi_isr_sample.fst size=69K |
| info_picorv32_ez | 0.025943 | waves=/opt/rtl-artifacts/picorv32_test_ez_vcd.fst size=17K |

## signal

| test | mean_s | meta |
| --- | --- | --- |
| signal_scr1_top_recursive_all_json | 0.040910 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst scope=TOP filter=.* recursive=True max_depth=default |
| signal_scr1_top_recursive_depth2_json | 0.036968 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst scope=TOP filter=.* recursive=True max_depth=2 |
| signal_scr1_top_recursive_filter_valid_json | 0.036775 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst scope=TOP filter=(?i).*valid.* recursive=True max_depth=default |
