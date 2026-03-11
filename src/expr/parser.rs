use super::ast::{
    BasicEventAst, DeferredLogicalExpr, EventExprAst, EventTermAst, IntegralBase, IntegralLiteral,
    LogicalBinaryOp, LogicalExprAst, LogicalExprNode,
};
use super::diagnostic::{DiagnosticLayer, ExprDiagnostic, Span};
use super::lexer::{Token, TokenKind, lex_event_expr};

pub fn parse_event_expr_ast(source: &str) -> Result<EventExprAst, ExprDiagnostic> {
    if source.trim().is_empty() {
        return Err(parse_diag(
            "C1-PARSE-EMPTY",
            "event expression cannot be empty",
            Span::new(0, source.len()),
            &["expected one event term"],
        ));
    }

    let tokens = lex_event_expr(source)?;
    if tokens.is_empty() {
        return Err(parse_diag(
            "C1-PARSE-EMPTY",
            "event expression cannot be empty",
            Span::new(0, source.len()),
            &["expected one event term"],
        ));
    }

    StrictParser {
        source,
        tokens,
        index: 0,
    }
    .parse_event_expr()
}

struct StrictParser<'a> {
    source: &'a str,
    tokens: Vec<Token>,
    index: usize,
}

impl<'a> StrictParser<'a> {
    fn parse_event_expr(&mut self) -> Result<EventExprAst, ExprDiagnostic> {
        let mut terms = Vec::new();
        terms.push(self.parse_event_term()?);

        while self.index < self.tokens.len() {
            let separator = self.current().ok_or_else(|| {
                parse_diag(
                    "C1-PARSE-BROKEN-UNION",
                    "broken event union segmentation",
                    Span::new(self.source.len(), self.source.len()),
                    &["expected 'or' or ',' between event terms"],
                )
            })?;

            let separator_kind = separator.kind.clone();
            let separator_span = separator.span;
            match separator_kind {
                TokenKind::KeywordOr | TokenKind::Comma => {
                    self.index += 1;
                    if self.index >= self.tokens.len() {
                        return Err(parse_diag(
                            "C1-PARSE-BROKEN-UNION",
                            "broken event union segmentation",
                            separator_span,
                            &["union separator must be followed by an event term"],
                        ));
                    }

                    if matches!(
                        self.current().map(|token| &token.kind),
                        Some(TokenKind::KeywordOr | TokenKind::Comma)
                    ) {
                        let duplicated = self.current().expect("token should exist");
                        return Err(parse_diag(
                            "C1-PARSE-BROKEN-UNION",
                            "broken event union segmentation",
                            duplicated.span,
                            &["duplicate union separator is not allowed"],
                        ));
                    }

                    terms.push(self.parse_event_term()?);
                }
                TokenKind::RightParen => {
                    return Err(parse_diag(
                        "C1-PARSE-UNMATCHED-CLOSE",
                        "unmatched closing parenthesis",
                        separator_span,
                        &["remove ')' or add a matching '(' in an iff payload"],
                    ));
                }
                TokenKind::LeftParen => {
                    return Err(parse_diag(
                        "C1-PARSE-UNMATCHED-OPEN",
                        "unmatched opening parenthesis",
                        separator_span,
                        &["event-level grouping is not supported; use parentheses only inside iff"],
                    ));
                }
                _ => {
                    return Err(parse_diag(
                        "C1-PARSE-BROKEN-UNION",
                        "broken event union segmentation",
                        separator_span,
                        &["expected 'or' or ',' between event terms"],
                    ));
                }
            }
        }

        let span = Span::new(
            terms.first().expect("one term exists").span.start,
            terms.last().expect("one term exists").span.end,
        );
        Ok(EventExprAst { terms, span })
    }

    fn parse_event_term(&mut self) -> Result<EventTermAst, ExprDiagnostic> {
        let (event, event_span) = self.parse_basic_event()?;
        let mut term_end = event_span.end;

        let iff = if matches!(
            self.current().map(|token| &token.kind),
            Some(TokenKind::KeywordIff)
        ) {
            let iff_span = self.current().expect("iff token should exist").span;
            self.index += 1;
            let (logical_source, logical_span) = self.capture_iff_payload(iff_span)?;
            term_end = logical_span.end;
            Some(DeferredLogicalExpr {
                source: logical_source,
                span: logical_span,
            })
        } else {
            None
        };

        Ok(EventTermAst {
            event,
            iff,
            span: Span::new(event_span.start, term_end),
        })
    }

    fn parse_basic_event(&mut self) -> Result<(BasicEventAst, Span), ExprDiagnostic> {
        let token = self.current().ok_or_else(|| {
            parse_diag(
                "C1-PARSE-BROKEN-UNION",
                "broken event union segmentation",
                Span::new(self.source.len(), self.source.len()),
                &["expected one event term"],
            )
        })?;

        let token_kind = token.kind.clone();
        let token_span = token.span;
        match token_kind {
            TokenKind::Star => {
                self.index += 1;
                Ok((BasicEventAst::AnyTracked { span: token_span }, token_span))
            }
            TokenKind::Identifier => {
                let name = std::mem::take(&mut self.tokens[self.index].lexeme);
                if let Some((offset, ch)) = invalid_name_char(name.as_str()) {
                    let message = format!("unexpected character '{ch}' in signal name");
                    return Err(parse_diag(
                        "C1-PARSE-LEX-CHAR",
                        message.as_str(),
                        Span::new(
                            token_span.start + offset,
                            token_span.start + offset + ch.len_utf8(),
                        ),
                        &["signal names must use [A-Za-z0-9_.$[]:]"],
                    ));
                }
                self.index += 1;
                Ok((
                    BasicEventAst::Named {
                        name,
                        span: token_span,
                    },
                    token_span,
                ))
            }
            TokenKind::KeywordPosedge => self.parse_edge_event(TokenKind::KeywordPosedge),
            TokenKind::KeywordNegedge => self.parse_edge_event(TokenKind::KeywordNegedge),
            TokenKind::KeywordEdge => self.parse_edge_event(TokenKind::KeywordEdge),
            TokenKind::KeywordOr | TokenKind::Comma => Err(parse_diag(
                "C1-PARSE-BROKEN-UNION",
                "broken event union segmentation",
                token_span,
                &["event term is missing before union separator"],
            )),
            TokenKind::LeftParen => Err(parse_diag(
                "C1-PARSE-UNMATCHED-OPEN",
                "unmatched opening parenthesis",
                token_span,
                &["event-level grouping is not supported; use parentheses only inside iff"],
            )),
            TokenKind::RightParen => Err(parse_diag(
                "C1-PARSE-UNMATCHED-CLOSE",
                "unmatched closing parenthesis",
                token_span,
                &["remove ')' or add a matching '(' in an iff payload"],
            )),
            TokenKind::KeywordIff => Err(parse_diag(
                "C1-PARSE-BROKEN-UNION",
                "broken event union segmentation",
                token_span,
                &["'iff' must follow a basic event term"],
            )),
        }
    }

    fn parse_edge_event(
        &mut self,
        kind: TokenKind,
    ) -> Result<(BasicEventAst, Span), ExprDiagnostic> {
        let keyword_span = self
            .current()
            .expect("edge keyword token should exist")
            .span;
        self.index += 1;
        let Some(name_token) = self.current() else {
            return Err(parse_diag(
                "C1-PARSE-MISSING-NAME",
                "missing signal name after edge keyword",
                keyword_span,
                &["expected a signal name after edge keyword"],
            ));
        };

        if name_token.kind != TokenKind::Identifier {
            let name_span = name_token.span;
            let diagnostic = match name_token.kind {
                TokenKind::LeftParen => parse_diag(
                    "C1-PARSE-UNMATCHED-OPEN",
                    "unmatched opening parenthesis",
                    name_span,
                    &["event-level grouping is not supported; use parentheses only inside iff"],
                ),
                TokenKind::RightParen => parse_diag(
                    "C1-PARSE-UNMATCHED-CLOSE",
                    "unmatched closing parenthesis",
                    name_span,
                    &["remove ')' or add a matching '(' in an iff payload"],
                ),
                _ => parse_diag(
                    "C1-PARSE-MISSING-NAME",
                    "missing signal name after edge keyword",
                    keyword_span,
                    &["expected a signal name after edge keyword"],
                ),
            };
            return Err(diagnostic);
        }

        let name_span = name_token.span;
        let name = std::mem::take(&mut self.tokens[self.index].lexeme);
        if let Some((offset, ch)) = invalid_name_char(name.as_str()) {
            let message = format!("unexpected character '{ch}' in signal name");
            return Err(parse_diag(
                "C1-PARSE-LEX-CHAR",
                message.as_str(),
                Span::new(
                    name_span.start + offset,
                    name_span.start + offset + ch.len_utf8(),
                ),
                &["signal names must use [A-Za-z0-9_.$[]:]"],
            ));
        }
        self.index += 1;
        let span = Span::new(keyword_span.start, name_span.end);
        let ast = if kind == TokenKind::KeywordPosedge {
            BasicEventAst::Posedge { name, span }
        } else if kind == TokenKind::KeywordNegedge {
            BasicEventAst::Negedge { name, span }
        } else if kind == TokenKind::KeywordEdge {
            BasicEventAst::Edge { name, span }
        } else {
            return Err(parse_diag(
                "C1-PARSE-BROKEN-UNION",
                "broken event union segmentation",
                keyword_span,
                &["internal parser keyword dispatch failure"],
            ));
        };

        Ok((ast, span))
    }

    fn capture_iff_payload(&mut self, iff_span: Span) -> Result<(String, Span), ExprDiagnostic> {
        if self.index >= self.tokens.len() {
            return Err(parse_diag(
                "C1-PARSE-EMPTY-IFF",
                "empty iff payload",
                iff_span,
                &["expected logical expression after 'iff'"],
            ));
        }

        let start = self.tokens[self.index].span.start;
        let mut end = start;
        let mut open_stack: Vec<Span> = Vec::new();

        while self.index < self.tokens.len() {
            let token = self.current().expect("token should exist");
            let token_kind = token.kind.clone();
            let token_span = token.span;
            match token_kind {
                TokenKind::LeftParen => {
                    open_stack.push(token_span);
                    end = token_span.end;
                    self.index += 1;
                }
                TokenKind::RightParen => {
                    if open_stack.pop().is_none() {
                        return Err(parse_diag(
                            "C1-PARSE-UNMATCHED-CLOSE",
                            "unmatched closing parenthesis",
                            token_span,
                            &["remove ')' or add a matching '(' in an iff payload"],
                        ));
                    }
                    end = token_span.end;
                    self.index += 1;
                }
                TokenKind::KeywordOr | TokenKind::Comma if open_stack.is_empty() => {
                    break;
                }
                _ => {
                    end = token_span.end;
                    self.index += 1;
                }
            }
        }

        if start == end {
            return Err(parse_diag(
                "C1-PARSE-EMPTY-IFF",
                "empty iff payload",
                iff_span,
                &["expected logical expression after 'iff'"],
            ));
        }

        if let Some(unmatched_open) = open_stack.first().copied() {
            return Err(parse_diag(
                "C1-PARSE-UNMATCHED-OPEN",
                "unmatched opening parenthesis",
                unmatched_open,
                &["close the '(' opened in iff payload"],
            ));
        }

        let raw = &self.source[start..end];
        let trimmed_start = raw
            .find(|ch: char| !ch.is_whitespace())
            .map(|offset| start + offset)
            .unwrap_or(start);
        let trimmed_end = raw
            .rfind(|ch: char| !ch.is_whitespace())
            .map(|offset| start + offset + 1)
            .unwrap_or(end);
        let payload = self.source[trimmed_start..trimmed_end].to_string();

        if payload.is_empty() {
            return Err(parse_diag(
                "C1-PARSE-EMPTY-IFF",
                "empty iff payload",
                iff_span,
                &["expected logical expression after 'iff'"],
            ));
        }

        Ok((payload, Span::new(trimmed_start, trimmed_end)))
    }

    fn current(&self) -> Option<&Token> {
        self.tokens.get(self.index)
    }
}

fn parse_diag(code: &'static str, message: &str, span: Span, notes: &[&str]) -> ExprDiagnostic {
    ExprDiagnostic {
        layer: DiagnosticLayer::Parse,
        code,
        message: message.to_string(),
        primary_span: span,
        notes: notes.iter().map(|note| (*note).to_string()).collect(),
    }
}

fn invalid_name_char(name: &str) -> Option<(usize, char)> {
    name.char_indices().find(|(_, ch)| !is_name_char(*ch))
}

fn is_name_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || matches!(ch, '_' | '.' | '$' | '[' | ']' | ':')
}

pub(crate) fn parse_bounded_logical_expr(
    source: &str,
    span_offset: usize,
) -> Result<LogicalExprAst, ExprDiagnostic> {
    if source.trim().is_empty() {
        return Err(logical_parse_diag(
            "C2-PARSE-LOGICAL-EMPTY",
            "empty iff logical expression",
            Span::new(span_offset, span_offset + source.len()),
            &["expected logical expression after 'iff'"],
        ));
    }

    let mut lexer = LogicalLexer::new(source, span_offset);
    let tokens = lexer.lex()?;
    let mut parser = LogicalParser {
        source,
        tokens,
        index: 0,
    };
    parser.parse()
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum LogicalTokenKind {
    Identifier(String),
    IntegralLiteral(IntegralLiteral),
    Bang,
    AndAnd,
    OrOr,
    Lt,
    Le,
    Gt,
    Ge,
    EqEq,
    NotEq,
    LeftParen,
    RightParen,
    Eof,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct LogicalToken {
    kind: LogicalTokenKind,
    span: Span,
}

struct LogicalLexer<'a> {
    source: &'a str,
    span_offset: usize,
    index: usize,
}

impl<'a> LogicalLexer<'a> {
    fn new(source: &'a str, span_offset: usize) -> Self {
        Self {
            source,
            span_offset,
            index: 0,
        }
    }

    fn lex(&mut self) -> Result<Vec<LogicalToken>, ExprDiagnostic> {
        let mut tokens = Vec::new();

        while let Some(ch) = self.peek_char() {
            if ch.is_whitespace() {
                self.bump_char();
                continue;
            }

            if self.starts_with("===")
                || self.starts_with("!==")
                || self.starts_with("==?")
                || self.starts_with("!=?")
                || self.starts_with("<<<")
                || self.starts_with(">>>")
                || self.starts_with("<<")
                || self.starts_with(">>")
            {
                let start = self.index;
                let token = if self.starts_with("<<") || self.starts_with(">>") {
                    self.take_exact(2)
                } else {
                    self.take_exact(3)
                };
                return Err(logical_parse_diag(
                    "C2-PARSE-LOGICAL-UNSUPPORTED",
                    "unsupported operator in C2 iff subset",
                    self.span(start, start + token.len()),
                    &["this operator is outside the bounded C2 logical subset"],
                ));
            }

            if self.starts_with("&&") {
                let start = self.index;
                self.take_exact(2);
                tokens.push(LogicalToken {
                    kind: LogicalTokenKind::AndAnd,
                    span: self.span(start, start + 2),
                });
                continue;
            }
            if self.starts_with("||") {
                let start = self.index;
                self.take_exact(2);
                tokens.push(LogicalToken {
                    kind: LogicalTokenKind::OrOr,
                    span: self.span(start, start + 2),
                });
                continue;
            }
            if self.starts_with("<=") {
                let start = self.index;
                self.take_exact(2);
                tokens.push(LogicalToken {
                    kind: LogicalTokenKind::Le,
                    span: self.span(start, start + 2),
                });
                continue;
            }
            if self.starts_with(">=") {
                let start = self.index;
                self.take_exact(2);
                tokens.push(LogicalToken {
                    kind: LogicalTokenKind::Ge,
                    span: self.span(start, start + 2),
                });
                continue;
            }
            if self.starts_with("==") {
                let start = self.index;
                self.take_exact(2);
                tokens.push(LogicalToken {
                    kind: LogicalTokenKind::EqEq,
                    span: self.span(start, start + 2),
                });
                continue;
            }
            if self.starts_with("!=") {
                let start = self.index;
                self.take_exact(2);
                tokens.push(LogicalToken {
                    kind: LogicalTokenKind::NotEq,
                    span: self.span(start, start + 2),
                });
                continue;
            }

            let start = self.index;
            match ch {
                '!' => {
                    self.bump_char();
                    tokens.push(LogicalToken {
                        kind: LogicalTokenKind::Bang,
                        span: self.span(start, self.index),
                    });
                }
                '<' => {
                    self.bump_char();
                    tokens.push(LogicalToken {
                        kind: LogicalTokenKind::Lt,
                        span: self.span(start, self.index),
                    });
                }
                '>' => {
                    self.bump_char();
                    tokens.push(LogicalToken {
                        kind: LogicalTokenKind::Gt,
                        span: self.span(start, self.index),
                    });
                }
                '(' => {
                    self.bump_char();
                    tokens.push(LogicalToken {
                        kind: LogicalTokenKind::LeftParen,
                        span: self.span(start, self.index),
                    });
                }
                ')' => {
                    self.bump_char();
                    tokens.push(LogicalToken {
                        kind: LogicalTokenKind::RightParen,
                        span: self.span(start, self.index),
                    });
                }
                '"' => {
                    return Err(logical_parse_diag(
                        "C2-PARSE-LOGICAL-UNSUPPORTED",
                        "string literals are outside the C2 iff subset",
                        self.span(start, start + 1),
                        &["C2 iff supports integral literals only"],
                    ));
                }
                '?' | ':' | '+' | '-' | '*' | '/' | '%' | '&' | '|' | '^' | '~' | '{' | '}'
                | '[' | ']' => {
                    self.bump_char();
                    return Err(logical_parse_diag(
                        "C2-PARSE-LOGICAL-UNSUPPORTED",
                        "unsupported operator in C2 iff subset",
                        self.span(start, self.index),
                        &["this operator is outside the bounded C2 logical subset"],
                    ));
                }
                '\'' => {
                    let literal = self.lex_based_literal(None, start)?;
                    tokens.push(LogicalToken {
                        kind: LogicalTokenKind::IntegralLiteral(literal.clone()),
                        span: literal.span,
                    });
                }
                _ if ch.is_ascii_digit() => {
                    let literal = self.lex_numeric_literal()?;
                    tokens.push(LogicalToken {
                        kind: LogicalTokenKind::IntegralLiteral(literal.clone()),
                        span: literal.span,
                    });
                }
                _ if logical_identifier_start(ch) => {
                    let token = self.lex_identifier_token()?;
                    tokens.push(token);
                }
                _ => {
                    self.bump_char();
                    return Err(logical_parse_diag(
                        "C2-PARSE-LOGICAL-UNSUPPORTED",
                        "unsupported token in C2 iff subset",
                        self.span(start, self.index),
                        &[
                            "only operand references, literals, !, comparisons, &&, ||, and parentheses are supported",
                        ],
                    ));
                }
            }
        }

        tokens.push(LogicalToken {
            kind: LogicalTokenKind::Eof,
            span: self.span(self.source.len(), self.source.len()),
        });

        Ok(tokens)
    }

    fn lex_identifier_token(&mut self) -> Result<LogicalToken, ExprDiagnostic> {
        let start = self.index;
        self.bump_char();
        while let Some(next) = self.peek_char() {
            if logical_identifier_char(next) {
                self.bump_char();
                continue;
            }
            break;
        }
        let end = self.index;
        let lexeme = self.source[start..end].to_string();
        let span = self.span(start, end);

        if lexeme == "inside" {
            return Err(logical_parse_diag(
                "C2-PARSE-LOGICAL-UNSUPPORTED",
                "unsupported operator in C2 iff subset",
                span,
                &["inside is outside the bounded C2 logical subset"],
            ));
        }

        if lexeme.ends_with(".triggered") {
            return Err(logical_parse_diag(
                "C2-PARSE-LOGICAL-UNSUPPORTED",
                "unsupported primary form in C2 iff subset",
                span,
                &[".triggered is deferred to later expression phases"],
            ));
        }

        if lexeme == "type" && matches!(self.peek_char(), Some('(')) {
            return Err(logical_parse_diag(
                "C2-PARSE-LOGICAL-UNSUPPORTED",
                "unsupported primary form in C2 iff subset",
                span,
                &["type(...) forms are deferred to later expression phases"],
            ));
        }

        if matches!(self.peek_char(), Some('\'')) {
            return Err(logical_parse_diag(
                "C2-PARSE-LOGICAL-UNSUPPORTED",
                "unsupported cast form in C2 iff subset",
                span,
                &["casts are deferred to later expression phases"],
            ));
        }

        Ok(LogicalToken {
            kind: LogicalTokenKind::Identifier(lexeme),
            span,
        })
    }

    fn lex_numeric_literal(&mut self) -> Result<IntegralLiteral, ExprDiagnostic> {
        let start = self.index;
        self.consume_while(|ch| ch.is_ascii_digit() || ch == '_');
        let integer_end = self.index;

        if matches!(self.peek_char(), Some('.')) {
            let dot_start = self.index;
            self.bump_char();
            if matches!(self.peek_char(), Some(ch) if ch.is_ascii_digit()) {
                self.consume_while(|ch| ch.is_ascii_digit() || ch == '_');
                return Err(logical_parse_diag(
                    "C2-PARSE-LOGICAL-UNSUPPORTED",
                    "real literals are outside the C2 iff subset",
                    self.span(start, self.index),
                    &["C2 iff supports integral literals only"],
                ));
            }

            return Err(logical_parse_diag(
                "C2-PARSE-LOGICAL-UNSUPPORTED",
                "unsupported token in C2 iff subset",
                self.span(dot_start, dot_start + 1),
                &[
                    "only operand references, literals, !, comparisons, &&, ||, and parentheses are supported",
                ],
            ));
        }

        if matches!(self.peek_char(), Some('\'')) {
            let size = self.source[start..integer_end]
                .chars()
                .filter(|ch| *ch != '_')
                .collect::<String>();
            let width = size.parse::<u32>().map_err(|_| {
                logical_parse_diag(
                    "C2-PARSE-LOGICAL-LITERAL",
                    "invalid sized integral literal",
                    self.span(start, integer_end),
                    &["integral literal size must be a positive integer"],
                )
            })?;
            if width == 0 {
                return Err(logical_parse_diag(
                    "C2-PARSE-LOGICAL-LITERAL",
                    "invalid sized integral literal",
                    self.span(start, integer_end),
                    &["integral literal size must be greater than zero"],
                ));
            }
            return self.lex_based_literal(Some(width), start);
        }

        let digits = self.source[start..integer_end]
            .chars()
            .filter(|ch| *ch != '_')
            .collect::<String>();
        if digits.is_empty() {
            return Err(logical_parse_diag(
                "C2-PARSE-LOGICAL-LITERAL",
                "invalid integral literal",
                self.span(start, integer_end),
                &["integral literals must contain digits"],
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

    fn lex_based_literal(
        &mut self,
        width: Option<u32>,
        literal_start: usize,
    ) -> Result<IntegralLiteral, ExprDiagnostic> {
        let Some('\'') = self.peek_char() else {
            return Err(logical_parse_diag(
                "C2-PARSE-LOGICAL-LITERAL",
                "invalid based integral literal",
                self.span(literal_start, literal_start),
                &["expected apostrophe in based literal"],
            ));
        };
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
                return Err(logical_parse_diag(
                    "C2-PARSE-LOGICAL-LITERAL",
                    "invalid based integral literal",
                    self.span(literal_start, self.index),
                    &["expected base specifier 'b', 'd', or 'h'"],
                ));
            }
        };

        let digits_start = self.index;
        self.consume_while(|ch| {
            ch.is_ascii_alphanumeric()
                || ch == '_'
                || ch == 'x'
                || ch == 'X'
                || ch == 'z'
                || ch == 'Z'
        });
        let digits_end = self.index;
        let digits = self.source[digits_start..digits_end]
            .chars()
            .filter(|ch| *ch != '_')
            .collect::<String>();
        if digits.is_empty() {
            return Err(logical_parse_diag(
                "C2-PARSE-LOGICAL-LITERAL",
                "invalid based integral literal",
                self.span(literal_start, self.index),
                &["based literals must include digits after the base specifier"],
            ));
        }

        for (offset, ch) in digits.char_indices() {
            let ok = match base {
                IntegralBase::Binary => matches!(ch, '0' | '1' | 'x' | 'X' | 'z' | 'Z'),
                IntegralBase::Decimal => ch.is_ascii_digit(),
                IntegralBase::Hex => ch.is_ascii_hexdigit() || matches!(ch, 'x' | 'X' | 'z' | 'Z'),
            };
            if !ok {
                return Err(logical_parse_diag(
                    "C2-PARSE-LOGICAL-LITERAL",
                    "invalid digit for integral literal base",
                    self.span(digits_start + offset, digits_start + offset + ch.len_utf8()),
                    &["integral literal digit is not valid for this base"],
                ));
            }
        }

        Ok(IntegralLiteral {
            width,
            signed,
            base,
            digits: digits.to_ascii_lowercase(),
            span: self.span(literal_start, self.index),
        })
    }

    fn starts_with(&self, prefix: &str) -> bool {
        self.source[self.index..].starts_with(prefix)
    }

    fn take_exact(&mut self, byte_len: usize) -> String {
        let token = self.source[self.index..self.index + byte_len].to_string();
        self.index += byte_len;
        token
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

    fn bump_char(&mut self) {
        if let Some(ch) = self.peek_char() {
            self.index += ch.len_utf8();
        }
    }

    fn span(&self, local_start: usize, local_end: usize) -> Span {
        Span::new(self.span_offset + local_start, self.span_offset + local_end)
    }
}

struct LogicalParser<'a> {
    source: &'a str,
    tokens: Vec<LogicalToken>,
    index: usize,
}

impl<'a> LogicalParser<'a> {
    fn parse(&mut self) -> Result<LogicalExprAst, ExprDiagnostic> {
        let root = self.parse_or_expr()?;
        if !matches!(self.current().kind, LogicalTokenKind::Eof) {
            return Err(logical_parse_diag(
                "C2-PARSE-LOGICAL-TRAILING",
                "trailing tokens in iff logical expression",
                self.current().span,
                &["remove extra tokens after a complete logical expression"],
            ));
        }
        Ok(LogicalExprAst { root })
    }

    fn parse_or_expr(&mut self) -> Result<LogicalExprNode, ExprDiagnostic> {
        let mut node = self.parse_and_expr()?;
        while matches!(self.current().kind, LogicalTokenKind::OrOr) {
            self.index += 1;
            let right = self.parse_and_expr()?;
            let span = Span::new(node.span().start, right.span().end);
            node = LogicalExprNode::Binary {
                op: LogicalBinaryOp::OrOr,
                left: Box::new(node),
                right: Box::new(right),
                span,
            };
        }
        Ok(node)
    }

    fn parse_and_expr(&mut self) -> Result<LogicalExprNode, ExprDiagnostic> {
        let mut node = self.parse_equality_expr()?;
        while matches!(self.current().kind, LogicalTokenKind::AndAnd) {
            self.index += 1;
            let right = self.parse_equality_expr()?;
            let span = Span::new(node.span().start, right.span().end);
            node = LogicalExprNode::Binary {
                op: LogicalBinaryOp::AndAnd,
                left: Box::new(node),
                right: Box::new(right),
                span,
            };
        }
        Ok(node)
    }

    fn parse_equality_expr(&mut self) -> Result<LogicalExprNode, ExprDiagnostic> {
        let mut node = self.parse_relation_expr()?;

        loop {
            let op = match self.current().kind {
                LogicalTokenKind::EqEq => LogicalBinaryOp::Eq,
                LogicalTokenKind::NotEq => LogicalBinaryOp::Ne,
                _ => break,
            };
            self.index += 1;
            let right = self.parse_relation_expr()?;
            let span = Span::new(node.span().start, right.span().end);
            node = LogicalExprNode::Binary {
                op,
                left: Box::new(node),
                right: Box::new(right),
                span,
            };
        }

        Ok(node)
    }

    fn parse_relation_expr(&mut self) -> Result<LogicalExprNode, ExprDiagnostic> {
        let mut node = self.parse_unary_expr()?;

        loop {
            let op = match self.current().kind {
                LogicalTokenKind::Lt => LogicalBinaryOp::Lt,
                LogicalTokenKind::Le => LogicalBinaryOp::Le,
                LogicalTokenKind::Gt => LogicalBinaryOp::Gt,
                LogicalTokenKind::Ge => LogicalBinaryOp::Ge,
                _ => break,
            };
            self.index += 1;
            let right = self.parse_unary_expr()?;
            let span = Span::new(node.span().start, right.span().end);
            node = LogicalExprNode::Binary {
                op,
                left: Box::new(node),
                right: Box::new(right),
                span,
            };
        }

        Ok(node)
    }

    fn parse_unary_expr(&mut self) -> Result<LogicalExprNode, ExprDiagnostic> {
        if matches!(self.current().kind, LogicalTokenKind::Bang) {
            let start = self.current().span.start;
            self.index += 1;
            let inner = self.parse_unary_expr()?;
            let span = Span::new(start, inner.span().end);
            return Ok(LogicalExprNode::UnaryNot {
                expr: Box::new(inner),
                span,
            });
        }

        self.parse_primary_expr()
    }

    fn parse_primary_expr(&mut self) -> Result<LogicalExprNode, ExprDiagnostic> {
        let token = self.current().clone();
        match token.kind {
            LogicalTokenKind::Identifier(name) => {
                self.index += 1;
                Ok(LogicalExprNode::OperandRef {
                    name,
                    span: token.span,
                })
            }
            LogicalTokenKind::IntegralLiteral(literal) => {
                self.index += 1;
                Ok(LogicalExprNode::IntegralLiteral {
                    literal,
                    span: token.span,
                })
            }
            LogicalTokenKind::LeftParen => {
                let open_span = token.span;
                self.index += 1;
                let expr = self.parse_or_expr()?;
                if !matches!(self.current().kind, LogicalTokenKind::RightParen) {
                    return Err(logical_parse_diag(
                        "C2-PARSE-LOGICAL-UNMATCHED-OPEN",
                        "unmatched opening parenthesis in iff payload",
                        open_span,
                        &["close this parenthesis in the iff logical expression"],
                    ));
                }
                let close_span = self.current().span;
                self.index += 1;
                let span = Span::new(open_span.start, close_span.end);
                Ok(re_span(expr, span))
            }
            LogicalTokenKind::RightParen => Err(logical_parse_diag(
                "C2-PARSE-LOGICAL-UNMATCHED-CLOSE",
                "unmatched closing parenthesis in iff payload",
                token.span,
                &["remove ')' or add a matching '(' in the iff logical expression"],
            )),
            LogicalTokenKind::Eof => Err(logical_parse_diag(
                "C2-PARSE-LOGICAL-EXPECTED-OPERAND",
                "incomplete iff logical expression",
                token.span,
                &["expected operand, literal, or '('"],
            )),
            _ => Err(logical_parse_diag(
                "C2-PARSE-LOGICAL-EXPECTED-OPERAND",
                "expected operand in iff logical expression",
                token.span,
                &["expected operand reference, integral literal, or '('"],
            )),
        }
    }

    fn current(&self) -> &LogicalToken {
        self.tokens
            .get(self.index)
            .expect("logical parser should keep eof sentinel")
    }
}

fn logical_identifier_start(ch: char) -> bool {
    ch.is_ascii_alphabetic() || matches!(ch, '_' | '$')
}

fn logical_identifier_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || matches!(ch, '_' | '.' | '$' | '[' | ']' | ':')
}

fn re_span(node: LogicalExprNode, span: Span) -> LogicalExprNode {
    match node {
        LogicalExprNode::OperandRef { name, .. } => LogicalExprNode::OperandRef { name, span },
        LogicalExprNode::IntegralLiteral { literal, .. } => {
            LogicalExprNode::IntegralLiteral { literal, span }
        }
        LogicalExprNode::UnaryNot { expr, .. } => LogicalExprNode::UnaryNot { expr, span },
        LogicalExprNode::Binary {
            op, left, right, ..
        } => LogicalExprNode::Binary {
            op,
            left,
            right,
            span,
        },
    }
}

fn logical_parse_diag(
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

#[cfg(test)]
mod tests {
    use super::parse_event_expr_ast;
    use crate::expr::{BasicEventAst, DiagnosticLayer};

    #[test]
    fn typed_parser_rejects_unmatched_open_parenthesis() {
        let error = parse_event_expr_ast("(").expect_err("source should fail");

        assert_eq!(error.layer, DiagnosticLayer::Parse);
        assert_eq!(error.code, "C1-PARSE-UNMATCHED-OPEN");
        assert_eq!(error.primary_span.start, 0);
        assert_eq!(error.primary_span.end, 1);
    }

    #[test]
    fn typed_parser_rejects_broken_union_segmentation() {
        let error = parse_event_expr_ast("posedge clk or , clk").expect_err("source should fail");

        assert_eq!(error.layer, DiagnosticLayer::Parse);
        assert_eq!(error.code, "C1-PARSE-BROKEN-UNION");
        assert_eq!(error.primary_span.start, 15);
        assert_eq!(error.primary_span.end, 16);
    }

    #[test]
    fn typed_parser_preserves_iff_binding_to_single_term() {
        let parsed =
            parse_event_expr_ast("negedge clk iff rstn or ready").expect("source should parse");

        assert_eq!(parsed.terms.len(), 2);
        assert!(matches!(
            parsed.terms[0].event,
            BasicEventAst::Negedge { ref name, .. } if name == "clk"
        ));
        assert_eq!(
            parsed.terms[0]
                .iff
                .as_ref()
                .expect("iff payload should exist")
                .source,
            "rstn"
        );
        assert!(matches!(
            parsed.terms[1].event,
            BasicEventAst::Named { ref name, .. } if name == "ready"
        ));
        assert!(parsed.terms[1].iff.is_none());
    }

    #[test]
    fn typed_parser_keeps_logical_operator_payload_deferred() {
        let parsed = parse_event_expr_ast("posedge clk iff a && b").expect("source should parse");

        assert_eq!(
            parsed.terms[0]
                .iff
                .as_ref()
                .expect("iff payload should exist")
                .source,
            "a && b"
        );
    }
}
