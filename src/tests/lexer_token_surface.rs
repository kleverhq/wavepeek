use crate::expr::Span;
use crate::expr::ast::{IntegralBase, IntegralLiteral, RealLiteral, StringLiteral};
use crate::expr::lexer::lex_logical_expr;

use super::*;

#[test]
fn derive_tokens_and_logical_token_kinds() {
    let token = Token {
        kind: TokenKind::KeywordIff,
        span: Span::new(1, 4),
        lexeme: "iff".to_string(),
    };
    assert_eq!(token.clone(), token);
    assert!(format!("{token:?}").contains("KeywordIff"));

    for kind in [
        TokenKind::Identifier,
        TokenKind::KeywordOr,
        TokenKind::KeywordPosedge,
        TokenKind::KeywordNegedge,
        TokenKind::KeywordEdge,
        TokenKind::Star,
        TokenKind::Comma,
        TokenKind::LeftParen,
        TokenKind::RightParen,
    ] {
        assert_eq!(kind.clone(), kind);
        assert!(format!("{kind:?}").len() > 1);
    }

    let kinds = vec![
        LogicalTokenKind::Identifier("top.sig".to_string()),
        LogicalTokenKind::IntegralLiteral(IntegralLiteral {
            width: Some(4),
            signed: false,
            base: IntegralBase::Binary,
            digits: "1010".to_string(),
            span: Span::new(0, 6),
        }),
        LogicalTokenKind::RealLiteral(RealLiteral {
            text: "1.5".to_string(),
            span: Span::new(0, 3),
        }),
        LogicalTokenKind::StringLiteral(StringLiteral {
            value: "hello".to_string(),
            span: Span::new(0, 7),
        }),
        LogicalTokenKind::LeftParen,
        LogicalTokenKind::RightParen,
        LogicalTokenKind::LeftBracket,
        LogicalTokenKind::RightBracket,
        LogicalTokenKind::LeftBrace,
        LogicalTokenKind::RightBrace,
        LogicalTokenKind::Comma,
        LogicalTokenKind::Colon,
        LogicalTokenKind::Question,
        LogicalTokenKind::Dot,
        LogicalTokenKind::Apostrophe,
        LogicalTokenKind::DoubleColon,
        LogicalTokenKind::Plus,
        LogicalTokenKind::Minus,
        LogicalTokenKind::Star,
        LogicalTokenKind::Power,
        LogicalTokenKind::Slash,
        LogicalTokenKind::Percent,
        LogicalTokenKind::Bang,
        LogicalTokenKind::Tilde,
        LogicalTokenKind::Amp,
        LogicalTokenKind::Pipe,
        LogicalTokenKind::Caret,
        LogicalTokenKind::TildeAmp,
        LogicalTokenKind::TildePipe,
        LogicalTokenKind::TildeCaret,
        LogicalTokenKind::CaretTilde,
        LogicalTokenKind::AndAnd,
        LogicalTokenKind::OrOr,
        LogicalTokenKind::Lt,
        LogicalTokenKind::Le,
        LogicalTokenKind::Gt,
        LogicalTokenKind::Ge,
        LogicalTokenKind::EqEq,
        LogicalTokenKind::NotEq,
        LogicalTokenKind::EqEqEq,
        LogicalTokenKind::NotEqEq,
        LogicalTokenKind::EqWildcard,
        LogicalTokenKind::NotEqWildcard,
        LogicalTokenKind::ShiftLeft,
        LogicalTokenKind::ShiftRight,
        LogicalTokenKind::ShiftArithLeft,
        LogicalTokenKind::ShiftArithRight,
        LogicalTokenKind::PlusColon,
        LogicalTokenKind::MinusColon,
        LogicalTokenKind::KeywordInside,
        LogicalTokenKind::Eof,
    ];
    for kind in kinds {
        let token = LogicalToken {
            kind: kind.clone(),
            span: Span::new(2, 3),
        };
        assert_eq!(token.clone(), token);
        assert!(format!("{kind:?}").len() > 1);
        assert!(format!("{token:?}").contains("span"));
    }
}

#[test]
fn lexer_exercises_multi_operator_and_cast_surface() {
    let tokens = lex_logical_expr(
        "lhs ==? rhs && mid !=? other && shift <<< 1 >>> 2 && bus[0 +: 2] == bus[1 -: 2] && signed'(data) == bit'(flag)",
        0,
    )
    .expect("lexer should accept the operator surface");
    let kinds = tokens
        .into_iter()
        .map(|token| token.kind)
        .collect::<Vec<_>>();

    assert!(kinds.contains(&LogicalTokenKind::EqWildcard));
    assert!(kinds.contains(&LogicalTokenKind::NotEqWildcard));
    assert!(kinds.contains(&LogicalTokenKind::ShiftArithLeft));
    assert!(kinds.contains(&LogicalTokenKind::ShiftArithRight));
    assert!(kinds.contains(&LogicalTokenKind::PlusColon));
    assert!(kinds.contains(&LogicalTokenKind::MinusColon));
    assert!(
        kinds
            .iter()
            .filter(|kind| matches!(kind, LogicalTokenKind::Apostrophe))
            .count()
            >= 2
    );
}

#[test]
fn lexer_exercises_based_literals_and_malformed_numeric_edges() {
    let tokens = lex_logical_expr("4'b1010 == bit'(flag) && 8'shff == signed'(data)", 0)
        .expect("lexer should keep based literals distinct from casts");
    assert!(
        tokens
            .iter()
            .any(|token| matches!(token.kind, LogicalTokenKind::IntegralLiteral(_)))
    );
    assert!(
        tokens
            .iter()
            .filter(|token| matches!(token.kind, LogicalTokenKind::Apostrophe))
            .count()
            >= 2
    );

    for source in ["8'sh", "12'", "4'b102"] {
        assert!(
            lex_logical_expr(source, 0).is_err(),
            "{source} should be rejected"
        );
    }
}
