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
    pub bits: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EventEvalFrame<'a> {
    pub timestamp: u64,
    pub previous_timestamp: Option<u64>,
    pub tracked_signals: &'a [SignalHandle],
}

pub trait ExpressionHost {
    fn resolve_signal(&self, name: &str) -> Result<SignalHandle, ExprDiagnostic>;
    fn signal_type(&self, handle: SignalHandle) -> Result<ExprType, ExprDiagnostic>;
    /// Returns the sampled signal state at or before `timestamp`.
    ///
    /// `SampledValue.bits == None` means no sampled state exists at or before
    /// this timestamp.
    fn sample_value(
        &self,
        handle: SignalHandle,
        timestamp: u64,
    ) -> Result<SampledValue, ExprDiagnostic>;
}
