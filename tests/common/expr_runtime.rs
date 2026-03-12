#![allow(dead_code)]

use std::cell::RefCell;
use std::collections::{HashMap, HashSet};

use serde::Deserialize;
use wavepeek::expr::{
    DiagnosticLayer, ExprDiagnostic, ExprStorage, ExprType, ExprTypeKind, ExpressionHost,
    IntegerLikeKind, SampledValue, SignalHandle, Span,
};

#[derive(Debug, Clone, Deserialize)]
pub struct SignalFixture {
    pub name: String,
    pub ty: TypeFixture,
    pub samples: Vec<SignalSample>,
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
}

#[derive(Debug, Clone, Deserialize)]
pub struct SignalSample {
    pub timestamp: u64,
    pub bits: Option<String>,
}

#[derive(Default)]
pub struct InMemoryExprHost {
    handles_by_name: HashMap<String, SignalHandle>,
    types_by_handle: HashMap<SignalHandle, ExprType>,
    timelines_by_handle: HashMap<SignalHandle, Vec<(u64, Option<String>)>>,
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
            host.timelines_by_handle.insert(
                handle,
                signal
                    .samples
                    .iter()
                    .map(|sample| (sample.timestamp, sample.bits.clone()))
                    .collect(),
            );
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

        let sampled = timeline
            .iter()
            .rev()
            .find(|(sample_time, _)| *sample_time <= timestamp)
            .map(|(_, bits)| bits.clone())
            .unwrap_or(None);
        Ok(SampledValue { bits: sampled })
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
    }
}
