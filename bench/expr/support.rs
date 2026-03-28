#![allow(dead_code)]

use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;

use criterion::Criterion;
use wavepeek::expr::{
    DiagnosticLayer, ExprDiagnostic, ExprStorage, ExprType, ExprTypeKind, ExpressionHost,
    IntegerLikeKind, SampledValue, SignalHandle, Span,
};

#[derive(Clone)]
pub struct BenchSignal {
    ty: ExprType,
    samples: Vec<(u64, SampledValue)>,
    event_timestamps: Vec<u64>,
}

impl BenchSignal {
    pub fn new(ty: ExprType, samples: Vec<(u64, SampledValue)>) -> Self {
        Self::with_events(ty, samples, vec![])
    }

    pub fn with_events(
        ty: ExprType,
        mut samples: Vec<(u64, SampledValue)>,
        mut event_timestamps: Vec<u64>,
    ) -> Self {
        samples.sort_by_key(|(timestamp, _)| *timestamp);
        event_timestamps.sort_unstable();
        Self {
            ty,
            samples,
            event_timestamps,
        }
    }
}

pub struct BenchHost {
    handles: HashMap<&'static str, SignalHandle>,
    signals: HashMap<SignalHandle, BenchSignal>,
}

impl BenchHost {
    pub fn new() -> Self {
        Self {
            handles: HashMap::new(),
            signals: HashMap::new(),
        }
    }

    pub fn insert_signal(&mut self, name: &'static str, handle: u32, signal: BenchSignal) {
        let signal_handle = SignalHandle(handle);
        self.handles.insert(name, signal_handle);
        self.signals.insert(signal_handle, signal);
    }

    pub fn handle(&self, name: &'static str) -> SignalHandle {
        *self.handles.get(name).expect("benchmark signal must exist")
    }
}

impl ExpressionHost for BenchHost {
    fn resolve_signal(&self, name: &str) -> Result<SignalHandle, ExprDiagnostic> {
        self.handles.get(name).copied().ok_or_else(|| {
            semantic_diagnostic("HOST-UNKNOWN-SIGNAL", format!("unknown signal '{name}'"))
        })
    }

    fn signal_type(&self, handle: SignalHandle) -> Result<ExprType, ExprDiagnostic> {
        self.signals
            .get(&handle)
            .map(|signal| signal.ty.clone())
            .ok_or_else(|| {
                semantic_diagnostic(
                    "HOST-UNKNOWN-TYPE",
                    format!("unknown signal handle {}", handle.0),
                )
            })
    }

    fn sample_value(
        &self,
        handle: SignalHandle,
        timestamp: u64,
    ) -> Result<SampledValue, ExprDiagnostic> {
        let signal = self.signals.get(&handle).ok_or_else(|| {
            runtime_diagnostic(
                "BENCH-MISSING-TIMELINE",
                format!("missing timeline for signal handle {}", handle.0),
            )
        })?;
        let insertion_index = signal
            .samples
            .partition_point(|(sample_time, _)| *sample_time <= timestamp);
        Ok(if insertion_index == 0 {
            empty_sample_for_type(&signal.ty)
        } else {
            signal.samples[insertion_index - 1].1.clone()
        })
    }

    fn event_occurred(&self, handle: SignalHandle, timestamp: u64) -> Result<bool, ExprDiagnostic> {
        let signal = self.signals.get(&handle).ok_or_else(|| {
            runtime_diagnostic(
                "BENCH-MISSING-EVENTS",
                format!("missing event set for signal handle {}", handle.0),
            )
        })?;
        Ok(signal.event_timestamps.contains(&timestamp))
    }
}

pub fn criterion_config() -> Criterion {
    Criterion::default()
        .sample_size(100)
        .warm_up_time(Duration::from_secs(3))
        .measurement_time(Duration::from_secs(5))
        .significance_level(0.05)
        .noise_threshold(0.01)
        .configure_from_args()
}

pub fn integral_sample(bits: &str) -> SampledValue {
    SampledValue::Integral {
        bits: Some(bits.to_string()),
        label: None,
    }
}

pub fn integral_signal(ty: ExprType, samples: Vec<(u64, &'static str)>) -> BenchSignal {
    BenchSignal::new(
        ty,
        samples
            .into_iter()
            .map(|(timestamp, bits)| (timestamp, integral_sample(bits)))
            .collect(),
    )
}

pub fn empty_sample_for_type(ty: &ExprType) -> SampledValue {
    match ty.kind {
        ExprTypeKind::Real => SampledValue::Real { value: None },
        ExprTypeKind::String => SampledValue::String { value: None },
        _ => SampledValue::Integral {
            bits: None,
            label: None,
        },
    }
}

pub fn bit_ty(width: u32, is_four_state: bool, is_signed: bool) -> ExprType {
    ExprType {
        kind: ExprTypeKind::BitVector,
        storage: if width > 1 {
            ExprStorage::PackedVector
        } else {
            ExprStorage::Scalar
        },
        width,
        is_four_state,
        is_signed,
        enum_type_id: None,
        enum_labels: None,
    }
}

pub fn int_ty(kind: IntegerLikeKind) -> ExprType {
    let (width, is_four_state) = match kind {
        IntegerLikeKind::Byte => (8, false),
        IntegerLikeKind::Shortint => (16, false),
        IntegerLikeKind::Int => (32, false),
        IntegerLikeKind::Longint => (64, false),
        IntegerLikeKind::Integer => (32, true),
        IntegerLikeKind::Time => (64, true),
    };
    ExprType {
        kind: ExprTypeKind::IntegerLike(kind),
        storage: ExprStorage::Scalar,
        width,
        is_four_state,
        is_signed: kind != IntegerLikeKind::Time,
        enum_type_id: None,
        enum_labels: None,
    }
}

pub fn real_ty() -> ExprType {
    ExprType {
        kind: ExprTypeKind::Real,
        storage: ExprStorage::Scalar,
        width: 64,
        is_four_state: false,
        is_signed: false,
        enum_type_id: None,
        enum_labels: None,
    }
}

pub fn string_ty() -> ExprType {
    ExprType {
        kind: ExprTypeKind::String,
        storage: ExprStorage::Scalar,
        width: 0,
        is_four_state: false,
        is_signed: false,
        enum_type_id: None,
        enum_labels: None,
    }
}

pub fn event_ty() -> ExprType {
    ExprType {
        kind: ExprTypeKind::Event,
        storage: ExprStorage::Scalar,
        width: 0,
        is_four_state: false,
        is_signed: false,
        enum_type_id: None,
        enum_labels: None,
    }
}

pub fn fixture_path(filename: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("hand")
        .join(filename)
}

fn semantic_diagnostic(code: &'static str, message: String) -> ExprDiagnostic {
    ExprDiagnostic {
        layer: DiagnosticLayer::Semantic,
        code,
        message,
        primary_span: Span::new(0, 0),
        notes: vec![],
    }
}

fn runtime_diagnostic(code: &'static str, message: String) -> ExprDiagnostic {
    ExprDiagnostic {
        layer: DiagnosticLayer::Runtime,
        code,
        message,
        primary_span: Span::new(0, 0),
        notes: vec![],
    }
}
