use crate::error::WavepeekError;

use super::ast::{BasicEventAst, DeferredLogicalExpr, EventExprAst, EventTermAst};
use super::diagnostic::{DiagnosticLayer, ExprDiagnostic, Span};
use super::lexer::{Token, TokenKind, lex_event_expr};
use super::{EventExpr, EventKind, EventTerm, Expression};

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
            let separator = self.current().cloned().ok_or_else(|| {
                parse_diag(
                    "C1-PARSE-BROKEN-UNION",
                    "broken event union segmentation",
                    Span::new(self.source.len(), self.source.len()),
                    &["expected 'or' or ',' between event terms"],
                )
            })?;

            match separator.kind {
                TokenKind::KeywordOr | TokenKind::Comma => {
                    self.index += 1;
                    if self.index >= self.tokens.len() {
                        return Err(parse_diag(
                            "C1-PARSE-BROKEN-UNION",
                            "broken event union segmentation",
                            separator.span,
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
                        separator.span,
                        &["remove ')' or add a matching '(' in an iff payload"],
                    ));
                }
                TokenKind::LeftParen => {
                    return Err(parse_diag(
                        "C1-PARSE-UNMATCHED-OPEN",
                        "unmatched opening parenthesis",
                        separator.span,
                        &["event-level grouping is not supported; use parentheses only inside iff"],
                    ));
                }
                _ => {
                    return Err(parse_diag(
                        "C1-PARSE-BROKEN-UNION",
                        "broken event union segmentation",
                        separator.span,
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
            let iff_token = self.bump().expect("iff token should exist");
            let (logical_source, logical_span) = self.capture_iff_payload(iff_token.span)?;
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
        let token = self.current().cloned().ok_or_else(|| {
            parse_diag(
                "C1-PARSE-BROKEN-UNION",
                "broken event union segmentation",
                Span::new(self.source.len(), self.source.len()),
                &["expected one event term"],
            )
        })?;

        match token.kind {
            TokenKind::Star => {
                self.index += 1;
                Ok((BasicEventAst::AnyTracked { span: token.span }, token.span))
            }
            TokenKind::Identifier => {
                self.index += 1;
                Ok((
                    BasicEventAst::Named {
                        name: token.lexeme,
                        span: token.span,
                    },
                    token.span,
                ))
            }
            TokenKind::KeywordPosedge => self.parse_edge_event(TokenKind::KeywordPosedge),
            TokenKind::KeywordNegedge => self.parse_edge_event(TokenKind::KeywordNegedge),
            TokenKind::KeywordEdge => self.parse_edge_event(TokenKind::KeywordEdge),
            TokenKind::KeywordOr | TokenKind::Comma => Err(parse_diag(
                "C1-PARSE-BROKEN-UNION",
                "broken event union segmentation",
                token.span,
                &["event term is missing before union separator"],
            )),
            TokenKind::LeftParen => Err(parse_diag(
                "C1-PARSE-UNMATCHED-OPEN",
                "unmatched opening parenthesis",
                token.span,
                &["event-level grouping is not supported; use parentheses only inside iff"],
            )),
            TokenKind::RightParen => Err(parse_diag(
                "C1-PARSE-UNMATCHED-CLOSE",
                "unmatched closing parenthesis",
                token.span,
                &["remove ')' or add a matching '(' in an iff payload"],
            )),
            TokenKind::KeywordIff => Err(parse_diag(
                "C1-PARSE-BROKEN-UNION",
                "broken event union segmentation",
                token.span,
                &["'iff' must follow a basic event term"],
            )),
        }
    }

    fn parse_edge_event(
        &mut self,
        kind: TokenKind,
    ) -> Result<(BasicEventAst, Span), ExprDiagnostic> {
        let keyword = self.bump().expect("edge keyword token should exist");
        let Some(name_token) = self.current().cloned() else {
            return Err(parse_diag(
                "C1-PARSE-MISSING-NAME",
                "missing signal name after edge keyword",
                keyword.span,
                &["expected a signal name after edge keyword"],
            ));
        };

        if name_token.kind != TokenKind::Identifier {
            let diagnostic = match name_token.kind {
                TokenKind::LeftParen => parse_diag(
                    "C1-PARSE-UNMATCHED-OPEN",
                    "unmatched opening parenthesis",
                    name_token.span,
                    &["event-level grouping is not supported; use parentheses only inside iff"],
                ),
                TokenKind::RightParen => parse_diag(
                    "C1-PARSE-UNMATCHED-CLOSE",
                    "unmatched closing parenthesis",
                    name_token.span,
                    &["remove ')' or add a matching '(' in an iff payload"],
                ),
                _ => parse_diag(
                    "C1-PARSE-MISSING-NAME",
                    "missing signal name after edge keyword",
                    keyword.span,
                    &["expected a signal name after edge keyword"],
                ),
            };
            return Err(diagnostic);
        }

        self.index += 1;
        let span = Span::new(keyword.span.start, name_token.span.end);
        let ast = match kind {
            TokenKind::KeywordPosedge => BasicEventAst::Posedge {
                name: name_token.lexeme,
                span,
            },
            TokenKind::KeywordNegedge => BasicEventAst::Negedge {
                name: name_token.lexeme,
                span,
            },
            TokenKind::KeywordEdge => BasicEventAst::Edge {
                name: name_token.lexeme,
                span,
            },
            _ => {
                return Err(parse_diag(
                    "C1-PARSE-BROKEN-UNION",
                    "broken event union segmentation",
                    keyword.span,
                    &["internal parser keyword dispatch failure"],
                ));
            }
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
            let token = self.current().cloned().expect("token should exist");
            match token.kind {
                TokenKind::LeftParen => {
                    open_stack.push(token.span);
                    end = token.span.end;
                    self.index += 1;
                }
                TokenKind::RightParen => {
                    if open_stack.pop().is_none() {
                        return Err(parse_diag(
                            "C1-PARSE-UNMATCHED-CLOSE",
                            "unmatched closing parenthesis",
                            token.span,
                            &["remove ')' or add a matching '(' in an iff payload"],
                        ));
                    }
                    end = token.span.end;
                    self.index += 1;
                }
                TokenKind::KeywordOr | TokenKind::Comma if open_stack.is_empty() => {
                    break;
                }
                _ => {
                    end = token.span.end;
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

    fn bump(&mut self) -> Option<Token> {
        let token = self.tokens.get(self.index).cloned();
        if token.is_some() {
            self.index += 1;
        }
        token
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

pub fn parse(source: &str) -> Result<Expression, WavepeekError> {
    if source.trim().is_empty() {
        return Err(WavepeekError::Args(
            "--eval expression cannot be empty".to_owned(),
        ));
    }

    Ok(Expression {
        source: source.to_owned(),
    })
}

pub fn parse_event_expr(source: &str) -> Result<EventExpr, WavepeekError> {
    let source = source.trim();
    if source.is_empty() {
        return Err(WavepeekError::Args(
            "--on expression cannot be empty. See 'wavepeek change --help'.".to_string(),
        ));
    }

    let term_chunks = split_terms(source)?;
    let mut terms = Vec::with_capacity(term_chunks.len());
    for chunk in term_chunks {
        terms.push(parse_event_term(chunk.as_str())?);
    }

    Ok(EventExpr { terms })
}

fn split_terms(source: &str) -> Result<Vec<String>, WavepeekError> {
    let mut terms = Vec::new();
    let mut depth = 0usize;
    let mut start = 0usize;
    let mut index = 0usize;

    while index < source.len() {
        let ch = source[index..].chars().next().ok_or_else(|| {
            WavepeekError::Internal("failed to decode event expression while splitting".to_string())
        })?;
        let ch_len = ch.len_utf8();

        match ch {
            '(' => {
                depth += 1;
                index += ch_len;
                continue;
            }
            ')' => {
                depth = depth.saturating_sub(1);
                index += ch_len;
                continue;
            }
            ',' if depth == 0 => {
                let segment = source[start..index].trim();
                if segment.is_empty() {
                    return Err(invalid_event_expr_error(source));
                }
                terms.push(segment.to_string());
                start = index + ch_len;
                index = start;
                continue;
            }
            _ => {}
        }

        if depth == 0 && source[index..].starts_with("or") && token_boundary(source, index, "or") {
            let segment = source[start..index].trim();
            if segment.is_empty() {
                return Err(invalid_event_expr_error(source));
            }
            terms.push(segment.to_string());
            start = index + 2;
            index = start;
            continue;
        }

        index += ch_len;
    }

    let tail = source[start..].trim();
    if tail.is_empty() {
        return Err(invalid_event_expr_error(source));
    }
    terms.push(tail.to_string());

    Ok(terms)
}

fn parse_event_term(segment: &str) -> Result<EventTerm, WavepeekError> {
    let segment = segment.trim();
    let (event, rest) = parse_basic_event(segment)?;
    let rest = rest.trim();
    if rest.is_empty() {
        return Ok(EventTerm {
            event,
            iff_expr: None,
        });
    }

    if !rest.starts_with("iff") || !token_boundary(rest, 0, "iff") {
        return Err(invalid_event_expr_error(segment));
    }

    let iff_expr = rest[3..].trim().to_string();
    Ok(EventTerm {
        event,
        iff_expr: Some(iff_expr),
    })
}

fn parse_basic_event(segment: &str) -> Result<(EventKind, &str), WavepeekError> {
    let segment = segment.trim();

    if let Some(rest) = segment.strip_prefix('*') {
        if rest.trim().is_empty() || rest.trim_start().starts_with("iff") {
            return Ok((EventKind::AnyTracked, rest));
        }
        return Err(invalid_event_expr_error(segment));
    }

    if let Some((name, rest)) = consume_prefixed_name(segment, "posedge") {
        return Ok((EventKind::Posedge(name), rest));
    }
    if let Some((name, rest)) = consume_prefixed_name(segment, "negedge") {
        return Ok((EventKind::Negedge(name), rest));
    }
    if let Some((name, rest)) = consume_prefixed_name(segment, "edge") {
        return Ok((EventKind::Edge(name), rest));
    }

    let (name, rest) = consume_name(segment)?;
    Ok((EventKind::AnyChange(name), rest))
}

fn consume_prefixed_name<'a>(segment: &'a str, prefix: &str) -> Option<(String, &'a str)> {
    if !segment.starts_with(prefix) || !token_boundary(segment, 0, prefix) {
        return None;
    }

    let rest = segment[prefix.len()..].trim_start();
    if rest.is_empty() {
        return Some((String::new(), rest));
    }

    let (name, tail) = consume_name(rest).ok()?;
    Some((name, tail))
}

fn consume_name(segment: &str) -> Result<(String, &str), WavepeekError> {
    let mut end = 0usize;
    for (index, ch) in segment.char_indices() {
        if is_name_char(ch) {
            end = index + ch.len_utf8();
            continue;
        }
        break;
    }

    if end == 0 {
        return Err(invalid_event_expr_error(segment));
    }

    Ok((segment[..end].to_string(), &segment[end..]))
}

fn is_name_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || matches!(ch, '_' | '.' | '$' | '[' | ']' | ':')
}

fn token_boundary(input: &str, start: usize, token: &str) -> bool {
    let end = start + token.len();
    let previous_ok = if start == 0 {
        true
    } else {
        !is_name_char(input[..start].chars().next_back().unwrap_or(' '))
    };
    let next_ok = if end >= input.len() {
        true
    } else {
        !is_name_char(input[end..].chars().next().unwrap_or(' '))
    };

    previous_ok && next_ok
}

fn invalid_event_expr_error(source: &str) -> WavepeekError {
    WavepeekError::Args(format!(
        "invalid --on expression '{source}'. See 'wavepeek change --help'."
    ))
}

#[cfg(test)]
mod tests {
    use super::{parse_event_expr, parse_event_expr_ast};
    use crate::expr::{BasicEventAst, DiagnosticLayer, EventKind, EventTerm};

    #[test]
    fn event_expr_iff_binding_with_union() {
        let parsed = parse_event_expr("negedge clk iff rstn or bar")
            .expect("event expression with iff+union should parse");

        assert_eq!(
            parsed.terms,
            vec![
                EventTerm {
                    event: EventKind::Negedge("clk".to_string()),
                    iff_expr: Some("rstn".to_string())
                },
                EventTerm {
                    event: EventKind::AnyChange("bar".to_string()),
                    iff_expr: None
                }
            ]
        );
    }

    #[test]
    fn event_expr_iff_capture_parenthesized_or() {
        let parsed = parse_event_expr("posedge clk iff (a or b) or bar")
            .expect("event expression with parenthesized iff should parse");

        assert_eq!(
            parsed.terms,
            vec![
                EventTerm {
                    event: EventKind::Posedge("clk".to_string()),
                    iff_expr: Some("(a or b)".to_string())
                },
                EventTerm {
                    event: EventKind::AnyChange("bar".to_string()),
                    iff_expr: None
                }
            ]
        );
    }

    #[test]
    fn event_expr_accepts_comma_union() {
        let parsed = parse_event_expr("posedge clk1, posedge clk2")
            .expect("comma-union expression should parse");

        assert_eq!(parsed.terms.len(), 2);
        assert!(matches!(parsed.terms[0].event, EventKind::Posedge(_)));
        assert!(matches!(parsed.terms[1].event, EventKind::Posedge(_)));
    }

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
}
