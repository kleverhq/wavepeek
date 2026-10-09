# Path and scope resolution

This page explains how `wavepeek` deals with names and paths, including canonical and relative paths and bit projections.

Normally, `wavepeek` uses canonical dump-derived paths (also called canonical full paths) for every query. Some commands also support shorter relative paths.

## Relative paths

To reduce boilerplate on deep hierarchies most query commands support `--scope` (`signal`, `value`, `change`, `property`, `extract`) and accept shorter names relative to the selected scope and canonical paths inside it. With `--scope top`, both `cpu.valid` and `top.cpu.valid` resolve to the same signal, and both forms may appear in one request. Every name that does not begin with the exact selected-scope prefix is interpreted relative to that scope, so it cannot select a canonical path outside the scope.

Full canonical paths (without `--scope`):

```text
wavepeek value --waves dump.vcd --at 10ns --signals top.cpu.clk,top.cpu.state,top.cpu.csr.status
```

With `--scope <path>`:

```text
wavepeek value --waves dump.vcd --at 10ns --scope top.cpu --signals clk,state,csr.status
# or
wavepeek value --waves dump.vcd --at 10ns --scope top.cpu --signals clk,top.cpu.state,csr.status
```

Commands that print signal paths in human output support `--abs` when relative output is possible. This includes `signal`, `value`, `change`, and the `extract` subcommands.

## Bit projections

`value --signals`, `change --signals`, and `extract generic` payloads may append one flat static decimal `[msb:lsb]` projection. Resolution first tries the complete token as a waveform path, then removes one trailing projection and resolves its base. `[n]` therefore remains ordinary waveform path syntax; use `[n:n]` for one projected bit. Projection indexes normalized sampled bits with bit zero at the right. These request lists preserve order and duplicates, including overlapping projections and a projection beside its complete source.

```text
wavepeek value --waves dump.vcd --at 10ns --scope top.cpu --signals 'state[7:4],top.cpu.data[31:16]'
```

## FSDB ambiguous paths

Older versions of Verdi can dump two distinct signals with exactly the same metadata under some circumstances. For example, a dump may contain `top.tx.opcode` (`opcode` is a struct field) and `top.opcode` (a logic signal), while the standalone FSDB does not expose enough information to recover both original paths.

If distinct FSDB records map to one canonical signal path, `wavepeek` quarantines that path instead of selecting a backing record. Scopes and unambiguous signals remain available. Signal listings omit quarantined paths with a diagnostic, while an explicit reference to one fails as an ambiguous signal.

Ambiguous-signal errors describe each candidate's scope, kind, width, and declared range in deterministic order. Candidate numbers are local to the diagnostic; they do not select a backing record.
