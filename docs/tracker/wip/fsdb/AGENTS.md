# FSDB WIP Guidance

## Scope

Temporary FSDB branch collateral: architecture notes, Verdi research notes, command-specific API notes, and FSDB execution plans.

## Source of Truth

- FSDB integration architecture proposal: `arch.md`
- Local Verdi installation map for FSDB work: `verdi_home_map.md`
- Command-specific FSDB Reader API research: `cmd_info.md`, `cmd_scope.md`, `cmd_signal.md`, `cmd_value.md`, `cmd_change.md`, `cmd_property.md`
- Completed FSDB execution plans: `plans/completed/`

## Local Guidance

- This directory is not part of the embedded public documentation corpus.
- Keep FSDB WIP material here while the branch is under review; fold durable content into `../../../dev/`, `../../../public/`, or source comments before final merge.
- Do not place Verdi headers, libraries, documentation excerpts, generated bindings, `.fsdb` fixtures, or converter logs here.
- Treat `.fst` and `.fsdb` waveform files as binary artifacts; inspect them only through `wavepeek`, Verdi tools, or binary-safe metadata commands.
