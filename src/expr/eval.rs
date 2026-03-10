use crate::error::WavepeekError;
use crate::expr::ast::DeferredLogicalExpr;
use crate::expr::diagnostic::{DiagnosticLayer, ExprDiagnostic};
use crate::expr::host::ExpressionHost;

use super::Expression;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TruthValue {
    Zero,
    One,
    Unknown,
}

pub(crate) fn eval_logical_expr(
    expr: &DeferredLogicalExpr,
    _host: &dyn ExpressionHost,
    _timestamp: u64,
) -> Result<TruthValue, ExprDiagnostic> {
    Err(ExprDiagnostic {
        layer: DiagnosticLayer::Runtime,
        code: "C1-RUNTIME-LOGICAL-NOT-IMPLEMENTED",
        message: "logical expression evaluation is not implemented yet".to_string(),
        primary_span: expr.span,
        notes: vec!["C1 keeps logical runtime semantics deferred".to_string()],
    })
}

pub fn eval(_expression: &Expression) -> Result<bool, WavepeekError> {
    Err(WavepeekError::Unimplemented(
        "expression evaluation is not implemented yet",
    ))
}

#[cfg(test)]
mod tests {
    use crate::expr::diagnostic::ExprDiagnostic;
    use crate::expr::diagnostic::Span;
    use crate::expr::host::{ExprType, ExpressionHost, SampledValue, SignalHandle};

    use super::*;

    struct HostStub;

    impl ExpressionHost for HostStub {
        fn resolve_signal(&self, _name: &str) -> Result<SignalHandle, ExprDiagnostic> {
            Ok(SignalHandle(1))
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

    #[test]
    fn eval_logical_expr_is_still_deferred_with_snapshot() {
        let expr = DeferredLogicalExpr {
            source: "rstn".to_string(),
            span: Span::new(0, 4),
        };
        let host = HostStub;

        let diagnostic = eval_logical_expr(&expr, &host, 0).expect_err("eval should fail");
        assert_eq!(diagnostic.layer, DiagnosticLayer::Runtime);
        assert_eq!(diagnostic.code, "C1-RUNTIME-LOGICAL-NOT-IMPLEMENTED");
        insta::assert_snapshot!(
            "eval_logical_expr_not_implemented",
            diagnostic.render(expr.source.as_str())
        );
    }
}
