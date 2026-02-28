# CLI E2E Bench Run: signal-recursive-baseline

- Generated at (UTC): 2026-02-28T11:44:42Z
- Run directory: `/workspaces/feat-signal-recursive/bench/e2e/runs/signal-recursive-baseline`
- Hyperfine JSON files: 3

## signal

| test | mean_s | meta |
| --- | --- | --- |
| signal_scr1_top_recursive_all_json | 0.008822 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst scope=TOP filter=.* recursive=True max_depth=default |
| signal_scr1_top_recursive_filter_valid_json | 0.008040 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst scope=TOP filter=(?i).*valid.* recursive=True max_depth=default |
| signal_scr1_top_recursive_depth2_json | 0.007396 | waves=/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst scope=TOP filter=.* recursive=True max_depth=2 |
