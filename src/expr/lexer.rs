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
    let mut tokens = Vec::with_capacity(source.len() / 2 + 1);
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
        while index < source.len() {
            let current = source[index..]
                .chars()
                .next()
                .ok_or_else(|| ExprDiagnostic {
                    layer: crate::expr::diagnostic::DiagnosticLayer::Parse,
                    code: "C1-PARSE-LEX-EOF",
                    message: "failed to decode event expression".to_string(),
                    primary_span: Span::new(index, index),
                    notes: vec![],
                })?;
            if current.is_whitespace() || matches!(current, '*' | ',' | '(' | ')') {
                break;
            }
            index += current.len_utf8();
        }

        let lexeme = source[start..index].to_string();
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
            span: Span::new(start, index),
            lexeme,
        });
    }

    Ok(tokens)
}

#[cfg(test)]
mod tests {
    use super::{TokenKind, lex_event_expr};

    #[test]
    fn lex_event_expr_tracks_keywords_and_spans() {
        let source = "posedge clk iff (a || b), *";
        let tokens = lex_event_expr(source).expect("lexing should succeed");

        let kinds = tokens.iter().map(|token| &token.kind).collect::<Vec<_>>();
        assert_eq!(
            kinds,
            vec![
                &TokenKind::KeywordPosedge,
                &TokenKind::Identifier,
                &TokenKind::KeywordIff,
                &TokenKind::LeftParen,
                &TokenKind::Identifier,
                &TokenKind::Identifier,
                &TokenKind::Identifier,
                &TokenKind::RightParen,
                &TokenKind::Comma,
                &TokenKind::Star,
            ]
        );
        assert_eq!(tokens[0].span.start, 0);
        assert_eq!(tokens[0].span.end, 7);
        assert_eq!(tokens[9].span.start, 26);
        assert_eq!(tokens[9].span.end, 27);
    }

    #[test]
    fn lex_event_expr_keeps_deferred_logical_operators() {
        let source = "posedge clk iff a && b";
        let tokens = lex_event_expr(source).expect("lexing should succeed");

        assert!(
            tokens
                .iter()
                .any(|token| token.lexeme == "&&" && token.kind == TokenKind::Identifier)
        );
    }

    #[test]
    fn lex_event_expr_classifies_event_union_or_keyword() {
        let source = "clk or rstn";
        let tokens = lex_event_expr(source).expect("lexing should succeed");

        assert_eq!(tokens[0].kind, TokenKind::Identifier);
        assert_eq!(tokens[1].kind, TokenKind::KeywordOr);
        assert_eq!(tokens[2].kind, TokenKind::Identifier);
    }

    #[test]
    fn lex_event_expr_keeps_invalid_name_chunks_for_parser_diagnostics() {
        let source = "posedge clk@";
        let tokens = lex_event_expr(source).expect("lexing should succeed");

        assert!(tokens.iter().any(|token| token.lexeme == "clk@"));
    }
}
