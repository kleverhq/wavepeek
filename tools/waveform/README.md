# Waveform Fixture Tools

`prepare_fixtures.py` regenerates source-backed VCD/FST test fixtures from `tests/fixtures/waveform_policy.json`.

Use the stable repository entrypoint:

    just prepare-waveform-fixtures

The script requires `iverilog`, `vvp`, and `vcd2fst` on `PATH`. Generated outputs live under ignored `tests/fixtures/generated/`.
