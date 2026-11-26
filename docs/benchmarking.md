# Benchmarking

Run benchmarks through `./dev` to keep the toolchain and fixtures fixed. Keep the harness deterministic and Python stdlib-only. Keep result artifacts under ignored paths such as `tmp/bench-gate/`.

## Comparing revisions

Use `./dev just bench-gate <baseline-ref> <revised-ref>` from a clean worktree. The gate builds the selected refs and uses the current worktree's scripts, catalogs, and fixtures for both. For separate captures, pass the `run` directories printed by `bench-capture` to `bench-compare`.

Compare both payloads and timing within the same format. The gate checks median regression against a 5% threshold with a 5 ms floor by default. For timing-only failures it compares the minimum recorded time for each binary. A timeout wrapper can quantize short timings. Use direct CLI timings to investigate small differences.

Cross-format checks compare payloads. FST and FSDB use different readers and can expose different metadata paths. Record any metadata exceptions and cases omitted from the runnable FSDB catalog in the compare and capture manifests.

## Workload constraints

Maintain the FST workload in `bench/e2e/tests.json` and regenerate the derived format catalogs with the corresponding `just` recipes. Keep the CI smoke workload small. Smoke checks CLI execution and saves artifacts under `tmp/bench-smoke/`. Use at least ten measured runs and five warmups for release-gate workloads.

Set `--on` explicitly for `change` and `property` scenarios. Use `--sample-mode native` for wildcard, plain-signal, and mixed-trigger scenarios.

VCD timing is opt-in. It can materialize tens of GiB of VCD beside the FST inputs for reuse in the worktree container. With an available FSDB reader, `auto` mode captures FSDB if both refs support it and fails the gate if only one does. SHM has a focused entrypoint, `./dev just bench-e2e-shm-run`. Prepare native-format inputs outside timed queries.
