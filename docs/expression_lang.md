# Expression Language Contract

This document captures the intended end-state semantics of the `wavepeek`
expression language used by `property` and by `change` through `--on` and
`--eval`. It describes the target contract, not rollout order.

The contract is based on IEEE 1800-2023 SystemVerilog and aims to preserve
SystemVerilog-compatible syntax and semantics wherever practical for dump-based
waveform values. This document is a whitelist specification: only the syntax
and semantics explicitly described here are supported; anything not described
here is out of scope.

## 1. Event Expressions

This section defines the event-language surface. It follows the
semantics of SystemVerilog clocking events - the forms ordinarily written inside
`@(...)` - but omits the outer `@` and parentheses.

### 1.1 Surface Forms

Event expressions support these forms:

- wildcard event: `*`
- named event: `name`
- edge events: `posedge name`, `negedge name`, `edge name`
- unions: `event or event`, `event, event`
- gated events: `event iff logical_expr`

`*` denotes any change in the command-defined tracked set. `change` binds that
set to the resolved `--signals`; `property` binds it to the signals referenced
by `--eval`.

### 1.2 Names and Resolution

A name may appear as a simple signal, a hierarchical path, or another canonical
dump-derived signal token accepted by the command surface. Names must resolve to
signals; unresolved names are errors.

### 1.3 Basic Event Semantics

Named event `name` means any value change of that signal. Wildcard event `*`
means any value change in the tracked set defined by the command. Edge events
use the previous sampled value strictly before the candidate timestamp and the
current sampled value at that timestamp.

Only the least-significant bit participates in edge classification. Nine-state
waveform values `h`, `u`, `w`, `l`, and `-` are normalized to `x` before edge
classification.

Edge classification follows SystemVerilog clocking-event semantics:

- `posedge` matches `0 -> 1/x/z` and `x/z -> 1`
- `negedge` matches `1 -> 0/x/z` and `x/z -> 0`
- `edge` matches either `posedge` or `negedge`

If no previous sampled value exists strictly before the timestamp, no edge is
detected at that timestamp.

### 1.4 Unions and `iff`

Union is logical OR over event terms. `or` and `,` are exact synonyms. If
multiple terms match at the same timestamp, they select the same candidate
timestamp rather than distinct duplicate events.

`iff` attaches only to the immediately preceding event term, not to the entire
union. For example, `negedge clk iff rstn or ready` means
`(negedge clk iff rstn) or ready`.

`logical_expr` uses the Boolean Expression language defined in section `2`.
Parentheses are part of that `logical_expr` syntax; Event Expressions do not
define an independent parenthesized grouping form.

### 1.5 Precedence and Grouping

Event expressions have one composition operator: union. `iff` binds tighter than
union and applies to a single preceding event term. `or` and `,` have equal
precedence and associate left-to-right.

This gives the practical precedence order:

1. basic event forms: `*`, `name`, `posedge name`, `negedge name`, `edge name`
2. gated event term: `event iff logical_expr`
3. union: `event or event`, `event, event`

### 1.6 Grammar Sketch

```text
event_expr ::= event_term { ("or" | ",") event_term }

event_term ::= basic_event
             | basic_event "iff" logical_expr

basic_event ::= "*"
              | operand_reference
              | "posedge" operand_reference
              | "negedge" operand_reference
              | "edge" operand_reference
```

Notes:

- `operand_reference` follows the same name-resolution rules as elsewhere in
  this document, with command-specific scope handling.
- `iff` binds only to the immediately preceding `basic_event`.
- `or` and `,` are exact synonyms.

## 2. Boolean Expressions

This section defines the value-expression language over sampled dump values:
operand types, operand forms, casts, implicit conversions, operators, and
grammar.

Boolean expressions follow IEEE 1800-2023 SystemVerilog as closely as
practical, with a small dump-oriented surface specialization:
- inline vector cast targets such as `bit[N]`, `logic[N]`, and `signed logic[N]` (for example, `logic[8]'(data)` or `signed bit[16]'(count)`);
- dump-derived operand-type casts such as `type(operand_reference)'(expr)` (for example, `type(fsm_state)'(next_value)`);
- dump-derived enum label
references such as `type(enum_operand_reference)::LABEL` (for example,
`type(fsm_state)::BUSY`).

### 2.1 Operand Types

Boolean expressions operate on typed operands. Operands may carry 2-state or
4-state values internally; final property decisions are reduced to 2-state.
The supported groups match the values and type metadata recoverable from
VCD/FST waveform dumps.

- `bit-vector`
  - `bit` - width `1`, sign `unsigned`, states `2-state`
  - `logic` - width `1`, sign `unsigned`, states `4-state`
  - `bit[N]` - width `N > 0`, sign `unsigned`, states `2-state`
  - `logic[N]` - width `N > 0`, sign `unsigned`, states `4-state`
  - `signed bit[N]` - width `N > 0`, sign `signed`, states `2-state`
  - `signed logic[N]` - width `N > 0`, sign `signed`, states `4-state`
  - `unsigned bit[N]` - width `N > 0`, sign `unsigned`, states `2-state`
  - `unsigned logic[N]` - width `N > 0`, sign `unsigned`, states `4-state`

- `integer-like`
  - `byte` - width `8`, sign `signed`, states `2-state`
  - `shortint` - width `16`, sign `signed`, states `2-state`
  - `int` - width `32`, sign `signed`, states `2-state`
  - `longint` - width `64`, sign `signed`, states `2-state`
  - `integer` - width `32`, sign `signed`, states `4-state`
  - `time` - width `64`, sign `unsigned`, states `4-state`

- `enum`
  - `enum` - width `N > 0`, sign `unsigned`, states `2-state` or `4-state`
  - enum values always carry an underlying bit-vector value and may carry an
    enum label

- `real`
  - `real` - width `64-bit floating-point`, sign `n/a`, states `n/a`

- `string`
  - `string` - width `n/a`, sign `n/a`, states `n/a`
  - modeled as a sequence of 8-bit bytes

- `event`
  - `event` - not a plain value operand; available only through `.triggered`
  - `event.triggered` - integral `bit` (width `1`, sign `unsigned`, states `2-state`)
  - raw `event` is not a value expression and does not
    participate directly in casts, implicit conversions, operators, selection,
    concatenation, or replication

### 2.2 Operand Forms

This section lists the leaf forms that can appear as direct operands.

#### 2.2.1 Literals

- `integral literal`
  - unsized decimal integer, for example `12`
  - unsized based integer, for example `'d12`, `'sd12`, `'hff`
  - sized based integer, for example `16'd12`, `8'shff`
  - decimal literals are `signed`
  - based literals are `unsigned` unless the base specifier includes `s`
  - sized based literals take width from the size prefix
  - based integral literals may contain `x` and `z`
  - integral literals are self-determined
  - unary `-` is an operator, not part of the literal itself

- `real literal`
  - produces `real`; self-determined

- `string literal`
  - produces `string`; self-determined

#### 2.2.2 Primary Expressions

- `operand reference`
  - a resolved signal reference uses the operand type defined by dump metadata

- `literal`
  - any supported literal form

- `parenthesized expression`
  - `(expr)` preserves the inner expression value and type

- `static cast`
  - `type'(expr)`

- `enum label reference`
  - `type(enum_operand_reference)::LABEL`
  - resolves `LABEL` within the enum type recovered from the referenced operand
  - produces an enum value of that enum type
  - invalid if the referenced operand is not enum-typed or if the label is
    absent

- `selection`
  - `expr[idx]`
  - `expr[msb:lsb]`
  - `expr[base +: width]`
  - `expr[base -: width]`

- `concatenation`
  - `{a, b, c}`

- `replication`
  - `{N{expr}}`

- `member-like primary`
  - `.triggered` is the only supported member-like primary form
  - valid only when applied to a raw event operand reference
  - chaining `.triggered` is invalid

### 2.3 Type Casts

This section defines explicit casts only. Implicit conversions and operator
coercion are specified separately.

#### 2.3.1 Cast Syntax

Casts use the SystemVerilog-style form `type'(expr)`.

#### 2.3.2 Supported Cast Targets

Supported cast targets are explicitly written `bit-vector`,
`integer-like`, `real`, and `string` types; signedness-only targets
`signed` and `unsigned`; and recovered operand-type targets through
`type(operand_reference)`. Enum labels are referenced through
`type(enum_operand_reference)::LABEL`.

#### 2.3.3 General Cast Rules

`type'(expr)` produces a value of the target type. Casts may change width,
signedness, state domain, and operand kind. Integral casts use deterministic
bit-level conversion.

#### 2.3.4 Signedness-Only Casts

`signed'(expr)` and `unsigned'(expr)` are width-preserving reinterpret casts.
They do not resize the operand or change the 2-state/4-state domain. They are
valid only for integral operands (`bit-vector`, `integer-like`, `enum`) and
invalid for `real`, `string`, and `event`.

#### 2.3.5 Integral Resize Rules

When an integral cast changes width, a wider signed source is sign-extended and
a wider unsigned source is zero-extended. A narrower target keeps the
least-significant `N` bits and truncates higher bits. After resize, the result
signedness is defined by the target type.

#### 2.3.6 State-Domain Rules

When casting between integral types, casting to a 4-state target preserves `x`
and `z`, while casting to a 2-state target converts `x -> 0` and `z -> 0`.

#### 2.3.7 Inline Vector Types

- `bit[N]` is an unsigned 2-state bit-vector of width `N`.
- `logic[N]` is an unsigned 4-state bit-vector of width `N`.
- `signed bit[N]` is a signed 2-state bit-vector of width `N`.
- `signed logic[N]` is a signed 4-state bit-vector of width `N`.
- `unsigned bit[N]` and `unsigned logic[N]` are allowed for symmetry and
  explicitness.
- `bit` is equivalent to `bit[1]`.
- `logic` is equivalent to `logic[1]`.

#### 2.3.8 Operand-Type Casts

`type(operand_reference)'(expr)` casts `expr` to the recovered operand type of
`operand_reference`. It is the primary way to cast into enum types recovered
from dump metadata. Example: `type(fsm_state)'(value)`.

#### 2.3.9 Enum Label References

`type(enum_operand_reference)::LABEL` references a label in the enum type
recovered from the referenced operand. It is a primary expression, not a cast
target, and produces a value of that enum type. It is invalid if the referenced
operand does not resolve to an enum type or if the label is not present in that
enum type.

#### 2.3.10 Enum Casts

Enum values are cast through their underlying bit-vector representation.
Casting to an enum type applies the target enum width and label set. If the
resulting bit pattern matches a declared enum label, the result carries that
label. Otherwise, it remains enum-typed but has no label.

#### 2.3.11 Real Casts

`real'(expr)` is a numeric cast, not a bit reinterpret cast. Casting from an
integral source to `real` first evaluates the source in its own self-determined
width, signedness, and state domain, then converts the resulting numeric value
to `real`. Such casts are invalid if the source contains `x` or `z`. Casting
from `real` to an integral target truncates toward zero, then applies the
target integral type's normal resize, signedness, and state-domain rules.
`signed'(expr)` and `unsigned'(expr)` are invalid for `real`.

#### 2.3.12 String Casts

Explicit string casts are intentionally narrow in this contract:
`string'(expr)` is supported only as an identity cast from `string` to
`string`; all other explicit casts to or from `string` are invalid.

#### 2.3.13 Event Casts

By the operand rules above, raw `event` is not castable as a plain value. After
`.triggered`, the result is a `bit` and follows the ordinary integral cast
rules of this section.

### 2.4 Implicit Conversions and Common-Type Rules

This section defines conversions applied by operator semantics when no explicit
cast is written. Except where an operator defines self-determined operands,
operands are converted to an operator common type before evaluation.

#### 2.4.1 Boolean Context

Boolean context applies to logical operators such as `!`, `&&`, `||`, and to
the condition operand of `?:`. For the integral family (`bit-vector`,
`integer-like`, `enum`), definitely zero is false, definitely non-zero is true,
and anything else is `x`. For `real`, `0.0` is false and any non-zero value is
true. In this contract, true `string` values do not implicitly convert in
boolean context. Boolean-context conversion produces a temporary 1-bit truth
value and does not otherwise change the operand type.

#### 2.4.2 Integral Common Type

The integral family consists of `bit-vector`, `integer-like`, and `enum`.
Within that family, width follows the operator's width rule or, if none is
defined, `max(operand widths)`; sign is `unsigned` if any participating operand
is `unsigned`, otherwise `signed`; and states are `4-state` if any
participating operand is `4-state`, otherwise `2-state`. When extending to the
common width, use sign extension for `signed` and zero extension for
`unsigned`. Conversion to a `2-state` common type maps `x -> 0` and `z -> 0`;
conversion to a `4-state` common type preserves `x` and `z`.

#### 2.4.3 Enum Participation

`enum` participates in implicit conversions through its underlying bit-vector
representation. For generic mixed-type common-type resolution, enum labels do
not affect width, sign, or state-domain selection. If both arms of `?:` have
the same enum type, the result preserves that enum type. In all other
mixed-type implicit conversions involving `enum`, the value is treated as an
integral operand and labels are not preserved by that conversion step.

#### 2.4.4 Real Common Type

`real` forms a separate numeric family. If an operator allows mixed `integral`
and `real` operands, the common type is `real`. Before conversion to `real`, an
integral operand is evaluated in its own self-determined width, signedness, and
state domain. Implicit conversion from `integral` to `real` is invalid if the
integral value contains `x` or `z`. `real` does not implicitly convert to
`string`, and `string` does not implicitly convert to `real`.

#### 2.4.5 String Restrictions

True `string` values do not participate in generic numeric common-type
resolution. Equality on `string` is defined separately as an operator-specific
rule. Mixed `string` with `integral` or `real` is invalid unless a later
operator-specific rule says otherwise. `?:` with both arms of type `string`
produces `string`; with one `string` arm and one non-`string` arm, it is
invalid unless a later operator-specific rule says otherwise.

#### 2.4.6 Result-Type Guidance

Logical operators consume boolean-context conversion and produce a 1-bit
4-state integral result. Operators over the integral family follow the
operator-specific width rule under the integral common-type model above.
Operators mixing `integral` and `real` produce `real` when the operator admits
mixed numeric operands. `?:` uses a self-determined condition and applies
common-type resolution only to its two result arms.

### 2.5 Operators

#### 2.5.1 Supported Operator Families

- selection: `expr[idx]`, `expr[msb:lsb]`, `expr[base +: width]`,
  `expr[base -: width]`
- logical: `!`, `&&`, `||`
- bitwise: `~`, `&`, `|`, `^`, `^~`, `~^`
- reduction: unary `&`, `~&`, `|`, `~|`, `^`, `^~`, `~^`
- exponentiation: `**`
- arithmetic: unary `+`, unary `-`, binary `+`, `-`, `*`, `/`, `%`
- shifts: `<<`, `>>`, `<<<`, `>>>`
- comparisons: `<`, `<=`, `>`, `>=`
- equalities: `==`, `!=`, `===`, `!==`, `==?`, `!=?`
- conditional: `?:`
- membership: `inside`
- concatenation: `{a, b, c}`
- replication: `{N{expr}}`

#### 2.5.2 Operator Matrix

| Family | Operators | Allowed operands | Conversion rule | Result |
|---|---|---|---|---|
| selection | `[]`, `[msb:lsb]`, `[base +: width]`, `[base -: width]` | packed `integral` | select indices are self-determined | unsigned bit-vector |
| logical | `!`, `&&`, `\|\|` | `integral`, `real` | boolean-context conversion | 1-bit unsigned 4-state |
| bitwise | `~`, `&`, `\|`, `^`, `^~`, `~^` | `integral` | integral common type | integral |
| reduction | unary `&`, `~&`, `\|`, `~\|`, `^`, `^~`, `~^` | `integral` | operand self-determined | 1-bit unsigned 4-state |
| exponentiation | `**` | `integral`, `real` | exponent is self-determined; result uses exponentiation rules | integral or `real` |
| arithmetic | unary `+`, unary `-`, binary `+`, `-`, `*`, `/`, `%` | `integral`, `real` | integral common type or promote to `real` | integral or `real` |
| shifts | `<<`, `>>`, `<<<`, `>>>` | `integral` | lhs integral result width; rhs self-determined | integral |
| comparisons | `<`, `<=`, `>`, `>=` | `integral`, `real` | integral compare or promote to `real` | 1-bit unsigned 4-state |
| equalities | `==`, `!=`, `===`, `!==`, `==?`, `!=?` | `==`, `!=`: `integral`, `real`, `string`/`string`; `===`, `!==`, `==?`, `!=?`: `integral` | operator-specific | 1-bit unsigned 4-state |
| conditional | `?:` | condition: `integral`, `real`; arms: common-type-compatible | condition self-determined; arms use common-type rules | common type of arms |
| inside | `inside` | lhs `integral`; rhs integral expressions and inclusive integral ranges `[lo:hi]` | item match uses wildcard-equality semantics | 1-bit unsigned 4-state |
| concatenation | `{...}` | `integral` | operands self-determined | unsigned bit-vector |
| replication | `{N{expr}}` | `integral` | `N` is a constant integer expression | unsigned bit-vector |

In this matrix, `integral` means the integral family defined earlier:
`bit-vector`, `integer-like`, and `enum`.

In this matrix, packed `integral` means an integral value treated as a packed
bit-vector, including derived packed values such as the result of selection,
concatenation, or replication. Selection of scalar integral values is still
invalid.

#### 2.5.3 Selection Operators

Selection supports bit-select `expr[idx]`, non-indexed part-select
`expr[msb:lsb]`, and indexed part-select `expr[base +: width]` /
`expr[base -: width]`. In non-indexed part-select, `msb` and `lsb` are
constant integer expressions. In indexed part-select, `width` is a positive
constant integer expression and `base` is an integral expression evaluated at
the current sample. Selection applies only to packed `integral` values;
selection of scalar integral values, `real`, or `string` is invalid.

Bit-select produces an unsigned `bit-vector` of width `1`. Part-select produces
an unsigned `bit-vector` whose width is the selected width. Selection never
preserves `integer-like` or `enum` identity; it always produces a plain
bit-vector result. For valid reads, the result preserves the source 2-state or
4-state domain. For bit-select, an invalid index yields `x` for a 4-state
source and `0` for a 2-state source. For part-select, a fully out-of-range read
yields all-`x`; an `x` or `z` select base or index yields all-`x`; and a
partially out-of-range read yields source bits for the in-range region and `x`
for the out-of-range region. Invalid part-select reads therefore produce a
4-state result even when the source is 2-state.

#### 2.5.4 Logical Operators

Logical operators apply boolean-context conversion to each operand. They
support `integral` and `real` operands, but not true `string` values. Results
are 1-bit unsigned 4-state values. `&&` and `||` short-circuit, and `!` maps
ambiguous truth to `x`.

#### 2.5.5 Bitwise Operators

Bitwise operators accept only `integral` operands. Binary bitwise operators use
the integral common type, and unary `~` preserves operand width. `enum`
operands participate through their underlying bit-vector values. Full 4-state
bitwise truth-table semantics are intended, including preserving controlling
values such as `0 & x -> 0` and `1 | x -> 1`.

#### 2.5.6 Reduction Operators

Reduction operators accept only `integral` operands. The operand is
self-determined. The result is a 1-bit unsigned 4-state value. `x` and `z`
participate through the reduction truth tables and may produce `x`.

#### 2.5.7 Exponentiation Operator

Exponentiation accepts `integral` and `real` operands. If either operand is
`real`, the result is `real`. Otherwise, exponentiation follows the ordinary
integral sizing and signedness rules of this section, and the exponent operand
is self-determined. Special cases follow SystemVerilog-style rules: exponent
`0` yields `1`, and integral `0 ** negative` yields all-`x`.

#### 2.5.8 Arithmetic Operators

Arithmetic operators support `integral` and `real` operands. Pure integral
arithmetic uses the integral common type; mixed `integral` and `real`
arithmetic promotes to `real`. For integral arithmetic, any `x` or `z` bit in
an operand makes the result all-`x`. Division truncates toward zero.
Divide-by-zero and modulo-by-zero produce all-`x` for integral arithmetic.

#### 2.5.9 Shift Operators

Shift operators accept only `integral` operands. Result width is the
left-operand width. The right operand is self-determined and treated as
unsigned. `<<`, `>>`, and `<<<` fill vacated bits with `0`; `>>>` fills them
with `0` for unsigned results and with the left operand MSB for signed results.
If the right operand contains `x` or `z`, the result is all-`x`.

#### 2.5.10 Comparison Operators

Comparison operators support `integral` and `real` operands. Mixed `integral`
and `real` comparison promotes the integral operand to `real`. The result is a
1-bit unsigned 4-state value. For integral comparison, any `x` or `z` bit in
either operand yields `x`.

#### 2.5.11 Equality Operators

`==` and `!=` support `integral`, `real`, and `string` operands. `===` and
`!==` support `integral` operands, and `==?` and `!=?` also support `integral`
operands. Mixed `integral` and `real` equality promotes the integral operand to
`real`. For integral operands, `==` and `!=` may produce `x`, while `===` and
`!==` always produce a known 1-bit result. `==?` and `!=?` treat `x` and `z`
in the right operand as wildcards; `x` and `z` in the left operand are not
wildcards.

For `string`, only `==` and `!=` are supported, and only when both operands are
`string`. String equality compares the full string values for exact match and
does not use numeric coercion, width extension, padding, truncation, or
wildcard semantics. String literals participate in string equality as `string`
values. String equality always returns a known `0` or `1`.

#### 2.5.12 Conditional Operator

The condition operand of `?:` is self-determined and evaluated in boolean
context. The two result arms use the common-type rules defined earlier. If both
arms are the same enum type, the result preserves that enum type. If the
condition is `x` or `z` and both arms are integral, the result is a bitwise
merge at the arms' common integral type. If the condition is `x` or `z` and
both arms are the same enum type, the result is that enum type with a merged
underlying value; the label is present only if the merged bit pattern matches a
declared label.

#### 2.5.13 Inside Operator

`inside` is supported in a narrowed form. The left operand must be `integral`.
Right-hand set items may be integral expressions or inclusive integral ranges
written as `[lo:hi]`. Item matching uses wildcard-equality semantics equivalent
to `==?`. If no set item matches, but at least one candidate match is `x`, the
result is `x`. Advanced `inside` forms are not supported in the initial target
semantics: open-range `$` bounds, `+/-`, `+%-`, unpacked-array matching, and
scan-order dependent behavior.

#### 2.5.14 Concatenation and Replication

Concatenation and replication accept only `integral` operands and produce plain
unsigned bit-vector results. They never preserve `integer-like` or `enum`
identity. Concatenation result width is the sum of operand widths;
replication result width is `N * operand_width`. The result is `4-state` if any
operand is `4-state`; otherwise it is `2-state`. `x` and `z` bits are
preserved. Unsized constants are not allowed in integral concatenation.
Replication multiplier `N` must be a non-negative constant integer expression.
Concatenation and replication results may themselves be selected.

### 2.6 Precedence and Associativity

SystemVerilog-style precedence is followed for the supported operator
families.

From highest precedence to lowest precedence:

1. primary and postfix forms
   - `(expr)`
   - `type'(expr)`
   - `type(enum_operand_reference)::LABEL`
   - concatenation: `{...}`
   - replication: `{N{...}}`
   - `.triggered`
   - bit-select and part-select: `[]`, `[msb:lsb]`, `[base +: width]`,
     `[base -: width]`
2. unary prefix operators
   - unary `+`, unary `-`
   - `!`
   - `~`
   - unary reduction `&`, `~&`, `|`, `~|`, `^`, `^~`, `~^`
3. exponentiation
   - `**`
4. multiplicative
   - `*`, `/`, `%`
5. additive
   - binary `+`, `-`
6. shifts
   - `<<`, `>>`, `<<<`, `>>>`
7. relational and membership
   - `<`, `<=`, `>`, `>=`, `inside`
8. equality
   - `==`, `!=`, `===`, `!==`, `==?`, `!=?`
9. bitwise AND
   - `&`
10. bitwise XOR/XNOR
   - `^`, `^~`, `~^`
11. bitwise OR
   - `|`
12. logical AND
   - `&&`
13. logical OR
   - `||`
14. conditional
   - `?:`

Associativity rules:

- Postfix forms chain left-to-right.
- Prefix unary operators bind to the following operand.
- Binary infix operators associate left-to-right.
- `?:` associates right-to-left.

Guidance:

- Parentheses should be used whenever mixing `?:`, `inside`, or concatenation
  and replication with other operators unless the intended grouping is obvious.
- `type'(expr)` is treated as a primary expression and binds tighter than any
  infix operator.
- `type(enum_operand_reference)::LABEL` is treated as a primary expression and
  binds tighter than any infix operator.
- `.triggered` is treated as a dedicated postfix/member form with the same
  tight binding as other primary postfix operations.

### 2.7 Parser-Level Grammar Sketch

This is a parser-oriented sketch of the supported surface syntax, not a full
lexical grammar. Semantic validation still enforces rules such as
constant-expression requirements, type restrictions, and packed-only selection.

```text
expr ::= conditional_expr

conditional_expr ::= logical_or_expr
                   | logical_or_expr "?" expr ":" conditional_expr

logical_or_expr ::= logical_and_expr { "||" logical_and_expr }
logical_and_expr ::= bitwise_or_expr { "&&" bitwise_or_expr }
bitwise_or_expr ::= bitwise_xor_expr { "|" bitwise_xor_expr }
bitwise_xor_expr ::= bitwise_and_expr { ("^" | "^~" | "~^") bitwise_and_expr }
bitwise_and_expr ::= equality_expr { "&" equality_expr }

equality_expr ::= relational_expr
                { ("==" | "!=" | "===" | "!==" | "==?" | "!=?")
                  relational_expr }

relational_expr ::= shift_expr
                  { ("<" | "<=" | ">" | ">=") shift_expr
                  | "inside" inside_set }

shift_expr ::= additive_expr { ("<<" | ">>" | "<<<" | ">>>") additive_expr }
additive_expr ::= multiplicative_expr { ("+" | "-") multiplicative_expr }
multiplicative_expr ::= power_expr { ("*" | "/" | "%") power_expr }
power_expr ::= unary_expr { "**" unary_expr }

unary_expr ::= postfix_expr
             | ("+" | "-" | "!" | "~"
             | "&" | "~&" | "|" | "~|" | "^" | "^~" | "~^") unary_expr

postfix_expr ::= primary_expr { postfix_suffix }

postfix_suffix ::= "[" expr "]"
                 | "[" constant_expr ":" constant_expr "]"
                 | "[" expr "+:" constant_expr "]"
                 | "[" expr "-:" constant_expr "]"
                 | ".triggered"

primary_expr ::= operand_reference
               | literal
               | "(" expr ")"
               | cast_expr
               | enum_label_expr
               | concatenation
               | replication

cast_expr ::= type_reference "'" "(" expr ")"
enum_label_expr ::= "type" "(" operand_reference ")" "::" enum_label_identifier

concatenation ::= "{" expr { "," expr } "}"
replication ::= "{" constant_expr "{" expr "}" "}"

inside_set ::= "{" inside_item { "," inside_item } "}"
inside_item ::= expr | "[" expr ":" expr "]"

literal ::= integral_literal | real_literal | string_literal

type_reference ::= bit_vector_type
                 | integer_like_type
                 | "real"
                 | "string"
                 | "signed"
                 | "unsigned"
                 | "type" "(" operand_reference ")"

bit_vector_type ::= [ ("signed" | "unsigned") ]
                    ("bit" | "logic") [ "[" positive_integer "]" ]

integer_like_type ::= "byte"
                    | "shortint"
                    | "int"
                    | "longint"
                    | "integer"
                    | "time"
```

Notes:

- `operand_reference` may be hierarchical, for example `top.cpu.data`.
- A trailing `.triggered` is a reserved postfix form and is not part of an
  operand reference.
- `.triggered` is semantically valid only on a raw event operand reference;
  broader parses are rejected during semantic validation.
- In `type(enum_operand_reference)::LABEL`, `enum_operand_reference` must
  resolve to an enum-typed operand reference and `LABEL` must be one of its
  declared labels.
- `constant_expr` means an expression later validated as a constant integer
  expression in the supported subset.
- `inside` ranges use inclusive bounds.
- `bit` and `logic` without an explicit `[N]` suffix mean width `1`.
