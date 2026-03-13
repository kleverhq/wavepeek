#![allow(dead_code)]

use std::fs;
use std::path::PathBuf;

use serde::Deserialize;
use serde::de::DeserializeOwned;
use wavepeek::expr::DiagnosticLayer;

#[derive(Debug, Deserialize)]
pub struct SpanRecord {
    pub start: usize,
    pub end: usize,
}

pub fn fixture_expr_path(file_name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("expr")
        .join(file_name)
}

pub fn load_expr_manifest<T: DeserializeOwned>(file_name: &str) -> T {
    let path = fixture_expr_path(file_name);
    let payload = fs::read_to_string(path)
        .unwrap_or_else(|error| panic!("manifest '{file_name}' should be readable: {error}"));
    serde_json::from_str(&payload)
        .unwrap_or_else(|error| panic!("manifest '{file_name}' should be valid JSON: {error}"))
}

pub fn expected_layer(raw: &str) -> DiagnosticLayer {
    match raw {
        "parse" => DiagnosticLayer::Parse,
        "semantic" => DiagnosticLayer::Semantic,
        "runtime" => DiagnosticLayer::Runtime,
        other => panic!("unsupported manifest layer '{other}'"),
    }
}
