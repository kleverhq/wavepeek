# Testing

Compare formats on equivalent inputs. Use snapshots only when rendered diagnostic text, notes, or source echo are part of the contract. Use structured assertions for other failures.

Keep command fixtures small and tied to durable behavior. Reuse named expression host profiles from `tests/common/expr_runtime.rs`. Test event-entrypoint runtime failures in Rust until the manifest schema can represent the event evaluation frame.

## Verification profiles

GitHub Actions runs `just ci` for pull requests and release tags. Run `./dev just ci` in the public profile to reproduce the VCD and FST CLI regressions and smoke, the help and disabled-feature diagnostics for optional readers, and the shared coverage, fixture, benchmark harness, documentation, and Playground checks. The tag workflow also checks the Cargo package and builds release distributions.

Run `./dev just ci-native` in the private profile. It checks each native reader independently. For each available reader, it runs feature lint, verifies the loader with a binary outside Cargo, and checks CLI output on native or converted inputs. It reports each unavailable reader as skipped. `./dev just ci` invokes the same native checks after the common quality gate.

The E2E smoke suite opens equivalent VCD, FST, FSDB, SHM, WLF, and VWDB dumps from `tests/fixtures/source/m2_core.v`. For each selected format it checks metadata and bounds, scope and signal paths, a point sample, and a change at the interval boundary. Each CLI call has a timeout and must produce the expected data. See `benchmarking.md` for the full E2E suites and release gate.

The WDB CLI tests record `m2_core.v` and `extract_apb3.v` with XSim and compare results with VCDs from the same HDL. They use a released WDB for enum labels and native names. The first `./dev just ci-wdb` run downloads it under `tmp/wdb/`. Each run verifies its SHA-256 and removes temporary captures after a successful run.

## Waveform fixtures

Prefer source-backed fixtures in `tests/fixtures/source/`. Register the outputs in `tests/fixtures/waveform_policy.json` and use `./dev just prepare-waveform-fixtures`. Keep hand dumps in `tests/fixtures/hand/` only when raw syntax or metadata is part of the case. Record the reason in the policy file. Keep derived native-format dumps ignored.

The public browser demo has a fixed checksum. Other large fixtures come from the pinned corpus in the container. Use native captures for types or names that conversion cannot preserve. See `environment.md`, `fsdb.md`, `shm.md`, `wlf.md`, and `vwdb.md`.

## Browser tests

`./dev just playground-test` tests the browser and native CLI against the same inputs. The local Pages preview combines the Playground and the current documentation. It also checks that a local VCD query can run without off-origin requests.
