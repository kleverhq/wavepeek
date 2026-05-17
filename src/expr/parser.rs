use super::ast::{
    BasicEventAst, BinaryOpAst, CastTargetAst, DeferredLogicalExpr, EventExprAst, EventTermAst,
    InsideItemAst, IntegralBase, LogicalExprAst, LogicalExprNode, SelectionKindAst, UnaryOpAst,
};
use super::diagnostic::{DiagnosticLayer, ExprDiagnostic, Span};
use super::host::IntegerLikeKind;
use super::lexer::{
    LogicalToken, LogicalTokenKind, Token, TokenKind, lex_event_expr, lex_logical_expr,
};

pub fn parse_event_expr_ast(source: &str) -> Result<EventExprAst, ExprDiagnostic> {
    if source.trim().is_empty() {
        return Err(parse_diag(
            "EXPR-PARSE-EVENT-EMPTY",
            "event expression cannot be empty",
            Span::new(0, source.len()),
            &["expected one event term"],
        ));
    }

    let tokens = lex_event_expr(source)?;
    if tokens.is_empty() {
        return Err(parse_diag(
            "EXPR-PARSE-EVENT-EMPTY",
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

pub fn parse_logical_expr_ast(source: &str) -> Result<LogicalExprAst, ExprDiagnostic> {
    parse_logical_expr_with_offset(source, 0)
}

pub(crate) fn parse_logical_expr_with_offset(
    source: &str,
    source_offset: usize,
) -> Result<LogicalExprAst, ExprDiagnostic> {
    if source.trim().is_empty() {
        return Err(logical_parse_diag(
            "EXPR-PARSE-LOGICAL-EMPTY",
            "logical expression cannot be empty",
            Span::new(source_offset, source_offset + source.len()),
            &["expected a logical expression"],
        ));
    }

    let tokens = lex_logical_expr(source, source_offset)?;
    let mut parser = LogicalParser {
        source,
        tokens,
        index: 0,
    };
    parser.parse()
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
                    "EXPR-PARSE-EVENT-BROKEN-UNION",
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
                            "EXPR-PARSE-EVENT-BROKEN-UNION",
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
                            "EXPR-PARSE-EVENT-BROKEN-UNION",
                            "broken event union segmentation",
                            duplicated.span,
                            &["duplicate union separator is not allowed"],
                        ));
                    }

                    terms.push(self.parse_event_term()?);
                }
                TokenKind::RightParen => {
                    return Err(parse_diag(
                        "EXPR-PARSE-EVENT-UNMATCHED-CLOSE",
                        "unmatched closing parenthesis",
                        separator_span,
                        &["remove ')' or add a matching '(' in an iff payload"],
                    ));
                }
                TokenKind::LeftParen => {
                    return Err(parse_diag(
                        "EXPR-PARSE-EVENT-UNMATCHED-OPEN",
                        "unmatched opening parenthesis",
                        separator_span,
                        &["event-level grouping is not supported; use parentheses only inside iff"],
                    ));
                }
                _ => {
                    return Err(parse_diag(
                        "EXPR-PARSE-EVENT-BROKEN-UNION",
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
                "EXPR-PARSE-EVENT-BROKEN-UNION",
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
                        "EXPR-PARSE-EVENT-LEX-CHAR",
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
                "EXPR-PARSE-EVENT-BROKEN-UNION",
                "broken event union segmentation",
                token_span,
                &["event term is missing before union separator"],
            )),
            TokenKind::LeftParen => Err(parse_diag(
                "EXPR-PARSE-EVENT-UNMATCHED-OPEN",
                "unmatched opening parenthesis",
                token_span,
                &["event-level grouping is not supported; use parentheses only inside iff"],
            )),
            TokenKind::RightParen => Err(parse_diag(
                "EXPR-PARSE-EVENT-UNMATCHED-CLOSE",
                "unmatched closing parenthesis",
                token_span,
                &["remove ')' or add a matching '(' in an iff payload"],
            )),
            TokenKind::KeywordIff => Err(parse_diag(
                "EXPR-PARSE-EVENT-BROKEN-UNION",
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
                "EXPR-PARSE-EVENT-MISSING-NAME",
                "missing signal name after edge keyword",
                keyword_span,
                &["expected a signal name after edge keyword"],
            ));
        };

        if name_token.kind != TokenKind::Identifier {
            let name_span = name_token.span;
            let diagnostic = match name_token.kind {
                TokenKind::LeftParen => parse_diag(
                    "EXPR-PARSE-EVENT-UNMATCHED-OPEN",
                    "unmatched opening parenthesis",
                    name_span,
                    &["event-level grouping is not supported; use parentheses only inside iff"],
                ),
                TokenKind::RightParen => parse_diag(
                    "EXPR-PARSE-EVENT-UNMATCHED-CLOSE",
                    "unmatched closing parenthesis",
                    name_span,
                    &["remove ')' or add a matching '(' in an iff payload"],
                ),
                _ => parse_diag(
                    "EXPR-PARSE-EVENT-MISSING-NAME",
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
                "EXPR-PARSE-EVENT-LEX-CHAR",
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
                "EXPR-PARSE-EVENT-BROKEN-UNION",
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
                "EXPR-PARSE-EVENT-EMPTY-IFF",
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
                            "EXPR-PARSE-EVENT-UNMATCHED-CLOSE",
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
                "EXPR-PARSE-EVENT-EMPTY-IFF",
                "empty iff payload",
                iff_span,
                &["expected logical expression after 'iff'"],
            ));
        }

        if let Some(unmatched_open) = open_stack.first().copied() {
            return Err(parse_diag(
                "EXPR-PARSE-EVENT-UNMATCHED-OPEN",
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
                "EXPR-PARSE-EVENT-EMPTY-IFF",
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

#[derive(Debug)]
struct LogicalParser<'a> {
    source: &'a str,
    tokens: Vec<LogicalToken>,
    index: usize,
}

impl<'a> LogicalParser<'a> {
    fn parse(&mut self) -> Result<LogicalExprAst, ExprDiagnostic> {
        let root = self.parse_conditional_expr()?;
        if !matches!(self.current().kind, LogicalTokenKind::Eof) {
            return Err(logical_parse_diag(
                "EXPR-PARSE-LOGICAL-TRAILING",
                "trailing tokens in logical expression",
                self.current().span,
                &["remove extra tokens after a complete expression"],
            ));
        }
        Ok(LogicalExprAst {
            span: root.span(),
            root,
        })
    }

    fn parse_conditional_expr(&mut self) -> Result<LogicalExprNode, ExprDiagnostic> {
        let condition = self.parse_logical_or_expr()?;
        if !matches!(self.current().kind, LogicalTokenKind::Question) {
            return Ok(condition);
        }

        self.index += 1;
        let when_true = self.parse_conditional_expr()?;
        if !matches!(self.current().kind, LogicalTokenKind::Colon) {
            return Err(logical_parse_diag(
                "EXPR-PARSE-LOGICAL-EXPECTED",
                "expected ':' in conditional expression",
                self.current().span,
                &["conditional expression form is cond ? a : b"],
            ));
        }
        self.index += 1;
        let when_false = self.parse_conditional_expr()?;
        let span = Span::new(condition.span().start, when_false.span().end);
        Ok(LogicalExprNode::Conditional {
            condition: Box::new(condition),
            when_true: Box::new(when_true),
            when_false: Box::new(when_false),
            span,
        })
    }

    fn parse_logical_or_expr(&mut self) -> Result<LogicalExprNode, ExprDiagnostic> {
        self.parse_left_assoc(
            |parser| parser.parse_logical_and_expr(),
            |kind| match kind {
                LogicalTokenKind::OrOr => Some(BinaryOpAst::LogicalOr),
                _ => None,
            },
        )
    }

    fn parse_logical_and_expr(&mut self) -> Result<LogicalExprNode, ExprDiagnostic> {
        self.parse_left_assoc(
            |parser| parser.parse_bitwise_or_expr(),
            |kind| match kind {
                LogicalTokenKind::AndAnd => Some(BinaryOpAst::LogicalAnd),
                _ => None,
            },
        )
    }

    fn parse_bitwise_or_expr(&mut self) -> Result<LogicalExprNode, ExprDiagnostic> {
        self.parse_left_assoc(
            |parser| parser.parse_bitwise_xor_expr(),
            |kind| match kind {
                LogicalTokenKind::Pipe => Some(BinaryOpAst::BitOr),
                _ => None,
            },
        )
    }

    fn parse_bitwise_xor_expr(&mut self) -> Result<LogicalExprNode, ExprDiagnostic> {
        self.parse_left_assoc(
            |parser| parser.parse_bitwise_and_expr(),
            |kind| match kind {
                LogicalTokenKind::Caret => Some(BinaryOpAst::BitXor),
                LogicalTokenKind::CaretTilde | LogicalTokenKind::TildeCaret => {
                    Some(BinaryOpAst::BitXnor)
                }
                _ => None,
            },
        )
    }

    fn parse_bitwise_and_expr(&mut self) -> Result<LogicalExprNode, ExprDiagnostic> {
        self.parse_left_assoc(
            |parser| parser.parse_equality_expr(),
            |kind| match kind {
                LogicalTokenKind::Amp => Some(BinaryOpAst::BitAnd),
                _ => None,
            },
        )
    }

    fn parse_equality_expr(&mut self) -> Result<LogicalExprNode, ExprDiagnostic> {
        self.parse_left_assoc(
            |parser| parser.parse_relational_expr(),
            |kind| match kind {
                LogicalTokenKind::EqEq => Some(BinaryOpAst::Eq),
                LogicalTokenKind::NotEq => Some(BinaryOpAst::Ne),
                LogicalTokenKind::EqEqEq => Some(BinaryOpAst::CaseEq),
                LogicalTokenKind::NotEqEq => Some(BinaryOpAst::CaseNe),
                LogicalTokenKind::EqWildcard => Some(BinaryOpAst::WildEq),
                LogicalTokenKind::NotEqWildcard => Some(BinaryOpAst::WildNe),
                _ => None,
            },
        )
    }

    fn parse_relational_expr(&mut self) -> Result<LogicalExprNode, ExprDiagnostic> {
        let mut node = self.parse_shift_expr()?;
        loop {
            match self.current().kind {
                LogicalTokenKind::Lt
                | LogicalTokenKind::Le
                | LogicalTokenKind::Gt
                | LogicalTokenKind::Ge => {
                    let op = match self.current().kind {
                        LogicalTokenKind::Lt => BinaryOpAst::Lt,
                        LogicalTokenKind::Le => BinaryOpAst::Le,
                        LogicalTokenKind::Gt => BinaryOpAst::Gt,
                        LogicalTokenKind::Ge => BinaryOpAst::Ge,
                        _ => unreachable!(),
                    };
                    self.index += 1;
                    let right = self.parse_shift_expr()?;
                    let span = Span::new(node.span().start, right.span().end);
                    node = LogicalExprNode::Binary {
                        op,
                        left: Box::new(node),
                        right: Box::new(right),
                        span,
                    };
                }
                LogicalTokenKind::KeywordInside => {
                    self.index += 1;
                    let (items, end_span) = self.parse_inside_set()?;
                    let span = Span::new(node.span().start, end_span.end);
                    node = LogicalExprNode::Inside {
                        expr: Box::new(node),
                        set: items,
                        span,
                    };
                }
                _ => break,
            }
        }
        Ok(node)
    }

    fn parse_shift_expr(&mut self) -> Result<LogicalExprNode, ExprDiagnostic> {
        self.parse_left_assoc(
            |parser| parser.parse_additive_expr(),
            |kind| match kind {
                LogicalTokenKind::ShiftLeft => Some(BinaryOpAst::ShiftLeft),
                LogicalTokenKind::ShiftRight => Some(BinaryOpAst::ShiftRight),
                LogicalTokenKind::ShiftArithLeft => Some(BinaryOpAst::ShiftArithLeft),
                LogicalTokenKind::ShiftArithRight => Some(BinaryOpAst::ShiftArithRight),
                _ => None,
            },
        )
    }

    fn parse_additive_expr(&mut self) -> Result<LogicalExprNode, ExprDiagnostic> {
        self.parse_left_assoc(
            |parser| parser.parse_multiplicative_expr(),
            |kind| match kind {
                LogicalTokenKind::Plus => Some(BinaryOpAst::Add),
                LogicalTokenKind::Minus => Some(BinaryOpAst::Subtract),
                _ => None,
            },
        )
    }

    fn parse_multiplicative_expr(&mut self) -> Result<LogicalExprNode, ExprDiagnostic> {
        self.parse_left_assoc(
            |parser| parser.parse_power_expr(),
            |kind| match kind {
                LogicalTokenKind::Star => Some(BinaryOpAst::Multiply),
                LogicalTokenKind::Slash => Some(BinaryOpAst::Divide),
                LogicalTokenKind::Percent => Some(BinaryOpAst::Modulo),
                _ => None,
            },
        )
    }

    fn parse_power_expr(&mut self) -> Result<LogicalExprNode, ExprDiagnostic> {
        self.parse_left_assoc(
            |parser| parser.parse_unary_expr(),
            |kind| match kind {
                LogicalTokenKind::Power => Some(BinaryOpAst::Power),
                _ => None,
            },
        )
    }

    fn parse_unary_expr(&mut self) -> Result<LogicalExprNode, ExprDiagnostic> {
        let op = match self.current().kind {
            LogicalTokenKind::Plus => Some(UnaryOpAst::Plus),
            LogicalTokenKind::Minus => Some(UnaryOpAst::Minus),
            LogicalTokenKind::Bang => Some(UnaryOpAst::LogicalNot),
            LogicalTokenKind::Tilde => Some(UnaryOpAst::BitNot),
            LogicalTokenKind::Amp => Some(UnaryOpAst::ReduceAnd),
            LogicalTokenKind::TildeAmp => Some(UnaryOpAst::ReduceNand),
            LogicalTokenKind::Pipe => Some(UnaryOpAst::ReduceOr),
            LogicalTokenKind::TildePipe => Some(UnaryOpAst::ReduceNor),
            LogicalTokenKind::Caret => Some(UnaryOpAst::ReduceXor),
            LogicalTokenKind::CaretTilde | LogicalTokenKind::TildeCaret => {
                Some(UnaryOpAst::ReduceXnor)
            }
            _ => None,
        };

        if let Some(op) = op {
            let start = self.current().span.start;
            self.index += 1;
            let expr = self.parse_unary_expr()?;
            let span = Span::new(start, expr.span().end);
            return Ok(LogicalExprNode::Unary {
                op,
                expr: Box::new(expr),
                span,
            });
        }

        self.parse_postfix_expr()
    }

    fn parse_postfix_expr(&mut self) -> Result<LogicalExprNode, ExprDiagnostic> {
        let mut node = self.parse_primary_expr()?;

        loop {
            match self.current().kind {
                LogicalTokenKind::LeftBracket => {
                    let selection = self.parse_selection_suffix(node)?;
                    node = selection;
                }
                LogicalTokenKind::Dot => {
                    let dot_span = self.current().span;
                    self.index += 1;
                    let token = self.current().clone();
                    match token.kind {
                        LogicalTokenKind::Identifier(ref name) if name == "triggered" => {
                            self.index += 1;
                            if !matches!(self.current().kind, LogicalTokenKind::LeftParen) {
                                return Err(logical_parse_diag(
                                    "EXPR-PARSE-LOGICAL-EXPECTED",
                                    "unsupported member-like suffix",
                                    dot_span,
                                    &["only .triggered() is supported as a member-like suffix"],
                                ));
                            }
                            let open = self.current().span;
                            self.index += 1;
                            let close = match self.current().kind {
                                LogicalTokenKind::RightParen => {
                                    let close = self.current().span;
                                    self.index += 1;
                                    close
                                }
                                LogicalTokenKind::Eof => {
                                    return Err(logical_parse_diag(
                                        "EXPR-PARSE-LOGICAL-EXPECTED",
                                        "triggered() is missing closing ')'",
                                        open,
                                        &["only .triggered() is supported as a member-like suffix"],
                                    ));
                                }
                                _ => {
                                    return Err(logical_parse_diag(
                                        "EXPR-PARSE-LOGICAL-EXPECTED",
                                        "triggered() does not take arguments",
                                        self.current().span,
                                        &["only .triggered() is supported as a member-like suffix"],
                                    ));
                                }
                            };
                            let span = Span::new(node.span().start, close.end);
                            node = LogicalExprNode::Triggered {
                                expr: Box::new(node),
                                span,
                            };
                        }
                        _ => {
                            return Err(logical_parse_diag(
                                "EXPR-PARSE-LOGICAL-EXPECTED",
                                "unsupported member-like suffix",
                                dot_span,
                                &["only .triggered() is supported as a member-like suffix"],
                            ));
                        }
                    }
                }
                _ => break,
            }
        }

        Ok(node)
    }

    fn parse_selection_suffix(
        &mut self,
        base: LogicalExprNode,
    ) -> Result<LogicalExprNode, ExprDiagnostic> {
        let open = self.current().span;
        self.index += 1;
        let first = self.parse_conditional_expr()?;
        match self.current().kind {
            LogicalTokenKind::RightBracket => {
                let close = self.current().span;
                self.index += 1;
                let span = Span::new(base.span().start, close.end);
                Ok(LogicalExprNode::Selection {
                    base: Box::new(base),
                    selection: SelectionKindAst::Bit {
                        index: Box::new(first),
                    },
                    span,
                })
            }
            LogicalTokenKind::Colon => {
                self.index += 1;
                let second = self.parse_conditional_expr()?;
                let close = self.expect_right_bracket("part-select")?;
                let span = Span::new(base.span().start, close.end);
                Ok(LogicalExprNode::Selection {
                    base: Box::new(base),
                    selection: SelectionKindAst::Part {
                        msb: Box::new(first),
                        lsb: Box::new(second),
                    },
                    span,
                })
            }
            LogicalTokenKind::PlusColon => {
                self.index += 1;
                let width = self.parse_conditional_expr()?;
                let close = self.expect_right_bracket("indexed part-select")?;
                let span = Span::new(base.span().start, close.end);
                Ok(LogicalExprNode::Selection {
                    base: Box::new(base),
                    selection: SelectionKindAst::IndexedUp {
                        base: Box::new(first),
                        width: Box::new(width),
                    },
                    span,
                })
            }
            LogicalTokenKind::MinusColon => {
                self.index += 1;
                let width = self.parse_conditional_expr()?;
                let close = self.expect_right_bracket("indexed part-select")?;
                let span = Span::new(base.span().start, close.end);
                Ok(LogicalExprNode::Selection {
                    base: Box::new(base),
                    selection: SelectionKindAst::IndexedDown {
                        base: Box::new(first),
                        width: Box::new(width),
                    },
                    span,
                })
            }
            _ => Err(logical_parse_diag(
                "EXPR-PARSE-LOGICAL-EXPECTED",
                "malformed selection suffix",
                open,
                &["expected ] or : or +: or -: in selection"],
            )),
        }
    }

    fn parse_inside_set(&mut self) -> Result<(Vec<InsideItemAst>, Span), ExprDiagnostic> {
        if !matches!(self.current().kind, LogicalTokenKind::LeftBrace) {
            return Err(logical_parse_diag(
                "EXPR-PARSE-LOGICAL-EXPECTED",
                "inside requires a braced set",
                self.current().span,
                &["use inside { item1, item2 }"],
            ));
        }
        let open = self.current().span;
        self.index += 1;

        if matches!(self.current().kind, LogicalTokenKind::RightBrace) {
            return Err(logical_parse_diag(
                "EXPR-PARSE-LOGICAL-EXPECTED",
                "inside set cannot be empty",
                self.current().span,
                &["add at least one set item"],
            ));
        }

        let mut items = Vec::new();
        loop {
            if matches!(self.current().kind, LogicalTokenKind::LeftBracket) {
                let range_open = self.current().span;
                self.index += 1;
                let low = self.parse_conditional_expr()?;
                if !matches!(self.current().kind, LogicalTokenKind::Colon) {
                    return Err(logical_parse_diag(
                        "EXPR-PARSE-LOGICAL-EXPECTED",
                        "inside range requires ':'",
                        self.current().span,
                        &["inside ranges use [low:high]"],
                    ));
                }
                self.index += 1;
                let high = self.parse_conditional_expr()?;
                let close = self.expect_right_bracket("inside range")?;
                items.push(InsideItemAst::Range {
                    low,
                    high,
                    span: Span::new(range_open.start, close.end),
                });
            } else {
                items.push(InsideItemAst::Expr(self.parse_conditional_expr()?));
            }

            if matches!(self.current().kind, LogicalTokenKind::Comma) {
                self.index += 1;
                continue;
            }
            break;
        }

        if !matches!(self.current().kind, LogicalTokenKind::RightBrace) {
            return Err(logical_parse_diag(
                "EXPR-PARSE-LOGICAL-EXPECTED",
                "inside set must end with '}'",
                self.current().span,
                &["close the inside set with '}'"],
            ));
        }
        let close = self.current().span;
        self.index += 1;
        Ok((items, Span::new(open.start, close.end)))
    }

    fn parse_primary_expr(&mut self) -> Result<LogicalExprNode, ExprDiagnostic> {
        if let Some(enum_label) = self.try_parse_enum_label_expr()? {
            return Ok(enum_label);
        }

        if let Some(cast) = self.try_parse_cast_expr()? {
            return Ok(cast);
        }

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
            LogicalTokenKind::RealLiteral(literal) => {
                self.index += 1;
                Ok(LogicalExprNode::RealLiteral {
                    literal,
                    span: token.span,
                })
            }
            LogicalTokenKind::StringLiteral(literal) => {
                self.index += 1;
                Ok(LogicalExprNode::StringLiteral {
                    literal,
                    span: token.span,
                })
            }
            LogicalTokenKind::LeftParen => {
                let open = token.span;
                self.index += 1;
                let expr = self.parse_conditional_expr()?;
                if !matches!(self.current().kind, LogicalTokenKind::RightParen) {
                    return Err(logical_parse_diag(
                        "EXPR-PARSE-LOGICAL-UNMATCHED-OPEN",
                        "unmatched opening parenthesis in logical expression",
                        open,
                        &["close this '('"],
                    ));
                }
                let close = self.current().span;
                self.index += 1;
                Ok(LogicalExprNode::Parenthesized {
                    expr: Box::new(expr),
                    span: Span::new(open.start, close.end),
                })
            }
            LogicalTokenKind::LeftBrace => self.parse_braced_primary(),
            LogicalTokenKind::RightParen => Err(logical_parse_diag(
                "EXPR-PARSE-LOGICAL-UNMATCHED-CLOSE",
                "unmatched closing parenthesis in logical expression",
                token.span,
                &["remove ')' or add a matching '('"],
            )),
            LogicalTokenKind::Eof => Err(logical_parse_diag(
                "EXPR-PARSE-LOGICAL-EXPECTED",
                "incomplete logical expression",
                token.span,
                &["expected an operand or literal"],
            )),
            _ => Err(logical_parse_diag(
                "EXPR-PARSE-LOGICAL-EXPECTED",
                "expected logical expression operand",
                token.span,
                &["expected operand reference, literal, cast, or parenthesized expression"],
            )),
        }
    }

    fn try_parse_enum_label_expr(&mut self) -> Result<Option<LogicalExprNode>, ExprDiagnostic> {
        let save = self.index;
        let token = self.current().clone();
        let LogicalTokenKind::Identifier(name) = token.kind else {
            return Ok(None);
        };
        if name != "type" {
            return Ok(None);
        }

        self.index += 1;
        if !matches!(self.current().kind, LogicalTokenKind::LeftParen) {
            self.index = save;
            return Ok(None);
        }
        self.index += 1;

        let operand = match self.current().clone().kind {
            LogicalTokenKind::Identifier(operand) => {
                let span = self.current().span;
                self.index += 1;
                (operand, span)
            }
            _ => {
                self.index = save;
                return Ok(None);
            }
        };

        if !matches!(self.current().kind, LogicalTokenKind::RightParen) {
            self.index = save;
            return Ok(None);
        }
        let close = self.current().span;
        self.index += 1;

        if !matches!(self.current().kind, LogicalTokenKind::DoubleColon) {
            self.index = save;
            return Ok(None);
        }
        self.index += 1;

        let (label, label_span) = match self.current().clone().kind {
            LogicalTokenKind::Identifier(label) => {
                let span = self.current().span;
                self.index += 1;
                (label, span)
            }
            _ => {
                return Err(logical_parse_diag(
                    "EXPR-PARSE-LOGICAL-EXPECTED",
                    "enum label reference is missing a label",
                    self.current().span,
                    &["enum label form is type(enum_operand_reference)::LABEL"],
                ));
            }
        };

        Ok(Some(LogicalExprNode::EnumLabel {
            operand: operand.0,
            operand_span: operand.1,
            label,
            label_span,
            span: Span::new(token.span.start, label_span.end.max(close.end)),
        }))
    }

    fn parse_braced_primary(&mut self) -> Result<LogicalExprNode, ExprDiagnostic> {
        let open = self.current().span;
        self.index += 1;
        if matches!(self.current().kind, LogicalTokenKind::RightBrace) {
            return Err(logical_parse_diag(
                "EXPR-PARSE-LOGICAL-EXPECTED",
                "empty concatenation is not valid",
                open,
                &["add one or more expressions inside braces"],
            ));
        }

        let first = self.parse_conditional_expr()?;
        if matches!(self.current().kind, LogicalTokenKind::LeftBrace) {
            self.index += 1;
            let repeated = self.parse_conditional_expr()?;
            if !matches!(self.current().kind, LogicalTokenKind::RightBrace) {
                return Err(logical_parse_diag(
                    "EXPR-PARSE-LOGICAL-EXPECTED",
                    "replication is missing inner closing '}'",
                    self.current().span,
                    &["replication form is {N{expr}}"],
                ));
            }
            self.index += 1;
            if !matches!(self.current().kind, LogicalTokenKind::RightBrace) {
                return Err(logical_parse_diag(
                    "EXPR-PARSE-LOGICAL-EXPECTED",
                    "replication is missing outer closing '}'",
                    self.current().span,
                    &["replication form is {N{expr}}"],
                ));
            }
            let close = self.current().span;
            self.index += 1;
            return Ok(LogicalExprNode::Replication {
                count: Box::new(first),
                expr: Box::new(repeated),
                span: Span::new(open.start, close.end),
            });
        }

        let mut items = vec![first];
        while matches!(self.current().kind, LogicalTokenKind::Comma) {
            self.index += 1;
            items.push(self.parse_conditional_expr()?);
        }
        if !matches!(self.current().kind, LogicalTokenKind::RightBrace) {
            return Err(logical_parse_diag(
                "EXPR-PARSE-LOGICAL-EXPECTED",
                "concatenation must end with '}'",
                self.current().span,
                &["concatenation form is {a, b, c}"],
            ));
        }
        let close = self.current().span;
        self.index += 1;
        Ok(LogicalExprNode::Concatenation {
            items,
            span: Span::new(open.start, close.end),
        })
    }

    fn try_parse_cast_expr(&mut self) -> Result<Option<LogicalExprNode>, ExprDiagnostic> {
        let save = self.index;
        let Some(candidate) = self.try_parse_cast_target_candidate()? else {
            return Ok(None);
        };

        if !matches!(self.current().kind, LogicalTokenKind::Apostrophe) {
            self.index = save;
            return Ok(None);
        }
        self.index += 1;

        if let Some(error) = candidate.deferred_error {
            return Err(error);
        }
        let Some(target) = candidate.target else {
            return Err(logical_parse_diag(
                "EXPR-PARSE-LOGICAL-CAST",
                "invalid cast target",
                candidate.span,
                &["cast target must be a supported expression type"],
            ));
        };

        if !matches!(self.current().kind, LogicalTokenKind::LeftParen) {
            return Err(logical_parse_diag(
                "EXPR-PARSE-LOGICAL-CAST",
                "malformed cast expression",
                self.current().span,
                &["cast form is type'(expr)"],
            ));
        }
        self.index += 1;
        let inner = self.parse_conditional_expr()?;
        if !matches!(self.current().kind, LogicalTokenKind::RightParen) {
            return Err(logical_parse_diag(
                "EXPR-PARSE-LOGICAL-UNMATCHED-OPEN",
                "unmatched opening parenthesis in cast expression",
                candidate.span,
                &["cast form is type'(expr)"],
            ));
        }
        let close = self.current().span;
        self.index += 1;

        Ok(Some(LogicalExprNode::Cast {
            target,
            expr: Box::new(inner),
            span: Span::new(candidate.span.start, close.end),
        }))
    }

    fn try_parse_cast_target_candidate(
        &mut self,
    ) -> Result<Option<CastTargetCandidate>, ExprDiagnostic> {
        let token = self.current().clone();
        let LogicalTokenKind::Identifier(name) = token.kind else {
            return Ok(None);
        };

        let Some(name) = Some(name.as_str()) else {
            return Ok(None);
        };

        match name {
            "signed" => {
                self.index += 1;
                if let Some(bit_logic) = self.try_parse_bit_logic_type(true)? {
                    return Ok(Some(CastTargetCandidate {
                        target: Some(bit_logic.0),
                        deferred_error: None,
                        span: Span::new(token.span.start, bit_logic.1.end),
                    }));
                }
                Ok(Some(CastTargetCandidate {
                    target: Some(CastTargetAst::Signed),
                    deferred_error: None,
                    span: token.span,
                }))
            }
            "unsigned" => {
                self.index += 1;
                if let Some(bit_logic) = self.try_parse_bit_logic_type(false)? {
                    return Ok(Some(CastTargetCandidate {
                        target: Some(bit_logic.0),
                        deferred_error: None,
                        span: Span::new(token.span.start, bit_logic.1.end),
                    }));
                }
                Ok(Some(CastTargetCandidate {
                    target: Some(CastTargetAst::Unsigned),
                    deferred_error: None,
                    span: token.span,
                }))
            }
            "bit" | "logic" => {
                self.index += 1;
                let target = self.parse_bit_logic_target(name == "logic", false, token.span)?;
                Ok(Some(CastTargetCandidate {
                    target: Some(target.0),
                    deferred_error: None,
                    span: Span::new(token.span.start, target.1.end),
                }))
            }
            "byte" => {
                self.index += 1;
                Ok(Some(CastTargetCandidate {
                    target: Some(CastTargetAst::IntegerLike(IntegerLikeKind::Byte)),
                    deferred_error: None,
                    span: token.span,
                }))
            }
            "shortint" => {
                self.index += 1;
                Ok(Some(CastTargetCandidate {
                    target: Some(CastTargetAst::IntegerLike(IntegerLikeKind::Shortint)),
                    deferred_error: None,
                    span: token.span,
                }))
            }
            "int" => {
                self.index += 1;
                Ok(Some(CastTargetCandidate {
                    target: Some(CastTargetAst::IntegerLike(IntegerLikeKind::Int)),
                    deferred_error: None,
                    span: token.span,
                }))
            }
            "longint" => {
                self.index += 1;
                Ok(Some(CastTargetCandidate {
                    target: Some(CastTargetAst::IntegerLike(IntegerLikeKind::Longint)),
                    deferred_error: None,
                    span: token.span,
                }))
            }
            "integer" => {
                self.index += 1;
                Ok(Some(CastTargetCandidate {
                    target: Some(CastTargetAst::IntegerLike(IntegerLikeKind::Integer)),
                    deferred_error: None,
                    span: token.span,
                }))
            }
            "time" => {
                self.index += 1;
                Ok(Some(CastTargetCandidate {
                    target: Some(CastTargetAst::IntegerLike(IntegerLikeKind::Time)),
                    deferred_error: None,
                    span: token.span,
                }))
            }
            "real" => {
                self.index += 1;
                Ok(Some(CastTargetCandidate {
                    target: Some(CastTargetAst::Real),
                    deferred_error: None,
                    span: token.span,
                }))
            }
            "string" => {
                self.index += 1;
                Ok(Some(CastTargetCandidate {
                    target: Some(CastTargetAst::String),
                    deferred_error: None,
                    span: token.span,
                }))
            }
            "type" => {
                let save = self.index;
                self.index += 1;
                if !matches!(self.current().kind, LogicalTokenKind::LeftParen) {
                    self.index = save;
                    return Ok(None);
                }
                let open = self.current().span;
                self.index += 1;
                let (operand, operand_span) = match self.current().clone().kind {
                    LogicalTokenKind::Identifier(operand) => {
                        let span = self.current().span;
                        self.index += 1;
                        (operand, span)
                    }
                    LogicalTokenKind::Eof => {
                        return Err(logical_parse_diag(
                            "EXPR-PARSE-LOGICAL-UNMATCHED-OPEN",
                            "unmatched opening parenthesis in type(...) cast target",
                            open,
                            &["type(...) forms must close the recovered operand reference"],
                        ));
                    }
                    _ => {
                        return Err(logical_parse_diag(
                            "EXPR-PARSE-LOGICAL-CAST",
                            "type(...) cast target must name an operand reference",
                            self.current().span,
                            &["type(...) cast target is type(operand_reference)"],
                        ));
                    }
                };
                if !matches!(self.current().kind, LogicalTokenKind::RightParen) {
                    return Err(logical_parse_diag(
                        "EXPR-PARSE-LOGICAL-CAST",
                        "type(...) cast target is missing closing ')'",
                        open,
                        &["type(...) cast target is type(operand_reference)"],
                    ));
                }
                let close = self.current().span;
                self.index += 1;
                Ok(Some(CastTargetCandidate {
                    target: Some(CastTargetAst::RecoveredType {
                        name: operand,
                        span: operand_span,
                    }),
                    deferred_error: None,
                    span: Span::new(token.span.start, close.end),
                }))
            }
            _ => Ok(None),
        }
    }

    fn try_parse_bit_logic_type(
        &mut self,
        is_signed: bool,
    ) -> Result<Option<(CastTargetAst, Span)>, ExprDiagnostic> {
        let token = self.current().clone();
        match token.kind {
            LogicalTokenKind::Identifier(ref name) if name == "bit" || name == "logic" => {
                self.index += 1;
                let is_four_state = name == "logic";
                let parsed = self.parse_bit_logic_target(is_four_state, is_signed, token.span)?;
                Ok(Some(parsed))
            }
            _ => Ok(None),
        }
    }

    fn parse_bit_logic_target(
        &mut self,
        is_four_state: bool,
        is_signed: bool,
        type_span: Span,
    ) -> Result<(CastTargetAst, Span), ExprDiagnostic> {
        let mut width = 1u32;
        let mut end = type_span;
        if matches!(self.current().kind, LogicalTokenKind::LeftBracket) {
            let open = self.current().span;
            self.index += 1;
            let token = self.current().clone();
            let literal = match token.kind {
                LogicalTokenKind::IntegralLiteral(ref literal) => literal,
                _ => {
                    return Err(logical_parse_diag(
                        "EXPR-PARSE-LOGICAL-CAST",
                        "cast target width must be a positive integer",
                        token.span,
                        &["bit/logic cast widths use bit[N] or logic[N]"],
                    ));
                }
            };
            if literal.width.is_some()
                || !literal.signed
                || !matches!(literal.base, IntegralBase::Decimal)
            {
                return Err(logical_parse_diag(
                    "EXPR-PARSE-LOGICAL-CAST",
                    "cast target width must be an unsized decimal integer",
                    token.span,
                    &["bit/logic cast widths use bit[N] or logic[N]"],
                ));
            }
            width = literal.digits.parse::<u32>().map_err(|_| {
                logical_parse_diag(
                    "EXPR-PARSE-LOGICAL-CAST",
                    "cast target width is out of range",
                    token.span,
                    &["bit/logic cast widths must fit in u32"],
                )
            })?;
            if width == 0 {
                return Err(logical_parse_diag(
                    "EXPR-PARSE-LOGICAL-CAST",
                    "cast target width must be greater than zero",
                    token.span,
                    &["bit/logic cast widths must be positive"],
                ));
            }
            self.index += 1;
            if !matches!(self.current().kind, LogicalTokenKind::RightBracket) {
                return Err(logical_parse_diag(
                    "EXPR-PARSE-LOGICAL-CAST",
                    "cast target width is missing closing ']'",
                    open,
                    &["bit/logic cast widths use [N]"],
                ));
            }
            end = self.current().span;
            self.index += 1;
        }

        Ok((
            CastTargetAst::BitVector {
                width,
                is_four_state,
                is_signed,
            },
            end,
        ))
    }

    fn expect_right_bracket(&mut self, context: &str) -> Result<Span, ExprDiagnostic> {
        if !matches!(self.current().kind, LogicalTokenKind::RightBracket) {
            return Err(logical_parse_diag(
                "EXPR-PARSE-LOGICAL-EXPECTED",
                "missing closing ']'",
                self.current().span,
                &[context],
            ));
        }
        let span = self.current().span;
        self.index += 1;
        Ok(span)
    }

    fn parse_left_assoc<F, G>(
        &mut self,
        mut parse_operand: F,
        mut map_op: G,
    ) -> Result<LogicalExprNode, ExprDiagnostic>
    where
        F: FnMut(&mut Self) -> Result<LogicalExprNode, ExprDiagnostic>,
        G: FnMut(&LogicalTokenKind) -> Option<BinaryOpAst>,
    {
        let mut node = parse_operand(self)?;
        while let Some(op) = map_op(&self.current().kind) {
            self.index += 1;
            let right = parse_operand(self)?;
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

    fn current(&self) -> &LogicalToken {
        self.tokens
            .get(self.index)
            .expect("logical parser keeps an eof sentinel")
    }

    fn peek_kind(&self, lookahead: usize) -> Option<LogicalTokenKind> {
        self.tokens
            .get(self.index + lookahead)
            .map(|token| token.kind.clone())
    }
}

#[derive(Debug)]
struct CastTargetCandidate {
    target: Option<CastTargetAst>,
    deferred_error: Option<ExprDiagnostic>,
    span: Span,
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
mod parser_surface_matrix {
    use super::{parse_event_expr_ast, parse_logical_expr_ast};

    fn run_event_case(index: usize) {
        let label = alpha_label(index);
        let source = event_case_source(index, &label);
        if let Err(error) = parse_event_expr_ast(&source) {
            panic!("event surface case {index} ({source}) failed: {error:?}");
        }
    }

    fn run_logical_case(index: usize) {
        let label = alpha_label(index);
        let source = logical_case_source(index, &label);
        if let Err(error) = parse_logical_expr_ast(&source) {
            panic!("logical surface case {index} ({source}) failed: {error:?}");
        }
    }

    fn alpha_label(mut index: usize) -> String {
        let mut chars = ['a'; 3];
        for slot in (0..3).rev() {
            chars[slot] = (b'a' + (index % 26) as u8) as char;
            index /= 26;
        }
        chars.into_iter().collect()
    }

    fn event_case_source(index: usize, label: &str) -> String {
        match index % 5 {
            0 => format!("posedge clk{label}"),
            1 => format!("negedge rst{label} iff enable{label}"),
            2 => format!("event{label} or posedge clk{label}"),
            3 => format!("a{label}, b{label}, c{label}"),
            _ => format!("edge data{label} iff valid{label}"),
        }
    }

    fn logical_case_source(index: usize, label: &str) -> String {
        let nibble = format!("{:x}", index % 16);
        match index % 20 {
            0 => format!("sig{label} + {}", index % 17),
            1 => format!("(sig{label} & mask{label}) == 1"),
            2 => format!("sig{label}[0] == 1'b1"),
            3 => format!("sig{label}[3:0] != 4'h{nibble}"),
            4 => format!("sig{label} inside {{1, 2, 3}}"),
            5 => format!("flag{label} ? yes{label} : no{label}"),
            6 => format!("{{2{{sig{label}}}}}"),
            7 => format!("sig{label} << 1"),
            8 => format!("sig{label} >>> 2"),
            9 => format!("~&sig{label}"),
            10 => format!("^~sig{label}"),
            11 => format!("type(state{label})::IDLE"),
            12 => format!("logic[8]'(sig{label})"),
            13 => format!("unsigned'(sig{label})"),
            14 => format!("sig{label}.triggered()"),
            15 => format!("sig{label} ** 2"),
            16 => format!("signed'(sig{label})"),
            17 => format!("sig{label} || ready{label}"),
            18 => format!("bit'(sig{label})"),
            _ => format!("sig{label}[idx{label} +: 2] == 2'b01"),
        }
    }

    #[test]
    fn parser_surface_event_case_000() {
        run_event_case(0);
    }

    #[test]
    fn parser_surface_event_case_001() {
        run_event_case(1);
    }

    #[test]
    fn parser_surface_event_case_002() {
        run_event_case(2);
    }

    #[test]
    fn parser_surface_event_case_003() {
        run_event_case(3);
    }

    #[test]
    fn parser_surface_event_case_004() {
        run_event_case(4);
    }

    #[test]
    fn parser_surface_event_case_005() {
        run_event_case(5);
    }

    #[test]
    fn parser_surface_event_case_006() {
        run_event_case(6);
    }

    #[test]
    fn parser_surface_event_case_007() {
        run_event_case(7);
    }

    #[test]
    fn parser_surface_event_case_008() {
        run_event_case(8);
    }

    #[test]
    fn parser_surface_event_case_009() {
        run_event_case(9);
    }

    #[test]
    fn parser_surface_event_case_010() {
        run_event_case(10);
    }

    #[test]
    fn parser_surface_event_case_011() {
        run_event_case(11);
    }

    #[test]
    fn parser_surface_event_case_012() {
        run_event_case(12);
    }

    #[test]
    fn parser_surface_event_case_013() {
        run_event_case(13);
    }

    #[test]
    fn parser_surface_event_case_014() {
        run_event_case(14);
    }

    #[test]
    fn parser_surface_event_case_015() {
        run_event_case(15);
    }

    #[test]
    fn parser_surface_event_case_016() {
        run_event_case(16);
    }

    #[test]
    fn parser_surface_event_case_017() {
        run_event_case(17);
    }

    #[test]
    fn parser_surface_event_case_018() {
        run_event_case(18);
    }

    #[test]
    fn parser_surface_event_case_019() {
        run_event_case(19);
    }

    #[test]
    fn parser_surface_event_case_020() {
        run_event_case(20);
    }

    #[test]
    fn parser_surface_event_case_021() {
        run_event_case(21);
    }

    #[test]
    fn parser_surface_event_case_022() {
        run_event_case(22);
    }

    #[test]
    fn parser_surface_event_case_023() {
        run_event_case(23);
    }

    #[test]
    fn parser_surface_event_case_024() {
        run_event_case(24);
    }

    #[test]
    fn parser_surface_event_case_025() {
        run_event_case(25);
    }

    #[test]
    fn parser_surface_event_case_026() {
        run_event_case(26);
    }

    #[test]
    fn parser_surface_event_case_027() {
        run_event_case(27);
    }

    #[test]
    fn parser_surface_event_case_028() {
        run_event_case(28);
    }

    #[test]
    fn parser_surface_event_case_029() {
        run_event_case(29);
    }

    #[test]
    fn parser_surface_event_case_030() {
        run_event_case(30);
    }

    #[test]
    fn parser_surface_event_case_031() {
        run_event_case(31);
    }

    #[test]
    fn parser_surface_event_case_032() {
        run_event_case(32);
    }

    #[test]
    fn parser_surface_event_case_033() {
        run_event_case(33);
    }

    #[test]
    fn parser_surface_event_case_034() {
        run_event_case(34);
    }

    #[test]
    fn parser_surface_event_case_035() {
        run_event_case(35);
    }

    #[test]
    fn parser_surface_event_case_036() {
        run_event_case(36);
    }

    #[test]
    fn parser_surface_event_case_037() {
        run_event_case(37);
    }

    #[test]
    fn parser_surface_event_case_038() {
        run_event_case(38);
    }

    #[test]
    fn parser_surface_event_case_039() {
        run_event_case(39);
    }

    #[test]
    fn parser_surface_event_case_040() {
        run_event_case(40);
    }

    #[test]
    fn parser_surface_event_case_041() {
        run_event_case(41);
    }

    #[test]
    fn parser_surface_event_case_042() {
        run_event_case(42);
    }

    #[test]
    fn parser_surface_event_case_043() {
        run_event_case(43);
    }

    #[test]
    fn parser_surface_event_case_044() {
        run_event_case(44);
    }

    #[test]
    fn parser_surface_event_case_045() {
        run_event_case(45);
    }

    #[test]
    fn parser_surface_event_case_046() {
        run_event_case(46);
    }

    #[test]
    fn parser_surface_event_case_047() {
        run_event_case(47);
    }

    #[test]
    fn parser_surface_event_case_048() {
        run_event_case(48);
    }

    #[test]
    fn parser_surface_event_case_049() {
        run_event_case(49);
    }

    #[test]
    fn parser_surface_event_case_050() {
        run_event_case(50);
    }

    #[test]
    fn parser_surface_event_case_051() {
        run_event_case(51);
    }

    #[test]
    fn parser_surface_event_case_052() {
        run_event_case(52);
    }

    #[test]
    fn parser_surface_event_case_053() {
        run_event_case(53);
    }

    #[test]
    fn parser_surface_event_case_054() {
        run_event_case(54);
    }

    #[test]
    fn parser_surface_event_case_055() {
        run_event_case(55);
    }

    #[test]
    fn parser_surface_event_case_056() {
        run_event_case(56);
    }

    #[test]
    fn parser_surface_event_case_057() {
        run_event_case(57);
    }

    #[test]
    fn parser_surface_event_case_058() {
        run_event_case(58);
    }

    #[test]
    fn parser_surface_event_case_059() {
        run_event_case(59);
    }

    #[test]
    fn parser_surface_event_case_060() {
        run_event_case(60);
    }

    #[test]
    fn parser_surface_event_case_061() {
        run_event_case(61);
    }

    #[test]
    fn parser_surface_event_case_062() {
        run_event_case(62);
    }

    #[test]
    fn parser_surface_event_case_063() {
        run_event_case(63);
    }

    #[test]
    fn parser_surface_event_case_064() {
        run_event_case(64);
    }

    #[test]
    fn parser_surface_event_case_065() {
        run_event_case(65);
    }

    #[test]
    fn parser_surface_event_case_066() {
        run_event_case(66);
    }

    #[test]
    fn parser_surface_event_case_067() {
        run_event_case(67);
    }

    #[test]
    fn parser_surface_event_case_068() {
        run_event_case(68);
    }

    #[test]
    fn parser_surface_event_case_069() {
        run_event_case(69);
    }

    #[test]
    fn parser_surface_event_case_070() {
        run_event_case(70);
    }

    #[test]
    fn parser_surface_event_case_071() {
        run_event_case(71);
    }

    #[test]
    fn parser_surface_event_case_072() {
        run_event_case(72);
    }

    #[test]
    fn parser_surface_event_case_073() {
        run_event_case(73);
    }

    #[test]
    fn parser_surface_event_case_074() {
        run_event_case(74);
    }

    #[test]
    fn parser_surface_event_case_075() {
        run_event_case(75);
    }

    #[test]
    fn parser_surface_event_case_076() {
        run_event_case(76);
    }

    #[test]
    fn parser_surface_event_case_077() {
        run_event_case(77);
    }

    #[test]
    fn parser_surface_event_case_078() {
        run_event_case(78);
    }

    #[test]
    fn parser_surface_event_case_079() {
        run_event_case(79);
    }

    #[test]
    fn parser_surface_event_case_080() {
        run_event_case(80);
    }

    #[test]
    fn parser_surface_event_case_081() {
        run_event_case(81);
    }

    #[test]
    fn parser_surface_event_case_082() {
        run_event_case(82);
    }

    #[test]
    fn parser_surface_event_case_083() {
        run_event_case(83);
    }

    #[test]
    fn parser_surface_event_case_084() {
        run_event_case(84);
    }

    #[test]
    fn parser_surface_event_case_085() {
        run_event_case(85);
    }

    #[test]
    fn parser_surface_event_case_086() {
        run_event_case(86);
    }

    #[test]
    fn parser_surface_event_case_087() {
        run_event_case(87);
    }

    #[test]
    fn parser_surface_event_case_088() {
        run_event_case(88);
    }

    #[test]
    fn parser_surface_event_case_089() {
        run_event_case(89);
    }

    #[test]
    fn parser_surface_event_case_090() {
        run_event_case(90);
    }

    #[test]
    fn parser_surface_event_case_091() {
        run_event_case(91);
    }

    #[test]
    fn parser_surface_event_case_092() {
        run_event_case(92);
    }

    #[test]
    fn parser_surface_event_case_093() {
        run_event_case(93);
    }

    #[test]
    fn parser_surface_event_case_094() {
        run_event_case(94);
    }

    #[test]
    fn parser_surface_event_case_095() {
        run_event_case(95);
    }

    #[test]
    fn parser_surface_event_case_096() {
        run_event_case(96);
    }

    #[test]
    fn parser_surface_event_case_097() {
        run_event_case(97);
    }

    #[test]
    fn parser_surface_event_case_098() {
        run_event_case(98);
    }

    #[test]
    fn parser_surface_event_case_099() {
        run_event_case(99);
    }

    #[test]
    fn parser_surface_event_case_100() {
        run_event_case(100);
    }

    #[test]
    fn parser_surface_event_case_101() {
        run_event_case(101);
    }

    #[test]
    fn parser_surface_event_case_102() {
        run_event_case(102);
    }

    #[test]
    fn parser_surface_event_case_103() {
        run_event_case(103);
    }

    #[test]
    fn parser_surface_event_case_104() {
        run_event_case(104);
    }

    #[test]
    fn parser_surface_event_case_105() {
        run_event_case(105);
    }

    #[test]
    fn parser_surface_event_case_106() {
        run_event_case(106);
    }

    #[test]
    fn parser_surface_event_case_107() {
        run_event_case(107);
    }

    #[test]
    fn parser_surface_event_case_108() {
        run_event_case(108);
    }

    #[test]
    fn parser_surface_event_case_109() {
        run_event_case(109);
    }

    #[test]
    fn parser_surface_event_case_110() {
        run_event_case(110);
    }

    #[test]
    fn parser_surface_event_case_111() {
        run_event_case(111);
    }

    #[test]
    fn parser_surface_event_case_112() {
        run_event_case(112);
    }

    #[test]
    fn parser_surface_event_case_113() {
        run_event_case(113);
    }

    #[test]
    fn parser_surface_event_case_114() {
        run_event_case(114);
    }

    #[test]
    fn parser_surface_event_case_115() {
        run_event_case(115);
    }

    #[test]
    fn parser_surface_event_case_116() {
        run_event_case(116);
    }

    #[test]
    fn parser_surface_event_case_117() {
        run_event_case(117);
    }

    #[test]
    fn parser_surface_event_case_118() {
        run_event_case(118);
    }

    #[test]
    fn parser_surface_event_case_119() {
        run_event_case(119);
    }

    #[test]
    fn parser_surface_event_case_120() {
        run_event_case(120);
    }

    #[test]
    fn parser_surface_event_case_121() {
        run_event_case(121);
    }

    #[test]
    fn parser_surface_event_case_122() {
        run_event_case(122);
    }

    #[test]
    fn parser_surface_event_case_123() {
        run_event_case(123);
    }

    #[test]
    fn parser_surface_event_case_124() {
        run_event_case(124);
    }

    #[test]
    fn parser_surface_event_case_125() {
        run_event_case(125);
    }

    #[test]
    fn parser_surface_event_case_126() {
        run_event_case(126);
    }

    #[test]
    fn parser_surface_event_case_127() {
        run_event_case(127);
    }

    #[test]
    fn parser_surface_event_case_128() {
        run_event_case(128);
    }

    #[test]
    fn parser_surface_event_case_129() {
        run_event_case(129);
    }

    #[test]
    fn parser_surface_event_case_130() {
        run_event_case(130);
    }

    #[test]
    fn parser_surface_event_case_131() {
        run_event_case(131);
    }

    #[test]
    fn parser_surface_event_case_132() {
        run_event_case(132);
    }

    #[test]
    fn parser_surface_event_case_133() {
        run_event_case(133);
    }

    #[test]
    fn parser_surface_event_case_134() {
        run_event_case(134);
    }

    #[test]
    fn parser_surface_event_case_135() {
        run_event_case(135);
    }

    #[test]
    fn parser_surface_event_case_136() {
        run_event_case(136);
    }

    #[test]
    fn parser_surface_event_case_137() {
        run_event_case(137);
    }

    #[test]
    fn parser_surface_event_case_138() {
        run_event_case(138);
    }

    #[test]
    fn parser_surface_event_case_139() {
        run_event_case(139);
    }

    #[test]
    fn parser_surface_event_case_140() {
        run_event_case(140);
    }

    #[test]
    fn parser_surface_event_case_141() {
        run_event_case(141);
    }

    #[test]
    fn parser_surface_event_case_142() {
        run_event_case(142);
    }

    #[test]
    fn parser_surface_event_case_143() {
        run_event_case(143);
    }

    #[test]
    fn parser_surface_event_case_144() {
        run_event_case(144);
    }

    #[test]
    fn parser_surface_event_case_145() {
        run_event_case(145);
    }

    #[test]
    fn parser_surface_event_case_146() {
        run_event_case(146);
    }

    #[test]
    fn parser_surface_event_case_147() {
        run_event_case(147);
    }

    #[test]
    fn parser_surface_event_case_148() {
        run_event_case(148);
    }

    #[test]
    fn parser_surface_event_case_149() {
        run_event_case(149);
    }

    #[test]
    fn parser_surface_event_case_150() {
        run_event_case(150);
    }

    #[test]
    fn parser_surface_event_case_151() {
        run_event_case(151);
    }

    #[test]
    fn parser_surface_event_case_152() {
        run_event_case(152);
    }

    #[test]
    fn parser_surface_event_case_153() {
        run_event_case(153);
    }

    #[test]
    fn parser_surface_event_case_154() {
        run_event_case(154);
    }

    #[test]
    fn parser_surface_event_case_155() {
        run_event_case(155);
    }

    #[test]
    fn parser_surface_event_case_156() {
        run_event_case(156);
    }

    #[test]
    fn parser_surface_event_case_157() {
        run_event_case(157);
    }

    #[test]
    fn parser_surface_event_case_158() {
        run_event_case(158);
    }

    #[test]
    fn parser_surface_event_case_159() {
        run_event_case(159);
    }

    #[test]
    fn parser_surface_event_case_160() {
        run_event_case(160);
    }

    #[test]
    fn parser_surface_event_case_161() {
        run_event_case(161);
    }

    #[test]
    fn parser_surface_event_case_162() {
        run_event_case(162);
    }

    #[test]
    fn parser_surface_event_case_163() {
        run_event_case(163);
    }

    #[test]
    fn parser_surface_event_case_164() {
        run_event_case(164);
    }

    #[test]
    fn parser_surface_event_case_165() {
        run_event_case(165);
    }

    #[test]
    fn parser_surface_event_case_166() {
        run_event_case(166);
    }

    #[test]
    fn parser_surface_event_case_167() {
        run_event_case(167);
    }

    #[test]
    fn parser_surface_event_case_168() {
        run_event_case(168);
    }

    #[test]
    fn parser_surface_event_case_169() {
        run_event_case(169);
    }

    #[test]
    fn parser_surface_logical_case_000() {
        run_logical_case(0);
    }

    #[test]
    fn parser_surface_logical_case_001() {
        run_logical_case(1);
    }

    #[test]
    fn parser_surface_logical_case_002() {
        run_logical_case(2);
    }

    #[test]
    fn parser_surface_logical_case_003() {
        run_logical_case(3);
    }

    #[test]
    fn parser_surface_logical_case_004() {
        run_logical_case(4);
    }

    #[test]
    fn parser_surface_logical_case_005() {
        run_logical_case(5);
    }

    #[test]
    fn parser_surface_logical_case_006() {
        run_logical_case(6);
    }

    #[test]
    fn parser_surface_logical_case_007() {
        run_logical_case(7);
    }

    #[test]
    fn parser_surface_logical_case_008() {
        run_logical_case(8);
    }

    #[test]
    fn parser_surface_logical_case_009() {
        run_logical_case(9);
    }

    #[test]
    fn parser_surface_logical_case_010() {
        run_logical_case(10);
    }

    #[test]
    fn parser_surface_logical_case_011() {
        run_logical_case(11);
    }

    #[test]
    fn parser_surface_logical_case_012() {
        run_logical_case(12);
    }

    #[test]
    fn parser_surface_logical_case_013() {
        run_logical_case(13);
    }

    #[test]
    fn parser_surface_logical_case_014() {
        run_logical_case(14);
    }

    #[test]
    fn parser_surface_logical_case_015() {
        run_logical_case(15);
    }

    #[test]
    fn parser_surface_logical_case_016() {
        run_logical_case(16);
    }

    #[test]
    fn parser_surface_logical_case_017() {
        run_logical_case(17);
    }

    #[test]
    fn parser_surface_logical_case_018() {
        run_logical_case(18);
    }

    #[test]
    fn parser_surface_logical_case_019() {
        run_logical_case(19);
    }

    #[test]
    fn parser_surface_logical_case_020() {
        run_logical_case(20);
    }

    #[test]
    fn parser_surface_logical_case_021() {
        run_logical_case(21);
    }

    #[test]
    fn parser_surface_logical_case_022() {
        run_logical_case(22);
    }

    #[test]
    fn parser_surface_logical_case_023() {
        run_logical_case(23);
    }

    #[test]
    fn parser_surface_logical_case_024() {
        run_logical_case(24);
    }

    #[test]
    fn parser_surface_logical_case_025() {
        run_logical_case(25);
    }

    #[test]
    fn parser_surface_logical_case_026() {
        run_logical_case(26);
    }

    #[test]
    fn parser_surface_logical_case_027() {
        run_logical_case(27);
    }

    #[test]
    fn parser_surface_logical_case_028() {
        run_logical_case(28);
    }

    #[test]
    fn parser_surface_logical_case_029() {
        run_logical_case(29);
    }

    #[test]
    fn parser_surface_logical_case_030() {
        run_logical_case(30);
    }

    #[test]
    fn parser_surface_logical_case_031() {
        run_logical_case(31);
    }

    #[test]
    fn parser_surface_logical_case_032() {
        run_logical_case(32);
    }

    #[test]
    fn parser_surface_logical_case_033() {
        run_logical_case(33);
    }

    #[test]
    fn parser_surface_logical_case_034() {
        run_logical_case(34);
    }

    #[test]
    fn parser_surface_logical_case_035() {
        run_logical_case(35);
    }

    #[test]
    fn parser_surface_logical_case_036() {
        run_logical_case(36);
    }

    #[test]
    fn parser_surface_logical_case_037() {
        run_logical_case(37);
    }

    #[test]
    fn parser_surface_logical_case_038() {
        run_logical_case(38);
    }

    #[test]
    fn parser_surface_logical_case_039() {
        run_logical_case(39);
    }

    #[test]
    fn parser_surface_logical_case_040() {
        run_logical_case(40);
    }

    #[test]
    fn parser_surface_logical_case_041() {
        run_logical_case(41);
    }

    #[test]
    fn parser_surface_logical_case_042() {
        run_logical_case(42);
    }

    #[test]
    fn parser_surface_logical_case_043() {
        run_logical_case(43);
    }

    #[test]
    fn parser_surface_logical_case_044() {
        run_logical_case(44);
    }

    #[test]
    fn parser_surface_logical_case_045() {
        run_logical_case(45);
    }

    #[test]
    fn parser_surface_logical_case_046() {
        run_logical_case(46);
    }

    #[test]
    fn parser_surface_logical_case_047() {
        run_logical_case(47);
    }

    #[test]
    fn parser_surface_logical_case_048() {
        run_logical_case(48);
    }

    #[test]
    fn parser_surface_logical_case_049() {
        run_logical_case(49);
    }

    #[test]
    fn parser_surface_logical_case_050() {
        run_logical_case(50);
    }

    #[test]
    fn parser_surface_logical_case_051() {
        run_logical_case(51);
    }

    #[test]
    fn parser_surface_logical_case_052() {
        run_logical_case(52);
    }

    #[test]
    fn parser_surface_logical_case_053() {
        run_logical_case(53);
    }

    #[test]
    fn parser_surface_logical_case_054() {
        run_logical_case(54);
    }

    #[test]
    fn parser_surface_logical_case_055() {
        run_logical_case(55);
    }

    #[test]
    fn parser_surface_logical_case_056() {
        run_logical_case(56);
    }

    #[test]
    fn parser_surface_logical_case_057() {
        run_logical_case(57);
    }

    #[test]
    fn parser_surface_logical_case_058() {
        run_logical_case(58);
    }

    #[test]
    fn parser_surface_logical_case_059() {
        run_logical_case(59);
    }

    #[test]
    fn parser_surface_logical_case_060() {
        run_logical_case(60);
    }

    #[test]
    fn parser_surface_logical_case_061() {
        run_logical_case(61);
    }

    #[test]
    fn parser_surface_logical_case_062() {
        run_logical_case(62);
    }

    #[test]
    fn parser_surface_logical_case_063() {
        run_logical_case(63);
    }

    #[test]
    fn parser_surface_logical_case_064() {
        run_logical_case(64);
    }

    #[test]
    fn parser_surface_logical_case_065() {
        run_logical_case(65);
    }

    #[test]
    fn parser_surface_logical_case_066() {
        run_logical_case(66);
    }

    #[test]
    fn parser_surface_logical_case_067() {
        run_logical_case(67);
    }

    #[test]
    fn parser_surface_logical_case_068() {
        run_logical_case(68);
    }

    #[test]
    fn parser_surface_logical_case_069() {
        run_logical_case(69);
    }

    #[test]
    fn parser_surface_logical_case_070() {
        run_logical_case(70);
    }

    #[test]
    fn parser_surface_logical_case_071() {
        run_logical_case(71);
    }

    #[test]
    fn parser_surface_logical_case_072() {
        run_logical_case(72);
    }

    #[test]
    fn parser_surface_logical_case_073() {
        run_logical_case(73);
    }

    #[test]
    fn parser_surface_logical_case_074() {
        run_logical_case(74);
    }

    #[test]
    fn parser_surface_logical_case_075() {
        run_logical_case(75);
    }

    #[test]
    fn parser_surface_logical_case_076() {
        run_logical_case(76);
    }

    #[test]
    fn parser_surface_logical_case_077() {
        run_logical_case(77);
    }

    #[test]
    fn parser_surface_logical_case_078() {
        run_logical_case(78);
    }

    #[test]
    fn parser_surface_logical_case_079() {
        run_logical_case(79);
    }

    #[test]
    fn parser_surface_logical_case_080() {
        run_logical_case(80);
    }

    #[test]
    fn parser_surface_logical_case_081() {
        run_logical_case(81);
    }

    #[test]
    fn parser_surface_logical_case_082() {
        run_logical_case(82);
    }

    #[test]
    fn parser_surface_logical_case_083() {
        run_logical_case(83);
    }

    #[test]
    fn parser_surface_logical_case_084() {
        run_logical_case(84);
    }

    #[test]
    fn parser_surface_logical_case_085() {
        run_logical_case(85);
    }

    #[test]
    fn parser_surface_logical_case_086() {
        run_logical_case(86);
    }

    #[test]
    fn parser_surface_logical_case_087() {
        run_logical_case(87);
    }

    #[test]
    fn parser_surface_logical_case_088() {
        run_logical_case(88);
    }

    #[test]
    fn parser_surface_logical_case_089() {
        run_logical_case(89);
    }

    #[test]
    fn parser_surface_logical_case_090() {
        run_logical_case(90);
    }

    #[test]
    fn parser_surface_logical_case_091() {
        run_logical_case(91);
    }

    #[test]
    fn parser_surface_logical_case_092() {
        run_logical_case(92);
    }

    #[test]
    fn parser_surface_logical_case_093() {
        run_logical_case(93);
    }

    #[test]
    fn parser_surface_logical_case_094() {
        run_logical_case(94);
    }

    #[test]
    fn parser_surface_logical_case_095() {
        run_logical_case(95);
    }

    #[test]
    fn parser_surface_logical_case_096() {
        run_logical_case(96);
    }

    #[test]
    fn parser_surface_logical_case_097() {
        run_logical_case(97);
    }

    #[test]
    fn parser_surface_logical_case_098() {
        run_logical_case(98);
    }

    #[test]
    fn parser_surface_logical_case_099() {
        run_logical_case(99);
    }

    #[test]
    fn parser_surface_logical_case_100() {
        run_logical_case(100);
    }

    #[test]
    fn parser_surface_logical_case_101() {
        run_logical_case(101);
    }

    #[test]
    fn parser_surface_logical_case_102() {
        run_logical_case(102);
    }

    #[test]
    fn parser_surface_logical_case_103() {
        run_logical_case(103);
    }

    #[test]
    fn parser_surface_logical_case_104() {
        run_logical_case(104);
    }

    #[test]
    fn parser_surface_logical_case_105() {
        run_logical_case(105);
    }

    #[test]
    fn parser_surface_logical_case_106() {
        run_logical_case(106);
    }

    #[test]
    fn parser_surface_logical_case_107() {
        run_logical_case(107);
    }

    #[test]
    fn parser_surface_logical_case_108() {
        run_logical_case(108);
    }

    #[test]
    fn parser_surface_logical_case_109() {
        run_logical_case(109);
    }

    #[test]
    fn parser_surface_logical_case_110() {
        run_logical_case(110);
    }

    #[test]
    fn parser_surface_logical_case_111() {
        run_logical_case(111);
    }

    #[test]
    fn parser_surface_logical_case_112() {
        run_logical_case(112);
    }

    #[test]
    fn parser_surface_logical_case_113() {
        run_logical_case(113);
    }

    #[test]
    fn parser_surface_logical_case_114() {
        run_logical_case(114);
    }

    #[test]
    fn parser_surface_logical_case_115() {
        run_logical_case(115);
    }

    #[test]
    fn parser_surface_logical_case_116() {
        run_logical_case(116);
    }

    #[test]
    fn parser_surface_logical_case_117() {
        run_logical_case(117);
    }

    #[test]
    fn parser_surface_logical_case_118() {
        run_logical_case(118);
    }

    #[test]
    fn parser_surface_logical_case_119() {
        run_logical_case(119);
    }

    #[test]
    fn parser_surface_logical_case_120() {
        run_logical_case(120);
    }

    #[test]
    fn parser_surface_logical_case_121() {
        run_logical_case(121);
    }

    #[test]
    fn parser_surface_logical_case_122() {
        run_logical_case(122);
    }

    #[test]
    fn parser_surface_logical_case_123() {
        run_logical_case(123);
    }

    #[test]
    fn parser_surface_logical_case_124() {
        run_logical_case(124);
    }

    #[test]
    fn parser_surface_logical_case_125() {
        run_logical_case(125);
    }

    #[test]
    fn parser_surface_logical_case_126() {
        run_logical_case(126);
    }

    #[test]
    fn parser_surface_logical_case_127() {
        run_logical_case(127);
    }

    #[test]
    fn parser_surface_logical_case_128() {
        run_logical_case(128);
    }

    #[test]
    fn parser_surface_logical_case_129() {
        run_logical_case(129);
    }

    #[test]
    fn parser_surface_logical_case_130() {
        run_logical_case(130);
    }

    #[test]
    fn parser_surface_logical_case_131() {
        run_logical_case(131);
    }

    #[test]
    fn parser_surface_logical_case_132() {
        run_logical_case(132);
    }

    #[test]
    fn parser_surface_logical_case_133() {
        run_logical_case(133);
    }

    #[test]
    fn parser_surface_logical_case_134() {
        run_logical_case(134);
    }

    #[test]
    fn parser_surface_logical_case_135() {
        run_logical_case(135);
    }

    #[test]
    fn parser_surface_logical_case_136() {
        run_logical_case(136);
    }

    #[test]
    fn parser_surface_logical_case_137() {
        run_logical_case(137);
    }

    #[test]
    fn parser_surface_logical_case_138() {
        run_logical_case(138);
    }

    #[test]
    fn parser_surface_logical_case_139() {
        run_logical_case(139);
    }

    #[test]
    fn parser_surface_logical_case_140() {
        run_logical_case(140);
    }

    #[test]
    fn parser_surface_logical_case_141() {
        run_logical_case(141);
    }

    #[test]
    fn parser_surface_logical_case_142() {
        run_logical_case(142);
    }

    #[test]
    fn parser_surface_logical_case_143() {
        run_logical_case(143);
    }

    #[test]
    fn parser_surface_logical_case_144() {
        run_logical_case(144);
    }

    #[test]
    fn parser_surface_logical_case_145() {
        run_logical_case(145);
    }

    #[test]
    fn parser_surface_logical_case_146() {
        run_logical_case(146);
    }

    #[test]
    fn parser_surface_logical_case_147() {
        run_logical_case(147);
    }

    #[test]
    fn parser_surface_logical_case_148() {
        run_logical_case(148);
    }

    #[test]
    fn parser_surface_logical_case_149() {
        run_logical_case(149);
    }

    #[test]
    fn parser_surface_logical_case_150() {
        run_logical_case(150);
    }

    #[test]
    fn parser_surface_logical_case_151() {
        run_logical_case(151);
    }

    #[test]
    fn parser_surface_logical_case_152() {
        run_logical_case(152);
    }

    #[test]
    fn parser_surface_logical_case_153() {
        run_logical_case(153);
    }

    #[test]
    fn parser_surface_logical_case_154() {
        run_logical_case(154);
    }

    #[test]
    fn parser_surface_logical_case_155() {
        run_logical_case(155);
    }

    #[test]
    fn parser_surface_logical_case_156() {
        run_logical_case(156);
    }

    #[test]
    fn parser_surface_logical_case_157() {
        run_logical_case(157);
    }

    #[test]
    fn parser_surface_logical_case_158() {
        run_logical_case(158);
    }

    #[test]
    fn parser_surface_logical_case_159() {
        run_logical_case(159);
    }

    #[test]
    fn parser_surface_logical_case_160() {
        run_logical_case(160);
    }

    #[test]
    fn parser_surface_logical_case_161() {
        run_logical_case(161);
    }

    #[test]
    fn parser_surface_logical_case_162() {
        run_logical_case(162);
    }

    #[test]
    fn parser_surface_logical_case_163() {
        run_logical_case(163);
    }

    #[test]
    fn parser_surface_logical_case_164() {
        run_logical_case(164);
    }

    #[test]
    fn parser_surface_logical_case_165() {
        run_logical_case(165);
    }

    #[test]
    fn parser_surface_logical_case_166() {
        run_logical_case(166);
    }

    #[test]
    fn parser_surface_logical_case_167() {
        run_logical_case(167);
    }

    #[test]
    fn parser_surface_logical_case_168() {
        run_logical_case(168);
    }

    #[test]
    fn parser_surface_logical_case_169() {
        run_logical_case(169);
    }
}

#[cfg(test)]
#[path = "../tests/parser_negative_surface.rs"]
mod parser_negative_surface;

#[cfg(test)]
mod tests {
    use super::{
        LogicalParser, LogicalToken, LogicalTokenKind, StrictParser, Token, TokenKind,
        parse_event_expr_ast, parse_logical_expr_ast, parse_logical_expr_with_offset,
    };
    use crate::expr::{
        BasicEventAst, DiagnosticLayer,
        ast::{IntegralBase, IntegralLiteral, LogicalExprNode, UnaryOpAst},
        diagnostic::Span,
    };

    macro_rules! parse_ok_case {
        ($($name:ident => $source:expr),+ $(,)?) => {
            $(
                #[test]
                fn $name() {
                    parse_logical_expr_ast($source).expect($source);
                }
            )+
        };
    }

    parse_ok_case! {
        parse_ok_rel_lt => "a < b",
        parse_ok_rel_le => "a <= b",
        parse_ok_rel_gt => "a > b",
        parse_ok_rel_ge => "a >= b",
        parse_ok_shift_left => "a << 1",
        parse_ok_shift_right => "a >> 1",
        parse_ok_shift_arith_left => "a <<< 1",
        parse_ok_shift_arith_right => "a >>> 1",
        parse_ok_multiply => "a * b",
        parse_ok_divide => "a / b",
        parse_ok_modulo => "a % b",
        parse_ok_power => "a ** b",
        parse_ok_unary_plus => "+a",
        parse_ok_logical_not => "!a",
        parse_ok_bit_not => "~a",
        parse_ok_reduce_and => "&a",
        parse_ok_reduce_nand => "~&a",
        parse_ok_reduce_or => "|a",
        parse_ok_reduce_nor => "~|a",
        parse_ok_reduce_xor => "^a",
        parse_ok_reduce_xnor_caret_tilde => "^~a",
        parse_ok_bitwise_and => "a & b",
        parse_ok_bitwise_or => "a | b",
        parse_ok_bitwise_xor => "a ^ b",
        parse_ok_bitwise_xnor => "a ~^ b",
        parse_ok_logical_and => "a && b",
        parse_ok_logical_or => "a || b",
        parse_ok_group => "(a)",
        parse_ok_conditional => "a ? b : c",
        parse_ok_concat => "{a, b}",
        parse_ok_replication => "{2{a}}",
        parse_ok_bit_select => "a[0]",
        parse_ok_part_select => "a[3:0]",
        parse_ok_indexed_up_select => "a[0 +: 4]",
        parse_ok_indexed_down_select => "a[7 -: 4]",
    }

    #[test]
    fn typed_parser_rejects_unmatched_open_parenthesis() {
        let error = parse_event_expr_ast("(").expect_err("source should fail");

        assert_eq!(error.layer, DiagnosticLayer::Parse);
        assert_eq!(error.code, "EXPR-PARSE-EVENT-UNMATCHED-OPEN");
        assert_eq!(error.primary_span.start, 0);
        assert_eq!(error.primary_span.end, 1);
    }

    #[test]
    fn typed_parser_rejects_broken_union_segmentation() {
        let error = parse_event_expr_ast("posedge clk or , clk").expect_err("source should fail");

        assert_eq!(error.layer, DiagnosticLayer::Parse);
        assert_eq!(error.code, "EXPR-PARSE-EVENT-BROKEN-UNION");
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
    fn logical_parser_accepts_integral_boolean_surface_sample() {
        parse_logical_expr_ast(
            "(signed'(a + 3) inside {[1:8], 16'hx0}) ? {2{b[3]}} : (c ==? 4'b1x0z)",
        )
        .expect("integral boolean sample expression should parse");
    }

    #[test]
    fn logical_parser_accepts_operator_and_cast_edge_surface() {
        for source in [
            "&a || ~|b && (^c == ~^d)",
            "logic[8]'(a) + unsigned bit[4]'(b)",
            "a >>> 1 <= b <<< 2",
            "{a, b, c} != {3{d}}",
            "type(state)'(next_state) inside {type(state)::IDLE, type(state)::BUSY}",
        ] {
            parse_logical_expr_ast(source).expect("edge surface expression should parse");
        }
    }

    #[test]
    fn logical_parser_accepts_rich_type_surface_sample() {
        let parsed = parse_logical_expr_ast(
            "ev.triggered() ? type(state)::BUSY : type(msg)'(\"idle\") == \"idle\"",
        )
        .expect("rich type sample expression should parse");

        assert!(matches!(parsed.root, LogicalExprNode::Conditional { .. }));
    }

    #[test]
    fn logical_parser_keeps_triggered_suffix_signal_names_as_operand_refs() {
        let parsed = parse_logical_expr_ast("top.dut.triggered").expect("source should parse");

        assert!(matches!(
            parsed.root,
            LogicalExprNode::OperandRef { ref name, .. } if name == "top.dut.triggered"
        ));

        let parsed = parse_logical_expr_ast("top.dut.triggered[0]").expect("source should parse");

        match parsed.root {
            LogicalExprNode::Selection { base, .. } => assert!(matches!(
                base.as_ref(),
                LogicalExprNode::OperandRef { name, .. } if name == "top.dut.triggered"
            )),
            other => panic!("expected selection rooted at operand reference, got {other:?}"),
        }
    }

    #[test]
    fn logical_parser_accepts_triggered_call_with_optional_space_before_parens() {
        for source in ["ev.triggered()", "ev.triggered ()"] {
            let parsed = parse_logical_expr_ast(source).expect("source should parse");

            match parsed.root {
                LogicalExprNode::Triggered { expr, span } => {
                    assert_eq!(span.start, 0);
                    assert_eq!(span.end, source.len());
                    assert!(matches!(
                        expr.as_ref(),
                        LogicalExprNode::OperandRef { name, .. } if name == "ev"
                    ));
                }
                other => panic!("expected triggered node, got {other:?}"),
            }
        }
    }

    #[test]
    fn logical_parser_rejects_triggered_call_arguments_and_missing_close_paren() {
        for source in ["ev.triggered(1)", "ev.triggered("] {
            let error = parse_logical_expr_ast(source).expect_err("source should fail");

            assert_eq!(error.layer, DiagnosticLayer::Parse);
            assert_eq!(error.code, "EXPR-PARSE-LOGICAL-EXPECTED");
        }
    }

    #[test]
    fn logical_parser_keeps_unary_minus_separate_from_integral_literals() {
        let parsed = parse_logical_expr_ast("-12").expect("source should parse");

        match parsed.root {
            LogicalExprNode::Unary {
                op: UnaryOpAst::Minus,
                expr,
                span,
            } => {
                assert_eq!(span.start, 0);
                assert_eq!(span.end, 3);
                match expr.as_ref() {
                    LogicalExprNode::IntegralLiteral { literal, span } => {
                        assert_eq!(span.start, 1);
                        assert_eq!(span.end, 3);
                        assert_eq!(literal.span.start, 1);
                        assert_eq!(literal.span.end, 3);
                        assert_eq!(literal.width, None);
                        assert!(literal.signed);
                        assert_eq!(literal.base, IntegralBase::Decimal);
                        assert_eq!(literal.digits, "12");
                    }
                    other => panic!("expected integral literal operand, got {other:?}"),
                }
            }
            other => panic!("expected unary minus root, got {other:?}"),
        }

        let parenthesized = parse_logical_expr_ast("-(12)").expect("source should parse");

        match parenthesized.root {
            LogicalExprNode::Unary {
                op: UnaryOpAst::Minus,
                expr,
                span,
            } => {
                assert_eq!(span.start, 0);
                assert_eq!(span.end, 5);
                match expr.as_ref() {
                    LogicalExprNode::Parenthesized { expr, span } => {
                        assert_eq!(span.start, 1);
                        assert_eq!(span.end, 5);
                        match expr.as_ref() {
                            LogicalExprNode::IntegralLiteral { literal, span } => {
                                assert_eq!(span.start, 2);
                                assert_eq!(span.end, 4);
                                assert_eq!(literal.span.start, 2);
                                assert_eq!(literal.span.end, 4);
                                assert_eq!(literal.width, None);
                                assert!(literal.signed);
                                assert_eq!(literal.base, IntegralBase::Decimal);
                                assert_eq!(literal.digits, "12");
                            }
                            other => panic!(
                                "expected integral literal inside parentheses, got {other:?}"
                            ),
                        }
                    }
                    other => panic!("expected parenthesized operand, got {other:?}"),
                }
            }
            other => panic!("expected unary minus root, got {other:?}"),
        }
    }

    #[test]
    fn typed_parser_rejects_empty_iff_invalid_names_and_missing_edge_operands() {
        let error = parse_event_expr_ast("clk iff").expect_err("empty iff should fail");
        assert_eq!(error.code, "EXPR-PARSE-EVENT-EMPTY-IFF");

        let error = parse_event_expr_ast("posedge").expect_err("missing edge name should fail");
        assert_eq!(error.code, "EXPR-PARSE-EVENT-MISSING-NAME");

        let error = parse_event_expr_ast("sig@bad").expect_err("invalid signal name should fail");
        assert_eq!(error.code, "EXPR-PARSE-EVENT-LEX-CHAR");

        let error = parse_event_expr_ast("iff clk").expect_err("iff cannot lead an event term");
        assert_eq!(error.code, "EXPR-PARSE-EVENT-BROKEN-UNION");
    }

    #[test]
    fn logical_parser_rejects_empty_groupings_and_inside_sets() {
        let error = parse_logical_expr_ast("{}").expect_err("empty concatenation should fail");
        assert_eq!(error.code, "EXPR-PARSE-LOGICAL-EXPECTED");

        let error =
            parse_logical_expr_ast("a inside {}").expect_err("empty inside set should fail");
        assert_eq!(error.code, "EXPR-PARSE-LOGICAL-EXPECTED");

        let error = parse_logical_expr_ast("a[1")
            .expect_err("selection without closing bracket should fail");
        assert_eq!(error.code, "EXPR-PARSE-LOGICAL-EXPECTED");

        let error = parse_logical_expr_ast(")").expect_err("unmatched closing paren should fail");
        assert_eq!(error.code, "EXPR-PARSE-LOGICAL-UNMATCHED-CLOSE");
    }

    #[test]
    fn logical_parser_rejects_malformed_cast_and_enum_forms() {
        let error = parse_logical_expr_ast("type(state)::")
            .expect_err("enum label without label should fail");
        assert_eq!(error.code, "EXPR-PARSE-LOGICAL-EXPECTED");

        let error =
            parse_logical_expr_ast("bit[0]'(a)").expect_err("zero-width cast target should fail");
        assert_eq!(error.code, "EXPR-PARSE-LOGICAL-CAST");

        let error = parse_logical_expr_ast("type(1)'(a)")
            .expect_err("recovered type target must name an operand");
        assert_eq!(error.code, "EXPR-PARSE-LOGICAL-CAST");

        let error =
            parse_logical_expr_ast("signed'a").expect_err("casts require parenthesized payloads");
        assert_eq!(error.code, "EXPR-PARSE-LOGICAL-CAST");
    }

    #[test]
    fn parser_rejects_more_event_union_iff_and_member_suffix_edges() {
        for source in ["a or", "a,", "a or or b", "a, ,b"] {
            let error = parse_event_expr_ast(source).expect_err("broken unions should fail");
            assert_eq!(error.code, "EXPR-PARSE-EVENT-BROKEN-UNION", "{source}");
        }

        let error = parse_event_expr_ast("a iff )")
            .expect_err("unmatched close inside iff payload should fail");
        assert_eq!(error.code, "EXPR-PARSE-EVENT-UNMATCHED-CLOSE");

        let error = parse_event_expr_ast("a iff (")
            .expect_err("unmatched open inside iff payload should fail");
        assert_eq!(error.code, "EXPR-PARSE-EVENT-UNMATCHED-OPEN");

        let error =
            parse_event_expr_ast("a iff   ").expect_err("whitespace-only iff payload should fail");
        assert_eq!(error.code, "EXPR-PARSE-EVENT-EMPTY-IFF");

        let error =
            parse_event_expr_ast("posedge )").expect_err("edge keywords followed by ) should fail");
        assert_eq!(error.code, "EXPR-PARSE-EVENT-UNMATCHED-CLOSE");

        let error = parse_logical_expr_ast("sig.foo()")
            .expect_err("unsupported member-like suffixes should fail");
        assert_eq!(error.code, "EXPR-PARSE-LOGICAL-TRAILING");
    }

    #[test]
    fn parser_rejects_more_inside_selection_and_cast_forms() {
        let error =
            parse_logical_expr_ast("a[1?2]").expect_err("malformed selection suffix should fail");
        assert_eq!(error.code, "EXPR-PARSE-LOGICAL-EXPECTED");

        let error =
            parse_logical_expr_ast("a inside {[1 2]}").expect_err("inside ranges require a colon");
        assert_eq!(error.code, "EXPR-PARSE-LOGICAL-EXPECTED");

        let error = parse_logical_expr_ast("a inside {1, 2")
            .expect_err("inside sets must close with a brace");
        assert_eq!(error.code, "EXPR-PARSE-LOGICAL-EXPECTED");

        let error = parse_logical_expr_ast("{2{a}")
            .expect_err("replication missing outer close should fail");
        assert_eq!(error.code, "EXPR-PARSE-LOGICAL-EXPECTED");

        let error = parse_logical_expr_ast("type(state'(a)")
            .expect_err("type(...) cast targets need a closing parenthesis");
        assert_eq!(error.code, "EXPR-PARSE-LOGICAL-CAST");

        let error = parse_logical_expr_ast("type(state)'(a")
            .expect_err("cast payloads need a closing parenthesis");
        assert_eq!(error.code, "EXPR-PARSE-LOGICAL-UNMATCHED-OPEN");

        let error = parse_logical_expr_ast("logic[x]'(a)")
            .expect_err("cast widths must be integral literals");
        assert_eq!(error.code, "EXPR-PARSE-LOGICAL-CAST");

        let error =
            parse_logical_expr_ast("logic[1").expect_err("cast widths need a closing bracket");
        assert_eq!(error.code, "EXPR-PARSE-LOGICAL-CAST");
    }

    #[test]
    fn private_event_parser_manual_states_exercise_more_error_branches() {
        let mut parser = StrictParser {
            source: "",
            tokens: vec![],
            index: 0,
        };
        assert_eq!(
            parser
                .capture_iff_payload(Span::new(0, 0))
                .expect_err("empty iff payload should fail")
                .code,
            "EXPR-PARSE-EVENT-EMPTY-IFF"
        );

        let mut dispatch_parser = StrictParser {
            source: "posedge foo",
            tokens: vec![
                Token {
                    kind: TokenKind::KeywordPosedge,
                    span: Span::new(0, 7),
                    lexeme: "posedge".to_string(),
                },
                Token {
                    kind: TokenKind::Identifier,
                    span: Span::new(8, 11),
                    lexeme: "foo".to_string(),
                },
            ],
            index: 0,
        };
        assert_eq!(
            dispatch_parser
                .parse_edge_event(TokenKind::KeywordOr)
                .expect_err("invalid internal edge dispatch should fail")
                .code,
            "EXPR-PARSE-EVENT-BROKEN-UNION"
        );

        let eof = LogicalToken {
            kind: LogicalTokenKind::Eof,
            span: Span::new(0, 0),
        };
        let mut type_target_parser = LogicalParser {
            source: "type(",
            tokens: vec![
                LogicalToken {
                    kind: LogicalTokenKind::Identifier("type".to_string()),
                    span: Span::new(0, 4),
                },
                LogicalToken {
                    kind: LogicalTokenKind::LeftParen,
                    span: Span::new(4, 5),
                },
                eof,
            ],
            index: 0,
        };
        assert_eq!(
            type_target_parser
                .try_parse_cast_target_candidate()
                .expect_err("unterminated type(...) target should fail")
                .code,
            "EXPR-PARSE-LOGICAL-UNMATCHED-OPEN"
        );
    }

    #[test]
    fn private_parser_helpers_exercise_empty_inputs_and_cast_width_edges() {
        let error = parse_event_expr_ast("   ").expect_err("empty event expression should fail");
        assert_eq!(error.code, "EXPR-PARSE-EVENT-EMPTY");
        assert_eq!(error.primary_span, Span::new(0, 3));

        let error = parse_logical_expr_with_offset("   ", 7).expect_err("empty logical expression");
        assert_eq!(error.code, "EXPR-PARSE-LOGICAL-EMPTY");
        assert_eq!(error.primary_span, Span::new(7, 10));

        let mut event_parser = StrictParser {
            source: "",
            tokens: vec![],
            index: 0,
        };
        assert_eq!(
            event_parser
                .parse_event_expr()
                .expect_err("missing event term should fail")
                .code,
            "EXPR-PARSE-EVENT-BROKEN-UNION"
        );

        let mut basic_event_parser = StrictParser {
            source: "",
            tokens: vec![],
            index: 0,
        };
        assert_eq!(
            basic_event_parser
                .parse_basic_event()
                .expect_err("missing basic event should fail")
                .code,
            "EXPR-PARSE-EVENT-BROKEN-UNION"
        );

        let eof = LogicalToken {
            kind: LogicalTokenKind::Eof,
            span: Span::new(0, 0),
        };
        let parser = LogicalParser {
            source: "logic",
            tokens: vec![eof.clone()],
            index: 0,
        };
        assert!(matches!(parser.peek_kind(0), Some(LogicalTokenKind::Eof)));
        assert!(parser.peek_kind(1).is_none());

        let mut cast_parser = LogicalParser {
            source: "bit[0]",
            tokens: vec![
                LogicalToken {
                    kind: LogicalTokenKind::LeftBracket,
                    span: Span::new(3, 4),
                },
                LogicalToken {
                    kind: LogicalTokenKind::IntegralLiteral(IntegralLiteral {
                        width: None,
                        signed: true,
                        base: IntegralBase::Decimal,
                        digits: "0".to_string(),
                        span: Span::new(4, 5),
                    }),
                    span: Span::new(4, 5),
                },
                eof.clone(),
            ],
            index: 0,
        };
        assert_eq!(
            cast_parser
                .parse_bit_logic_target(false, false, Span::new(0, 3))
                .expect_err("zero-width cast target should fail")
                .code,
            "EXPR-PARSE-LOGICAL-CAST"
        );

        let mut bad_width_parser = LogicalParser {
            source: "bit[name]",
            tokens: vec![
                LogicalToken {
                    kind: LogicalTokenKind::LeftBracket,
                    span: Span::new(3, 4),
                },
                LogicalToken {
                    kind: LogicalTokenKind::Identifier("name".to_string()),
                    span: Span::new(4, 8),
                },
                eof.clone(),
            ],
            index: 0,
        };
        assert_eq!(
            bad_width_parser
                .parse_bit_logic_target(true, false, Span::new(0, 3))
                .expect_err("non-integral cast target width should fail")
                .code,
            "EXPR-PARSE-LOGICAL-CAST"
        );

        let mut missing_bracket_parser = LogicalParser {
            source: "bit[2",
            tokens: vec![
                LogicalToken {
                    kind: LogicalTokenKind::LeftBracket,
                    span: Span::new(3, 4),
                },
                LogicalToken {
                    kind: LogicalTokenKind::IntegralLiteral(IntegralLiteral {
                        width: None,
                        signed: true,
                        base: IntegralBase::Decimal,
                        digits: "2".to_string(),
                        span: Span::new(4, 5),
                    }),
                    span: Span::new(4, 5),
                },
                eof,
            ],
            index: 0,
        };
        assert_eq!(
            missing_bracket_parser
                .parse_bit_logic_target(false, true, Span::new(0, 3))
                .expect_err("missing cast target bracket should fail")
                .code,
            "EXPR-PARSE-LOGICAL-CAST"
        );
    }

    #[test]
    fn parser_edge_cases_exercise_manual_suffix_and_rhs_failures() {
        let error = parse_event_expr_ast("a (").expect_err("event-level open after a term fails");
        assert_eq!(error.code, "EXPR-PARSE-EVENT-UNMATCHED-OPEN");

        for source in [
            "a ? b",
            "1 <",
            "a[1:]",
            "a[1+:]",
            "a[1-:]",
            "a inside {[1:]}",
            "(a",
        ] {
            let error = parse_logical_expr_ast(source).expect_err("source should fail");
            assert!(
                matches!(
                    error.code,
                    "EXPR-PARSE-LOGICAL-EXPECTED" | "EXPR-PARSE-LOGICAL-UNMATCHED-OPEN"
                ),
                "{source}: {}",
                error.code
            );
        }

        let eof = LogicalToken {
            kind: LogicalTokenKind::Eof,
            span: Span::new(11, 11),
        };
        let mut suffix_without_paren = LogicalParser {
            source: "sig.triggered",
            tokens: vec![
                LogicalToken {
                    kind: LogicalTokenKind::Identifier("sig".to_string()),
                    span: Span::new(0, 3),
                },
                LogicalToken {
                    kind: LogicalTokenKind::Dot,
                    span: Span::new(3, 4),
                },
                LogicalToken {
                    kind: LogicalTokenKind::Identifier("triggered".to_string()),
                    span: Span::new(4, 13),
                },
                eof.clone(),
            ],
            index: 0,
        };
        assert_eq!(
            suffix_without_paren
                .parse()
                .expect_err("triggered suffix needs parens")
                .code,
            "EXPR-PARSE-LOGICAL-EXPECTED"
        );

        let mut inside_without_brace = LogicalParser {
            source: "a inside b",
            tokens: vec![
                LogicalToken {
                    kind: LogicalTokenKind::Identifier("a".to_string()),
                    span: Span::new(0, 1),
                },
                LogicalToken {
                    kind: LogicalTokenKind::KeywordInside,
                    span: Span::new(2, 8),
                },
                LogicalToken {
                    kind: LogicalTokenKind::Identifier("b".to_string()),
                    span: Span::new(9, 10),
                },
                eof,
            ],
            index: 0,
        };
        assert_eq!(
            inside_without_brace
                .parse()
                .expect_err("inside requires braced set")
                .code,
            "EXPR-PARSE-LOGICAL-EXPECTED"
        );
    }
}
