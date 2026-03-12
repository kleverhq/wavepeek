#![allow(dead_code)]

use std::cell::RefCell;
use std::collections::{HashMap, HashSet};

use serde::Deserialize;
use wavepeek::expr::{
    DiagnosticLayer, EnumLabelInfo, ExprDiagnostic, ExprStorage, ExprType, ExprTypeKind,
    ExpressionHost, IntegerLikeKind, SampledValue, SignalHandle, Span,
};

#[derive(Debug, Clone, Deserialize)]
pub struct SignalFixture {
    pub name: String,
    pub ty: TypeFixture,
    pub samples: Vec<SignalSample>,
    #[serde(default)]
    pub event_timestamps: Vec<u64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TypeFixture {
    pub kind: String,
    pub integer_like_kind: Option<String>,
    pub storage: String,
    pub width: u32,
    pub is_four_state: bool,
    pub is_signed: bool,
    pub enum_type_id: Option<String>,
    #[serde(default)]
    pub enum_labels: Option<Vec<EnumLabelFixture>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EnumLabelFixture {
    pub name: String,
    pub bits: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SignalSample {
    pub timestamp: u64,
    pub bits: Option<String>,
    #[serde(default)]
    pub label: Option<String>,
    #[serde(default)]
    pub real: Option<f64>,
    #[serde(default)]
    pub string: Option<String>,
}

#[derive(Default)]
pub struct InMemoryExprHost {
    handles_by_name: HashMap<String, SignalHandle>,
    types_by_handle: HashMap<SignalHandle, ExprType>,
    timelines_by_handle: HashMap<SignalHandle, Vec<(u64, SampledValue)>>,
    events_by_handle: HashMap<SignalHandle, Vec<u64>>,
    trap_handles: HashSet<SignalHandle>,
    sample_counts: RefCell<HashMap<SignalHandle, usize>>,
}

impl InMemoryExprHost {
    pub fn from_fixtures(signals: &[SignalFixture]) -> Self {
        let mut host = Self::default();
        for (index, signal) in signals.iter().enumerate() {
            let handle = SignalHandle((index + 1) as u32);
            host.handles_by_name.insert(signal.name.clone(), handle);
            host.types_by_handle
                .insert(handle, expr_type_from_fixture(&signal.ty));
            host.timelines_by_handle
                .insert(handle, samples_from_fixture(signal));
            host.events_by_handle
                .insert(handle, signal.event_timestamps.clone());
        }
        host
    }

    pub fn tracked_handles(&self, names: &[String]) -> Vec<SignalHandle> {
        names
            .iter()
            .map(|name| {
                *self
                    .handles_by_name
                    .get(name)
                    .unwrap_or_else(|| panic!("tracked signal '{name}' must exist"))
            })
            .collect()
    }

    pub fn handle(&self, name: &str) -> SignalHandle {
        *self
            .handles_by_name
            .get(name)
            .unwrap_or_else(|| panic!("signal '{name}' must exist"))
    }

    pub fn enable_sample_trap(&mut self, name: &str) {
        self.trap_handles.insert(self.handle(name));
    }

    pub fn sample_count(&self, name: &str) -> usize {
        let handle = self.handle(name);
        self.sample_counts
            .borrow()
            .get(&handle)
            .copied()
            .unwrap_or(0)
    }
}

impl ExpressionHost for InMemoryExprHost {
    fn resolve_signal(&self, name: &str) -> Result<SignalHandle, ExprDiagnostic> {
        self.handles_by_name
            .get(name)
            .copied()
            .ok_or_else(|| ExprDiagnostic {
                layer: DiagnosticLayer::Semantic,
                code: "HOST-UNKNOWN-SIGNAL",
                message: format!("unknown signal '{name}'"),
                primary_span: Span::new(0, 0),
                notes: vec![],
            })
    }

    fn signal_type(&self, handle: SignalHandle) -> Result<ExprType, ExprDiagnostic> {
        self.types_by_handle
            .get(&handle)
            .cloned()
            .ok_or_else(|| ExprDiagnostic {
                layer: DiagnosticLayer::Semantic,
                code: "HOST-UNKNOWN-TYPE",
                message: format!("unknown type for handle {}", handle.0),
                primary_span: Span::new(0, 0),
                notes: vec![],
            })
    }

    fn sample_value(
        &self,
        handle: SignalHandle,
        timestamp: u64,
    ) -> Result<SampledValue, ExprDiagnostic> {
        *self.sample_counts.borrow_mut().entry(handle).or_insert(0) += 1;

        if self.trap_handles.contains(&handle) {
            return Err(ExprDiagnostic {
                layer: DiagnosticLayer::Runtime,
                code: "C3-RUNTIME-UNEXPECTED-SAMPLE",
                message: format!("signal {} was sampled unexpectedly", handle.0),
                primary_span: Span::new(0, 0),
                notes: vec!["short-circuited branch must not sample this signal".to_string()],
            });
        }

        let timeline = self
            .timelines_by_handle
            .get(&handle)
            .ok_or_else(|| ExprDiagnostic {
                layer: DiagnosticLayer::Runtime,
                code: "HOST-MISSING-TIMELINE",
                message: format!("missing timeline for handle {}", handle.0),
                primary_span: Span::new(0, 0),
                notes: vec![],
            })?;

        if matches!(
            self.types_by_handle.get(&handle).map(|ty| &ty.kind),
            Some(ExprTypeKind::Event)
        ) {
            return Err(ExprDiagnostic {
                layer: DiagnosticLayer::Runtime,
                code: "HOST-EVENT-SAMPLE-MISUSE",
                message: format!("event handle {} cannot be sampled as a value", handle.0),
                primary_span: Span::new(0, 0),
                notes: vec!["use event_occurred for raw event operands".to_string()],
            });
        }

        let sampled = timeline
            .iter()
            .rev()
            .find(|(sample_time, _)| *sample_time <= timestamp)
            .map(|(_, value)| value.clone())
            .unwrap_or_else(
                || match self.types_by_handle.get(&handle).map(|ty| &ty.kind) {
                    Some(ExprTypeKind::Real) => SampledValue::Real { value: None },
                    Some(ExprTypeKind::String) => SampledValue::String { value: None },
                    _ => SampledValue::Integral {
                        bits: None,
                        label: None,
                    },
                },
            );
        Ok(sampled)
    }

    fn event_occurred(&self, handle: SignalHandle, timestamp: u64) -> Result<bool, ExprDiagnostic> {
        if !matches!(
            self.types_by_handle.get(&handle).map(|ty| &ty.kind),
            Some(ExprTypeKind::Event)
        ) {
            return Err(ExprDiagnostic {
                layer: DiagnosticLayer::Runtime,
                code: "HOST-EVENT-OCCURRED-MISUSE",
                message: format!(
                    "non-event handle {} cannot be queried as an event",
                    handle.0
                ),
                primary_span: Span::new(0, 0),
                notes: vec!["event_occurred is reserved for raw event operands".to_string()],
            });
        }

        Ok(self
            .events_by_handle
            .get(&handle)
            .is_some_and(|events| events.contains(&timestamp)))
    }
}

pub fn expr_type_from_fixture(fixture: &TypeFixture) -> ExprType {
    let kind = match fixture.kind.as_str() {
        "bit_vector" => ExprTypeKind::BitVector,
        "integer_like" => {
            let raw = fixture
                .integer_like_kind
                .as_deref()
                .unwrap_or_else(|| panic!("integer_like_kind is required for integer_like type"));
            ExprTypeKind::IntegerLike(match raw {
                "byte" => IntegerLikeKind::Byte,
                "shortint" => IntegerLikeKind::Shortint,
                "int" => IntegerLikeKind::Int,
                "longint" => IntegerLikeKind::Longint,
                "integer" => IntegerLikeKind::Integer,
                "time" => IntegerLikeKind::Time,
                other => panic!("unsupported integer_like_kind '{other}'"),
            })
        }
        "enum_core" => ExprTypeKind::EnumCore,
        "real" => ExprTypeKind::Real,
        "string" => ExprTypeKind::String,
        "event" => ExprTypeKind::Event,
        other => panic!("unsupported type kind '{other}'"),
    };

    let storage = match fixture.storage.as_str() {
        "packed_vector" => ExprStorage::PackedVector,
        "scalar" => ExprStorage::Scalar,
        other => panic!("unsupported storage kind '{other}'"),
    };

    ExprType {
        kind,
        storage,
        width: fixture.width,
        is_four_state: fixture.is_four_state,
        is_signed: fixture.is_signed,
        enum_type_id: fixture.enum_type_id.clone(),
        enum_labels: fixture.enum_labels.as_ref().map(|labels| {
            labels
                .iter()
                .map(|entry| EnumLabelInfo {
                    name: entry.name.clone(),
                    bits: entry.bits.clone(),
                })
                .collect()
        }),
    }
}

fn samples_from_fixture(signal: &SignalFixture) -> Vec<(u64, SampledValue)> {
    signal
        .samples
        .iter()
        .map(|sample| {
            let value = match signal.ty.kind.as_str() {
                "real" => SampledValue::Real { value: sample.real },
                "string" => SampledValue::String {
                    value: sample.string.clone(),
                },
                _ => SampledValue::Integral {
                    bits: sample.bits.clone(),
                    label: sample.label.clone(),
                },
            };
            (sample.timestamp, value)
        })
        .collect()
}
