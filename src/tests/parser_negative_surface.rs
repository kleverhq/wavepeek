use crate::expr::Span;
use crate::expr::ast::CastTargetAst;
use crate::expr::lexer::{LogicalToken, LogicalTokenKind};

use super::*;

#[test]
fn debug_private_parser_state() {
    let parser = LogicalParser {
        source: "a",
        tokens: vec![LogicalToken {
            kind: LogicalTokenKind::Eof,
            span: Span::new(1, 1),
        }],
        index: 0,
    };
    assert!(format!("{parser:?}").contains("source"));

    let candidate = CastTargetCandidate {
        target: Some(CastTargetAst::Signed),
        deferred_error: None,
        span: Span::new(0, 1),
    };
    assert!(format!("{candidate:?}").contains("Signed"));
}

#[test]
fn negative_branches_for_error_construction() {
    for source in [
        "sig iff   ",
        "{a, b",
        "type '(a)",
        "bit[4'b1]'(a)",
        "bit[999999999999999999999999999999]'(a)",
        "bit[0]'(a)",
        "type(sig)::LABEL",
        "a <",
    ] {
        assert!(parse_logical_expr_ast(source).is_err() || parse_event_expr_ast(source).is_err());
    }
    assert!(parse_event_expr_ast("sig iff   ").is_err());
}

#[test]
fn parser_negative_surface_exercises_event_separator_and_name_failures() {
    for source in [
        "sig or or clk",
        "sig , , clk",
        "(posedge clk",
        "posedge",
        "edge iff ready",
        "posedge bad-name",
    ] {
        assert!(
            parse_event_expr_ast(source).is_err(),
            "{source} should fail"
        );
    }
}

#[test]
fn parser_negative_surface_exercises_positive_cast_selection_and_inside_forms() {
    for source in [
        "bus[idx +: 2] == 2'b01",
        "bus[idx -: 2] == 2'b01",
        "inside_sig inside {1, [2:3], 4}",
        "type(state)::BUSY == unsigned'(data)",
        "bit[4]'(signed'(data)) == 4'hf",
        "{2{flag}} == 2'b11",
    ] {
        parse_logical_expr_ast(source).expect(source);
    }
}
