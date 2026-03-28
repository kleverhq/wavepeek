use std::cell::RefCell;
use std::collections::HashSet;
use std::path::Path;
use std::rc::Rc;

use crate::error::WavepeekError;
use crate::expr::sema::{
    BoundEventKind, BoundInsideItem, BoundLogicalKind, BoundLogicalNode, BoundSelection,
};
use crate::expr::{
    BoundEventExpr, BoundLogicalExpr, EventEvalFrame, ExprDiagnostic, ExprValuePayload,
    ExpressionHost, SampledValue, SignalHandle, Span, bind_event_expr_ast, bind_logical_expr_ast,
    eval_logical_expr_at, event_matches_at, parse_event_expr_ast, parse_logical_expr_ast,
};
use crate::waveform::{ExprResolvedSignal, Waveform, expr_host::WaveformExprHost};

pub(crate) type SharedWaveform = Rc<RefCell<Waveform>>;

pub(crate) fn open_shared_waveform(path: &Path) -> Result<SharedWaveform, WavepeekError> {
    Ok(Rc::new(RefCell::new(Waveform::open(path)?)))
}

pub(crate) struct ScopedExprHost<'a> {
    inner: &'a dyn ExpressionHost,
    scope: Option<&'a str>,
}

impl<'a> ScopedExprHost<'a> {
    pub(crate) fn new(inner: &'a dyn ExpressionHost, scope: Option<&'a str>) -> Self {
        Self { inner, scope }
    }
}

impl ExpressionHost for ScopedExprHost<'_> {
    fn resolve_signal(&self, name: &str) -> Result<SignalHandle, ExprDiagnostic> {
        let resolved_name = match self.scope {
            Some(_) if name.contains('.') => return Err(unknown_signal_diagnostic(name)),
            Some(scope) => format!("{scope}.{name}"),
            None => name.to_string(),
        };
        self.inner.resolve_signal(resolved_name.as_str())
    }

    fn signal_type(&self, handle: SignalHandle) -> Result<crate::expr::ExprType, ExprDiagnostic> {
        self.inner.signal_type(handle)
    }

    fn sample_value(
        &self,
        handle: SignalHandle,
        timestamp: u64,
    ) -> Result<SampledValue, ExprDiagnostic> {
        self.inner.sample_value(handle, timestamp)
    }

    fn event_occurred(&self, handle: SignalHandle, timestamp: u64) -> Result<bool, ExprDiagnostic> {
        self.inner.event_occurred(handle, timestamp)
    }
}

pub(crate) fn bind_waveform_event_expr(
    waveform: SharedWaveform,
    scope: Option<&str>,
    source: &str,
) -> Result<(WaveformExprHost, BoundEventExpr), WavepeekError> {
    let host = WaveformExprHost::from_shared(waveform);
    let scoped = ScopedExprHost::new(&host, scope);
    let ast =
        parse_event_expr_ast(source).map_err(|diagnostic| expr_diagnostic(source, diagnostic))?;
    let bound = bind_event_expr_ast(&ast, &scoped)
        .map_err(|diagnostic| expr_diagnostic(source, diagnostic))?;
    Ok((host, bound))
}

pub(crate) fn bind_waveform_logical_expr(
    host: &WaveformExprHost,
    scope: Option<&str>,
    source: &str,
) -> Result<BoundLogicalExpr, WavepeekError> {
    let scoped = ScopedExprHost::new(host, scope);
    let ast =
        parse_logical_expr_ast(source).map_err(|diagnostic| expr_diagnostic(source, diagnostic))?;
    bind_logical_expr_ast(&ast, &scoped).map_err(|diagnostic| expr_diagnostic(source, diagnostic))
}

pub(crate) fn eval_bound_logical_truth(
    source: &str,
    expr: &BoundLogicalExpr,
    host: &dyn ExpressionHost,
    timestamp: u64,
) -> Result<bool, WavepeekError> {
    let value = eval_logical_expr_at(expr, host, timestamp)
        .map_err(|diagnostic| expr_diagnostic(source, diagnostic))?;
    match value.payload {
        ExprValuePayload::Integral { bits, .. } => Ok(bits.chars().any(|bit| bit == '1')),
        ExprValuePayload::Real { value } => Ok(value != 0.0),
        ExprValuePayload::String { .. } => Ok(false),
    }
}

pub(crate) fn event_expr_matches(
    source: &str,
    expr: &BoundEventExpr,
    host: &dyn ExpressionHost,
    frame: &EventEvalFrame<'_>,
) -> Result<bool, WavepeekError> {
    event_matches_at(expr, host, frame).map_err(|diagnostic| expr_diagnostic(source, diagnostic))
}

pub(crate) fn expr_diagnostic(source: &str, diagnostic: ExprDiagnostic) -> WavepeekError {
    WavepeekError::Expr(diagnostic.render(source))
}

pub(crate) fn referenced_signal_handles(expr: &BoundLogicalExpr) -> Vec<SignalHandle> {
    let mut handles = Vec::new();
    let mut seen = HashSet::new();
    collect_logical_handles(&expr.root, &mut seen, &mut handles);
    handles
}

pub(crate) fn event_candidate_handles(expr: &BoundEventExpr) -> Vec<SignalHandle> {
    let mut handles = Vec::new();
    let mut seen = HashSet::new();
    for term in &expr.terms {
        let handle = match term.event {
            BoundEventKind::AnyTracked => None,
            BoundEventKind::Named(handle)
            | BoundEventKind::Posedge(handle)
            | BoundEventKind::Negedge(handle)
            | BoundEventKind::Edge(handle) => Some(handle),
        };
        if let Some(handle) = handle
            && seen.insert(handle)
        {
            handles.push(handle);
        }
    }
    handles
}

pub(crate) fn event_expr_contains_wildcard(expr: &BoundEventExpr) -> bool {
    expr.terms
        .iter()
        .any(|term| matches!(term.event, BoundEventKind::AnyTracked))
}

pub(crate) fn event_expr_is_any_tracked_only(expr: &BoundEventExpr) -> bool {
    !expr.terms.is_empty()
        && expr
            .terms
            .iter()
            .all(|term| matches!(term.event, BoundEventKind::AnyTracked))
}

pub(crate) fn event_expr_is_edge_only(expr: &BoundEventExpr) -> bool {
    !expr.terms.is_empty()
        && expr.terms.iter().all(|term| {
            matches!(
                term.event,
                BoundEventKind::Posedge(_) | BoundEventKind::Negedge(_) | BoundEventKind::Edge(_)
            )
        })
}

pub(crate) fn candidate_sources_for_handles(
    host: &WaveformExprHost,
    handles: &[SignalHandle],
) -> Result<Vec<ExprResolvedSignal>, WavepeekError> {
    let mut sources = Vec::with_capacity(handles.len());
    let mut seen = HashSet::new();
    for handle in handles {
        let resolved = host.resolved_signal_for_handle(*handle)?;
        if seen.insert(resolved.signal_ref) {
            sources.push(resolved);
        }
    }
    Ok(sources)
}

fn collect_logical_handles(
    node: &BoundLogicalNode,
    seen: &mut HashSet<SignalHandle>,
    handles: &mut Vec<SignalHandle>,
) {
    match &node.kind {
        BoundLogicalKind::SignalRef { handle } | BoundLogicalKind::Triggered { handle } => {
            if seen.insert(*handle) {
                handles.push(*handle);
            }
        }
        BoundLogicalKind::Parenthesized { expr }
        | BoundLogicalKind::Cast { expr, .. }
        | BoundLogicalKind::Unary { expr, .. }
        | BoundLogicalKind::Replication { expr, .. } => {
            collect_logical_handles(expr, seen, handles)
        }
        BoundLogicalKind::Selection { base, selection } => {
            collect_logical_handles(base, seen, handles);
            collect_selection_handles(selection, seen, handles);
        }
        BoundLogicalKind::Binary { left, right, .. } => {
            collect_logical_handles(left, seen, handles);
            collect_logical_handles(right, seen, handles);
        }
        BoundLogicalKind::Conditional {
            condition,
            when_true,
            when_false,
        } => {
            collect_logical_handles(condition, seen, handles);
            collect_logical_handles(when_true, seen, handles);
            collect_logical_handles(when_false, seen, handles);
        }
        BoundLogicalKind::Inside { expr, set } => {
            collect_logical_handles(expr, seen, handles);
            for item in set {
                match item {
                    BoundInsideItem::Expr(expr) => collect_logical_handles(expr, seen, handles),
                    BoundInsideItem::Range { low, high } => {
                        collect_logical_handles(low, seen, handles);
                        collect_logical_handles(high, seen, handles);
                    }
                }
            }
        }
        BoundLogicalKind::Concatenation { items } => {
            for item in items {
                collect_logical_handles(item, seen, handles);
            }
        }
        BoundLogicalKind::IntegralLiteral { .. }
        | BoundLogicalKind::RealLiteral { .. }
        | BoundLogicalKind::StringLiteral { .. }
        | BoundLogicalKind::EnumLabel { .. } => {}
    }
}

fn collect_selection_handles(
    selection: &BoundSelection,
    seen: &mut HashSet<SignalHandle>,
    handles: &mut Vec<SignalHandle>,
) {
    match selection {
        BoundSelection::Bit { index } => collect_logical_handles(index, seen, handles),
        BoundSelection::IndexedUp { base, .. } | BoundSelection::IndexedDown { base, .. } => {
            collect_logical_handles(base, seen, handles);
        }
        BoundSelection::Part { .. } => {}
    }
}

fn unknown_signal_diagnostic(name: &str) -> ExprDiagnostic {
    ExprDiagnostic {
        layer: crate::expr::DiagnosticLayer::Semantic,
        code: "HOST-UNKNOWN-SIGNAL",
        message: format!("unknown signal '{name}'"),
        primary_span: Span::new(0, 0),
        notes: vec![],
    }
}

#[cfg(test)]
mod tests {
    use crate::expr::{
        ExprDiagnostic, ExprStorage, ExprType, ExprTypeKind, ExpressionHost, IntegerLikeKind,
        SampledValue, SignalHandle, Span,
    };

    use super::{
        ScopedExprHost, event_candidate_handles, event_expr_contains_wildcard,
        referenced_signal_handles,
    };

    #[derive(Default)]
    struct StubHost;

    impl ExpressionHost for StubHost {
        fn resolve_signal(&self, name: &str) -> Result<SignalHandle, ExprDiagnostic> {
            match name {
                "top.clk" => Ok(SignalHandle(1)),
                "top.sig" => Ok(SignalHandle(2)),
                "top.ev" => Ok(SignalHandle(3)),
                other => Err(ExprDiagnostic {
                    layer: crate::expr::DiagnosticLayer::Semantic,
                    code: "HOST-UNKNOWN-SIGNAL",
                    message: format!("unknown signal '{other}'"),
                    primary_span: Span::new(0, 0),
                    notes: vec![],
                }),
            }
        }

        fn signal_type(&self, handle: SignalHandle) -> Result<ExprType, ExprDiagnostic> {
            let kind = match handle {
                SignalHandle(3) => ExprTypeKind::Event,
                _ => ExprTypeKind::IntegerLike(IntegerLikeKind::Int),
            };
            Ok(ExprType {
                kind,
                storage: ExprStorage::Scalar,
                width: 1,
                is_four_state: true,
                is_signed: false,
                enum_type_id: None,
                enum_labels: None,
            })
        }

        fn sample_value(
            &self,
            _handle: SignalHandle,
            _timestamp: u64,
        ) -> Result<SampledValue, ExprDiagnostic> {
            Ok(SampledValue::Integral {
                bits: Some("1".to_string()),
                label: None,
            })
        }

        fn event_occurred(
            &self,
            _handle: SignalHandle,
            _timestamp: u64,
        ) -> Result<bool, ExprDiagnostic> {
            Ok(true)
        }
    }

    #[test]
    fn scoped_host_rejects_dotted_names_when_scope_is_active() {
        let host = StubHost;
        let scoped = ScopedExprHost::new(&host, Some("top"));
        let error = scoped
            .resolve_signal("top.clk")
            .expect_err("dotted scoped token should fail");

        assert_eq!(error.code, "HOST-UNKNOWN-SIGNAL");
        assert_eq!(error.message, "unknown signal 'top.clk'");
    }

    #[test]
    fn bound_handle_helpers_preserve_unique_signal_order() {
        let host = StubHost;
        let scoped = ScopedExprHost::new(&host, Some("top"));
        let logical_ast = crate::expr::parse_logical_expr_ast("sig && ev.triggered() && sig")
            .expect("logical expr should parse");
        let bound_logical = crate::expr::bind_logical_expr_ast(&logical_ast, &scoped)
            .expect("logical expr should bind");
        assert_eq!(
            referenced_signal_handles(&bound_logical),
            vec![SignalHandle(2), SignalHandle(3)]
        );

        let event_ast =
            crate::expr::parse_event_expr_ast("* or posedge clk").expect("event expr should parse");
        let bound_event =
            crate::expr::bind_event_expr_ast(&event_ast, &scoped).expect("event expr should bind");
        assert!(event_expr_contains_wildcard(&bound_event));
        assert_eq!(event_candidate_handles(&bound_event), vec![SignalHandle(1)]);
    }
}
