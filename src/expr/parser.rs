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

pub fn parse_logical_expr_ast(source: &str) -> Result<LogicalExprAst, ExprDiagnostic> {
    parse_logical_expr_with_offset(source, 0)
}

pub(crate) fn parse_logical_expr_with_offset(
    source: &str,
    source_offset: usize,
) -> Result<LogicalExprAst, ExprDiagnostic> {
    if source.trim().is_empty() {
        return Err(logical_parse_diag(
            "C3-PARSE-LOGICAL-EMPTY",
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
                "C3-PARSE-LOGICAL-TRAILING",
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
                "C3-PARSE-LOGICAL-EXPECTED",
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
                            let span = Span::new(dot_span.start, token.span.end);
                            return Err(logical_parse_diag(
                                "C3-PARSE-LOGICAL-DEFERRED",
                                ".triggered is deferred to C4",
                                span,
                                &["event-member primaries are outside C3"],
                            ));
                        }
                        _ => {
                            return Err(logical_parse_diag(
                                "C3-PARSE-LOGICAL-EXPECTED",
                                "unsupported member-like suffix",
                                dot_span,
                                &["only .triggered is reserved and it is deferred to C4"],
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
                "C3-PARSE-LOGICAL-EXPECTED",
                "malformed selection suffix",
                open,
                &["expected ] or : or +: or -: in selection"],
            )),
        }
    }

    fn parse_inside_set(&mut self) -> Result<(Vec<InsideItemAst>, Span), ExprDiagnostic> {
        if !matches!(self.current().kind, LogicalTokenKind::LeftBrace) {
            return Err(logical_parse_diag(
                "C3-PARSE-LOGICAL-EXPECTED",
                "inside requires a braced set",
                self.current().span,
                &["use inside { item1, item2 }"],
            ));
        }
        let open = self.current().span;
        self.index += 1;

        if matches!(self.current().kind, LogicalTokenKind::RightBrace) {
            return Err(logical_parse_diag(
                "C3-PARSE-LOGICAL-EXPECTED",
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
                        "C3-PARSE-LOGICAL-EXPECTED",
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
                "C3-PARSE-LOGICAL-EXPECTED",
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
        if let Some(cast) = self.try_parse_cast_expr()? {
            return Ok(cast);
        }

        let token = self.current().clone();
        match token.kind {
            LogicalTokenKind::Identifier(name) => {
                if name.ends_with(".triggered") {
                    return Err(logical_parse_diag(
                        "C3-PARSE-LOGICAL-DEFERRED",
                        ".triggered is deferred to C4",
                        token.span,
                        &["event-member primaries are outside C3"],
                    ));
                }
                if name == "type" && matches!(self.peek_kind(1), Some(LogicalTokenKind::LeftParen))
                {
                    return Err(logical_parse_diag(
                        "C3-PARSE-LOGICAL-DEFERRED",
                        "type(...) enum-label forms are deferred to C4",
                        token.span,
                        &["type(enum_operand_reference)::LABEL is outside C3"],
                    ));
                }
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
                let open = token.span;
                self.index += 1;
                let expr = self.parse_conditional_expr()?;
                if !matches!(self.current().kind, LogicalTokenKind::RightParen) {
                    return Err(logical_parse_diag(
                        "C3-PARSE-LOGICAL-UNMATCHED-OPEN",
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
                "C3-PARSE-LOGICAL-UNMATCHED-CLOSE",
                "unmatched closing parenthesis in logical expression",
                token.span,
                &["remove ')' or add a matching '('"],
            )),
            LogicalTokenKind::Eof => Err(logical_parse_diag(
                "C3-PARSE-LOGICAL-EXPECTED",
                "incomplete logical expression",
                token.span,
                &["expected an operand or literal"],
            )),
            _ => Err(logical_parse_diag(
                "C3-PARSE-LOGICAL-EXPECTED",
                "expected logical expression operand",
                token.span,
                &["expected operand reference, literal, cast, or parenthesized expression"],
            )),
        }
    }

    fn parse_braced_primary(&mut self) -> Result<LogicalExprNode, ExprDiagnostic> {
        let open = self.current().span;
        self.index += 1;
        if matches!(self.current().kind, LogicalTokenKind::RightBrace) {
            return Err(logical_parse_diag(
                "C3-PARSE-LOGICAL-EXPECTED",
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
                    "C3-PARSE-LOGICAL-EXPECTED",
                    "replication is missing inner closing '}'",
                    self.current().span,
                    &["replication form is {N{expr}}"],
                ));
            }
            self.index += 1;
            if !matches!(self.current().kind, LogicalTokenKind::RightBrace) {
                return Err(logical_parse_diag(
                    "C3-PARSE-LOGICAL-EXPECTED",
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
                "C3-PARSE-LOGICAL-EXPECTED",
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
                "C3-PARSE-LOGICAL-CAST",
                "invalid cast target",
                candidate.span,
                &["cast target must be an integral C3 target"],
            ));
        };

        if !matches!(self.current().kind, LogicalTokenKind::LeftParen) {
            return Err(logical_parse_diag(
                "C3-PARSE-LOGICAL-CAST",
                "malformed cast expression",
                self.current().span,
                &["cast form is type'(expr)"],
            ));
        }
        self.index += 1;
        let inner = self.parse_conditional_expr()?;
        if !matches!(self.current().kind, LogicalTokenKind::RightParen) {
            return Err(logical_parse_diag(
                "C3-PARSE-LOGICAL-UNMATCHED-OPEN",
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
                    target: None,
                    deferred_error: Some(logical_parse_diag(
                        "C3-PARSE-LOGICAL-DEFERRED",
                        "real casts are deferred to C4",
                        token.span,
                        &["C3 supports integral cast targets only"],
                    )),
                    span: token.span,
                }))
            }
            "string" => {
                self.index += 1;
                Ok(Some(CastTargetCandidate {
                    target: None,
                    deferred_error: Some(logical_parse_diag(
                        "C3-PARSE-LOGICAL-DEFERRED",
                        "string casts are deferred to C4",
                        token.span,
                        &["C3 supports integral cast targets only"],
                    )),
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
                let mut depth = 1usize;
                let mut end = open;
                while depth > 0 {
                    let token = self.current().clone();
                    match token.kind {
                        LogicalTokenKind::LeftParen => {
                            depth += 1;
                            end = token.span;
                            self.index += 1;
                        }
                        LogicalTokenKind::RightParen => {
                            depth = depth.saturating_sub(1);
                            end = token.span;
                            self.index += 1;
                            if depth == 0 {
                                break;
                            }
                        }
                        LogicalTokenKind::Eof => {
                            return Err(logical_parse_diag(
                                "C3-PARSE-LOGICAL-UNMATCHED-OPEN",
                                "unmatched opening parenthesis in type(...) cast target",
                                open,
                                &["type(...) forms are deferred to C4"],
                            ));
                        }
                        _ => {
                            end = token.span;
                            self.index += 1;
                        }
                    }
                }

                Ok(Some(CastTargetCandidate {
                    target: None,
                    deferred_error: Some(logical_parse_diag(
                        "C3-PARSE-LOGICAL-DEFERRED",
                        "type(operand_reference) casts are deferred to C4",
                        Span::new(token.span.start, end.end),
                        &["type(...) cast targets are outside C3"],
                    )),
                    span: Span::new(token.span.start, end.end),
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
                        "C3-PARSE-LOGICAL-CAST",
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
                    "C3-PARSE-LOGICAL-CAST",
                    "cast target width must be an unsized decimal integer",
                    token.span,
                    &["bit/logic cast widths use bit[N] or logic[N]"],
                ));
            }
            width = literal.digits.parse::<u32>().map_err(|_| {
                logical_parse_diag(
                    "C3-PARSE-LOGICAL-CAST",
                    "cast target width is out of range",
                    token.span,
                    &["bit/logic cast widths must fit in u32"],
                )
            })?;
            if width == 0 {
                return Err(logical_parse_diag(
                    "C3-PARSE-LOGICAL-CAST",
                    "cast target width must be greater than zero",
                    token.span,
                    &["bit/logic cast widths must be positive"],
                ));
            }
            self.index += 1;
            if !matches!(self.current().kind, LogicalTokenKind::RightBracket) {
                return Err(logical_parse_diag(
                    "C3-PARSE-LOGICAL-CAST",
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
                "C3-PARSE-LOGICAL-EXPECTED",
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
mod tests {
    use super::{parse_event_expr_ast, parse_logical_expr_ast};
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
    fn logical_parser_accepts_c3_integral_surface_sample() {
        parse_logical_expr_ast(
            "(signed'(a + 3) inside {[1:8], 16'hx0}) ? {2{b[3]}} : (c ==? 4'b1x0z)",
        )
        .expect("c3 sample expression should parse");
    }
}
