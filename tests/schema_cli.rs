use assert_cmd::prelude::*;
use serde_json::{Value, json};
use std::fs;
use std::path::PathBuf;

mod common;
use common::{expected_schema_url, expected_stream_schema_url, wavepeek_cmd};

const EXPECTED_SCOPE_KINDS: &[&str] = &[
    "module",
    "task",
    "function",
    "begin",
    "fork",
    "generate",
    "struct",
    "union",
    "class",
    "interface",
    "package",
    "program",
    "unknown",
];

const EXPECTED_SIGNAL_KINDS: &[&str] = &[
    "event",
    "integer",
    "parameter",
    "real",
    "reg",
    "supply0",
    "supply1",
    "time",
    "tri",
    "triand",
    "trior",
    "trireg",
    "tri0",
    "tri1",
    "wand",
    "wire",
    "wor",
    "string",
    "port",
    "sparse_array",
    "real_time",
    "real_parameter",
    "bit",
    "logic",
    "int",
    "short_int",
    "long_int",
    "byte",
    "enum",
    "short_real",
    "boolean",
    "bit_vector",
];

const EXCLUDED_SCOPE_KINDS: &[&str] = &[
    "vhdl_architecture",
    "vhdl_procedure",
    "vhdl_function",
    "vhdl_record",
    "vhdl_process",
    "vhdl_block",
    "vhdl_for_generate",
    "vhdl_if_generate",
    "vhdl_generate",
    "vhdl_package",
    "vhdl_array",
    "ghw_generic",
];

const EXCLUDED_SIGNAL_KINDS: &[&str] = &[
    "std_logic",
    "std_ulogic",
    "std_logic_vector",
    "std_ulogic_vector",
];

fn canonical_schema_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("schema")
        .join("output.json")
}

fn canonical_stream_schema_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("schema")
        .join("stream.json")
}

fn canonical_catalog_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("schema")
        .join("catalog.json")
}

fn run_schema_command() -> Vec<u8> {
    wavepeek_cmd()
        .args(["schema"])
        .output()
        .expect("schema command should execute")
        .stdout
}

fn run_stream_schema_command() -> Vec<u8> {
    wavepeek_cmd()
        .args(["schema", "--stream"])
        .output()
        .expect("schema --stream command should execute")
        .stdout
}

fn schema_json() -> Value {
    serde_json::from_slice(&run_schema_command()).expect("schema output should be valid json")
}

fn stream_schema_json() -> Value {
    serde_json::from_slice(&run_stream_schema_command())
        .expect("stream schema output should be valid json")
}

fn stream_schema_validator() -> jsonschema::Validator {
    jsonschema::validator_for(&stream_schema_json()).expect("stream schema should compile")
}

fn info_json() -> Value {
    let output = wavepeek_cmd()
        .args([
            "info",
            "--waves",
            "tests/fixtures/hand/m2_core.vcd",
            "--json",
        ])
        .output()
        .expect("info command should execute");
    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    serde_json::from_slice(&output.stdout).expect("info output should be valid json")
}

fn schema_enum(schema: &Value, def_name: &str) -> Vec<String> {
    schema["$defs"][def_name]["enum"]
        .as_array()
        .unwrap_or_else(|| panic!("schema definition {def_name} should expose enum array"))
        .iter()
        .map(|entry| {
            entry
                .as_str()
                .unwrap_or_else(|| panic!("schema definition {def_name} should use strings"))
                .to_string()
        })
        .collect()
}

#[test]
fn schema_command_prints_canonical_artifact_bytes() {
    let mut command = wavepeek_cmd();
    let assert = command.args(["schema"]).assert().success();

    let expected = fs::read(canonical_schema_path()).expect("schema file should be readable");
    assert_eq!(assert.get_output().stdout, expected);
    assert!(assert.get_output().stderr.is_empty());
}

#[test]
fn schema_command_ignores_debug_trace() {
    let mut command = wavepeek_cmd();
    let assert = command
        .env("DEBUG", "1")
        .args(["schema"])
        .assert()
        .success();

    let expected = fs::read(canonical_schema_path()).expect("schema file should be readable");
    assert_eq!(assert.get_output().stdout, expected);
    assert!(assert.get_output().stderr.is_empty());
}

#[test]
fn schema_command_output_is_valid_json() {
    let value = schema_json();

    assert_eq!(value["type"], "object");
    assert!(value["$defs"].is_object());
}

#[test]
fn schema_command_output_is_deterministic_across_runs() {
    let first = wavepeek_cmd()
        .args(["schema"])
        .output()
        .expect("first run should execute");
    let second = wavepeek_cmd()
        .args(["schema"])
        .output()
        .expect("second run should execute");

    assert!(first.status.success());
    assert!(second.status.success());
    assert_eq!(first.stdout, second.stdout);
    assert_eq!(first.stderr, second.stderr);
}

#[test]
fn schema_command_schema_url_matches_exact_family_artifact() {
    let value = schema_json();
    assert_eq!(
        value["properties"]["$schema"]["const"],
        expected_schema_url()
    );
    assert_eq!(value["$id"], expected_schema_url());
    assert!(value["properties"]["$schema"].get("pattern").is_none());
}

#[test]
fn schema_command_exposes_typed_diagnostics_contract() {
    let value = schema_json();

    assert!(
        value["required"]
            .as_array()
            .unwrap()
            .contains(&json!("diagnostics"))
    );
    assert!(
        !value["required"]
            .as_array()
            .unwrap()
            .contains(&json!("warnings"))
    );
    assert!(value["properties"].get("diagnostics").is_some());
    assert!(value["properties"].get("warnings").is_none());
    assert_eq!(
        value["properties"]["diagnostics"]["items"]["$ref"],
        "#/$defs/diagnostic"
    );
    assert!(
        value["properties"]["data"].get("anyOf").is_some(),
        "root data must allow ambiguous empty arrays and rely on command branches"
    );
    assert!(value["properties"]["data"].get("oneOf").is_none());

    let diagnostic = &value["$defs"]["diagnostic"];
    assert_eq!(diagnostic["type"], "object");
    assert_ne!(diagnostic["additionalProperties"], false);
    assert_eq!(diagnostic["required"], json!(["kind", "message"]));
    let diagnostic_property_keys = diagnostic["properties"]
        .as_object()
        .expect("diagnostic properties should be an object")
        .keys()
        .cloned()
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(
        diagnostic_property_keys,
        [
            "code".to_string(),
            "kind".to_string(),
            "message".to_string()
        ]
        .into_iter()
        .collect::<std::collections::BTreeSet<_>>()
    );
    assert_eq!(
        diagnostic["properties"]["kind"]["enum"],
        json!(["info", "warning", "error"])
    );
    assert_eq!(
        diagnostic["properties"]["code"]["pattern"],
        "^WPK-[WE][0-9]{4}$"
    );
    assert_eq!(diagnostic["properties"]["message"]["type"], "string");

    let rules = diagnostic["allOf"]
        .as_array()
        .expect("diagnostic rules should be array");
    assert!(rules.iter().any(|rule| {
        rule["if"]["properties"]["kind"]["const"] == "warning"
            && rule["then"]["required"] == json!(["code"])
            && rule["then"]["properties"]["code"]["pattern"] == "^WPK-W[0-9]{4}$"
    }));
    assert!(rules.iter().any(|rule| {
        rule["if"]["properties"]["kind"]["const"] == "error"
            && rule["then"]["required"] == json!(["code"])
            && rule["then"]["properties"]["code"]["pattern"] == "^WPK-E[0-9]{4}$"
    }));
    assert!(rules.iter().any(|rule| {
        rule["if"]["properties"]["kind"]["const"] == "info"
            && rule["then"]["not"] == json!({"required": ["code"]})
    }));

    let code_pattern = diagnostic["properties"]["code"]["pattern"]
        .as_str()
        .expect("diagnostic code pattern should be a string");
    let regex = regex::Regex::new(code_pattern).expect("diagnostic code pattern should compile");
    assert!(regex.is_match("WPK-W0001"));
    assert!(regex.is_match("WPK-E0001"));
    assert!(!regex.is_match("WPK-I0001"));
}

#[test]
fn runtime_info_envelope_uses_diagnostics_not_warnings() {
    let value = info_json();

    assert_eq!(value["$schema"], expected_schema_url());
    assert_eq!(value["command"], "info");
    assert_eq!(value["diagnostics"], json!([]));
    assert!(value.get("warnings").is_none());
}

#[test]
fn schema_validator_accepts_extension_fields() {
    let schema = schema_json();
    let validator = jsonschema::validator_for(&schema).expect("schema should compile");
    let envelope = json!({
        "$schema": expected_schema_url(),
        "command": "info",
        "data": {
            "time_unit": "1ns",
            "time_start": "0ns",
            "time_end": "10ns",
            "x_client_note": "accepted extension"
        },
        "diagnostics": [{
            "kind": "warning",
            "code": "WPK-W0001",
            "message": "warning text",
            "x_trace_id": "abc123"
        }],
        "x_envelope": true
    });

    validator.validate(&envelope).unwrap_or_else(|error| {
        panic!("extension-friendly envelope rejected: {error}\n{envelope}")
    });
}

#[test]
fn schema_command_includes_property_command_branch() {
    let value = schema_json();

    let commands = value["properties"]["command"]["enum"]
        .as_array()
        .expect("command enum should be array");
    assert!(
        commands.iter().any(|entry| entry == "property"),
        "schema command enum should include property"
    );

    let data_variants = value["properties"]["data"]["anyOf"]
        .as_array()
        .expect("data variants should be array");
    assert!(
        data_variants
            .iter()
            .any(|entry| entry["$ref"] == "#/$defs/propertyData"),
        "schema data variants should include propertyData"
    );
}

#[test]
fn schema_command_includes_docs_command_branches() {
    let value = schema_json();

    let commands = value["properties"]["command"]["enum"]
        .as_array()
        .expect("command enum should be array");
    assert!(
        commands.iter().any(|entry| entry == "docs topics"),
        "schema command enum should include docs topics"
    );
    assert!(
        commands.iter().any(|entry| entry == "docs search"),
        "schema command enum should include docs search"
    );

    let data_variants = value["properties"]["data"]["anyOf"]
        .as_array()
        .expect("data variants should be array");
    assert!(
        data_variants
            .iter()
            .any(|entry| entry["$ref"] == "#/$defs/docsTopicsData"),
        "schema data variants should include docsTopicsData"
    );
    assert!(
        data_variants
            .iter()
            .any(|entry| entry["$ref"] == "#/$defs/docsSearchData"),
        "schema data variants should include docsSearchData"
    );
    assert_eq!(
        value["$defs"]["docsSearchMatch"]["properties"]["matched_tokens"]["minimum"], 1,
        "docs search matched_tokens should match the runtime minimum"
    );
}

#[test]
fn schema_command_hardens_scope_and_signal_kind_inventories() {
    let value = schema_json();

    assert_eq!(
        schema_enum(&value, "scopeKind"),
        EXPECTED_SCOPE_KINDS
            .iter()
            .map(|entry| entry.to_string())
            .collect::<Vec<_>>()
    );
    assert_eq!(
        value["$defs"]["scopeEntry"]["properties"]["kind"]["$ref"],
        "#/$defs/scopeKind"
    );

    assert_eq!(
        schema_enum(&value, "signalKind"),
        EXPECTED_SIGNAL_KINDS
            .iter()
            .map(|entry| entry.to_string())
            .collect::<Vec<_>>()
    );
    assert_eq!(
        value["$defs"]["signalEntry"]["properties"]["kind"]["$ref"],
        "#/$defs/signalKind"
    );
}

#[test]
fn schema_command_excludes_backend_specific_kind_aliases() {
    let value = schema_json();
    let scope_kinds = schema_enum(&value, "scopeKind");
    let signal_kinds = schema_enum(&value, "signalKind");

    for alias in EXCLUDED_SCOPE_KINDS {
        assert!(
            !scope_kinds.iter().any(|entry| entry == alias),
            "excluded scope alias {alias:?} leaked into schema"
        );
    }
    for alias in EXCLUDED_SIGNAL_KINDS {
        assert!(
            !signal_kinds.iter().any(|entry| entry == alias),
            "excluded signal alias {alias:?} leaked into schema"
        );
    }
}

#[test]
fn schema_command_exposes_value_data_as_snapshot_array() {
    let value = schema_json();

    assert_eq!(value["$defs"]["valueData"]["type"], "array");
    assert_eq!(
        value["$defs"]["valueData"]["items"]["$ref"],
        "#/$defs/valueSnapshot"
    );
    assert_eq!(value["$defs"]["changeData"]["type"], "array");
    assert_eq!(
        value["$defs"]["changeData"]["items"]["$ref"],
        "#/$defs/changeSnapshot"
    );
}

#[test]
fn schema_command_exposes_field_descriptions_for_machine_clients() {
    let value = schema_json();

    for path in [
        &["$defs", "scopeEntry", "properties", "path", "description"][..],
        &["$defs", "scopeEntry", "properties", "kind", "description"],
        &["$defs", "signalEntry", "properties", "kind", "description"],
        &["$defs", "valueData", "description"],
        &[
            "$defs",
            "changeSnapshot",
            "properties",
            "time",
            "description",
        ],
        &[
            "$defs",
            "changeSnapshot",
            "properties",
            "sample_time",
            "description",
        ],
        &[
            "$defs",
            "changeSignalValue",
            "properties",
            "value",
            "description",
        ],
        &["$defs", "propertyRow", "properties", "kind", "description"],
        &[
            "$defs",
            "propertyRow",
            "properties",
            "sample_time",
            "description",
        ],
        &[
            "$defs",
            "topicSummary",
            "properties",
            "description",
            "description",
        ],
        &[
            "$defs",
            "docsSearchData",
            "properties",
            "query",
            "description",
        ],
    ] {
        let mut cursor = &value;
        for segment in path {
            cursor = &cursor[*segment];
        }
        let description = cursor
            .as_str()
            .expect("description field should be a string");
        assert!(
            !description.trim().is_empty(),
            "description at {path:?} should not be empty"
        );
    }
}

#[test]
fn schema_stream_command_prints_canonical_artifact_bytes() {
    let mut command = wavepeek_cmd();
    let assert = command.args(["schema", "--stream"]).assert().success();

    let expected =
        fs::read(canonical_stream_schema_path()).expect("stream schema file should read");
    assert_eq!(assert.get_output().stdout, expected);
    assert!(assert.get_output().stderr.is_empty());
}

#[test]
fn schema_catalog_points_to_current_family_snapshots() {
    let catalog: Value = serde_json::from_slice(
        &fs::read(canonical_catalog_path()).expect("schema catalog should be readable"),
    )
    .expect("schema catalog should parse");
    let families = catalog["families"]
        .as_array()
        .expect("families should be array");
    assert_eq!(families.len(), 2);
    assert!(families.iter().any(|entry| {
        entry["id"] == "wavepeek.output"
            && entry["version"] == "2.0"
            && entry["path"] == "schema/output.json"
            && entry["url"] == expected_schema_url()
    }));
    assert!(families.iter().any(|entry| {
        entry["id"] == "wavepeek.stream-record"
            && entry["version"] == "2.0"
            && entry["path"] == "schema/stream.json"
            && entry["url"] == expected_stream_schema_url()
    }));
}

#[test]
fn schema_stream_command_output_is_valid_json() {
    let value = stream_schema_json();

    assert_eq!(
        value["$schema"],
        "https://json-schema.org/draft/2020-12/schema"
    );
    assert_eq!(value["title"], "wavepeek JSONL stream record");
    assert!(value["oneOf"].is_array());
    assert!(value["$defs"].is_object());
}

#[test]
fn schema_stream_command_exposes_waveform_command_contract() {
    let value = stream_schema_json();

    assert_eq!(
        value["$defs"]["streamCommand"]["enum"],
        json!(["info", "scope", "signal", "value", "change", "property"])
    );
    let root_refs = value["oneOf"]
        .as_array()
        .expect("stream schema root should use oneOf")
        .iter()
        .map(|entry| {
            entry["$ref"]
                .as_str()
                .expect("root variant should be a ref")
        })
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(
        root_refs,
        [
            "#/$defs/beginRecord",
            "#/$defs/itemRecord",
            "#/$defs/diagnosticRecord",
            "#/$defs/endRecord",
        ]
        .into_iter()
        .collect::<std::collections::BTreeSet<_>>()
    );
    assert_eq!(
        value["$defs"]["beginRecord"]["properties"]["$schema"]["const"],
        expected_stream_schema_url()
    );
    assert_eq!(value["$id"], expected_stream_schema_url());
    assert!(
        value["$defs"]["beginRecord"]["properties"]["$schema"]
            .get("pattern")
            .is_none()
    );
}

#[test]
fn schema_stream_validator_accepts_representative_waveform_records() {
    let validator = stream_schema_validator();
    let cases = [
        json!({
            "type": "begin",
            "seq": 0,
            "command": "change",
            "$schema": expected_stream_schema_url()
        }),
        json!({
            "type": "item",
            "seq": 1,
            "command": "info",
            "item": {"time_unit": "1ns", "time_start": "0ns", "time_end": "10ns"}
        }),
        json!({
            "type": "item",
            "seq": 1,
            "command": "scope",
            "item": {"path": "top", "depth": 0, "kind": "module"}
        }),
        json!({
            "type": "item",
            "seq": 1,
            "command": "signal",
            "item": {"name": "clk", "path": "top.clk", "kind": "wire", "width": 1}
        }),
        json!({
            "type": "item",
            "seq": 1,
            "command": "value",
            "item": {"time": "5ns", "signals": [{"path": "top.clk", "value": "1'h1"}]}
        }),
        json!({
            "type": "item",
            "seq": 1,
            "command": "change",
            "item": {"time": "5ns", "sample_time": "5ns", "signals": [{"path": "top.clk", "value": "1'h1"}]}
        }),
        json!({
            "type": "item",
            "seq": 1,
            "command": "property",
            "item": {"time": "5ns", "sample_time": "5ns", "kind": "assert"}
        }),
        json!({
            "type": "diagnostic",
            "seq": 2,
            "command": "change",
            "diagnostic": {"kind": "warning", "code": "WPK-W0002", "message": "truncated"}
        }),
        json!({
            "type": "end",
            "seq": 3,
            "command": "change",
            "summary": {"status": "ok", "items": 1, "diagnostics": 1, "truncated": true}
        }),
    ];

    for case in cases {
        validator
            .validate(&case)
            .unwrap_or_else(|error| panic!("valid stream record rejected: {error}\n{case}"));
    }
}

#[test]
fn schema_stream_validator_accepts_extension_fields() {
    let validator = stream_schema_validator();
    let record = json!({
        "type": "item",
        "seq": 1,
        "command": "change",
        "item": {
            "time": "5ns",
            "sample_time": "5ns",
            "signals": [{"path": "top.clk", "value": "1'h1", "x_signal": true}],
            "x_row": "accepted extension"
        },
        "x_record": true
    });

    validator.validate(&record).unwrap_or_else(|error| {
        panic!("extension-friendly stream record rejected: {error}\n{record}")
    });
}

#[test]
fn schema_stream_validator_rejects_command_payload_mismatches() {
    let validator = stream_schema_validator();
    let invalid_cases = [
        json!({
            "type": "item",
            "seq": 1,
            "command": "property",
            "item": {"time": "5ns", "signals": [{"path": "top.clk", "value": "1'h1"}]}
        }),
        json!({
            "type": "item",
            "seq": 1,
            "command": "change",
            "item": {"time": "5ns", "kind": "assert"}
        }),
        json!({
            "type": "item",
            "seq": 1,
            "command": "docs topics",
            "item": {}
        }),
    ];

    for case in invalid_cases {
        assert!(
            validator.validate(&case).is_err(),
            "invalid stream record should fail validation: {case}"
        );
    }
}
