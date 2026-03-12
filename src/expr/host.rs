use crate::expr::diagnostic::ExprDiagnostic;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SignalHandle(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntegerLikeKind {
    Byte,
    Shortint,
    Int,
    Longint,
    Integer,
    Time,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExprTypeKind {
    BitVector,
    IntegerLike(IntegerLikeKind),
    EnumCore,
    Real,
    String,
    Event,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExprStorage {
    PackedVector,
    Scalar,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnumLabelInfo {
    pub name: String,
    pub bits: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExprType {
    pub kind: ExprTypeKind,
    pub storage: ExprStorage,
    pub width: u32,
    pub is_four_state: bool,
    pub is_signed: bool,
    pub enum_type_id: Option<String>,
    pub enum_labels: Option<Vec<EnumLabelInfo>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SampledValue {
    Integral {
        bits: Option<String>,
        label: Option<String>,
    },
    Real {
        value: Option<f64>,
    },
    String {
        value: Option<String>,
    },
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
    /// `None` inside the returned payload means no sampled state exists at or
    /// before this timestamp.
    ///
    /// Raw `event` operands are not sampled through this method.
    fn sample_value(
        &self,
        handle: SignalHandle,
        timestamp: u64,
    ) -> Result<SampledValue, ExprDiagnostic>;

    /// Returns whether a raw `event` operand occurred at `timestamp`.
    ///
    /// Non-event operands are not queried through this method.
    fn event_occurred(&self, handle: SignalHandle, timestamp: u64) -> Result<bool, ExprDiagnostic>;
}
