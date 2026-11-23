# Path and scope resolution

WavePeek formats paths from exact Ondas hierarchy components. A path is a sequence of components separated by dots. In the default `logical` style, names with dots, literal brackets, commas, quotes, backslashes, or whitespace use JSON string syntax:

```text
top.cpu.valid
top."a.b".c
top."a.b.c"
top."literal[7:0]"
```

The middle two paths select different declarations. An index follows the name, as in `top.u_reg[0].q`. Brackets inside a quoted or escaped name are literal characters, as in `top."u_reg[0]".q`. Listing paths can be reused in selections. Scope depth counts components, regardless of punctuation inside a name.

Use `--path-style sv` after `scope`, `signal`, `value`, `change`, or an `extract` subcommand to print SystemVerilog paths. A backslash starts an escaped name. An ASCII space terminates it, even at the end of a path. Both styles are accepted as input. Path suggestions in diagnostics use logical syntax.

An unrepresentable result path under `--path-style sv` returns `WPK-F0001` with the message `--path-style sv: path component cannot be represented in Verilog` followed by the component name.

Use shell single quotes to preserve quotes, backslashes, and spaces. Commas separate `--signals` and CLI `--payload` entries outside quoted or escaped components. Order and duplicates are preserved.

```sh
wavepeek value --waves dump.vcd --at 10ns \
  --signals 'top."literal[7:0]",top.literal'
wavepeek value --path-style sv --waves dump.vcd --at 0ns \
  --signals 'top."u_reg[0]".q'
# @0ns top.\u_reg[0] .q=1'h1
```

## Relative paths

`signal`, `value`, `change`, `property`, and `extract` support `--scope`. With `--scope top`, both `cpu.valid` and `top.cpu.valid` resolve inside that scope. Names without the exact selected scope's component prefix are relative to it. Use `signal --scope ''` for root-level declarations; add `--recursive` to include signals in child scopes.

```sh
wavepeek value --waves dump.vcd --at 10ns \
  --scope 'top."a.b"' --signals c
```

Human output supports `--abs` for full paths. Machine output reports full paths in `path` and omits the selected scope from `relative_path`. Both fields use the selected style. Reuse the full `path` as a selector. A relative suffix can start with the same components as `--scope`, which the selector parser reads as an already-qualified path.

## Expression names

Event and Boolean expressions accept qualified paths with quoted components, such as `top."a.b".c`. A standalone quoted token in a Boolean expression remains a string literal. Use `$signal("path")` to name a signal explicitly; its argument is a JSON string containing the path. This also addresses quoted root names and names containing expression operators or reserved words. Scope resolution is the same as for ordinary signal operands. An expression selection after `$signal(...)` applies to that resolved operand, rather than looking up another declaration with an appended index.

```sh
wavepeek property --waves dump.vcd --on 'posedge top.clk' \
  --eval 'top."a.b".c == 34'
wavepeek property --waves dump.vcd --on 'posedge top.clk' \
  --eval '$signal("\"root.name\"") == 1'
```

SystemVerilog escaped identifiers are also accepted, with terminating whitespace: `top.\a.b .c` identifies the same components as `top."a.b".c`.

## Bit projections and composite signals

`value --signals`, `change --signals`, and `extract generic` payloads may append one flat static decimal `[msb:lsb]` projection. Resolution first tries the complete name, then a trailing projection outside quoted or escaped components. `[n]` names a declaration. Use `[n:n]` for one projected bit. Only one trailing projection is applied to an exact declaration. An indexed declaration such as `mem[0]` can be the base. Bit zero is the rightmost normalized sampled bit. When a literal declaration conflicts with a projection spelling, the projection uses an explicit base name, such as `top."mem"[0][3:0]` in logical or `top.\mem [0][3:0]` in SV.

```sh
wavepeek value --waves dump.vcd --at 10ns \
  --signals 'top.data[7:4],top.mem[0][3:0]'
```

Names such as `memory[0]` do not create scopes or prove array structure. A trailing projection can select bits of such a declaration, for example `top.mem[0][7:0]`. WavePeek delegates selector resolution and scope qualification to Ondas.

Ondas normalizes confirmed VCD split declarations into one ordinary signal before publishing the hierarchy. The parts must share an exact name and owning scope, have compatible metadata, and cover contiguous, non-overlapping, consistently directed ranges. For scalar parts, higher indices occupy more significant bits. Independent aliases remain available. Other formats retain their Ondas declarations without WavePeek reconstruction. If some parts have observations, missing parts contribute X; if all parts are missing, the whole signal and its projections are missing.

## Ambiguous declarations

Distinct source declarations can have the same exact path. Listings omit an ambiguous path with `WPK-W0005`, and explicit selection fails without choosing a candidate. This rule applies to every format. Shared alias histories do not merge declarations or make different paths ambiguous. Canonical quoting prevents collisions caused by flattening different component sequences.
