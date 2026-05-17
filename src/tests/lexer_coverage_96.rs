use crate::expr::Span;
use crate::expr::ast::{IntegralBase, IntegralLiteral, RealLiteral, StringLiteral};

use super::*;

#[test]
fn lexer96_derive_tokens_and_logical_token_kinds() {
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
