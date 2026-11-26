# Path and scope resolution

WavePeek formats paths from exact Ondas hierarchy components separated by dots. The default `sv` style preserves explicit SystemVerilog escapes supplied by the waveform reader and escapes names outside the bare-name pattern when representable. An ASCII space terminates each escaped name, even at the end of a path.

Use `--path-style logical` when a consumer expects logical spelling or for names that SV cannot represent. Logical style writes names matching `[A-Za-z_][A-Za-z0-9_$]*` bare and uses JSON string syntax for other names.

```text
top.cpu.valid
top."a.b".c
top."a.b.c"
top."literal[7:0]"
```

The middle two paths select different declarations. One or more indices can follow a name in the same component, as in `top.u_reg[0].q` and `top.matrix[1][2]`. Brackets inside a quoted or escaped name are literal characters. For example, `top."matrix[1][2]"` selects the literal name. Listing paths can be reused in selections. Scope depth counts components, regardless of punctuation inside a name.

Both styles are accepted as input. Filters, includes, and path suggestions use the canonical logical spelling, including its quotes.

An unrepresentable result path in SV style returns `WPK-F0001` with the message `--path-style sv: path component cannot be represented in Verilog` followed by the component name. For example, use `--path-style logical` to list a name such as `zété`.

Use shell single quotes to preserve quotes, backslashes, and spaces. Commas separate `--signals` and CLI `--payload` entries outside quoted or escaped components. Order and duplicates are preserved.

```sh
wavepeek value --waves dump.vcd --at 10ns \
  --signals 'top."literal[7:0]",top.literal'
wavepeek value --waves dump.vcd --at 0ns \
  --signals 'top."u_reg[0]".q'
# @0ns top.\u_reg[0] .q=1'h1
```

## Relative paths

`signal`, `value`, `change`, `property`, and `extract` support `--scope`. With `--scope top`, both `cpu.valid` and `top.cpu.valid` resolve inside that scope. Names without the exact selected scope's component prefix are relative to it. Use `signal --scope ''` for root-level declarations; add `--recursive` to include signals in child scopes.

Protocol `--map` with `--scope` takes one local component and an optional projection, for example `tdata=data[3:0]`.

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

SystemVerilog escaped identifiers require terminating whitespace. `top.\a.b .c` identifies the same components as `top."a.b".c`. Event and Boolean operands accept signed hierarchy indices such as `top.\a.b [-1].c`. An escaped event name can be followed by `.triggered()`, as in `top.\ev.pulse .triggered()`.

## Bit projections and composite signals

`value --signals`, `change --signals`, and `extract` payload selectors accept one trailing static decimal `[msb:lsb]` projection. A range outside quotes or an escaped name selects bits from the base declaration. Use `top.data[3:0]` for a projection and `top."data[3:0]"` for the literal name. `[n]` selects an array element. Use `[n:n]` for one projected bit. An indexed declaration such as `mem[0]` or `matrix[1][2]` can be the base. Bit zero is the rightmost normalized sampled bit.

```sh
wavepeek value --waves dump.vcd --at 10ns \
  --signals 'top.data[7:4],top.matrix[1][2][3:0]'
```

Names such as `memory[0]` do not create scopes or prove array structure. A trailing projection can select bits of such a declaration, for example `top.mem[0][7:0]`. WavePeek delegates selector resolution and scope qualification to Ondas.

Ondas normalizes confirmed VCD split declarations into one ordinary signal before publishing the hierarchy. The parts must share an exact name and owning scope, have compatible metadata, and cover contiguous, non-overlapping, consistently directed ranges. For scalar parts, higher indices occupy more significant bits. Independent aliases remain available. Other formats retain their Ondas declarations without WavePeek reconstruction. If some parts have observations, missing parts contribute X; if all parts are missing, the whole signal and its projections are missing.

## Ambiguous declarations

Distinct source declarations can have the same exact path. Listings omit an ambiguous path with `WPK-W0005`, and explicit selection fails without choosing a candidate. This rule applies to every format. Shared alias histories do not merge declarations or make different paths ambiguous. Canonical quoting prevents collisions caused by flattening different component sequences.
