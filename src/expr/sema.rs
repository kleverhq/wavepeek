use crate::expr::ast::{BasicEventAst, DeferredLogicalExpr, EventExprAst};
use crate::expr::diagnostic::{DiagnosticLayer, ExprDiagnostic};
use crate::expr::host::{ExpressionHost, SignalHandle};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoundEventExpr {
    pub terms: Vec<BoundEventTerm>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoundEventTerm {
    pub event: BoundEventKind,
    pub iff: Option<DeferredLogicalExpr>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BoundEventKind {
    AnyTracked,
    Named(SignalHandle),
    Posedge(SignalHandle),
    Negedge(SignalHandle),
    Edge(SignalHandle),
}

pub(crate) fn bind_event_expr(
    ast: &EventExprAst,
    host: &dyn ExpressionHost,
) -> Result<BoundEventExpr, ExprDiagnostic> {
    let mut terms = Vec::with_capacity(ast.terms.len());
    for term in &ast.terms {
        let event = match &term.event {
            BasicEventAst::AnyTracked { .. } => BoundEventKind::AnyTracked,
            BasicEventAst::Named { name, .. } => BoundEventKind::Named(host.resolve_signal(name)?),
            BasicEventAst::Posedge { name, .. } => {
                BoundEventKind::Posedge(host.resolve_signal(name)?)
            }
            BasicEventAst::Negedge { name, .. } => {
                BoundEventKind::Negedge(host.resolve_signal(name)?)
            }
            BasicEventAst::Edge { name, .. } => BoundEventKind::Edge(host.resolve_signal(name)?),
        };
        terms.push(BoundEventTerm {
            event,
            iff: term.iff.clone(),
        });
    }

    Ok(BoundEventExpr { terms })
}

pub(crate) fn bind_logical_expr(
    expr: &DeferredLogicalExpr,
    _host: &dyn ExpressionHost,
) -> Result<(), ExprDiagnostic> {
    Err(ExprDiagnostic {
        layer: DiagnosticLayer::Semantic,
        code: "C1-SEMANTIC-LOGICAL-NOT-IMPLEMENTED",
        message: "logical expression binding is not implemented yet".to_string(),
        primary_span: expr.span,
        notes: vec!["C1 keeps logical semantics deferred".to_string()],
    })
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;
    use crate::expr::host::{ExprType, SampledValue};
    use crate::expr::{EventTermAst, Span, parse_event_expr_ast};

    struct HostStub {
        handles: HashMap<String, SignalHandle>,
    }

    impl HostStub {
        fn new() -> Self {
            let mut handles = HashMap::new();
            handles.insert("clk".to_string(), SignalHandle(1));
            handles.insert("rstn".to_string(), SignalHandle(2));
            Self { handles }
        }
    }

    impl ExpressionHost for HostStub {
        fn resolve_signal(&self, name: &str) -> Result<SignalHandle, ExprDiagnostic> {
            self.handles
                .get(name)
                .copied()
                .ok_or_else(|| ExprDiagnostic {
                    layer: DiagnosticLayer::Semantic,
                    code: "C1-SEMANTIC-UNKNOWN-SIGNAL",
                    message: format!("unknown signal '{name}'"),
                    primary_span: Span::new(0, 0),
                    notes: vec![],
                })
        }

        fn signal_type(&self, _handle: SignalHandle) -> Result<ExprType, ExprDiagnostic> {
            Ok(ExprType {
                width: 1,
                is_four_state: true,
                is_signed: false,
            })
        }

        fn sample_value(
            &self,
            _handle: SignalHandle,
            _timestamp: u64,
        ) -> Result<SampledValue, ExprDiagnostic> {
            Ok(SampledValue {
                bits: "0".to_string(),
            })
        }
    }

    fn first_term_with_iff(ast: &EventExprAst) -> &EventTermAst {
        ast.terms
            .first()
            .expect("ast should have at least one term")
    }

    #[test]
    fn bind_event_expr_resolves_named_terms() {
        let ast = parse_event_expr_ast("posedge clk or rstn").expect("source should parse");
        let host = HostStub::new();

        let bound = bind_event_expr(&ast, &host).expect("binding should succeed");

        assert_eq!(bound.terms.len(), 2);
        assert_eq!(
            bound.terms[0].event,
            BoundEventKind::Posedge(SignalHandle(1))
        );
        assert_eq!(bound.terms[1].event, BoundEventKind::Named(SignalHandle(2)));
    }

    #[test]
    fn bind_logical_expr_is_still_deferred_with_snapshot() {
        let ast = parse_event_expr_ast("posedge clk iff rstn").expect("source should parse");
        let host = HostStub::new();
        let logical = first_term_with_iff(&ast)
            .iff
            .as_ref()
            .expect("iff payload should exist");

        let diagnostic = bind_logical_expr(logical, &host).expect_err("binding should fail");
        assert_eq!(diagnostic.layer, DiagnosticLayer::Semantic);
        assert_eq!(diagnostic.code, "C1-SEMANTIC-LOGICAL-NOT-IMPLEMENTED");
        insta::assert_snapshot!(
            "bind_logical_expr_not_implemented",
            diagnostic.render(logical.source.as_str())
        );
    }
}
