# Testing

Compare formats on equivalent inputs. Use snapshots only when rendered diagnostic text, notes, or source echo are part of the contract. Use structured assertions for other failures.

Keep command fixtures small and tied to durable behavior. Reuse named expression host profiles from `tests/common/expr_runtime.rs`. Test event-entrypoint runtime failures in Rust until the manifest schema can represent the event evaluation frame.

## Waveform fixtures

Prefer source-backed fixtures in `tests/fixtures/source/`. Register the outputs in `tests/fixtures/waveform_policy.json` and use `./dev just prepare-waveform-fixtures`. Keep hand dumps in `tests/fixtures/hand/` only when raw syntax or metadata is part of the case. Record the reason in the policy file. Keep derived native-format dumps ignored.

The public browser demo has a fixed checksum. Other large fixtures come from the pinned corpus in the container. Use native captures for types or names that conversion cannot preserve. See `environment.md`, `fsdb.md`, and `shm.md`.

## Browser tests

`./dev just playground-test` tests the browser and native CLI against the same inputs. The local Pages preview combines the Playground and the current documentation. It also checks that a local VCD query can run without off-origin requests.
