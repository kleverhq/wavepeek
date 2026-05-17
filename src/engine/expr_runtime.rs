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
    use std::fs;
    use std::path::Path;

    use tempfile::NamedTempFile;

    use crate::expr::sema::{
        BoundInsideItem, BoundLogicalExpr, BoundLogicalKind, BoundLogicalNode, BoundSelection,
    };
    use crate::expr::{
        DiagnosticLayer, ExprDiagnostic, ExprStorage, ExprType, ExprTypeKind, ExpressionHost,
        IntegerLikeKind, SampledValue, SignalHandle, Span,
    };
    use crate::waveform::expr_host::WaveformExprHost;

    use super::{
        ScopedExprHost, bind_waveform_event_expr, bind_waveform_logical_expr,
        candidate_sources_for_handles, eval_bound_logical_truth, event_candidate_handles,
        event_expr_contains_wildcard, event_expr_is_any_tracked_only, event_expr_is_edge_only,
        event_expr_matches, expr_diagnostic, open_shared_waveform, referenced_signal_handles,
        unknown_signal_diagnostic,
    };

    const TEST_VCD: &str = concat!(
        "$date\n  today\n$end\n",
        "$version\n  wavepeek-test\n$end\n",
        "$timescale 1ns $end\n",
        "$scope module top $end\n",
        "$var wire 1 ! clk $end\n",
        "$var wire 1 \" sig $end\n",
        "$var event 1 # ev $end\n",
        "$var real 1 $ temp $end\n",
        "$var string 1 % msg $end\n",
        "$upscope $end\n",
        "$enddefinitions $end\n",
        "#0\n",
        "0!\n",
        "0\"\n",
        "r0.0 $\n",
        "shello %\n",
        "#5\n",
        "1!\n",
        "1\"\n",
        "1#\n",
        "r2.5 $\n",
        "sworld %\n"
    );

    #[derive(Default)]
    struct StubHost;

    impl ExpressionHost for StubHost {
        fn resolve_signal(&self, name: &str) -> Result<SignalHandle, ExprDiagnostic> {
            match name {
                "top.clk" => Ok(SignalHandle(1)),
                "top.sig" => Ok(SignalHandle(2)),
                "top.ev" => Ok(SignalHandle(3)),
                "top.temp" => Ok(SignalHandle(4)),
                "top.msg" => Ok(SignalHandle(5)),
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
                SignalHandle(4) => ExprTypeKind::Real,
                SignalHandle(5) => ExprTypeKind::String,
                _ => ExprTypeKind::IntegerLike(IntegerLikeKind::Int),
            };
            Ok(ExprType {
                kind,
                storage: ExprStorage::Scalar,
                width: if handle == SignalHandle(4) { 64 } else { 1 },
                is_four_state: handle != SignalHandle(4) && handle != SignalHandle(5),
                is_signed: false,
                enum_type_id: None,
                enum_labels: None,
            })
        }

        fn sample_value(
            &self,
            handle: SignalHandle,
            _timestamp: u64,
        ) -> Result<SampledValue, ExprDiagnostic> {
            Ok(match handle {
                SignalHandle(2) => SampledValue::Integral {
                    bits: Some("1".to_string()),
                    label: None,
                },
                SignalHandle(4) => SampledValue::Real { value: Some(2.5) },
                SignalHandle(5) => SampledValue::String {
                    value: Some("hello".to_string()),
                },
                _ => SampledValue::Integral {
                    bits: Some("0".to_string()),
                    label: None,
                },
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
    fn scoped_host_prefixes_short_names_and_passthrough_without_scope() {
        let host = StubHost;
        let scoped = ScopedExprHost::new(&host, Some("top"));
        assert_eq!(
            scoped.resolve_signal("clk").expect("scoped short name"),
            SignalHandle(1)
        );

        let unscoped = ScopedExprHost::new(&host, None);
        assert_eq!(
            unscoped.resolve_signal("top.sig").expect("canonical name"),
            SignalHandle(2)
        );
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
        assert!(!event_expr_is_any_tracked_only(&bound_event));
        assert!(!event_expr_is_edge_only(&bound_event));

        let edge_only = crate::expr::bind_event_expr_ast(
            &crate::expr::parse_event_expr_ast("posedge clk or edge sig").expect("parse"),
            &scoped,
        )
        .expect("bind");
        assert!(event_expr_is_edge_only(&edge_only));

        let any_only = crate::expr::bind_event_expr_ast(
            &crate::expr::parse_event_expr_ast("*").expect("parse"),
            &scoped,
        )
        .expect("bind");
        assert!(event_expr_is_any_tracked_only(&any_only));
    }

    #[test]
    fn eval_bound_logical_truth_handles_integral_real_and_string_results() {
        let host = StubHost;
        for (source, expected) in [("top.sig", true), ("top.temp", true), ("top.msg", false)] {
            let expr = crate::expr::bind_logical_expr_ast(
                &crate::expr::parse_logical_expr_ast(source).expect("parse"),
                &host,
            )
            .expect("bind");
            assert_eq!(
                eval_bound_logical_truth(source, &expr, &host, 0).expect("eval"),
                expected,
                "{source}"
            );
        }
    }

    #[test]
    fn expr_helpers_open_waveforms_and_map_diagnostics() {
        let fixture = write_fixture(TEST_VCD, "expr-runtime.vcd");
        let shared = open_shared_waveform(fixture.path()).expect("waveform should open");
        assert_eq!(
            shared.borrow().metadata().expect("metadata").time_end,
            "5ns"
        );

        let host = WaveformExprHost::open(fixture.path()).expect("host should open");
        let bound = bind_waveform_logical_expr(&host, Some("top"), "sig")
            .expect("scoped logical expr should bind");
        assert!(eval_bound_logical_truth("sig", &bound, &host, 5).expect("eval"));

        let wrapped = expr_diagnostic(
            "sig",
            ExprDiagnostic {
                layer: DiagnosticLayer::Semantic,
                code: "HOST-UNKNOWN-SIGNAL",
                message: "unknown signal 'sig'".to_string(),
                primary_span: Span::new(0, 3),
                notes: vec![],
            },
        );
        assert!(wrapped.to_string().starts_with("error: expr:"));
    }

    #[test]
    fn candidate_sources_are_deduplicated_and_unknown_handles_fail() {
        let fixture = write_fixture(TEST_VCD, "expr-runtime.vcd");
        let host = WaveformExprHost::open(fixture.path()).expect("host should open");
        let clk = host.resolve_signal("top.clk").expect("clk should resolve");
        let sig = host.resolve_signal("top.sig").expect("sig should resolve");
        let deduped = candidate_sources_for_handles(&host, &[clk, sig, clk]).expect("sources");
        assert_eq!(deduped.len(), 2);
        assert_eq!(deduped[0].path, "top.clk");
        assert_eq!(deduped[1].path, "top.sig");

        let error = candidate_sources_for_handles(&host, &[SignalHandle(999)])
            .expect_err("unknown handle should fail");
        assert_eq!(
            error.to_string(),
            "error: internal: unknown signal handle 999"
        );
    }

    #[test]
    fn helper_wrappers_exercise_open_failures_event_matching_and_nested_handle_walks() {
        let error = open_shared_waveform(Path::new("/definitely/missing.vcd"))
            .expect_err("missing waveform should fail");
        assert!(error.to_string().contains("No such file or directory"));

        let fixture = write_fixture(TEST_VCD, "expr-runtime.vcd");
        let shared = open_shared_waveform(fixture.path()).expect("waveform should open");
        let (host, bound_event) = bind_waveform_event_expr(shared, Some("top"), "posedge clk")
            .expect("event binding should succeed");
        let frame = crate::expr::host::EventEvalFrame {
            timestamp: 5,
            previous_timestamp: Some(0),
            tracked_signals: &[SignalHandle(1)],
        };
        assert!(event_expr_matches("posedge clk", &bound_event, &host, &frame).expect("match"));

        let error = bind_waveform_event_expr(
            open_shared_waveform(fixture.path()).expect("waveform should reopen"),
            Some("top"),
            "top.clk",
        )
        .expect_err("scoped dotted token should fail");
        assert!(error.to_string().contains("unknown signal 'top.clk'"));

        let nested = BoundLogicalExpr {
            root: BoundLogicalNode {
                ty: ExprType {
                    kind: ExprTypeKind::BitVector,
                    storage: ExprStorage::PackedVector,
                    width: 4,
                    is_four_state: true,
                    is_signed: false,
                    enum_type_id: None,
                    enum_labels: None,
                },
                span: Span::new(0, 0),
                kind: BoundLogicalKind::Conditional {
                    condition: Box::new(BoundLogicalNode {
                        ty: ExprType {
                            kind: ExprTypeKind::BitVector,
                            storage: ExprStorage::Scalar,
                            width: 1,
                            is_four_state: true,
                            is_signed: false,
                            enum_type_id: None,
                            enum_labels: None,
                        },
                        span: Span::new(0, 0),
                        kind: BoundLogicalKind::Triggered {
                            handle: SignalHandle(3),
                        },
                    }),
                    when_true: Box::new(BoundLogicalNode {
                        ty: ExprType {
                            kind: ExprTypeKind::BitVector,
                            storage: ExprStorage::PackedVector,
                            width: 2,
                            is_four_state: true,
                            is_signed: false,
                            enum_type_id: None,
                            enum_labels: None,
                        },
                        span: Span::new(0, 0),
                        kind: BoundLogicalKind::Inside {
                            expr: Box::new(BoundLogicalNode {
                                ty: ExprType {
                                    kind: ExprTypeKind::BitVector,
                                    storage: ExprStorage::PackedVector,
                                    width: 2,
                                    is_four_state: true,
                                    is_signed: false,
                                    enum_type_id: None,
                                    enum_labels: None,
                                },
                                span: Span::new(0, 0),
                                kind: BoundLogicalKind::SignalRef {
                                    handle: SignalHandle(1),
                                },
                            }),
                            set: vec![BoundInsideItem::Range {
                                low: BoundLogicalNode {
                                    ty: ExprType {
                                        kind: ExprTypeKind::BitVector,
                                        storage: ExprStorage::Scalar,
                                        width: 1,
                                        is_four_state: true,
                                        is_signed: false,
                                        enum_type_id: None,
                                        enum_labels: None,
                                    },
                                    span: Span::new(0, 0),
                                    kind: BoundLogicalKind::SignalRef {
                                        handle: SignalHandle(2),
                                    },
                                },
                                high: BoundLogicalNode {
                                    ty: ExprType {
                                        kind: ExprTypeKind::BitVector,
                                        storage: ExprStorage::Scalar,
                                        width: 1,
                                        is_four_state: true,
                                        is_signed: false,
                                        enum_type_id: None,
                                        enum_labels: None,
                                    },
                                    span: Span::new(0, 0),
                                    kind: BoundLogicalKind::Selection {
                                        base: Box::new(BoundLogicalNode {
                                            ty: ExprType {
                                                kind: ExprTypeKind::BitVector,
                                                storage: ExprStorage::PackedVector,
                                                width: 2,
                                                is_four_state: true,
                                                is_signed: false,
                                                enum_type_id: None,
                                                enum_labels: None,
                                            },
                                            span: Span::new(0, 0),
                                            kind: BoundLogicalKind::SignalRef {
                                                handle: SignalHandle(4),
                                            },
                                        }),
                                        selection: BoundSelection::IndexedUp {
                                            base: Box::new(BoundLogicalNode {
                                                ty: ExprType {
                                                    kind: ExprTypeKind::BitVector,
                                                    storage: ExprStorage::Scalar,
                                                    width: 1,
                                                    is_four_state: true,
                                                    is_signed: false,
                                                    enum_type_id: None,
                                                    enum_labels: None,
                                                },
                                                span: Span::new(0, 0),
                                                kind: BoundLogicalKind::SignalRef {
                                                    handle: SignalHandle(5),
                                                },
                                            }),
                                            width: 1,
                                        },
                                    },
                                },
                            }],
                        },
                    }),
                    when_false: Box::new(BoundLogicalNode {
                        ty: ExprType {
                            kind: ExprTypeKind::BitVector,
                            storage: ExprStorage::PackedVector,
                            width: 2,
                            is_four_state: true,
                            is_signed: false,
                            enum_type_id: None,
                            enum_labels: None,
                        },
                        span: Span::new(0, 0),
                        kind: BoundLogicalKind::Concatenation {
                            items: vec![BoundLogicalNode {
                                ty: ExprType {
                                    kind: ExprTypeKind::BitVector,
                                    storage: ExprStorage::Scalar,
                                    width: 1,
                                    is_four_state: true,
                                    is_signed: false,
                                    enum_type_id: None,
                                    enum_labels: None,
                                },
                                span: Span::new(0, 0),
                                kind: BoundLogicalKind::SignalRef {
                                    handle: SignalHandle(6),
                                },
                            }],
                        },
                    }),
                },
            },
        };
        assert_eq!(
            referenced_signal_handles(&nested),
            vec![
                SignalHandle(3),
                SignalHandle(1),
                SignalHandle(2),
                SignalHandle(4),
                SignalHandle(5),
                SignalHandle(6),
            ]
        );

        let error = unknown_signal_diagnostic("missing.sig");
        assert_eq!(error.code, "HOST-UNKNOWN-SIGNAL");
        assert_eq!(error.message, "unknown signal 'missing.sig'");
    }

    #[test]
    fn scoped_host_forwarders_and_error_wrappers_exercise_edge_branches() {
        struct FailingHost;

        impl ExpressionHost for FailingHost {
            fn resolve_signal(&self, _name: &str) -> Result<SignalHandle, ExprDiagnostic> {
                Ok(SignalHandle(1))
            }

            fn signal_type(&self, _handle: SignalHandle) -> Result<ExprType, ExprDiagnostic> {
                Ok(ExprType {
                    kind: ExprTypeKind::BitVector,
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
                Err(ExprDiagnostic {
                    layer: DiagnosticLayer::Runtime,
                    code: "HOST-SAMPLE-FAIL",
                    message: "sample blew up".to_string(),
                    primary_span: Span::new(0, 0),
                    notes: vec![],
                })
            }

            fn event_occurred(
                &self,
                _handle: SignalHandle,
                _timestamp: u64,
            ) -> Result<bool, ExprDiagnostic> {
                Err(ExprDiagnostic {
                    layer: DiagnosticLayer::Runtime,
                    code: "HOST-EVENT-FAIL",
                    message: "event blew up".to_string(),
                    primary_span: Span::new(0, 0),
                    notes: vec![],
                })
            }
        }

        let host = StubHost;
        let scoped = ScopedExprHost::new(&host, Some("top"));
        assert_eq!(
            scoped
                .sample_value(SignalHandle(2), 5)
                .expect("forwarded sample"),
            SampledValue::Integral {
                bits: Some("1".to_string()),
                label: None,
            }
        );
        assert!(
            scoped
                .event_occurred(SignalHandle(1), 5)
                .expect("forwarded event")
        );

        let fixture = write_fixture(TEST_VCD, "expr-runtime-errors.vcd");
        let wf_host = WaveformExprHost::open(fixture.path()).expect("host should open");
        assert!(
            bind_waveform_event_expr(
                open_shared_waveform(fixture.path()).expect("waveform should reopen"),
                Some("top"),
                "posedge )",
            )
            .expect_err("parse failures should map to expr errors")
            .to_string()
            .starts_with("error: expr:")
        );
        assert!(
            bind_waveform_logical_expr(&wf_host, Some("top"), "ev")
                .expect_err("semantic bind failures should map to expr errors")
                .to_string()
                .contains("triggered()")
        );

        let logical = crate::expr::bind_logical_expr_ast(
            &crate::expr::parse_logical_expr_ast("top.sig").expect("parse"),
            &host,
        )
        .expect("bind");
        assert!(
            eval_bound_logical_truth("top.sig", &logical, &FailingHost, 0)
                .expect_err("runtime sample failures should map to expr errors")
                .to_string()
                .contains("sample blew up")
        );

        let event = crate::expr::bind_event_expr_ast(
            &crate::expr::parse_event_expr_ast("posedge top.clk").expect("parse"),
            &host,
        )
        .expect("event bind");
        let frame = crate::expr::host::EventEvalFrame {
            timestamp: 5,
            previous_timestamp: Some(0),
            tracked_signals: &[SignalHandle(1)],
        };
        assert!(
            event_expr_matches("posedge top.clk", &event, &FailingHost, &frame)
                .expect_err("runtime event failures should map to expr errors")
                .to_string()
                .contains("sample blew up")
        );
        assert!(
            FailingHost
                .event_occurred(SignalHandle(1), 5)
                .expect_err("direct event failure should remain available")
                .message
                .contains("event blew up")
        );

        let rich_tree = BoundLogicalExpr {
            root: BoundLogicalNode {
                ty: ExprType {
                    kind: ExprTypeKind::BitVector,
                    storage: ExprStorage::PackedVector,
                    width: 4,
                    is_four_state: true,
                    is_signed: false,
                    enum_type_id: None,
                    enum_labels: None,
                },
                span: Span::new(0, 0),
                kind: BoundLogicalKind::Concatenation {
                    items: vec![
                        BoundLogicalNode {
                            ty: ExprType {
                                kind: ExprTypeKind::BitVector,
                                storage: ExprStorage::Scalar,
                                width: 1,
                                is_four_state: true,
                                is_signed: false,
                                enum_type_id: None,
                                enum_labels: None,
                            },
                            span: Span::new(0, 0),
                            kind: BoundLogicalKind::Parenthesized {
                                expr: Box::new(BoundLogicalNode {
                                    ty: ExprType {
                                        kind: ExprTypeKind::BitVector,
                                        storage: ExprStorage::Scalar,
                                        width: 1,
                                        is_four_state: true,
                                        is_signed: false,
                                        enum_type_id: None,
                                        enum_labels: None,
                                    },
                                    span: Span::new(0, 0),
                                    kind: BoundLogicalKind::SignalRef {
                                        handle: SignalHandle(10),
                                    },
                                }),
                            },
                        },
                        BoundLogicalNode {
                            ty: ExprType {
                                kind: ExprTypeKind::BitVector,
                                storage: ExprStorage::Scalar,
                                width: 1,
                                is_four_state: true,
                                is_signed: false,
                                enum_type_id: None,
                                enum_labels: None,
                            },
                            span: Span::new(0, 0),
                            kind: BoundLogicalKind::Cast {
                                kind: crate::expr::sema::BoundCastKind::Static,
                                expr: Box::new(BoundLogicalNode {
                                    ty: ExprType {
                                        kind: ExprTypeKind::BitVector,
                                        storage: ExprStorage::Scalar,
                                        width: 1,
                                        is_four_state: true,
                                        is_signed: false,
                                        enum_type_id: None,
                                        enum_labels: None,
                                    },
                                    span: Span::new(0, 0),
                                    kind: BoundLogicalKind::SignalRef {
                                        handle: SignalHandle(11),
                                    },
                                }),
                            },
                        },
                        BoundLogicalNode {
                            ty: ExprType {
                                kind: ExprTypeKind::BitVector,
                                storage: ExprStorage::Scalar,
                                width: 1,
                                is_four_state: true,
                                is_signed: false,
                                enum_type_id: None,
                                enum_labels: None,
                            },
                            span: Span::new(0, 0),
                            kind: BoundLogicalKind::Unary {
                                op: crate::expr::ast::UnaryOpAst::Plus,
                                expr: Box::new(BoundLogicalNode {
                                    ty: ExprType {
                                        kind: ExprTypeKind::BitVector,
                                        storage: ExprStorage::Scalar,
                                        width: 1,
                                        is_four_state: true,
                                        is_signed: false,
                                        enum_type_id: None,
                                        enum_labels: None,
                                    },
                                    span: Span::new(0, 0),
                                    kind: BoundLogicalKind::SignalRef {
                                        handle: SignalHandle(12),
                                    },
                                }),
                            },
                        },
                        BoundLogicalNode {
                            ty: ExprType {
                                kind: ExprTypeKind::BitVector,
                                storage: ExprStorage::Scalar,
                                width: 1,
                                is_four_state: true,
                                is_signed: false,
                                enum_type_id: None,
                                enum_labels: None,
                            },
                            span: Span::new(0, 0),
                            kind: BoundLogicalKind::Replication {
                                count: 2,
                                expr: Box::new(BoundLogicalNode {
                                    ty: ExprType {
                                        kind: ExprTypeKind::BitVector,
                                        storage: ExprStorage::Scalar,
                                        width: 1,
                                        is_four_state: true,
                                        is_signed: false,
                                        enum_type_id: None,
                                        enum_labels: None,
                                    },
                                    span: Span::new(0, 0),
                                    kind: BoundLogicalKind::SignalRef {
                                        handle: SignalHandle(13),
                                    },
                                }),
                            },
                        },
                        BoundLogicalNode {
                            ty: ExprType {
                                kind: ExprTypeKind::BitVector,
                                storage: ExprStorage::PackedVector,
                                width: 2,
                                is_four_state: true,
                                is_signed: false,
                                enum_type_id: None,
                                enum_labels: None,
                            },
                            span: Span::new(0, 0),
                            kind: BoundLogicalKind::Selection {
                                base: Box::new(BoundLogicalNode {
                                    ty: ExprType {
                                        kind: ExprTypeKind::BitVector,
                                        storage: ExprStorage::PackedVector,
                                        width: 2,
                                        is_four_state: true,
                                        is_signed: false,
                                        enum_type_id: None,
                                        enum_labels: None,
                                    },
                                    span: Span::new(0, 0),
                                    kind: BoundLogicalKind::SignalRef {
                                        handle: SignalHandle(14),
                                    },
                                }),
                                selection: BoundSelection::IndexedDown {
                                    base: Box::new(BoundLogicalNode {
                                        ty: ExprType {
                                            kind: ExprTypeKind::BitVector,
                                            storage: ExprStorage::Scalar,
                                            width: 1,
                                            is_four_state: true,
                                            is_signed: false,
                                            enum_type_id: None,
                                            enum_labels: None,
                                        },
                                        span: Span::new(0, 0),
                                        kind: BoundLogicalKind::SignalRef {
                                            handle: SignalHandle(15),
                                        },
                                    }),
                                    width: 1,
                                },
                            },
                        },
                    ],
                },
            },
        };
        assert_eq!(
            referenced_signal_handles(&rich_tree),
            vec![
                SignalHandle(10),
                SignalHandle(11),
                SignalHandle(12),
                SignalHandle(13),
                SignalHandle(14),
                SignalHandle(15),
            ]
        );
    }

    fn write_fixture(contents: &str, suffix: &str) -> NamedTempFile {
        let fixture = NamedTempFile::with_suffix(suffix).expect("fixture should create");
        fs::write(fixture.path(), contents).expect("fixture should write");
        fixture
    }
}
