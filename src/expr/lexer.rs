use crate::expr::diagnostic::{ExprDiagnostic, Span};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenKind {
    Identifier,
    KeywordOr,
    KeywordIff,
    KeywordPosedge,
    KeywordNegedge,
    KeywordEdge,
    Star,
    Comma,
    LeftParen,
    RightParen,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
    pub lexeme: String,
}

pub fn lex_event_expr(source: &str) -> Result<Vec<Token>, ExprDiagnostic> {
    let mut tokens = Vec::new();
    let mut index = 0usize;

    while index < source.len() {
        let ch = source[index..]
            .chars()
            .next()
            .ok_or_else(|| ExprDiagnostic {
                layer: crate::expr::diagnostic::DiagnosticLayer::Parse,
                code: "C1-PARSE-LEX-EOF",
                message: "failed to decode event expression".to_string(),
                primary_span: Span::new(index, index),
                notes: vec![],
            })?;
        let ch_len = ch.len_utf8();

        if ch.is_whitespace() {
            index += ch_len;
            continue;
        }

        let simple = match ch {
            '*' => Some(TokenKind::Star),
            ',' => Some(TokenKind::Comma),
            '(' => Some(TokenKind::LeftParen),
            ')' => Some(TokenKind::RightParen),
            _ => None,
        };
        if let Some(kind) = simple {
            tokens.push(Token {
                kind,
                span: Span::new(index, index + ch_len),
                lexeme: ch.to_string(),
            });
            index += ch_len;
            continue;
        }

        let start = index;
        let mut end = index;
        for (offset, current) in source[index..].char_indices() {
            if current.is_whitespace() || matches!(current, '*' | ',' | '(' | ')') {
                break;
            }
            end = index + offset + current.len_utf8();
        }

        if end == start {
            return Err(ExprDiagnostic {
                layer: crate::expr::diagnostic::DiagnosticLayer::Parse,
                code: "C1-PARSE-LEX-CHAR",
                message: format!("unexpected character '{ch}'"),
                primary_span: Span::new(start, start + ch_len),
                notes: vec![],
            });
        }

        let lexeme = source[start..end].to_string();
        let kind = match lexeme.as_str() {
            "or" => TokenKind::KeywordOr,
            "iff" => TokenKind::KeywordIff,
            "posedge" => TokenKind::KeywordPosedge,
            "negedge" => TokenKind::KeywordNegedge,
            "edge" => TokenKind::KeywordEdge,
            _ => TokenKind::Identifier,
        };
        tokens.push(Token {
            kind,
            span: Span::new(start, end),
            lexeme,
        });
        index = end;
    }

    Ok(tokens)
}
