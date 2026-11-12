# Path and scope resolution

WavePeek formats canonical paths from exact Ondas hierarchy components. A path is a sequence of exact hierarchy components separated by dots. Components containing dots, brackets, commas, quotes, backslashes, or whitespace use JSON string syntax:

```text
top.cpu.valid
top."a.b".c
top."a.b.c"
top."literal[7:0]"
```

The middle two paths select different declarations. Brackets inside a quoted component are literal name characters. Quoting affects spelling, not identity. Legacy spellings that flatten components or remove literal suffixes are not aliases. Listing paths can be reused in selections and expressions. Scope depth counts components, regardless of punctuation inside a name.

Use shell single quotes to preserve path quotes. Commas separate `--signals` and CLI `--payload` entries only outside quoted components. Order and duplicates are preserved.

```sh
wavepeek value --waves dump.vcd --at 10ns \
  --signals 'top."literal[7:0]",top.literal'
```

## Relative paths

`signal`, `value`, `change`, `property`, and `extract` support `--scope`. With `--scope top`, both `cpu.valid` and `top.cpu.valid` resolve inside that scope. Names without the exact selected scope's component prefix are relative to it. Use `signal --scope ''` for root-level declarations; add `--recursive` to include signals in child scopes.

```sh
wavepeek value --waves dump.vcd --at 10ns \
  --scope 'top."a.b"' --signals c
```

Human output supports `--abs` for canonical paths. Machine path fields are always canonical; `relative_path` omits the selected scope.

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

`value --signals`, `change --signals`, and `extract generic` payloads may append one flat static decimal `[msb:lsb]` projection. Resolution first tries the complete name, then a trailing projection outside quoted or escaped components. `[n]` names a declaration; use `[n:n]` for one projected bit. Bit zero is the rightmost normalized sampled bit. If a literal declaration conflicts with a projection spelling, the projection quotes its base component, such as `top."data"[3:0]`, so the returned selector remains unambiguous.

```sh
wavepeek value --waves dump.vcd --at 10ns \
  --signals 'top.data[7:4],top."memory[0]"[3:0]'
```

Names such as `memory[0]` do not create scopes or prove array structure. WavePeek preserves the hierarchy provided by Ondas. Declarations sharing an exact name and owning scope form a composite vector only when explicit ranges establish a contiguous, non-overlapping, consistently directed layout. When all parts are scalar ranges, higher indices occupy the more significant bits. Ondas reads the concatenation and its projections. If some parts have observations, missing parts contribute X; if all parts are missing, the whole composition and its projections are missing.

## Ambiguous declarations

Distinct source declarations can have the same exact path. When confirmed ranges cannot form one vector, listings omit that path with `WPK-W0005`, and explicit selection fails without choosing a candidate. This rule applies to every format. Shared alias histories do not merge declarations or make different paths ambiguous. Canonical quoting prevents collisions caused by flattening different component sequences.
