use crate::expr::ast::{IntegralBase, IntegralLiteral};
use crate::expr::diagnostic::{DiagnosticLayer, ExprDiagnostic, Span};

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
                layer: DiagnosticLayer::Parse,
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
                    layer: DiagnosticLayer::Parse,
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum LogicalTokenKind {
    Identifier(String),
    KeywordInside,
    IntegralLiteral(IntegralLiteral),
    LeftParen,
    RightParen,
    LeftBracket,
    RightBracket,
    LeftBrace,
    RightBrace,
    Comma,
    Colon,
    Question,
    Dot,
    Apostrophe,
    DoubleColon,
    Plus,
    Minus,
    Star,
    Power,
    Slash,
    Percent,
    Bang,
    Tilde,
    Amp,
    Pipe,
    Caret,
    TildeAmp,
    TildePipe,
    TildeCaret,
    CaretTilde,
    AndAnd,
    OrOr,
    Lt,
    Le,
    Gt,
    Ge,
    EqEq,
    NotEq,
    EqEqEq,
    NotEqEq,
    EqWildcard,
    NotEqWildcard,
    ShiftLeft,
    ShiftRight,
    ShiftArithLeft,
    ShiftArithRight,
    PlusColon,
    MinusColon,
    Eof,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct LogicalToken {
    pub kind: LogicalTokenKind,
    pub span: Span,
}

pub(crate) fn lex_logical_expr(
    source: &str,
    span_offset: usize,
) -> Result<Vec<LogicalToken>, ExprDiagnostic> {
    LogicalLexer {
        source,
        span_offset,
        index: 0,
    }
    .lex()
}

struct LogicalLexer<'a> {
    source: &'a str,
    span_offset: usize,
    index: usize,
}

impl<'a> LogicalLexer<'a> {
    fn lex(&mut self) -> Result<Vec<LogicalToken>, ExprDiagnostic> {
        let mut tokens = Vec::new();
        while self.index < self.source.len() {
            let Some(ch) = self.peek_char() else {
                break;
            };

            if ch.is_whitespace() {
                self.bump_char();
                continue;
            }

            if let Some((kind, width)) = self.lex_multi_operator() {
                let start = self.index;
                self.index += width;
                tokens.push(LogicalToken {
                    kind,
                    span: self.span(start, self.index),
                });
                continue;
            }

            if ch.is_ascii_digit() {
                let literal = self.lex_numeric_literal()?;
                let span = literal.span;
                tokens.push(LogicalToken {
                    kind: LogicalTokenKind::IntegralLiteral(literal),
                    span,
                });
                continue;
            }

            if ch == '\'' {
                if self.next_char_is_based_marker() {
                    let literal = self.lex_based_literal(None, self.index)?;
                    let span = literal.span;
                    tokens.push(LogicalToken {
                        kind: LogicalTokenKind::IntegralLiteral(literal),
                        span,
                    });
                } else {
                    let start = self.index;
                    self.bump_char();
                    tokens.push(LogicalToken {
                        kind: LogicalTokenKind::Apostrophe,
                        span: self.span(start, self.index),
                    });
                }
                continue;
            }

            if ch == '"' {
                return Err(self.parse_diag(
                    "C3-PARSE-LOGICAL-DEFERRED",
                    "string literals are deferred to C4",
                    self.span(self.index, self.index + 1),
                    &["C3 supports integral operands only"],
                ));
            }

            if logical_identifier_start(ch) {
                let start = self.index;
                self.bump_char();
                while let Some(next) = self.peek_char() {
                    if logical_identifier_char(next) {
                        self.bump_char();
                    } else {
                        break;
                    }
                }
                let end = self.index;
                let word = self.source[start..end].to_string();
                let kind = if word == "inside" {
                    LogicalTokenKind::KeywordInside
                } else {
                    LogicalTokenKind::Identifier(word)
                };
                tokens.push(LogicalToken {
                    kind,
                    span: self.span(start, end),
                });
                continue;
            }

            let start = self.index;
            let kind = match ch {
                '(' => {
                    self.bump_char();
                    LogicalTokenKind::LeftParen
                }
                ')' => {
                    self.bump_char();
                    LogicalTokenKind::RightParen
                }
                '[' => {
                    self.bump_char();
                    LogicalTokenKind::LeftBracket
                }
                ']' => {
                    self.bump_char();
                    LogicalTokenKind::RightBracket
                }
                '{' => {
                    self.bump_char();
                    LogicalTokenKind::LeftBrace
                }
                '}' => {
                    self.bump_char();
                    LogicalTokenKind::RightBrace
                }
                ',' => {
                    self.bump_char();
                    LogicalTokenKind::Comma
                }
                ':' => {
                    self.bump_char();
                    LogicalTokenKind::Colon
                }
                '?' => {
                    self.bump_char();
                    LogicalTokenKind::Question
                }
                '.' => {
                    self.bump_char();
                    LogicalTokenKind::Dot
                }
                '+' => {
                    self.bump_char();
                    LogicalTokenKind::Plus
                }
                '-' => {
                    self.bump_char();
                    LogicalTokenKind::Minus
                }
                '*' => {
                    self.bump_char();
                    LogicalTokenKind::Star
                }
                '/' => {
                    self.bump_char();
                    LogicalTokenKind::Slash
                }
                '%' => {
                    self.bump_char();
                    LogicalTokenKind::Percent
                }
                '!' => {
                    self.bump_char();
                    LogicalTokenKind::Bang
                }
                '~' => {
                    self.bump_char();
                    LogicalTokenKind::Tilde
                }
                '&' => {
                    self.bump_char();
                    LogicalTokenKind::Amp
                }
                '|' => {
                    self.bump_char();
                    LogicalTokenKind::Pipe
                }
                '^' => {
                    self.bump_char();
                    LogicalTokenKind::Caret
                }
                '<' => {
                    self.bump_char();
                    LogicalTokenKind::Lt
                }
                '>' => {
                    self.bump_char();
                    LogicalTokenKind::Gt
                }
                _ => {
                    self.bump_char();
                    return Err(self.parse_diag(
                        "C3-PARSE-LOGICAL-TOKEN",
                        "unsupported token in logical expression",
                        self.span(start, self.index),
                        &["C3 supports the integral Boolean surface only"],
                    ));
                }
            };

            tokens.push(LogicalToken {
                kind,
                span: self.span(start, self.index),
            });
        }

        let eof_span = self.span(self.source.len(), self.source.len());
        tokens.push(LogicalToken {
            kind: LogicalTokenKind::Eof,
            span: eof_span,
        });
        Ok(tokens)
    }

    fn lex_multi_operator(&self) -> Option<(LogicalTokenKind, usize)> {
        let tail = &self.source[self.index..];
        for (text, kind) in [
            ("!==", LogicalTokenKind::NotEqEq),
            ("===", LogicalTokenKind::EqEqEq),
            ("==?", LogicalTokenKind::EqWildcard),
            ("!=?", LogicalTokenKind::NotEqWildcard),
            ("<<<", LogicalTokenKind::ShiftArithLeft),
            (">>>", LogicalTokenKind::ShiftArithRight),
            ("**", LogicalTokenKind::Power),
            ("<<", LogicalTokenKind::ShiftLeft),
            (">>", LogicalTokenKind::ShiftRight),
            ("&&", LogicalTokenKind::AndAnd),
            ("||", LogicalTokenKind::OrOr),
            ("<=", LogicalTokenKind::Le),
            (">=", LogicalTokenKind::Ge),
            ("==", LogicalTokenKind::EqEq),
            ("!=", LogicalTokenKind::NotEq),
            ("+:", LogicalTokenKind::PlusColon),
            ("-:", LogicalTokenKind::MinusColon),
            ("::", LogicalTokenKind::DoubleColon),
            ("~&", LogicalTokenKind::TildeAmp),
            ("~|", LogicalTokenKind::TildePipe),
            ("~^", LogicalTokenKind::TildeCaret),
            ("^~", LogicalTokenKind::CaretTilde),
        ] {
            if tail.starts_with(text) {
                return Some((kind, text.len()));
            }
        }
        None
    }

    fn lex_numeric_literal(&mut self) -> Result<IntegralLiteral, ExprDiagnostic> {
        let start = self.index;
        self.consume_while(|ch| ch.is_ascii_digit() || ch == '_');
        let integer_end = self.index;

        if matches!(self.peek_char(), Some('.'))
            && matches!(self.peek_nth_char(1), Some(ch) if ch.is_ascii_digit())
        {
            self.bump_char();
            self.consume_while(|ch| ch.is_ascii_digit() || ch == '_');
            return Err(self.parse_diag(
                "C3-PARSE-LOGICAL-DEFERRED",
                "real literals are deferred to C4",
                self.span(start, self.index),
                &["C3 supports integral operands only"],
            ));
        }

        if matches!(self.peek_char(), Some('\'')) {
            let size_digits = self.source[start..integer_end]
                .chars()
                .filter(|ch| *ch != '_')
                .collect::<String>();
            let width = size_digits.parse::<u32>().map_err(|_| {
                self.parse_diag(
                    "C3-PARSE-LOGICAL-LITERAL",
                    "invalid sized integral literal",
                    self.span(start, integer_end),
                    &["literal size must be a positive integer"],
                )
            })?;
            if width == 0 {
                return Err(self.parse_diag(
                    "C3-PARSE-LOGICAL-LITERAL",
                    "invalid sized integral literal",
                    self.span(start, integer_end),
                    &["literal size must be greater than zero"],
                ));
            }
            return self.lex_based_literal(Some(width), start);
        }

        let digits = self.source[start..integer_end]
            .chars()
            .filter(|ch| *ch != '_')
            .collect::<String>();
        if digits.is_empty() {
            return Err(self.parse_diag(
                "C3-PARSE-LOGICAL-LITERAL",
                "invalid integral literal",
                self.span(start, integer_end),
                &["integral literals must contain at least one digit"],
            ));
        }

        Ok(IntegralLiteral {
            width: None,
            signed: true,
            base: IntegralBase::Decimal,
            digits,
            span: self.span(start, integer_end),
        })
    }

    fn next_char_is_based_marker(&self) -> bool {
        let Some(next) = self.peek_nth_char(1) else {
            return false;
        };
        if matches!(next, 'b' | 'B' | 'd' | 'D' | 'h' | 'H') {
            return true;
        }
        matches!(next, 's' | 'S')
            && matches!(
                self.peek_nth_char(2),
                Some('b' | 'B' | 'd' | 'D' | 'h' | 'H')
            )
    }

    fn lex_based_literal(
        &mut self,
        width: Option<u32>,
        start: usize,
    ) -> Result<IntegralLiteral, ExprDiagnostic> {
        self.bump_char();

        let signed = if matches!(self.peek_char(), Some('s' | 'S')) {
            self.bump_char();
            true
        } else {
            false
        };

        let base = match self.peek_char() {
            Some('b' | 'B') => {
                self.bump_char();
                IntegralBase::Binary
            }
            Some('d' | 'D') => {
                self.bump_char();
                IntegralBase::Decimal
            }
            Some('h' | 'H') => {
                self.bump_char();
                IntegralBase::Hex
            }
            _ => {
                return Err(self.parse_diag(
                    "C3-PARSE-LOGICAL-LITERAL",
                    "invalid based integral literal",
                    self.span(start, self.index),
                    &["expected base specifier b, d, or h"],
                ));
            }
        };

        let digits_start = self.index;
        self.consume_while(|ch| {
            ch.is_ascii_alphanumeric() || matches!(ch, '_' | 'x' | 'X' | 'z' | 'Z' | '-')
        });
        let digits_end = self.index;
        let digits = self.source[digits_start..digits_end]
            .chars()
            .filter(|ch| *ch != '_')
            .collect::<String>();
        if digits.is_empty() {
            return Err(self.parse_diag(
                "C3-PARSE-LOGICAL-LITERAL",
                "invalid based integral literal",
                self.span(start, self.index),
                &["based literal must include at least one digit"],
            ));
        }

        for (offset, ch) in digits.char_indices() {
            let ch = ch.to_ascii_lowercase();
            let valid = match base {
                IntegralBase::Binary => {
                    matches!(ch, '0' | '1' | 'x' | 'z' | 'h' | 'u' | 'w' | 'l' | '-')
                }
                IntegralBase::Decimal => {
                    ch.is_ascii_digit() || matches!(ch, 'x' | 'z' | 'h' | 'u' | 'w' | 'l' | '-')
                }
                IntegralBase::Hex => {
                    ch.is_ascii_hexdigit() || matches!(ch, 'x' | 'z' | 'h' | 'u' | 'w' | 'l' | '-')
                }
            };
            if !valid {
                return Err(self.parse_diag(
                    "C3-PARSE-LOGICAL-LITERAL",
                    "invalid digit for integral literal base",
                    self.span(digits_start + offset, digits_start + offset + ch.len_utf8()),
                    &["digit is not valid for the literal base"],
                ));
            }
        }

        Ok(IntegralLiteral {
            width,
            signed,
            base,
            digits: digits.to_ascii_lowercase(),
            span: self.span(start, self.index),
        })
    }

    fn consume_while<F>(&mut self, mut predicate: F)
    where
        F: FnMut(char) -> bool,
    {
        while let Some(ch) = self.peek_char() {
            if !predicate(ch) {
                break;
            }
            self.bump_char();
        }
    }

    fn peek_char(&self) -> Option<char> {
        self.source[self.index..].chars().next()
    }

    fn peek_nth_char(&self, offset: usize) -> Option<char> {
        self.source[self.index..].chars().nth(offset)
    }

    fn bump_char(&mut self) {
        if let Some(ch) = self.peek_char() {
            self.index += ch.len_utf8();
        }
    }

    fn span(&self, start: usize, end: usize) -> Span {
        Span::new(self.span_offset + start, self.span_offset + end)
    }

    fn parse_diag(
        &self,
        code: &'static str,
        message: &str,
        span: Span,
        notes: &[&str],
    ) -> ExprDiagnostic {
        ExprDiagnostic {
            layer: DiagnosticLayer::Parse,
            code,
            message: message.to_string(),
            primary_span: span,
            notes: notes.iter().map(|note| (*note).to_string()).collect(),
        }
    }
}

fn logical_identifier_start(ch: char) -> bool {
    ch.is_ascii_alphabetic() || matches!(ch, '_' | '$')
}

fn logical_identifier_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || matches!(ch, '_' | '$' | '.')
}

#[cfg(test)]
mod tests {
    use super::{TokenKind, lex_event_expr, lex_logical_expr};

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
    fn lex_logical_expr_accepts_core_c3_operators() {
        let tokens = lex_logical_expr("logic[8]'({a,b}) inside {[3:4], 8'hx?}", 0).expect("lexes");

        assert!(tokens.len() > 8, "token stream should be non-trivial");
    }
}
