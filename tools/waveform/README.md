# Waveform Fixture Tools

`prepare_fixtures.py` regenerates source-backed VCD, FST, and GHW test fixtures from `tests/fixtures/waveform_policy.json`.

For VCD and FST fixtures, use:

    just prepare-waveform-fixtures

For the GHW CLI fixtures, use:

    just prepare-ghw-fixtures

Verilog fixtures require `iverilog`, `vvp`, and `vcd2fst` on `PATH`. GHW fixtures use `ghdl` with the mcode backend. Generated outputs live under ignored `tests/fixtures/generated/`. GHDL writes the GHW and the reference VCD under `tests/fixtures/generated/ghw/`. FSDB fixture preparation excludes this directory.
