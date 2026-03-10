use crate::expr::diagnostic::ExprDiagnostic;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SignalHandle(pub u32);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExprType {
    pub width: u32,
    pub is_four_state: bool,
    pub is_signed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SampledValue {
    pub bits: String,
}

pub trait ExpressionHost {
    fn resolve_signal(&self, name: &str) -> Result<SignalHandle, ExprDiagnostic>;
    fn signal_type(&self, handle: SignalHandle) -> Result<ExprType, ExprDiagnostic>;
    fn sample_value(
        &self,
        handle: SignalHandle,
        timestamp: u64,
    ) -> Result<SampledValue, ExprDiagnostic>;
}
