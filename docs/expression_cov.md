# Expression Language Coverage Checklist

This checklist is derived only from `docs/expression_lang.md`.
Each numbered point is a future test target. Many points are intentionally
parameterized matrices that should expand into multiple concrete cases.
Overlap is intentional where the contract states the same rule in more than one
chapter.

Because the contract is a whitelist specification, each section should also
spawn representative negative tests for nearby unsupported syntax or semantics,
even when that rejection is called out only indirectly.

Maintenance policy: this checklist is append-only. Preserve historical
numbering, append new items only at the end, and use bracketed source-section
tags such as `[2.5.3]` to locate the originating contract clause instead of
inserting new items back into earlier chapters.

1. [1.1] Wildcard event surface form
   - Assertion: `*` is a valid event expression surface form.
   - Verification: Parse and evaluate `*` in a command context with a non-empty tracked set.
   - Expected result: Parsing succeeds, and the event is eligible whenever any tracked signal changes.

2. [1.1] Named event surface form
   - Assertion: `name` is a valid event expression surface form.
   - Verification: Parse and evaluate a resolved signal name as the whole event expression.
   - Expected result: Parsing succeeds, and the event follows named-change semantics for that signal.

3. [1.1] Edge event surface forms
   - Assertion: `posedge name`, `negedge name`, and `edge name` are all valid surface forms.
   - Verification: Parse and evaluate one expression for each edge keyword against a resolved signal.
   - Expected result: All three forms are accepted and use edge-classification semantics rather than plain change semantics.

4. [1.1] Union surface forms
   - Assertion: `event or event` and `event, event` are both valid union forms.
   - Verification: Parse equivalent unions written once with `or` and once with `,`.
   - Expected result: Both spellings are accepted.

5. [1.1] Gated event surface form
   - Assertion: `event iff logical_expr` is a valid gated-event form.
   - Verification: Parse an event term gated by a boolean expression, for example `posedge clk iff ready && !stall`.
   - Expected result: Parsing succeeds and the gate is evaluated in boolean context.

6. [1.1] Wildcard tracked-set binding for `change`
   - Assertion: In `change`, `*` denotes any change in the command-defined tracked set resolved from `--signals`.
   - Verification: Run `change` with `--signals` naming some signals, then toggle both tracked and untracked signals.
   - Expected result: `*` reacts only to changes in the resolved `--signals` set.

7. [1.1] Wildcard tracked-set binding for `property`
   - Assertion: In `property`, `*` denotes any change in the set of signals referenced by `--eval`.
   - Verification: Run `property` with an `--eval` expression that references a subset of available signals, then toggle referenced and unreferenced signals.
   - Expected result: `*` reacts only to changes in signals referenced by `--eval`.

8. [1.2] Simple signal name resolution
   - Assertion: A simple signal name is a valid event operand reference when it resolves.
   - Verification: Use an event expression that names a simple signal token present in the dump.
   - Expected result: Resolution succeeds and evaluation uses that signal.

9. [1.2] Hierarchical name resolution
   - Assertion: A hierarchical signal path is a valid event operand reference when it resolves.
   - Verification: Use an event expression with a hierarchical path such as `top.cpu.clk`.
   - Expected result: Resolution succeeds and evaluation uses that hierarchical signal.

10. [1.2] Canonical dump-token resolution
   - Assertion: Another canonical dump-derived signal token accepted by the command surface is also valid in event expressions.
   - Verification: Use an event expression with a non-simple canonical token that the command surface already accepts.
   - Expected result: Resolution succeeds and the token is treated exactly like any other signal reference.

11. [1.2] Unresolved or non-signal names are errors
   - Assertion: Names must resolve to signals; unresolved names or non-signal targets are errors.
   - Verification: Evaluate an event expression with an unknown token and, separately, with a token that is not a signal if such a fixture exists.
   - Expected result: Semantic validation fails with an error instead of silently ignoring the term.

12. [1.3] Named event means any value change
   - Assertion: A named event matches any value change of that signal.
   - Verification: Use a signal with several timestamps, including repeated samples with no change and samples with changes.
   - Expected result: The event matches every timestamp where the signal value changes and no timestamps where it does not.

13. [1.3] Wildcard event means any tracked-set change
   - Assertion: `*` matches any value change in the tracked set.
   - Verification: Use a tracked set with multiple signals and create changes on different members at different timestamps.
   - Expected result: The event matches each timestamp where any tracked signal changes.

14. [1.3] Edge sampling uses the previous sample strictly before the candidate timestamp
   - Assertion: Edge events compare the current sampled value at the candidate timestamp with the previous sampled value strictly before that timestamp.
   - Verification: Use a signal with known samples at `t0 < t1 < t2`, and evaluate edge events at `t1` and `t2`.
   - Expected result: Each edge decision uses the value from the immediately preceding sampled timestamp, never a same-timestamp pseudo-previous value.

15. [1.3] Only the least-significant bit participates in edge classification
   - Assertion: Edge classification depends only on the least-significant bit.
   - Verification: Use a multi-bit signal where higher bits change while the least-significant bit stays constant, then compare named-event behavior with edge-event behavior.
   - Expected result: The named event matches the value change, but `posedge`/`negedge`/`edge` do not match unless the least-significant bit changes classification.

16. [1.3] Nine-state `h/u/w/l/-` normalization before edge classification
   - Assertion: `h`, `u`, `w`, `l`, and `-` are normalized to `x` before edge classification.
   - Verification: Parameterize transitions that include each of `h`, `u`, `w`, `l`, and `-`, and compare them with equivalent transitions where those values are replaced by `x`.
   - Expected result: Edge matching is identical to the corresponding `x`-based case for every normalized value.

17. [1.3] `posedge` classification matrix
   - Assertion: `posedge` matches `0 -> 1/x/z` and `x/z -> 1`.
   - Verification: Parameterize transitions `0->1`, `0->x`, `0->z`, `x->1`, and `z->1` on the least-significant bit.
   - Expected result: All listed transitions match `posedge`, and control transitions outside that matrix do not match this rule.

18. [1.3] `negedge` classification matrix
   - Assertion: `negedge` matches `1 -> 0/x/z` and `x/z -> 0`.
   - Verification: Parameterize transitions `1->0`, `1->x`, `1->z`, `x->0`, and `z->0` on the least-significant bit.
   - Expected result: All listed transitions match `negedge`, and control transitions outside that matrix do not match this rule.

19. [1.3] `edge` is the union of `posedge` and `negedge`
   - Assertion: `edge name` matches exactly when either `posedge name` or `negedge name` would match.
   - Verification: Evaluate all positive-edge and negative-edge transition cases plus several non-edge cases.
   - Expected result: `edge` matches every `posedge` and `negedge` case and no other cases.

20. [1.3] No previous sample means no edge
   - Assertion: If no previous sampled value exists strictly before the timestamp, no edge is detected.
   - Verification: Evaluate an edge event at the first sampled timestamp of a signal.
   - Expected result: No edge is reported at that first timestamp even if the current value would otherwise look like an edge destination.

21. [1.4] Union is logical OR over event terms
   - Assertion: A union matches when any member term matches.
   - Verification: Build a union with disjoint matching terms at different timestamps.
   - Expected result: The union selects every timestamp selected by any constituent term.

22. [1.4] `or` and `,` are exact semantic synonyms
   - Assertion: `or` and `,` have identical event semantics.
   - Verification: Evaluate the same union once with `or` and once with `,` over the same waveform.
   - Expected result: Both expressions produce identical candidate timestamps.

23. [1.4] Same-timestamp union matches are deduplicated
   - Assertion: If multiple union terms match at the same timestamp, they select one candidate timestamp, not duplicates.
   - Verification: Use a union where two terms are guaranteed to match on the same timestamp, such as a named event and `*` over the same signal set.
   - Expected result: The timestamp appears once in the selected event stream.

24. [1.4] `iff` gates only the immediately preceding event term
   - Assertion: `iff` attaches to one event term, not to the full union.
   - Verification: Compare `a iff cond or b` with a fixture where applying the gate to the full union would change the answer.
   - Expected result: Only `a` is gated; `b` remains ungated.

25. [1.4] Example grouping for `negedge clk iff rstn or ready`
   - Assertion: `negedge clk iff rstn or ready` means `(negedge clk iff rstn) or ready`.
   - Verification: Use a waveform where `ready` is true at timestamps where `rstn` is false and `negedge clk` also occurs.
   - Expected result: The expression still matches on `ready` timestamps because the gate does not apply to the `ready` term.

26. [1.4] `iff` uses the boolean expression language from section 2
   - Assertion: The `logical_expr` after `iff` is parsed and evaluated using the boolean-expression contract from section 2.
   - Verification: Use `iff` with a compound boolean expression that exercises logical precedence, casts, or selection.
   - Expected result: The gate expression behaves exactly like the same expression used as a standalone boolean expression.

27. [1.4] Event expressions have no independent parenthesized grouping form
   - Assertion: Parentheses belong only to `logical_expr`, not to event-expression grouping.
   - Verification: Parse `posedge clk iff (a || b)` and, separately, attempt `(posedge clk or ready)`.
   - Expected result: The parenthesized gate expression is accepted, while parenthesized event grouping is rejected.

28. [1.5] Union is the only event-composition operator
   - Assertion: Event expressions compose only through union; there is no second event-level infix operator.
   - Verification: Attempt to write event-level constructs other than union and `iff`, such as parenthesized event grouping or a second event operator.
   - Expected result: Only the documented union and `iff` forms are accepted.

29. [1.5] `iff` binds tighter than union
   - Assertion: `iff` is evaluated before `or` or `,`.
   - Verification: Use an expression such as `a or b iff cond` with a waveform where the wrong grouping would change the result.
   - Expected result: The expression behaves as `a or (b iff cond)`.

30. [1.5] `or` and `,` have equal precedence and left-to-right association
   - Assertion: Both union spellings share precedence and associate left-to-right.
   - Verification: Parse a mixed expression such as `a or b, c or d` and inspect the parser shape or use an AST-focused parser test.
   - Expected result: The parser builds a left-associated union chain with no precedence distinction between `or` and `,`.

31. [1.5] Event precedence order is basic event, then gated term, then union
   - Assertion: The practical event precedence order is `basic_event` > `event iff logical_expr` > union.
   - Verification: Use expressions whose meaning changes if the precedence is wrong, such as `posedge a iff b or c`.
   - Expected result: Behavior matches the documented precedence order.

32. [1.6] Event grammar accepts repeated unions of event terms
   - Assertion: `event_expr ::= event_term { ("or" | ",") event_term }` allows multiple unioned terms.
   - Verification: Parse a four-term expression that mixes both union spellings.
   - Expected result: Parsing succeeds and yields a flat or left-associated union chain.

33. [1.6] `event_term` is limited to `basic_event` and `basic_event iff logical_expr`
   - Assertion: A gated event term must start from a `basic_event`, not an arbitrary union.
   - Verification: Attempt `a iff cond`, `posedge a iff cond`, and `(a or b) iff cond`.
   - Expected result: The first two are accepted; the grouped-union form is rejected.

34. [1.6] `basic_event` grammar matrix
   - Assertion: `basic_event` is exactly one of `*`, an operand reference, or `posedge`/`negedge`/`edge` followed by an operand reference.
   - Verification: Parse one valid example of each form.
   - Expected result: Each documented form is accepted as a `basic_event`.

35. [1.6] Edge keywords require operand references, not arbitrary expressions
   - Assertion: `posedge`, `negedge`, and `edge` accept operand references only.
   - Verification: Attempt forms such as `posedge 1'b1` or `posedge (a)`.
   - Expected result: Semantic or parse validation rejects non-reference operands after edge keywords.

36. [2.0] Inline vector cast targets are supported
   - Assertion: Inline cast targets like `bit[N]`, `logic[N]`, `signed logic[N]`, and related forms are part of the supported surface.
   - Verification: Parse representative expressions such as `logic[8]'(data)`, `bit[4]'(x)`, `signed bit[16]'(count)`, and `unsigned logic[3]'(v)`.
   - Expected result: All documented inline vector cast syntaxes are accepted.

37. [2.0] Dump-derived operand-type casts are supported
   - Assertion: `type(operand_reference)'(expr)` is a supported cast form.
   - Verification: Parse representative uses for both enum-typed and non-enum operand references.
   - Expected result: Parsing succeeds, and the target type is taken from the referenced operand metadata.

38. [2.0] Dump-derived enum label references are supported
   - Assertion: `type(enum_operand_reference)::LABEL` is a supported primary form.
   - Verification: Parse representative enum-label expressions against an enum-typed operand reference.
   - Expected result: Parsing succeeds and yields a primary expression of that enum type.

39. [2.1] Scalar `bit` and `logic` metadata
   - Assertion: `bit` is width 1 unsigned 2-state, and `logic` is width 1 unsigned 4-state.
   - Verification: Use casts and width/state-sensitive operations to observe the width, sign, and state-domain behavior of `bit` and `logic`.
   - Expected result: `bit` behaves as a 1-bit unsigned 2-state value, and `logic` behaves as a 1-bit unsigned 4-state value.

40. [2.1] Packed bit-vector family metadata matrix
   - Assertion: `bit[N]`, `logic[N]`, `signed bit[N]`, `signed logic[N]`, `unsigned bit[N]`, and `unsigned logic[N]` follow the documented width/sign/state matrix for `N > 0`.
   - Verification: Parameterize casts and operators across those six families for a few widths.
   - Expected result: Width, signedness, and 2-state/4-state behavior match the contract for every family member.

41. [2.1] Integer-like builtin metadata matrix
   - Assertion: `byte`, `shortint`, `int`, `longint`, `integer`, and `time` follow the documented width/sign/state matrix.
   - Verification: Parameterize casts and arithmetic across each integer-like type.
   - Expected result: Each builtin exposes the documented width, signedness, and state domain.

42. [2.1] Enum type metadata and label carriage
   - Assertion: `enum` values have width `N > 0`, unsigned sign, 2-state or 4-state storage, an underlying bit-vector value, and an optional label.
   - Verification: Use fixtures with both labeled and unlabeled enum values and with both 2-state and 4-state enum encodings if available.
   - Expected result: Enum values behave as integral values with optional label metadata exactly as documented.

43. [2.1] Real type metadata
   - Assertion: `real` is a 64-bit floating-point numeric type.
   - Verification: Use `real` literals, casts, and mixed numeric operators.
   - Expected result: The operand participates as a floating-point numeric value rather than a packed integral value.

44. [2.1] String type metadata
   - Assertion: `string` is a distinct non-numeric type modeled as a sequence of 8-bit bytes.
   - Verification: Use string literals and string equality cases with different lengths and contents.
   - Expected result: The value behaves as a byte sequence with string-only equality semantics, not as an integral vector.

45. [2.1] Event operand and `.triggered`
   - Assertion: Raw `event` is not a plain value operand, while `event.triggered` is an unsigned 1-bit 2-state integral value.
   - Verification: Use a raw event reference and the same reference with `.triggered` in value contexts.
   - Expected result: The raw event is not accepted as a plain value, while `.triggered` is accepted and behaves as a `bit`.

46. [2.1] Raw event is excluded from ordinary value operations
   - Assertion: Raw `event` does not participate directly in casts, implicit conversions, operators, selection, concatenation, or replication.
   - Verification: Attempt one representative use of a raw event in each forbidden category.
   - Expected result: Each use is rejected until `.triggered` is applied.

47. [2.2.1] Integral literal surface-form matrix
   - Assertion: Supported integral literals include unsized decimal integers, unsized based integers, and sized based integers.
   - Verification: Parse representative literals such as `12`, `'d12`, `'sd12`, `'hff`, `16'd12`, and `8'shff`.
   - Expected result: All documented integral literal spellings are accepted.

48. [2.2.1] Integral literal sign and width defaults
   - Assertion: Decimal literals are signed, based literals are unsigned unless the base specifier includes `s`, and sized based literals take width from the size prefix.
   - Verification: Use casts or operators that reveal sign and width differences among `12`, `'d12`, `'sd12`, `'hff`, and `8'shff`.
   - Expected result: Each literal uses the documented default signedness and width rule.

49. [2.2.1] Based integral literals may contain `x` and `z`
   - Assertion: Based literals may include unknown and high-impedance digits.
   - Verification: Parse literals such as `4'b10xz` and feed them into 4-state-sensitive operators.
   - Expected result: Parsing succeeds and the `x`/`z` digits are preserved in 4-state contexts.

50. [2.2.1] Integral literals are self-determined
   - Assertion: Integral literals keep their own width and signedness until an operator or cast applies other rules.
   - Verification: Combine literals of different widths and signs in operators whose width or sign depends on operand self-determination.
   - Expected result: Results reflect the literal's own width and signedness before common-type conversion.

51. [2.2.1] Unary minus is an operator, not part of the literal token
   - Assertion: A leading `-` belongs to unary operator parsing, not the literal grammar.
   - Verification: Parse expressions such as `-12` and `-(12)` and compare them with parser expectations for literals.
   - Expected result: The literal token is positive, and unary `-` is represented as a separate operator node.

52. [2.2.1] Real literals produce self-determined `real`
   - Assertion: A real literal produces a `real` operand and is self-determined.
   - Verification: Use real literals in mixed numeric expressions and casts.
   - Expected result: The literal participates as `real` without requiring an explicit cast.

53. [2.2.1] String literals produce self-determined `string`
   - Assertion: A string literal produces a `string` operand and is self-determined.
   - Verification: Use string literals in string equality and invalid numeric contexts.
   - Expected result: The literal behaves as `string` and is rejected where strings are not allowed.

54. [2.2.2] Operand-reference primaries use dump-recovered operand types
   - Assertion: A resolved signal reference takes its type from dump metadata.
   - Verification: Use operand references with known metadata for integral, enum, real, and string signals.
   - Expected result: Each reference behaves with the type recovered from the dump.

55. [2.2.2] Literals are valid primary expressions
   - Assertion: Any supported literal form may appear as a primary expression.
   - Verification: Use integral, real, and string literals as standalone expressions and as operands to higher-precedence operators.
   - Expected result: Each literal form is accepted wherever a primary expression is allowed.

56. [2.2.2] Parenthesized expressions preserve inner value and type
   - Assertion: `(expr)` preserves the inner expression's value and type.
   - Verification: Wrap expressions with observable width, sign, enum identity, or state-domain behavior in parentheses.
   - Expected result: Parentheses change grouping only; the value and type are otherwise unchanged.

57. [2.2.2] Static casts are valid primaries
   - Assertion: `type'(expr)` is a primary-expression form.
   - Verification: Use casts in larger expressions where primary precedence matters.
   - Expected result: The cast parses as a primary and binds tighter than infix operators.

58. [2.2.2] Enum label reference primary semantics
   - Assertion: `type(enum_operand_reference)::LABEL` resolves a label within the referenced enum type and produces a value of that enum type.
   - Verification: Compare an enum signal against a label reference of its own recovered enum type.
   - Expected result: The label reference resolves and yields an enum value of the expected type.

59. [2.2.2] Enum label reference invalid cases
   - Assertion: An enum label reference is invalid if the operand is not enum-typed or if the label is absent.
   - Verification: Attempt `type(non_enum_ref)::LABEL` and `type(enum_ref)::MISSING_LABEL`.
   - Expected result: Semantic validation rejects both cases.

60. [2.2.2] Selection primary syntax matrix
   - Assertion: The primary-expression surface includes `expr[idx]`, `expr[msb:lsb]`, `expr[base +: width]`, and `expr[base -: width]`.
   - Verification: Parse one valid example of each selection form.
   - Expected result: All four forms are accepted as postfix selections.

61. [2.2.2] Concatenation primary syntax
   - Assertion: `{a, b, c}` is a valid primary-expression form.
   - Verification: Parse a concatenation with multiple integral operands.
   - Expected result: Parsing succeeds and the expression is treated as a primary.

62. [2.2.2] Replication primary syntax
   - Assertion: `{N{expr}}` is a valid primary-expression form.
   - Verification: Parse replications with several constant multipliers, including zero if zero is intended to be supported.
   - Expected result: Parsing succeeds whenever `N` is a valid constant integer expression.

63. [2.2.2] `.triggered` is the only supported member-like primary
   - Assertion: The language supports no member-like primary form other than `.triggered`.
   - Verification: Parse `.triggered` on a raw event operand reference and attempt another member-like suffix such as `.foo`.
   - Expected result: Only `.triggered` is accepted.

64. [2.2.2] `.triggered` is restricted to raw event references and may not chain
   - Assertion: `.triggered` is valid only on a raw event operand reference, and `.triggered.triggered` is invalid.
   - Verification: Attempt `.triggered` on non-event operands and attempt chaining on an event.
   - Expected result: Non-event use and chained use are rejected.

65. [2.3.1] Cast syntax
   - Assertion: Casts use the SystemVerilog-style syntax `type'(expr)`.
   - Verification: Parse representative casts to each supported target family.
   - Expected result: The parser accepts the exact `type'(expr)` shape and rejects non-conforming variants.

66. [2.3.2] Supported explicit cast-target families
   - Assertion: Explicit cast targets include documented bit-vector, integer-like, `real`, `string`, `signed`, `unsigned`, and `type(operand_reference)` targets.
   - Verification: Parse one representative cast for each supported target family.
   - Expected result: Every documented target family is accepted.

67. [2.3.3] Casts may change width, signedness, state domain, and operand kind
   - Assertion: `type'(expr)` produces the target type even when that changes width, sign, state domain, or type family.
   - Verification: Cast the same source value into several target families and compare width-sensitive, sign-sensitive, and type-sensitive behavior.
   - Expected result: The result always has the target type's properties.

68. [2.3.3] Integral casts use deterministic bit-level conversion
   - Assertion: Integral-to-integral casts are defined by deterministic bit-level rules rather than host-language coercions.
   - Verification: Use integral casts that combine resizing, sign changes, and 2-state/4-state conversion.
   - Expected result: Results match the documented bit-level resize and state-domain rules exactly.

69. [2.3.4] Signedness-only casts are width-preserving reinterpretations
   - Assertion: `signed'(expr)` and `unsigned'(expr)` preserve width and reinterpret sign only.
   - Verification: Apply `signed'` and `unsigned'` to the same integral source and observe numeric interpretation without changing the bit pattern width.
   - Expected result: Width is unchanged and the underlying bit pattern is preserved.

70. [2.3.4] Signedness-only casts do not change the state domain
   - Assertion: `signed'` and `unsigned'` do not convert between 2-state and 4-state domains.
   - Verification: Apply them to 4-state values containing `x` or `z` and to 2-state values.
   - Expected result: 4-state operands remain 4-state with `x`/`z` preserved, and 2-state operands remain 2-state.

71. [2.3.4] Signedness-only casts are valid only for integral operands
   - Assertion: `signed'` and `unsigned'` are valid for `bit-vector`, `integer-like`, and `enum` operands.
   - Verification: Apply both casts to representatives of each integral operand family.
   - Expected result: The casts are accepted for all integral families.

72. [2.3.4] Signedness-only casts reject `real`, `string`, and `event`
   - Assertion: `signed'` and `unsigned'` are invalid for `real`, `string`, and raw `event` operands.
   - Verification: Attempt both casts on `real`, `string`, and raw event values.
   - Expected result: Semantic validation rejects all such casts.

73. [2.3.5] Widening resize extension rules
   - Assertion: Widening a signed integral source sign-extends, while widening an unsigned source zero-extends.
   - Verification: Cast representative signed and unsigned narrow values into wider integral targets.
   - Expected result: Signed sources replicate the sign bit, and unsigned sources pad with zero.

74. [2.3.5] Narrowing integral casts keep least-significant bits
   - Assertion: Casting to a narrower integral target keeps the least-significant `N` bits and truncates higher bits.
   - Verification: Cast wider values with distinctive upper and lower bits into narrower targets.
   - Expected result: Only the least-significant target-width bits remain.

75. [2.3.5] Post-resize signedness comes from the target type
   - Assertion: After resize, result signedness is defined by the target type rather than the source.
   - Verification: Cast the same bit pattern into signed and unsigned targets of the same width.
   - Expected result: Numeric interpretation changes with the target signedness even when the bits are the same.

76. [2.3.6] Integral state-domain conversion rules
   - Assertion: Casting to a 4-state integral target preserves `x` and `z`, while casting to a 2-state target converts `x -> 0` and `z -> 0`.
   - Verification: Cast 4-state sources containing `x` and `z` into both 4-state and 2-state targets.
   - Expected result: 4-state targets preserve unknowns, and 2-state targets zero them.

77. [2.3.7] Inline vector cast-target semantics matrix
   - Assertion: `bit[N]`, `logic[N]`, `signed bit[N]`, `signed logic[N]`, `unsigned bit[N]`, and `unsigned logic[N]` produce the documented width/sign/state properties.
   - Verification: Cast the same source into each inline vector target family across a few widths.
   - Expected result: The result type matches the documented matrix for every inline vector target.

78. [2.3.7] `bit` and `logic` are width-1 aliases
   - Assertion: `bit` is equivalent to `bit[1]`, and `logic` is equivalent to `logic[1]`.
   - Verification: Compare casts and operator behavior for `bit'(expr)` versus `bit[1]'(expr)` and for `logic'(expr)` versus `logic[1]'(expr)`.
   - Expected result: Each pair is semantically identical.

79. [2.3.8] Operand-type casts use recovered operand metadata
   - Assertion: `type(operand_reference)'(expr)` casts to the referenced operand's recovered type.
   - Verification: Use `type(ref)'(expr)` where `ref` is a non-enum typed operand with known metadata.
   - Expected result: The cast result takes the referenced operand's exact recovered type.

80. [2.3.8] Operand-type casts support non-enum recovered types
   - Assertion: `type(operand_reference)'(expr)` is not limited to enums.
   - Verification: Cast into recovered bit-vector, integer-like, real, or string operand types where fixtures exist.
   - Expected result: The cast works for any supported recovered operand type, subject to general cast rules.

81. [2.3.8] Operand-type casts are the primary route into dump-derived enum types
   - Assertion: `type(enum_ref)'(expr)` is the main way to cast into an enum type recovered from dump metadata.
   - Verification: Cast integral values into the recovered type of an enum signal.
   - Expected result: The result is enum-typed and follows the enum-cast rules.

82. [2.3.9] Enum label references are primaries, not cast targets
   - Assertion: `type(enum_operand_reference)::LABEL` is a primary expression rather than a cast target.
   - Verification: Use the label form where a primary is expected and attempt to use it as the left side of a cast target.
   - Expected result: Primary use succeeds, while treating it as a cast target is rejected.

83. [2.3.10] Enum casts use underlying bit-vector values
   - Assertion: Enum values are cast through their underlying bit-vector representation.
   - Verification: Cast into and out of enum types using integral values with known bit patterns.
   - Expected result: The bit pattern, after ordinary integral conversion rules, determines the enum result.

84. [2.3.10] Enum casts apply target enum width and label set
   - Assertion: Casting to an enum type uses the target enum's width and declared labels.
   - Verification: Cast the same source into two different enum types that differ in width or label set.
   - Expected result: The result reflects the selected target enum's width and label universe.

85. [2.3.10] Enum casts keep labels on exact bit-pattern matches
   - Assertion: If the resulting bit pattern matches a declared enum label, the result carries that label.
   - Verification: Cast a value whose post-conversion bit pattern matches a declared label.
   - Expected result: The enum result is labeled with the matching enumerator.

86. [2.3.10] Enum casts yield unlabeled enum values on non-matching patterns
   - Assertion: If the resulting bit pattern matches no declared label, the result stays enum-typed but has no label.
   - Verification: Cast a value whose post-conversion bit pattern is valid for the enum width but not declared as a label.
   - Expected result: The result has the enum type with no attached label.

87. [2.3.11] `real'(integral)` is numeric conversion from the source's self-determined value
   - Assertion: Casting integral to `real` uses the source's own width, signedness, and state domain before numeric conversion.
   - Verification: Convert integral sources whose numeric value depends on source width or signedness.
   - Expected result: The produced `real` matches the source's self-determined numeric value, not a reinterpreted bit pattern.

88. [2.3.11] `real'(integral)` rejects `x` and `z`
   - Assertion: Integral-to-real casts are invalid if the source contains `x` or `z`.
   - Verification: Attempt `real'(...)` on integral values containing `x` or `z`.
   - Expected result: Semantic validation or evaluation reports the cast as invalid.

89. [2.3.11] Real-to-integral casts truncate toward zero before target rules
   - Assertion: Casting from `real` to an integral target truncates toward zero, then applies normal integral resize, signedness, and state-domain rules.
   - Verification: Cast positive and negative non-integer real values into several integral targets.
   - Expected result: Fractional parts are dropped toward zero first, then the target integral rules are applied.

90. [2.3.12] `string'(string)` identity cast
   - Assertion: `string'(expr)` is supported only as an identity cast from `string` to `string`.
   - Verification: Cast a string literal or string operand with `string'(...)`.
   - Expected result: The cast is accepted and yields the same string value.

91. [2.3.12] Other explicit string casts are invalid
   - Assertion: All other explicit casts to or from `string` are invalid.
   - Verification: Attempt casts from integral or real to `string` and from `string` to integral or real.
   - Expected result: All such casts are rejected.

92. [2.3.13] Raw `event` is not castable, but `.triggered` follows ordinary integral cast rules
   - Assertion: Raw `event` cannot be cast directly, while `.triggered` can be cast like any other integral `bit`.
   - Verification: Attempt a direct cast of a raw event, then cast the corresponding `.triggered` value.
   - Expected result: The raw event cast is rejected, and the `.triggered` cast succeeds under normal integral cast rules.

93. [2.4] Common-type conversion is the default outside self-determined operands
   - Assertion: Operators convert operands to a common type unless the operator defines a self-determined operand.
   - Verification: Use several binary operators with mismatched operand widths, signs, and state domains, and compare them with operators that explicitly keep an operand self-determined.
   - Expected result: Operands are coerced according to the common-type rules except where the operator says otherwise.

94. [2.4.1] Boolean-context consumers
   - Assertion: Boolean context applies to `!`, `&&`, `||`, and the condition operand of `?:`.
   - Verification: Use the same operand under each of those operators and compare the truth result.
   - Expected result: All four contexts use the boolean-conversion rules rather than numeric arithmetic rules.

95. [2.4.1] Integral boolean-context truth table
   - Assertion: For integral operands, definitely zero is false, definitely non-zero is true, and anything else is `x`.
   - Verification: Parameterize boolean-context cases with integral `0`, integral non-zero, `x`, `z`, and mixed unknown-containing vectors.
   - Expected result: The truth result is `0`, `1`, or `x` exactly as documented.

96. [2.4.1] Real boolean-context truth table
   - Assertion: For `real`, `0.0` is false and any non-zero value is true.
   - Verification: Evaluate `!`, `&&`, `||`, or `?:` with `0.0`, positive non-zero, and negative non-zero real operands.
   - Expected result: `0.0` behaves as false, and any non-zero real behaves as true.

97. [2.4.1] Strings do not implicitly convert in boolean context
   - Assertion: True `string` values do not participate in boolean-context conversion.
   - Verification: Attempt to use a string operand with `!`, `&&`, `||`, or as the condition of `?:`.
   - Expected result: Semantic validation rejects the expression.

98. [2.4.1] Boolean conversion is temporary 1-bit truth only
   - Assertion: Boolean-context conversion produces a temporary 1-bit truth value and does not otherwise change the operand type.
   - Verification: Use operands with observable width or enum identity in boolean contexts and in neighboring non-boolean contexts.
   - Expected result: Only the boolean operator sees the temporary truth value; the operand's own type rules remain intact elsewhere.

99. [2.4.2] Integral common-type family and default width rule
   - Assertion: The integral family is `bit-vector`, `integer-like`, and `enum`, and the default common width is `max(operand widths)` when no operator-specific width rule overrides it.
   - Verification: Use integral-family operands of different widths under operators that do not define a special width rule.
   - Expected result: The common type stays within the integral family and uses the maximum operand width.

100. [2.4.2] Integral common sign and state rules
   - Assertion: Integral common sign is unsigned if any participating operand is unsigned, otherwise signed; the common state domain is 4-state if any operand is 4-state, otherwise 2-state.
   - Verification: Parameterize mixed signed/unsigned and mixed 2-state/4-state integral pairs.
   - Expected result: The chosen common sign and state domain match the documented rules.

101. [2.4.2] Integral extension and domain-conversion rules
   - Assertion: Extending to the common width uses sign extension for signed values and zero extension for unsigned values; conversion to 2-state maps `x/z -> 0`, while conversion to 4-state preserves `x` and `z`.
   - Verification: Use integral-family operands that require width extension and state-domain conversion before evaluation.
   - Expected result: The operands are converted exactly as documented before the operator is applied.

102. [2.4.3] Enum participation and label-loss rules
   - Assertion: Enums participate in implicit conversions through underlying bit-vector values, and enum labels do not affect width, sign, or state-domain selection.
   - Verification: Mix enum operands with other integral operands under operators that force implicit conversion.
   - Expected result: Common-type resolution uses the enum's underlying bits and ignores label names for width/sign/state selection.

103. [2.4.3] `?:` preserves enum type only for matching enum arms
   - Assertion: If both `?:` arms have the same enum type, the result preserves that enum type; otherwise mixed enum conversions drop labels and act as integral.
   - Verification: Compare ternaries with same-enum arms, different-enum arms, and enum-plus-non-enum arms.
   - Expected result: Only the same-enum-arm case preserves enum type and labels per the later merge rules.

104. [2.4.4] Real-family common-type rules
   - Assertion: `real` is a separate numeric family, and operators that admit mixed `integral` and `real` operands use `real` as the common type.
   - Verification: Use mixed integral/real arithmetic, comparison, and equality operators that are documented as admitting both families.
   - Expected result: The operation is evaluated in `real`, not in an integral common type.

105. [2.4.4] Implicit integral-to-real conversion rejects `x` and `z`
   - Assertion: Implicit conversion from integral to `real` is invalid if the integral value contains `x` or `z`.
   - Verification: Use mixed integral/real operators with an integral operand containing `x` or `z`.
   - Expected result: The expression is rejected or fails evaluation as invalid rather than fabricating a real value.

106. [2.4.4] No implicit `real`/`string` cross-conversion
   - Assertion: `real` does not implicitly convert to `string`, and `string` does not implicitly convert to `real`.
   - Verification: Attempt operators that would require implicit conversion between those families.
   - Expected result: The expression is rejected.

107. [2.4.5] String common-type restrictions
   - Assertion: Strings do not participate in generic numeric common-type resolution; mixed `string` with `integral` or `real` is invalid unless an operator-specific rule says otherwise; `?:` with two string arms yields string, while a string/non-string arm mix is invalid unless a later rule permits it.
   - Verification: Use strings in arithmetic, logical, comparison, and ternary cases with same-type and mixed-type arms.
   - Expected result: Only the documented string cases succeed.

108. [2.4.6] Result-type guidance matrix
   - Assertion: Logical operators produce a 1-bit 4-state integral result; integral-family operators follow operator-specific width rules within the integral common-type model; mixed integral/real operators produce `real`; and `?:` uses a self-determined condition with common-type resolution only on its arms.
   - Verification: Sample one operator from each category and inspect width, sign, state, and family of the result.
   - Expected result: Each category produces the documented result type.

109. [2.5.3] Selection works only on packed integral values, including derived packed values
   - Assertion: Selection applies to packed integral values, including derived packed values such as the results of selection, concatenation, or replication.
   - Verification: Select from a packed signal and from derived packed values built with concatenation or replication.
   - Expected result: Selection is accepted on all packed integral sources covered by the contract.

110. [2.5.3] Selection rejects scalar integral, `real`, and `string`
   - Assertion: Selection of scalar integral values, `real`, or `string` is invalid.
   - Verification: Attempt bit-select and part-select on a scalar integral, a real, and a string operand.
   - Expected result: Semantic validation rejects all such selections.

111. [2.5.3] Selection bound and runtime rules matrix
   - Assertion: Bit-select indices are self-determined expressions; non-indexed part-select uses constant integer `msb` and `lsb`; indexed part-select uses a positive constant integer `width` and a runtime integral `base`.
   - Verification: Combine valid and invalid cases for `expr[msb:lsb]`, `expr[base +: width]`, and `expr[base -: width]`.
   - Expected result: Only the documented constant and runtime positions are accepted.

112. [2.5.3] Selection result type and identity rules
   - Assertion: Bit-select yields an unsigned width-1 bit-vector, part-select yields an unsigned bit-vector of the selected width, and selection never preserves `integer-like` or `enum` identity.
   - Verification: Select from integer-like and enum operands and inspect result width, sign, family, and label preservation.
   - Expected result: Every selection result is a plain unsigned bit-vector of the documented width.

113. [2.5.3] Invalid bit-select behavior matrix
   - Assertion: For bit-select, an invalid index yields `x` from a 4-state source and `0` from a 2-state source.
   - Verification: Bit-select out of range on both 4-state and 2-state packed integral sources.
   - Expected result: The 4-state case returns `x`, and the 2-state case returns `0`.

114. [2.5.3] Invalid part-select behavior matrix
   - Assertion: Fully out-of-range part-select reads yield all-`x`; `x` or `z` in the base or index yields all-`x`; partially out-of-range reads return in-range source bits plus `x` for the out-of-range region; and invalid part-select reads therefore produce a 4-state result even from a 2-state source.
   - Verification: Parameterize fully out-of-range, partially out-of-range, and unknown-index/base part-select cases against both 2-state and 4-state sources.
   - Expected result: Every invalid case matches the documented all-`x` or mixed-bits-plus-`x` behavior, with a 4-state result type.

115. [2.5.4] Logical operator acceptance, result type, and short-circuit matrix
   - Assertion: Logical operators accept `integral` and `real`, reject true `string`, produce a 1-bit unsigned 4-state result, `&&`/`||` short-circuit, and `!` maps ambiguous truth to `x`.
   - Verification: Parameterize `!`, `&&`, and `||` with integral and real operands, add a string rejection case, and use side-effect-sensitive or invalid-right-operand fixtures to confirm short-circuiting.
   - Expected result: Accepted operand families and result types match the contract, string operands are rejected, and short-circuiting suppresses evaluation of the unnecessary side.

116. [2.5.5] Bitwise operator acceptance and common-type rules
   - Assertion: Bitwise operators accept only integral operands; binary bitwise operators use the integral common type; unary `~` preserves operand width; and enum operands participate through their underlying bit-vector values.
   - Verification: Use unary and binary bitwise operators across mixed integral widths, signs, and state domains, and include rejected real/string cases.
   - Expected result: Only integral operands are accepted, binary operations use the common integral type, and unary `~` keeps the operand width.

117. [2.5.5] Bitwise truth-table controlling-value cases
   - Assertion: Full 4-state bitwise truth-table semantics are intended, including controlling values such as `0 & x -> 0` and `1 | x -> 1`.
   - Verification: Evaluate representative truth-table entries for `&`, `|`, `^`, `^~`, and `~^`, with special emphasis on controlling-value examples.
   - Expected result: Results follow 4-state SystemVerilog-style truth tables, including the documented controlling cases.

118. [2.5.6] Reduction operator rules matrix
   - Assertion: Reduction operators accept only integral operands, keep the operand self-determined, produce a 1-bit unsigned 4-state result, and allow `x/z` to propagate to `x` through the reduction truth tables.
   - Verification: Parameterize unary reduction operators over 2-state and 4-state integral operands with and without `x/z` bits.
   - Expected result: Accepted operands, result type, and `x/z` handling all match the contract.

119. [2.5.7] Exponentiation acceptance and result-family rules
   - Assertion: Exponentiation accepts integral and real operands; if either operand is real the result is real; otherwise the result follows integral sizing and signedness rules, and the exponent operand is self-determined.
   - Verification: Use integral/integral, integral/real, and real/real exponentiation cases, including width- and sign-sensitive integral examples.
   - Expected result: Result family and operand coercion follow the documented rules.

120. [2.5.7] Exponentiation special-case matrix
   - Assertion: Exponent `0` yields `1`, and integral `0 ** negative` yields all-`x`.
   - Verification: Evaluate exponentiation cases with zero exponents and negative exponents applied to zero.
   - Expected result: The result is exactly `1` for exponent zero and all-`x` for integral zero raised to a negative exponent.

121. [2.5.8] Arithmetic supported-operator family
   - Assertion: Arithmetic includes unary `+`, unary `-`, binary `+`, `-`, `*`, `/`, and `%`.
   - Verification: Parse and evaluate one representative case for each arithmetic operator.
   - Expected result: Every documented arithmetic operator is accepted.

122. [2.5.8] Arithmetic common-type and promotion rules
   - Assertion: Pure integral arithmetic uses the integral common type, while mixed integral/real arithmetic promotes to `real`.
   - Verification: Compare pure-integral arithmetic with mixed integral/real arithmetic over the same numeric values.
   - Expected result: Pure integral cases stay integral, and mixed cases produce `real` results.

123. [2.5.8] Integral arithmetic `x/z` contagion and zero-divisor matrix
   - Assertion: In integral arithmetic, any `x` or `z` bit in an operand makes the result all-`x`, and divide-by-zero or modulo-by-zero also produce all-`x`.
   - Verification: Parameterize arithmetic cases with `x` or `z` bits and with zero divisors for `/` and `%`.
   - Expected result: Every such integral arithmetic case yields all-`x`.

124. [2.5.8] Integral division truncates toward zero
   - Assertion: Integral division truncates toward zero.
   - Verification: Divide positive and negative integral operands whose mathematical quotients are non-integer.
   - Expected result: The quotient is truncated toward zero in each case.

125. [2.5.9] Shift acceptance and width/count rules
   - Assertion: Shift operators accept only integral operands; result width is the left-operand width; and the right operand is self-determined and treated as unsigned.
   - Verification: Use shifts with varied left widths and right operands of different widths and signs, plus rejected real/string cases.
   - Expected result: Only integral operands are accepted, the result width equals the left width, and the shift count uses the right operand's self-determined value as unsigned.

126. [2.5.9] Shift fill-rule matrix
   - Assertion: `<<`, `>>`, and `<<<` fill vacated bits with `0`, while `>>>` fills with `0` for unsigned results and with the left operand's MSB for signed results.
   - Verification: Parameterize shifts over signed and unsigned left operands and all four shift operators.
   - Expected result: Each operator uses the documented fill behavior.

127. [2.5.9] Unknown shift counts yield all-`x`
   - Assertion: If the right operand of a shift contains `x` or `z`, the result is all-`x`.
   - Verification: Use shift counts containing `x` and `z` with otherwise valid left operands.
   - Expected result: The shift result is all-`x`.

128. [2.5.10] Comparison acceptance, promotion, and result rules
   - Assertion: Comparisons support `integral` and `real`; mixed integral/real comparison promotes the integral operand to `real`; the result is a 1-bit unsigned 4-state value; and any `x/z` bit in either integral operand yields `x`.
   - Verification: Parameterize integral-only, real-only, mixed integral/real, and integral-with-unknown comparison cases.
   - Expected result: Accepted operand families, real promotion, result type, and `x` propagation match the contract.

129. [2.5.11] Equality operator acceptance matrix
   - Assertion: `==` and `!=` support `integral`, `real`, and `string`/`string`; `===`, `!==`, `==?`, and `!=?` support only integral operands; and the equality-family result is a 1-bit unsigned 4-state value except where string equality is later constrained to known `0/1`.
   - Verification: Use valid and invalid examples for each equality operator across integral, real, and string operands.
   - Expected result: Each operator accepts only the documented operand families.

130. [2.5.11] Integral equality result semantics matrix
   - Assertion: Mixed integral/real equality promotes the integral operand to `real`; integral `==` and `!=` may produce `x`; integral `===` and `!==` always produce known 0/1; and `==?`/`!=?` treat `x/z` in the right operand as wildcards but never in the left operand.
   - Verification: Parameterize equality cases that distinguish all four behaviors.
   - Expected result: Each operator yields exactly the documented known/unknown and wildcard behavior.

131. [2.5.11] String equality exact-match semantics matrix
   - Assertion: For strings, only `==` and `!=` are supported; both operands must be strings; comparison is exact over the full string value; there is no numeric coercion, width extension, padding, truncation, or wildcarding; string literals participate as strings; and the result is always known 0 or 1.
   - Verification: Parameterize string equality with equal strings, unequal strings, different lengths, string literals, and invalid mixed-type cases.
   - Expected result: Only string/string `==` and `!=` succeed, and the result is exact and always known.

132. [2.5.12] Conditional operator condition and arm-resolution rules
   - Assertion: The condition operand of `?:` is self-determined and evaluated in boolean context, while the two result arms use the earlier common-type rules and preserve enum type when both arms share the same enum type.
   - Verification: Use ternaries with width-sensitive conditions and arm pairs from different compatible families.
   - Expected result: The condition uses boolean conversion only, and the result type is determined solely by the arms.

133. [2.5.12] Conditional `x/z` merge rules matrix
   - Assertion: If the condition is `x` or `z` and both arms are integral, the result is a bitwise merge at the common integral type; if both arms are the same enum type, the result keeps that enum type with merged underlying bits and preserves a label only when the merged pattern matches a declared label.
   - Verification: Parameterize ternaries with ambiguous conditions for integral arms and same-enum arms.
   - Expected result: Integral arms merge bitwise, same-enum arms merge within the enum type, and labels survive only on exact declared-label matches.

134. [2.5.13] `inside` accepted narrowed form and item kinds
   - Assertion: `inside` is supported only in the narrowed form where the left operand is integral, right-hand set items are integral expressions or inclusive integral ranges `[lo:hi]`, and the result is a 1-bit unsigned 4-state value.
   - Verification: Use valid `inside` expressions with integral lhs, expression items, and range items.
   - Expected result: The narrowed form is accepted and evaluated.

135. [2.5.13] `inside` matching semantics and `x` result rule
   - Assertion: `inside` item matching uses wildcard-equality semantics equivalent to `==?`, and if nothing matches but at least one candidate match is `x`, the result is `x`.
   - Verification: Use `inside` cases that produce direct matches, definite misses, and ambiguous candidate matches.
   - Expected result: Matches follow `==?` semantics, and ambiguous-no-match cases return `x`.

136. [2.5.13] Unsupported advanced `inside` forms matrix
   - Assertion: Advanced `inside` forms are unsupported, including open-range `$` bounds, `+/-`, `+%-`, unpacked-array matching, and scan-order-dependent behavior.
   - Verification: Attempt one representative example of each unsupported advanced form.
   - Expected result: Every advanced form is rejected.

137. [2.5.14] Concatenation and replication acceptance and identity rules
   - Assertion: Concatenation and replication accept only integral operands, return plain unsigned bit-vectors, never preserve `integer-like` or `enum` identity, and their results may themselves be selected.
   - Verification: Build concatenations and replications from integral, integer-like, and enum operands, then select from the results.
   - Expected result: Only integral operands are accepted, result identity is plain unsigned bit-vector, and later selection is allowed.

138. [2.5.14] Concatenation and replication width/state/value rules matrix
   - Assertion: Concatenation width is the sum of operand widths; replication width is `N * operand_width`; the result is 4-state if any operand is 4-state, otherwise 2-state; `x` and `z` are preserved; unsized constants are not allowed in integral concatenation; and replication multiplier `N` must be a non-negative constant integer expression.
   - Verification: Parameterize concatenation and replication cases that distinguish width, 2-state/4-state, `x/z` preservation, unsized constants, negative `N`, and non-constant `N`.
   - Expected result: All valid cases match the documented width and state rules, and all invalid `N` or unsized-constant cases are rejected.

139. [2.6] Primary and postfix forms bind tighter than infix operators
   - Assertion: Parenthesized expressions, casts, enum labels, concatenation, replication, `.triggered`, and selection bind tighter than any infix operator.
   - Verification: Use mixed expressions where infix regrouping would change the result, such as selection or cast next to addition or equality.
   - Expected result: The primary or postfix form is evaluated before any infix operator.

140. [2.6] Concatenation and replication behave as primary expressions in mixed expressions
   - Assertion: `{...}` and `{N{...}}` have primary-expression precedence.
   - Verification: Mix concatenation or replication with arithmetic, shifts, or equality without extra parentheses.
   - Expected result: The concatenation or replication groups as a whole primary expression.

141. [2.6] `.triggered` has postfix binding like selection
   - Assertion: `.triggered` binds as a dedicated postfix/member form with the same tight binding as other primary postfix operations.
   - Verification: Use `.triggered` adjacent to unary and binary operators and compare with explicitly parenthesized forms.
   - Expected result: `.triggered` groups with its operand before surrounding operators are applied.

142. [2.6] Postfix forms chain left-to-right
   - Assertion: Postfix forms associate left-to-right.
   - Verification: Use expressions with repeated postfix selections such as `expr[a][b]` or mixed selection suffixes on derived packed values.
   - Expected result: The parser and evaluator apply postfix suffixes left-to-right.

143. [2.6] Prefix unary operators bind to the following operand
   - Assertion: Prefix unary operators apply to the immediately following expression.
   - Verification: Compare unparenthesized unary forms with explicitly parenthesized equivalents.
   - Expected result: Each prefix operator groups with the following operand only.

144. [2.6] Exponentiation precedence relative to unary and multiplicative operators
   - Assertion: Exponentiation sits below prefix unary operators and above multiplicative operators in the supported precedence order.
   - Verification: Use expressions such as `-a ** b` and `a ** b * c` with fixtures that distinguish possible groupings.
   - Expected result: Grouping follows the documented order.

145. [2.6] Multiplicative, additive, and shift precedence chain
   - Assertion: Multiplicative operators bind tighter than additive operators, which bind tighter than shifts.
   - Verification: Use mixed expressions such as `a + b * c << d` and compare with explicit parentheses.
   - Expected result: The unparenthesized expression matches the documented precedence chain.

146. [2.6] Relational and `inside` precedence relative to shifts and equality
   - Assertion: Relational operators and `inside` are below shifts and above equality.
   - Verification: Use expressions mixing shifts, relational operators, `inside`, and equality, then compare with parenthesized equivalents.
   - Expected result: Grouping matches the documented precedence level.

147. [2.6] Equality and bitwise precedence chain
   - Assertion: Equality binds tighter than bitwise AND, bitwise AND binds tighter than bitwise XOR/XNOR, and bitwise XOR/XNOR binds tighter than bitwise OR.
   - Verification: Use mixed expressions spanning those operators and compare with explicit parentheses.
   - Expected result: The evaluator follows the documented equality/bitwise precedence chain.

148. [2.6] Logical and conditional precedence chain
   - Assertion: Logical AND binds tighter than logical OR, and logical OR binds tighter than the conditional operator.
   - Verification: Use mixed logical and ternary expressions whose meaning changes under different grouping.
   - Expected result: Grouping follows the documented logical and conditional precedence.

149. [2.6] Binary infix operators associate left-to-right
   - Assertion: Supported binary infix operators associate left-to-right unless otherwise specified.
   - Verification: Use repeated binary chains at one precedence level, such as subtraction, shifts, equality, and bitwise operators.
   - Expected result: The parse tree is left-associated.

150. [2.6] Conditional operator associates right-to-left
   - Assertion: `?:` associates right-to-left.
   - Verification: Use nested ternaries without parentheses and compare with explicit right-associated parentheses.
   - Expected result: The unparenthesized form matches right-associated grouping.

151. [2.6] Casts and enum-label primaries bind as primary expressions
   - Assertion: `type'(expr)` and `type(enum_operand_reference)::LABEL` both bind tighter than any infix operator.
   - Verification: Place each form next to additive, equality, or logical operators without extra parentheses.
   - Expected result: The cast or enum-label form groups as a primary expression.

152. [2.7] Start symbol and ternary grammar
   - Assertion: The parser entry point is `expr ::= conditional_expr`, and ternary syntax is `logical_or_expr ? expr : conditional_expr`.
   - Verification: Parse a bare logical-or expression and a nested ternary expression.
   - Expected result: Both forms are accepted, and nested ternaries follow the right-recursive grammar.

153. [2.7] Logical-chain grammar for `||` and `&&`
   - Assertion: `logical_or_expr` and `logical_and_expr` allow repeated `||` and `&&` chains.
   - Verification: Parse multi-term `||` and `&&` expressions.
   - Expected result: Repeated logical chains are accepted.

154. [2.7] Bitwise, equality, and relational chain grammar
   - Assertion: The grammar supports repeated bitwise, equality, and relational operator chains using the documented operator sets.
   - Verification: Parse representative repeated chains for bitwise AND/XOR/OR, equality operators, and relational operators including `inside`.
   - Expected result: Each documented chain form is accepted by the parser.

155. [2.7] Shift, additive, multiplicative, and power chain grammar
   - Assertion: `shift_expr`, `additive_expr`, `multiplicative_expr`, and `power_expr` support repeated chains of their documented operators.
   - Verification: Parse expressions containing repeated shifts, additions/subtractions, multiplications/divisions/modulos, and exponentiations.
   - Expected result: The parser accepts repeated chains at each grammar level.

156. [2.7] Recursive unary grammar
   - Assertion: `unary_expr` accepts `postfix_expr` or a recursively nested supported prefix operator applied to another `unary_expr`.
   - Verification: Parse stacked prefix operators such as `!!a`, `~~a`, `-+a`, and reduction-prefix combinations.
   - Expected result: Recursive prefix-unary chains are accepted.

157. [2.7] Repeated postfix grammar
   - Assertion: `postfix_expr ::= primary_expr { postfix_suffix }` allows repeated postfix suffixes.
   - Verification: Parse expressions with multiple postfix suffixes, such as nested selections.
   - Expected result: Multiple postfix suffixes are accepted in sequence.

158. [2.7] Postfix suffix grammar matrix
   - Assertion: The supported postfix suffixes are `[expr]`, `[constant_expr : constant_expr]`, `[expr +: constant_expr]`, `[expr -: constant_expr]`, and `.triggered`.
   - Verification: Parse one representative example of each postfix suffix form.
   - Expected result: All documented postfix suffixes are accepted.

159. [2.7] Primary-expression grammar matrix
   - Assertion: `primary_expr` includes operand references, literals, parenthesized expressions, casts, enum-label expressions, concatenations, and replications.
   - Verification: Parse one representative example of each primary-expression alternative.
   - Expected result: Every documented primary form is accepted.

160. [2.7] Concatenation and replication grammar details
   - Assertion: Concatenation is `{ expr { , expr } }`, and replication is `{ constant_expr { expr } }`.
   - Verification: Parse concatenations with multiple elements and replications with constant expressions as multipliers.
   - Expected result: Parser behavior matches the documented grammar shapes.

161. [2.7] `inside` set grammar details
   - Assertion: `inside_set` is `{ inside_item { , inside_item } }`, and each `inside_item` is either an expression or a range `[expr : expr]`.
   - Verification: Parse `inside` sets containing mixed expression items and range items.
   - Expected result: The parser accepts both item forms inside the set.

162. [2.7] Literal grammar matrix
   - Assertion: The literal grammar covers integral, real, and string literals.
   - Verification: Parse one representative literal of each class.
   - Expected result: All three literal categories are accepted.

163. [2.7] `type_reference` grammar matrix
   - Assertion: `type_reference` includes bit-vector types, integer-like types, `real`, `string`, `signed`, `unsigned`, and `type(operand_reference)`.
   - Verification: Parse one cast for each `type_reference` alternative.
   - Expected result: Every documented type-reference form is accepted.

164. [2.7] `bit_vector_type` grammar details
   - Assertion: A bit-vector type supports optional `signed`/`unsigned`, `bit` or `logic`, and an optional `[positive_integer]` width suffix.
   - Verification: Parse bare `bit`/`logic`, signed and unsigned variants, and width-suffixed variants, then attempt an invalid non-positive width.
   - Expected result: All documented forms are accepted, and invalid widths are rejected.

165. [2.7] `integer_like_type` grammar keyword matrix
   - Assertion: `byte`, `shortint`, `int`, `longint`, `integer`, and `time` are the supported integer-like type keywords.
   - Verification: Parse casts using each keyword.
   - Expected result: All six keywords are accepted as integer-like type references.

166. [2.7] Hierarchical operand references are part of the grammar surface
   - Assertion: Operand references may be hierarchical, for example `top.cpu.data`.
   - Verification: Parse expressions and event forms that use hierarchical operand references.
   - Expected result: Hierarchical names are accepted wherever operand references are allowed.

167. [2.7] `.triggered` is a reserved postfix form with semantic restrictions
   - Assertion: A trailing `.triggered` is not part of an operand reference name, and semantic validation allows it only on raw event operand references.
   - Verification: Parse tokens that could be confused with a longer operand name and separately attempt `.triggered` on non-event operands.
   - Expected result: The parser treats `.triggered` as a postfix suffix, and semantic validation rejects non-event uses.

168. [2.7] Semantic validation for enum labels, constant expressions, and inclusive ranges
   - Assertion: In `type(enum_operand_reference)::LABEL`, the operand must resolve to an enum-typed reference and the label must be declared; `constant_expr` positions must later validate as constant integer expressions in the supported subset; and `inside` ranges use inclusive bounds.
   - Verification: Use valid and invalid enum-label cases, valid and invalid constant-expression positions, and `inside` range cases that touch both bounds.
   - Expected result: Enum-label validation, constant-expression validation, and inclusive range semantics all match the contract.

169. [1] Event syntax rejects the outer `@(...)` wrapper
   - Assertion: Event expressions admit only the inner clocking-event surface, not the surrounding SystemVerilog `@(...)` wrapper.
   - Verification: Attempt forms such as `@(posedge clk)`, `@(a or b)`, and `@(*)`.
   - Expected result: The outer wrapper is rejected in every case, while the corresponding inner event expressions remain valid.

170. [2.3.13] `.triggered` follows ordinary integral cast semantics
   - Assertion: After `.triggered`, an event operand behaves exactly like an ordinary integral `bit` under width-changing, signedness-changing, and state-domain-changing casts.
   - Verification: Cast `event_ref.triggered` through representative integral targets and compare the results with the same casts applied to an ordinary `bit` operand carrying the same 0/1 value.
   - Expected result: Every cast result matches ordinary integral cast rules rather than a special-case event-specific path.

171. [2.4.4] Implicit integral-to-`real` promotion uses the source's self-determined value
   - Assertion: In mixed `integral`/`real` operators, an integral operand converts to `real` from its own self-determined width, signedness, and state domain before any wider operator context is considered.
   - Verification: Use mixed `integral`/`real` operators with integral operands whose numeric value changes with width or signedness, then compare the result with explicitly parenthesized reference cases.
   - Expected result: Promotion to `real` reflects the integral operand's self-determined value, not a value recomputed after external coercion.

172. [2.5.1] Non-whitelisted operator syntax is rejected
   - Assertion: The expression language accepts only the operator families explicitly listed in the contract and rejects nearby unsupported operator syntax.
   - Verification: Attempt representative non-whitelisted forms near supported families, such as `++a`, `a++`, `--a`, `a--`, and binary `a ~& b` or `a ~| b`.
   - Expected result: Every non-whitelisted operator form is rejected instead of being parsed as an extension.

173. [2.5.3] Bit-select unknown indices use the invalid-index rule
   - Assertion: A bit-select index containing `x` or `z` is an invalid index and therefore yields `x` for a 4-state source and `0` for a 2-state source.
   - Verification: Bit-select packed 4-state and 2-state integral sources with indices that evaluate to `x` and `z`.
   - Expected result: Unknown indices follow the same invalid-index result rule as other invalid bit-select indices.

174. [2.6] Exponentiation chains associate left-to-right
   - Assertion: Repeated `**` chains follow the contract's general binary-infix left-to-right associativity rule.
   - Verification: Evaluate expressions such as `a ** b ** c` and compare them with `(a ** b) ** c` and `a ** (b ** c)`.
   - Expected result: The unparenthesized chain matches `(a ** b) ** c`.
