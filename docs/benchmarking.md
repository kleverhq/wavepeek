# Benchmarking

Run benchmarks through `./dev` to keep the toolchain and fixtures fixed. Keep the harness deterministic and Python stdlib-only. Keep result artifacts under ignored paths such as `tmp/bench-gate/`.

## Comparing revisions

Use `./dev just bench-release-gate vPREVIOUS HEAD` from a clean worktree in the private profile. It requires VCD, FST, FSDB, SHM, and WLF. To run the release gate for the open formats in the public profile, use `./dev just bench-release-gate vPREVIOUS HEAD public`. The gate builds the selected refs with their lockfiles and uses the current worktree's scripts and catalogs.

Compare both payloads and timing within the same format. The gate checks median regression against a 5% threshold with a 5 ms floor by default. For timing-only failures it compares the minimum recorded time for each binary. A timeout wrapper can quantize short timings. Use direct CLI timings to investigate small differences.

Cross-format checks compare payloads on equivalent inputs. The gate manifest identifies both revisions and the results. The run manifest records the input and catalog SHA-256 checksums for the binaries in each suite. The comparison records format-specific exceptions. A newly introduced optional format records its first baseline from the revised ref. A missing required reader or input fails the gate.

Use `./dev just bench-gate <baseline-ref> <revised-ref>` for diagnostics. For separate diagnostic captures, pass the `run` directories printed by `bench-capture` to `bench-compare`. For all formats, use `./dev just bench-capture <ref> always always always always` in the private profile. The comparison reports `shared-run` for paired release runs and `matched` for matching independently recorded inputs. Diagnostic comparisons without recorded checksums report `unverified`.

## Workload constraints

The full E2E recipes are `bench-e2e-run` for FST, `bench-e2e-vcd-run` for VCD, `bench-e2e-fsdb-run` for FSDB, `bench-e2e-shm-run` for SHM, and `bench-e2e-wlf-run` for WLF. Run each through `./dev` in its profile. The full recipes and release gate use the same format catalogs and fail on a missing or failed scenario or a timeout. Input preparation and checksums run before timing. Run manifests contain the catalog and input SHA-256 checksums. The container pins the original fixture corpus by revision and verifies its published checksums during installation.

Maintain the FST workload in `bench/e2e/tests.json`. Each derived-format run generates a catalog under `tmp/bench-catalogs/`. The run manifest records its path and checksum. Use at least ten measured runs and five warmups for release-gate workloads. The short CI smoke saves checked payloads under `tmp/bench-smoke/`.

Use selectors that both release revisions accept. Twelve large selection cases use ordinary signal paths in place of legacy array-element selectors. This keeps the same selection counts and lets the gate run one command per case against both revisions. CLI regressions cover indexed selectors.

Set `--on` explicitly for `change` and `property` scenarios. Use `--sample-mode native` for wildcard, plain-signal, and mixed-trigger scenarios.

The two SCR1 instruction-memory `change` scenarios with a wildcard trigger and end time `1000ps` start at `2ps`. Conversion to SHM or WLF can add an initial tick at zero, so both queries start after the initial sample.

VCD timing can materialize tens of GiB of VCD beside the FST inputs for reuse in the worktree container. Diagnostic `bench-gate` and `bench-capture` require `vcd=always` to time VCD. With an available FSDB reader, diagnostic `bench-gate` in `auto` mode captures FSDB if both refs support it and fails if only one does.
