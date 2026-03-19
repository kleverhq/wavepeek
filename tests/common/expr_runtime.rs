#![allow(dead_code)]

use std::cell::RefCell;
use std::collections::{HashMap, HashSet};

use serde::Deserialize;
use wavepeek::expr::{
    BoundEventExpr, BoundLogicalExpr, DiagnosticLayer, EnumLabelInfo, ExprDiagnostic, ExprStorage,
    ExprType, ExprTypeKind, ExprValue, ExprValuePayload, ExpressionHost, IntegerLikeKind,
    SampledValue, SignalHandle, Span, bind_event_expr_ast, bind_logical_expr_ast,
    eval_logical_expr_at, event_matches_at, parse_event_expr_ast, parse_logical_expr_ast,
};

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct SignalFixture {
    pub name: String,
    pub ty: TypeFixture,
    pub samples: Vec<SignalSample>,
    #[serde(default)]
    pub event_timestamps: Vec<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
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

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct EnumLabelFixture {
    pub name: String,
    pub bits: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
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

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct ExpectedValueFixture {
    pub kind: ExpectedValueKind,
    pub bits: Option<String>,
    pub label: Option<String>,
    pub real: Option<f64>,
    pub string: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExpectedValueKind {
    Integral,
    Real,
    String,
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
                code: "TEST-RUNTIME-UNEXPECTED-SAMPLE",
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

pub fn supported_host_profiles() -> &'static [&'static str] {
    &[
        "event_runtime_baseline",
        "integral_boolean_baseline",
        "rich_types_baseline",
    ]
}

pub fn is_supported_host_profile(profile: &str) -> bool {
    supported_host_profiles().contains(&profile)
}

pub fn host_from_profile(profile: &str) -> InMemoryExprHost {
    let fixtures = match profile {
        "event_runtime_baseline" => event_runtime_baseline_signals(),
        "integral_boolean_baseline" => integral_boolean_baseline_signals(),
        "rich_types_baseline" => rich_types_baseline_signals(),
        other => panic!("unsupported host profile '{other}'"),
    };
    InMemoryExprHost::from_fixtures(fixtures.as_slice())
}

pub fn bind_event_expr(
    source: &str,
    host: &dyn ExpressionHost,
) -> Result<BoundEventExpr, ExprDiagnostic> {
    let ast = parse_event_expr_ast(source)?;
    bind_event_expr_ast(&ast, host)
}

pub fn bind_logical_expr(
    source: &str,
    host: &dyn ExpressionHost,
) -> Result<BoundLogicalExpr, ExprDiagnostic> {
    let ast = parse_logical_expr_ast(source)?;
    bind_logical_expr_ast(&ast, host)
}

pub fn eval_logical_expr_source_at(
    source: &str,
    host: &dyn ExpressionHost,
    timestamp: u64,
) -> Result<ExprValue, ExprDiagnostic> {
    let bound = bind_logical_expr(source, host)?;
    eval_logical_expr_at(&bound, host, timestamp)
}

pub fn collect_event_matches(
    source: &str,
    host: &InMemoryExprHost,
    tracked_signals: &[String],
    probes: &[u64],
) -> Result<Vec<u64>, ExprDiagnostic> {
    let expr = bind_event_expr(source, host)?;
    let tracked_handles = host.tracked_handles(tracked_signals);
    collect_bound_event_matches(&expr, host, tracked_handles.as_slice(), probes)
}

pub fn collect_bound_event_matches(
    expr: &BoundEventExpr,
    host: &dyn ExpressionHost,
    tracked_handles: &[SignalHandle],
    probes: &[u64],
) -> Result<Vec<u64>, ExprDiagnostic> {
    let mut previous_timestamp = None;
    let mut matches = Vec::new();
    for probe in probes {
        let frame = wavepeek::expr::EventEvalFrame {
            timestamp: *probe,
            previous_timestamp,
            tracked_signals: tracked_handles,
        };
        if event_matches_at(expr, host, &frame)? {
            matches.push(*probe);
        }
        previous_timestamp = Some(*probe);
    }
    Ok(matches)
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

pub fn assert_expr_type_eq(actual: &ExprType, expected: &ExprType, case_name: &str) {
    assert_eq!(actual.kind, expected.kind, "case '{case_name}' kind");
    assert_eq!(
        actual.storage, expected.storage,
        "case '{case_name}' storage"
    );
    assert_eq!(actual.width, expected.width, "case '{case_name}' width");
    assert_eq!(
        actual.is_four_state, expected.is_four_state,
        "case '{case_name}' is_four_state"
    );
    assert_eq!(
        actual.is_signed, expected.is_signed,
        "case '{case_name}' is_signed"
    );
    assert_eq!(
        actual.enum_type_id, expected.enum_type_id,
        "case '{case_name}' enum_type_id"
    );
    assert_eq!(
        actual.enum_labels, expected.enum_labels,
        "case '{case_name}' enum_labels"
    );
}

pub fn assert_expected_value_eq(
    actual: &ExprValue,
    expected: &ExpectedValueFixture,
    case_name: &str,
) {
    match (&actual.payload, expected.kind) {
        (ExprValuePayload::Integral { bits, label }, ExpectedValueKind::Integral) => {
            assert_eq!(
                Some(bits.as_str()),
                expected.bits.as_deref(),
                "case '{case_name}' bits"
            );
            assert_eq!(
                label.as_deref(),
                expected.label.as_deref(),
                "case '{case_name}' label"
            );
        }
        (ExprValuePayload::Real { value }, ExpectedValueKind::Real) => {
            assert_eq!(Some(*value), expected.real, "case '{case_name}' real");
        }
        (ExprValuePayload::String { value }, ExpectedValueKind::String) => {
            assert_eq!(
                Some(value.as_str()),
                expected.string.as_deref(),
                "case '{case_name}' string"
            );
        }
        (other, kind) => panic!("case '{case_name}' expected {kind:?} payload, got {other:?}"),
    }
}

pub fn integral_bits(value: &ExprValue) -> &str {
    match &value.payload {
        ExprValuePayload::Integral { bits, .. } => bits.as_str(),
        other => panic!("expected integral payload, got {other:?}"),
    }
}

pub fn assert_integral_bits(value: &ExprValue, expected: &str) {
    assert_eq!(integral_bits(value), expected);
}

pub fn assert_integral_label(value: &ExprValue, expected: Option<&str>) {
    match &value.payload {
        ExprValuePayload::Integral { label, .. } => assert_eq!(label.as_deref(), expected),
        other => panic!("expected integral payload, got {other:?}"),
    }
}

impl ExpectedValueFixture {
    pub fn validate(&self, case_name: &str) -> Result<(), String> {
        match self.kind {
            ExpectedValueKind::Integral => {
                if self.bits.is_none() {
                    return Err(format!(
                        "logical case '{case_name}' uses integral expected_result without bits"
                    ));
                }
            }
            ExpectedValueKind::Real => {
                if self.real.is_none() {
                    return Err(format!(
                        "logical case '{case_name}' uses real expected_result without real"
                    ));
                }
            }
            ExpectedValueKind::String => {
                if self.string.is_none() {
                    return Err(format!(
                        "logical case '{case_name}' uses string expected_result without string"
                    ));
                }
            }
        }
        Ok(())
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

fn event_runtime_baseline_signals() -> Vec<SignalFixture> {
    vec![
        bit_signal("clk", 1, &[(0, "0")]),
        bit_signal("a", 1, &[(0, "0")]),
        bit_signal("b", 1, &[(0, "1")]),
        bit_signal("c", 1, &[(0, "1")]),
        bit_signal("data", 8, &[(0, "00000000")]),
        bit_signal("ev", 1, &[(0, "0")]),
        bit_signal("state", 2, &[(0, "00")]),
    ]
}

fn integral_boolean_baseline_signals() -> Vec<SignalFixture> {
    vec![
        bit_signal("clk", 1, &[(0, "0")]),
        bit_signal("a", 8, &[(0, "00000000")]),
        bit_signal("idx", 4, &[(0, "0001")]),
        integer_like_signal(
            "count",
            "int",
            32,
            false,
            true,
            &[(0, "00000000000000000000000000000001")],
        ),
        enum_signal("state", 2, "fsm_state", None, &[(0, "00", None)]),
        bit_signal("ev", 1, &[(0, "0")]),
    ]
}

fn rich_types_baseline_signals() -> Vec<SignalFixture> {
    vec![
        bit_signal("data", 8, &[(0, "00000011")]),
        integer_like_signal(
            "count",
            "int",
            32,
            false,
            true,
            &[(0, "00000000000000000000000000000010")],
        ),
        SignalFixture {
            name: "temp".to_string(),
            ty: real_type(),
            samples: vec![real_sample(0, 0.25), real_sample(10, 1.5)],
            event_timestamps: vec![],
        },
        SignalFixture {
            name: "msg".to_string(),
            ty: string_type(),
            samples: vec![string_sample(0, "idle"), string_sample(10, "go")],
            event_timestamps: vec![],
        },
        enum_signal(
            "state",
            2,
            "fsm_state",
            Some(vec![
                EnumLabelFixture {
                    name: "IDLE".to_string(),
                    bits: "00".to_string(),
                },
                EnumLabelFixture {
                    name: "BUSY".to_string(),
                    bits: "01".to_string(),
                },
                EnumLabelFixture {
                    name: "DONE".to_string(),
                    bits: "10".to_string(),
                },
            ]),
            &[(0, "01", Some("BUSY"))],
        ),
        enum_signal("state_no_labels", 2, "fsm_state", None, &[(0, "01", None)]),
        bit_signal("sel", 1, &[(0, "1")]),
        bit_signal("xsel", 1, &[(0, "x")]),
        bit_signal("xbus", 4, &[(0, "x101")]),
        SignalFixture {
            name: "clk".to_string(),
            ty: bit_vector_type(1, true, false),
            samples: vec![
                bits_sample(0, "0"),
                bits_sample(10, "1"),
                bits_sample(20, "0"),
            ],
            event_timestamps: vec![],
        },
        event_signal("ev", &[10]),
    ]
}

fn bit_signal(name: &str, width: u32, samples: &[(u64, &str)]) -> SignalFixture {
    SignalFixture {
        name: name.to_string(),
        ty: bit_vector_type(width, true, false),
        samples: samples
            .iter()
            .map(|(timestamp, bits)| bits_sample(*timestamp, bits))
            .collect(),
        event_timestamps: vec![],
    }
}

fn integer_like_signal(
    name: &str,
    integer_like_kind: &str,
    width: u32,
    is_four_state: bool,
    is_signed: bool,
    samples: &[(u64, &str)],
) -> SignalFixture {
    SignalFixture {
        name: name.to_string(),
        ty: TypeFixture {
            kind: "integer_like".to_string(),
            integer_like_kind: Some(integer_like_kind.to_string()),
            storage: "scalar".to_string(),
            width,
            is_four_state,
            is_signed,
            enum_type_id: None,
            enum_labels: None,
        },
        samples: samples
            .iter()
            .map(|(timestamp, bits)| bits_sample(*timestamp, bits))
            .collect(),
        event_timestamps: vec![],
    }
}

fn enum_signal(
    name: &str,
    width: u32,
    enum_type_id: &str,
    enum_labels: Option<Vec<EnumLabelFixture>>,
    samples: &[(u64, &str, Option<&str>)],
) -> SignalFixture {
    SignalFixture {
        name: name.to_string(),
        ty: TypeFixture {
            kind: "enum_core".to_string(),
            integer_like_kind: None,
            storage: "scalar".to_string(),
            width,
            is_four_state: true,
            is_signed: false,
            enum_type_id: Some(enum_type_id.to_string()),
            enum_labels,
        },
        samples: samples
            .iter()
            .map(|(timestamp, bits, label)| SignalSample {
                timestamp: *timestamp,
                bits: Some((*bits).to_string()),
                label: label.map(str::to_string),
                real: None,
                string: None,
            })
            .collect(),
        event_timestamps: vec![],
    }
}

fn event_signal(name: &str, event_timestamps: &[u64]) -> SignalFixture {
    SignalFixture {
        name: name.to_string(),
        ty: TypeFixture {
            kind: "event".to_string(),
            integer_like_kind: None,
            storage: "scalar".to_string(),
            width: 0,
            is_four_state: false,
            is_signed: false,
            enum_type_id: None,
            enum_labels: None,
        },
        samples: vec![],
        event_timestamps: event_timestamps.to_vec(),
    }
}

fn bit_vector_type(width: u32, is_four_state: bool, is_signed: bool) -> TypeFixture {
    TypeFixture {
        kind: "bit_vector".to_string(),
        integer_like_kind: None,
        storage: if width > 1 {
            "packed_vector".to_string()
        } else {
            "scalar".to_string()
        },
        width,
        is_four_state,
        is_signed,
        enum_type_id: None,
        enum_labels: None,
    }
}

fn real_type() -> TypeFixture {
    TypeFixture {
        kind: "real".to_string(),
        integer_like_kind: None,
        storage: "scalar".to_string(),
        width: 64,
        is_four_state: false,
        is_signed: false,
        enum_type_id: None,
        enum_labels: None,
    }
}

fn string_type() -> TypeFixture {
    TypeFixture {
        kind: "string".to_string(),
        integer_like_kind: None,
        storage: "scalar".to_string(),
        width: 0,
        is_four_state: false,
        is_signed: false,
        enum_type_id: None,
        enum_labels: None,
    }
}

fn bits_sample(timestamp: u64, bits: &str) -> SignalSample {
    SignalSample {
        timestamp,
        bits: Some(bits.to_string()),
        label: None,
        real: None,
        string: None,
    }
}

fn real_sample(timestamp: u64, value: f64) -> SignalSample {
    SignalSample {
        timestamp,
        bits: None,
        label: None,
        real: Some(value),
        string: None,
    }
}

fn string_sample(timestamp: u64, value: &str) -> SignalSample {
    SignalSample {
        timestamp,
        bits: None,
        label: None,
        real: None,
        string: Some(value.to_string()),
    }
}
