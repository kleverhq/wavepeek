use super::ast::{BasicEventAst, DeferredLogicalExpr, EventExprAst, EventTermAst};
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
