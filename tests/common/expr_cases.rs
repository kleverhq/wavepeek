#![allow(dead_code)]

use std::fs;
use std::path::PathBuf;

use serde::Deserialize;
use serde::de::DeserializeOwned;
use wavepeek::expr::{
    DiagnosticLayer, ExprDiagnostic, Span, bind_event_expr_ast, bind_logical_expr_ast,
    eval_logical_expr_at, parse_event_expr_ast, parse_logical_expr_ast,
};

use super::expr_runtime::{
    ExpectedValueFixture, InMemoryExprHost, SignalFixture, TypeFixture, host_from_profile,
    is_supported_host_profile,
};

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct SpanRecord {
    pub start: usize,
    pub end: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ManifestEntrypoint {
    Parse,
    Logical,
    Event,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ManifestLayer {
    Parse,
    Semantic,
    Runtime,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct PositiveManifest {
    pub cases: Vec<PositiveCase>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum PositiveCase {
    EventParse(EventParseCase),
    LogicalEval(LogicalEvalCase),
    EventEval(EventEvalCase),
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct EventParseCase {
    pub name: String,
    pub source: String,
    pub terms: Vec<NormalizedTerm>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct NormalizedTerm {
    pub event: String,
    pub name: Option<String>,
    pub iff: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct LogicalEvalCase {
    pub name: String,
    pub source: String,
    pub signals: Vec<SignalFixture>,
    pub timestamp: u64,
    pub expected_type: TypeFixture,
    pub expected_result: ExpectedValueFixture,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct EventEvalCase {
    pub name: String,
    pub source: String,
    pub tracked_signals: Vec<String>,
    pub signals: Vec<SignalFixture>,
    pub probes: Vec<u64>,
    pub matches: Vec<u64>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct NegativeManifest {
    pub cases: Vec<NegativeCase>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct NegativeCase {
    pub name: String,
    pub entrypoint: ManifestEntrypoint,
    pub source: String,
    pub layer: ManifestLayer,
    pub code: String,
    pub span: SpanRecord,
    pub snapshot: Option<String>,
    #[serde(default)]
    pub host_profile: Option<String>,
    #[serde(default)]
    pub signals: Vec<SignalFixture>,
    #[serde(default)]
    pub timestamp: Option<u64>,
}

pub fn fixture_expr_path(file_name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("expr")
        .join(file_name)
}

pub fn expression_snapshot_path(file_name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("snapshots")
        .join(file_name)
}

pub fn expression_manifest_file_names() -> Vec<String> {
    let mut entries = fs::read_dir(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("fixtures")
            .join("expr"),
    )
    .expect("expression fixture directory should be readable")
    .map(|entry| entry.expect("fixture entry should be readable"))
    .filter_map(|entry| {
        entry
            .file_type()
            .expect("fixture entry type should be readable")
            .is_file()
            .then(|| entry.file_name().to_string_lossy().into_owned())
    })
    .collect::<Vec<_>>();
    entries.sort();
    entries
}

pub fn expression_snapshot_file_names() -> Vec<String> {
    let mut entries = fs::read_dir(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("snapshots"),
    )
    .expect("snapshot directory should be readable")
    .map(|entry| entry.expect("snapshot entry should be readable"))
    .filter_map(|entry| {
        let file_name = entry.file_name().to_string_lossy().into_owned();
        (entry
            .file_type()
            .expect("snapshot entry type should be readable")
            .is_file()
            && file_name.starts_with("expression_")
            && file_name.ends_with(".snap"))
        .then_some(file_name)
    })
    .collect::<Vec<_>>();
    entries.sort();
    entries
}

pub fn load_expr_manifest<T: DeserializeOwned>(file_name: &str) -> T {
    let path = fixture_expr_path(file_name);
    let payload = fs::read_to_string(path)
        .unwrap_or_else(|error| panic!("manifest '{file_name}' should be readable: {error}"));
    serde_json::from_str(&payload)
        .unwrap_or_else(|error| panic!("manifest '{file_name}' should be valid JSON: {error}"))
}

pub fn parse_positive_manifest_payload(payload: &str) -> Result<PositiveManifest, String> {
    let manifest = serde_json::from_str::<PositiveManifest>(payload)
        .map_err(|error| format!("positive manifest JSON should deserialize: {error}"))?;
    validate_positive_manifest(manifest)
}

pub fn load_positive_manifest(file_name: &str) -> PositiveManifest {
    let payload = fs::read_to_string(fixture_expr_path(file_name))
        .unwrap_or_else(|error| panic!("manifest '{file_name}' should be readable: {error}"));
    parse_positive_manifest_payload(&payload).unwrap_or_else(|error| {
        panic!("manifest '{file_name}' should match shared contract: {error}")
    })
}

pub fn parse_negative_manifest_payload(payload: &str) -> Result<NegativeManifest, String> {
    let manifest = serde_json::from_str::<NegativeManifest>(payload)
        .map_err(|error| format!("negative manifest JSON should deserialize: {error}"))?;
    validate_negative_manifest(manifest)
}

pub fn load_negative_manifest(file_name: &str) -> NegativeManifest {
    let payload = fs::read_to_string(fixture_expr_path(file_name))
        .unwrap_or_else(|error| panic!("manifest '{file_name}' should be readable: {error}"));
    parse_negative_manifest_payload(&payload).unwrap_or_else(|error| {
        panic!("manifest '{file_name}' should match shared contract: {error}")
    })
}

pub fn expected_layer(raw: &str) -> DiagnosticLayer {
    match raw {
        "parse" => DiagnosticLayer::Parse,
        "semantic" => DiagnosticLayer::Semantic,
        "runtime" => DiagnosticLayer::Runtime,
        other => panic!("unsupported manifest layer '{other}'"),
    }
}

pub fn snapshot_suite_prefix(manifest_file_name: &str) -> &'static str {
    match manifest_file_name {
        "parse_negative_manifest.json" => "expression_parse",
        "event_runtime_negative_manifest.json" => "expression_event_runtime",
        "integral_boolean_negative_manifest.json" => "expression_integral_boolean",
        "rich_types_negative_manifest.json" => "expression_rich_types",
        other => panic!("unsupported negative manifest '{other}' for snapshot mapping"),
    }
}

pub fn snapshot_file_name(manifest_file_name: &str, snapshot_name: &str) -> String {
    format!(
        "{}__{}.snap",
        snapshot_suite_prefix(manifest_file_name),
        snapshot_name
    )
}

pub fn snapshot_identifier(manifest_file_name: &str, snapshot_name: &str) -> String {
    format!(
        "{}__{}",
        snapshot_suite_prefix(manifest_file_name),
        snapshot_name
    )
}

pub fn assert_negative_diagnostic(
    manifest_file_name: &str,
    case: &NegativeCase,
    diagnostic: &ExprDiagnostic,
) {
    assert_eq!(
        diagnostic.layer,
        case.layer.as_diagnostic_layer(),
        "case '{}'",
        case.name
    );
    assert_eq!(diagnostic.code, case.code, "case '{}'", case.name);
    assert_eq!(
        diagnostic.primary_span,
        case.span.as_span(),
        "case '{}'",
        case.name
    );

    if let Some(snapshot_name) = case.snapshot.as_deref() {
        let snapshot_identifier = snapshot_identifier(manifest_file_name, snapshot_name);
        insta::with_settings!({
            prepend_module_to_snapshot => false,
            snapshot_path => "../snapshots",
        }, {
            insta::assert_snapshot!(snapshot_identifier.as_str(), diagnostic.render(case.source.as_str()));
        });
    }
}

pub fn negative_case_host(case: &NegativeCase) -> InMemoryExprHost {
    let profile = case
        .host_profile
        .as_deref()
        .unwrap_or_else(|| panic!("case '{}' must declare host_profile", case.name));
    if profile == "custom" {
        return InMemoryExprHost::from_fixtures(case.signals.as_slice());
    }
    host_from_profile(profile)
}

pub fn negative_case_runtime_timestamp(case: &NegativeCase) -> u64 {
    case.timestamp
        .unwrap_or_else(|| panic!("case '{}' must declare runtime timestamp", case.name))
}

pub fn run_negative_case(case: &NegativeCase) -> ExprDiagnostic {
    match case.entrypoint {
        ManifestEntrypoint::Parse => parse_event_expr_ast(case.source.as_str())
            .expect_err(&format!("{} should fail", case.name)),
        ManifestEntrypoint::Logical => {
            let host = negative_case_host(case);
            match parse_logical_expr_ast(case.source.as_str()) {
                Ok(ast) => match bind_logical_expr_ast(&ast, &host) {
                    Ok(bound) if matches!(case.layer, ManifestLayer::Runtime) => {
                        eval_logical_expr_at(&bound, &host, negative_case_runtime_timestamp(case))
                            .expect_err(&format!("{} should fail at runtime", case.name))
                    }
                    Ok(_) => panic!("{} should fail", case.name),
                    Err(diagnostic) => diagnostic,
                },
                Err(diagnostic) => diagnostic,
            }
        }
        ManifestEntrypoint::Event => {
            let host = negative_case_host(case);
            match parse_event_expr_ast(case.source.as_str()) {
                Ok(ast) => bind_event_expr_ast(&ast, &host)
                    .expect_err(&format!("{} should fail", case.name)),
                Err(diagnostic) => diagnostic,
            }
        }
    }
}

impl SpanRecord {
    pub fn as_span(&self) -> Span {
        Span {
            start: self.start,
            end: self.end,
        }
    }
}

impl ManifestLayer {
    pub fn as_diagnostic_layer(self) -> DiagnosticLayer {
        match self {
            Self::Parse => DiagnosticLayer::Parse,
            Self::Semantic => DiagnosticLayer::Semantic,
            Self::Runtime => DiagnosticLayer::Runtime,
        }
    }
}

fn validate_positive_manifest(manifest: PositiveManifest) -> Result<PositiveManifest, String> {
    for case in &manifest.cases {
        match case {
            PositiveCase::LogicalEval(case) => case.expected_result.validate(case.name.as_str())?,
            PositiveCase::EventParse(_) | PositiveCase::EventEval(_) => {}
        }
    }
    Ok(manifest)
}

fn validate_negative_manifest(manifest: NegativeManifest) -> Result<NegativeManifest, String> {
    for case in &manifest.cases {
        validate_negative_case(case)?;
    }
    Ok(manifest)
}

fn validate_negative_case(case: &NegativeCase) -> Result<(), String> {
    match case.entrypoint {
        ManifestEntrypoint::Parse => {
            if case.host_profile.is_some() || !case.signals.is_empty() || case.timestamp.is_some() {
                return Err(format!(
                    "negative case '{}' is parse-only and must not declare host context",
                    case.name
                ));
            }
        }
        ManifestEntrypoint::Logical | ManifestEntrypoint::Event => {
            let Some(profile) = case.host_profile.as_deref() else {
                return Err(format!(
                    "negative case '{}' must declare host_profile for non-parse entrypoints",
                    case.name
                ));
            };

            if profile == "custom" {
                if case.signals.is_empty() {
                    return Err(format!(
                        "negative case '{}' uses custom host_profile but provides no signals",
                        case.name
                    ));
                }
            } else {
                if !is_supported_host_profile(profile) {
                    return Err(format!(
                        "negative case '{}' references unsupported host_profile '{}'",
                        case.name, profile
                    ));
                }
                if !case.signals.is_empty() {
                    return Err(format!(
                        "negative case '{}' must not mix named host_profile '{}' with inline signals",
                        case.name, profile
                    ));
                }
            }
        }
    }

    if matches!(case.layer, ManifestLayer::Runtime) && case.timestamp.is_none() {
        return Err(format!(
            "negative case '{}' must declare timestamp for runtime diagnostics",
            case.name
        ));
    }

    if matches!(case.entrypoint, ManifestEntrypoint::Event)
        && matches!(case.layer, ManifestLayer::Runtime)
    {
        return Err(format!(
            "negative case '{}' uses unsupported event runtime diagnostics in shared manifests",
            case.name
        ));
    }

    Ok(())
}
