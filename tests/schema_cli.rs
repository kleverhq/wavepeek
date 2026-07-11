use assert_cmd::prelude::*;
use serde_json::{Value, json};
use std::fs;
use std::path::PathBuf;

mod common;
use common::{
    expected_input_schema_url, expected_schema_url, expected_stream_schema_url, wavepeek_cmd,
};

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

fn canonical_input_schema_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("schema")
        .join("input.json")
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

fn run_input_schema_command() -> Vec<u8> {
    wavepeek_cmd()
        .args(["schema", "--input"])
        .output()
        .expect("schema --input command should execute")
        .stdout
}

fn schema_json() -> Value {
    serde_json::from_slice(&run_schema_command()).expect("schema output should be valid json")
}

fn stream_schema_json() -> Value {
    serde_json::from_slice(&run_stream_schema_command())
        .expect("stream schema output should be valid json")
}

fn input_schema_json() -> Value {
    serde_json::from_slice(&run_input_schema_command())
        .expect("input schema output should be valid json")
}

fn output_schema_validator() -> jsonschema::Validator {
    let schema: Value = serde_json::from_slice(
        &fs::read(canonical_schema_path()).expect("schema file should be readable"),
    )
    .expect("schema file should parse");
    jsonschema::validator_for(&schema).expect("schema should compile")
}

fn stream_schema_validator() -> jsonschema::Validator {
    jsonschema::validator_for(&stream_schema_json()).expect("stream schema should compile")
}

fn input_schema_validator() -> jsonschema::Validator {
    jsonschema::validator_for(&input_schema_json()).expect("input schema should compile")
}

fn run_json_command(args: &[&str], expected_command: &str) -> Value {
    let output = wavepeek_cmd()
        .args(args)
        .output()
        .expect("json command should execute");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "json command {args:?} failed with status {}\nstdout:\n{stdout}\nstderr:\n{stderr}",
        output.status
    );
    assert!(
        output.stderr.is_empty(),
        "json command {args:?} wrote stderr\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    let value: Value = serde_json::from_slice(&output.stdout).unwrap_or_else(|error| {
        panic!("json command {args:?} stdout should be valid json: {error}\nstdout:\n{stdout}")
    });
    assert_eq!(
        value["$schema"],
        expected_schema_url(),
        "json command {args:?} emitted unexpected schema URL: {value}"
    );
    assert_eq!(
        value["command"], expected_command,
        "json command {args:?} emitted unexpected command: {value}"
    );

    let validator = output_schema_validator();
    validator.validate(&value).unwrap_or_else(|error| {
        panic!("json command {args:?} output should match schema: {error}\n{value}")
    });
    value
}

fn info_json() -> Value {
    run_json_command(
        &[
            "info",
            "--waves",
            "tests/fixtures/generated/m2_core.vcd",
            "--json",
        ],
        "info",
    )
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
fn runtime_metadata_json_outputs_validate_against_schema() {
    let fixture = "tests/fixtures/generated/m2_core.vcd";

    let info = run_json_command(&["info", "--waves", fixture, "--json"], "info");
    assert_eq!(info["data"]["time_unit"], "1ns");

    let scope = run_json_command(&["scope", "--waves", fixture, "--json"], "scope");
    assert!(
        scope["data"]
            .as_array()
            .is_some_and(|entries| { entries.iter().any(|entry| entry["path"] == "top") })
    );

    let signal = run_json_command(
        &["signal", "--waves", fixture, "--scope", "top", "--json"],
        "signal",
    );
    assert!(
        signal["data"]
            .as_array()
            .is_some_and(|entries| { entries.iter().any(|entry| entry["path"] == "top.clk") })
    );
}

#[test]
fn runtime_waveform_data_json_outputs_validate_against_schema() {
    let fixture = "tests/fixtures/generated/m2_core.vcd";

    let value = run_json_command(
        &[
            "value",
            "--waves",
            fixture,
            "--at",
            "5ns",
            "--signals",
            "top.clk,top.data",
            "--json",
        ],
        "value",
    );
    assert_eq!(value["data"][0]["time"], "5ns");
    assert_eq!(value["data"][0]["signals"].as_array().unwrap().len(), 2);

    let change = run_json_command(
        &[
            "change",
            "--waves",
            fixture,
            "--from",
            "1ns",
            "--to",
            "10ns",
            "--signals",
            "top.clk,top.data",
            "--on",
            "*",
            "--sample-mode",
            "native",
            "--max",
            "2",
            "--json",
        ],
        "change",
    );
    assert_eq!(change["data"].as_array().unwrap().len(), 2);
    assert_eq!(change["data"][0]["sample_time"], "5ns");

    let property = run_json_command(
        &[
            "property",
            "--waves",
            fixture,
            "--scope",
            "top",
            "--on",
            "*",
            "--sample-mode",
            "native",
            "--eval",
            "clk",
            "--capture",
            "switch",
            "--max",
            "2",
            "--json",
        ],
        "property",
    );
    assert_eq!(property["data"][0]["kind"], "assert");

    let extract = run_json_command(
        &[
            "extract",
            "generic",
            "--waves",
            fixture,
            "--scope",
            "top",
            "--on",
            "posedge clk",
            "--when",
            "1",
            "--payload",
            "data",
            "--max",
            "1",
            "--json",
        ],
        "extract generic",
    );
    assert_eq!(extract["data"][0]["source"], "transfer");
}

#[test]
fn runtime_docs_json_outputs_validate_against_schema() {
    let topics = run_json_command(&["docs", "topics", "--json"], "docs topics");
    assert!(
        topics["data"]["topics"]
            .as_array()
            .is_some_and(|entries| { entries.iter().any(|entry| entry["id"] == "intro") })
    );

    let search = run_json_command(&["docs", "search", "schema", "--json"], "docs search");
    assert_eq!(search["data"]["query"], "schema");
    assert!(search["data"]["matches"].as_array().is_some_and(|matches| {
        matches
            .iter()
            .any(|entry| entry["topic"]["id"] == "commands/schema")
    }));
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
fn schema_command_includes_property_and_extract_command_branches() {
    let value = schema_json();

    let commands = value["properties"]["command"]["enum"]
        .as_array()
        .expect("command enum should be array");
    assert!(
        commands.iter().any(|entry| entry == "property"),
        "schema command enum should include property"
    );
    assert!(
        commands.iter().any(|entry| entry == "extract axi"),
        "schema command enum should include extract axi"
    );
    assert!(
        commands.iter().any(|entry| entry == "extract generic"),
        "schema command enum should include extract generic"
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
    assert!(
        data_variants
            .iter()
            .any(|entry| entry["$ref"] == "#/$defs/extractAxiData"),
        "schema data variants should include extractAxiData"
    );
    assert!(
        data_variants
            .iter()
            .any(|entry| entry["$ref"] == "#/$defs/extractGenericData"),
        "schema data variants should include extractGenericData"
    );
}

#[test]
fn schema_output_validator_enforces_axi_profile_channel_payloads() {
    let validator = output_schema_validator();
    let axi4_lite_mappings = json!({
        "aclk": {"path": "top.clk"},
        "awvalid": {"path": "top.awvalid"},
        "awready": {"path": "top.awready"},
        "awaddr": {"path": "top.awaddr"},
        "awprot": {"path": "top.awprot"}
    });
    let axi3_mappings = json!({
        "aclk": {"path": "top.clk"},
        "wvalid": {"path": "top.wvalid"},
        "wready": {"path": "top.wready"},
        "wdata": {"path": "top.wdata"},
        "wid": {"path": "top.wid"}
    });
    let axi4_mappings = json!({
        "aclk": {"path": "top.clk"},
        "wvalid": {"path": "top.wvalid"},
        "wready": {"path": "top.wready"},
        "wdata": {"path": "top.wdata"}
    });
    let ace_mappings = json!({
        "aclk": {"path": "top.clk"},
        "acvalid": {"path": "top.acvalid"},
        "acready": {"path": "top.acready"},
        "acaddr": {"path": "top.acaddr"}
    });
    let ace_lite_mappings = json!({
        "aclk": {"path": "top.clk"},
        "awvalid": {"path": "top.awvalid"},
        "awready": {"path": "top.awready"},
        "awunique": {"path": "top.awunique"}
    });
    let ace5_mappings = json!({
        "aclk": {"path": "top.clk"},
        "cdvalid": {"path": "top.cdvalid"},
        "cdready": {"path": "top.cdready"},
        "cdpoison": {"path": "top.cdpoison"}
    });

    let valid_axi4_lite_aw = json!({
        "$schema": expected_schema_url(),
        "command": "extract axi",
        "data": {
            "name": "axi",
            "profile": "axi4-lite",
            "issue": "H.c",
            "mappings": axi4_lite_mappings,
            "transfers": [{
                "time": "5ns",
                "sample_time": "4ns",
                "profile": "axi4-lite",
                "channel": "aw",
                "payload": {"awaddr": "32'h40", "awprot": "3'h0"}
            }]
        },
        "diagnostics": []
    });
    validator
        .validate(&valid_axi4_lite_aw)
        .unwrap_or_else(|error| {
            panic!("valid AXI4-Lite AW output rejected: {error}\n{valid_axi4_lite_aw}")
        });

    let mut invalid_axi4_lite_aw = valid_axi4_lite_aw.clone();
    invalid_axi4_lite_aw["data"]["transfers"][0]["payload"]["awlen"] = json!("8'h0");
    assert!(
        validator.validate(&invalid_axi4_lite_aw).is_err(),
        "AXI4-Lite AW output must reject AXI4-only awlen payload: {invalid_axi4_lite_aw}"
    );

    let valid_axi3_w = json!({
        "$schema": expected_schema_url(),
        "command": "extract axi",
        "data": {
            "name": "axi",
            "profile": "axi3",
            "issue": "H.c",
            "mappings": axi3_mappings,
            "transfers": [{
                "time": "5ns",
                "sample_time": "4ns",
                "profile": "axi3",
                "channel": "w",
                "payload": {"wid": "4'h1", "wdata": "8'haa"}
            }]
        },
        "diagnostics": []
    });
    validator
        .validate(&valid_axi3_w)
        .unwrap_or_else(|error| panic!("valid AXI3 W output rejected: {error}\n{valid_axi3_w}"));

    let invalid_axi4_w = json!({
        "$schema": expected_schema_url(),
        "command": "extract axi",
        "data": {
            "name": "axi",
            "profile": "axi4",
            "issue": "H.c",
            "mappings": axi4_mappings,
            "transfers": [{
                "time": "5ns",
                "sample_time": "4ns",
                "profile": "axi4",
                "channel": "w",
                "payload": {"wid": "4'h1", "wdata": "8'haa"}
            }]
        },
        "diagnostics": []
    });
    assert!(
        validator.validate(&invalid_axi4_w).is_err(),
        "AXI4 W output must reject AXI3-only wid payload: {invalid_axi4_w}"
    );

    let valid_ace_ac = json!({
        "$schema": expected_schema_url(),
        "command": "extract axi",
        "data": {
            "name": "ace",
            "profile": "ace",
            "issue": "H.c",
            "mappings": ace_mappings,
            "transfers": [{
                "time": "5ns",
                "sample_time": "4ns",
                "profile": "ace",
                "channel": "ac",
                "payload": {"acaddr": "40'h1234"}
            }]
        },
        "diagnostics": []
    });
    validator
        .validate(&valid_ace_ac)
        .unwrap_or_else(|error| panic!("valid ACE AC output rejected: {error}\n{valid_ace_ac}"));

    let valid_ace_lite_awunique = json!({
        "$schema": expected_schema_url(),
        "command": "extract axi",
        "data": {
            "name": "ace-lite",
            "profile": "ace-lite",
            "issue": "H.c",
            "mappings": ace_lite_mappings,
            "transfers": [{
                "time": "5ns",
                "sample_time": "4ns",
                "profile": "ace-lite",
                "channel": "aw",
                "payload": {"awunique": "1'h1"}
            }]
        },
        "diagnostics": []
    });
    validator
        .validate(&valid_ace_lite_awunique)
        .unwrap_or_else(|error| {
            panic!("valid ACE-Lite AW output rejected: {error}\n{valid_ace_lite_awunique}")
        });

    let mut valid_ace_lite_empty_aw = valid_ace_lite_awunique.clone();
    valid_ace_lite_empty_aw["data"]["transfers"][0]["payload"] = json!({});
    validator
        .validate(&valid_ace_lite_empty_aw)
        .unwrap_or_else(|error| {
            panic!("valid empty ACE-Lite AW payload rejected: {error}\n{valid_ace_lite_empty_aw}")
        });

    let mut invalid_ace_lite_channel = valid_ace_lite_empty_aw.clone();
    invalid_ace_lite_channel["data"]["transfers"][0]["channel"] = json!("ac");
    assert!(
        validator.validate(&invalid_ace_lite_channel).is_err(),
        "ACE-Lite output must reject AC channels: {invalid_ace_lite_channel}"
    );

    let mut invalid_ace_lite_payload = valid_ace_lite_awunique.clone();
    invalid_ace_lite_payload["data"]["transfers"][0]["payload"] = json!({"acaddr": "40'h1234"});
    assert!(
        validator.validate(&invalid_ace_lite_payload).is_err(),
        "ACE-Lite AW output must reject AC payload keys: {invalid_ace_lite_payload}"
    );

    let mut invalid_ace_lite_mapping = valid_ace_lite_awunique.clone();
    invalid_ace_lite_mapping["data"]["mappings"]["acvalid"] = json!({"path": "top.acvalid"});
    assert!(
        validator.validate(&invalid_ace_lite_mapping).is_err(),
        "ACE-Lite output must reject AC mappings: {invalid_ace_lite_mapping}"
    );

    let mut invalid_ace_lite_transfer_profile = valid_ace_lite_awunique.clone();
    invalid_ace_lite_transfer_profile["data"]["transfers"][0]["profile"] = json!("ace");
    assert!(
        validator
            .validate(&invalid_ace_lite_transfer_profile)
            .is_err(),
        "ACE-Lite data must reject ACE transfer rows: {invalid_ace_lite_transfer_profile}"
    );

    let valid_ace5_cd = json!({
        "$schema": expected_schema_url(),
        "command": "extract axi",
        "data": {
            "name": "ace5",
            "profile": "ace5",
            "issue": "H.c",
            "mappings": ace5_mappings,
            "transfers": [{
                "time": "5ns",
                "sample_time": "4ns",
                "profile": "ace5",
                "channel": "cd",
                "payload": {"cdpoison": "1'h1"}
            }]
        },
        "diagnostics": []
    });
    validator
        .validate(&valid_ace5_cd)
        .unwrap_or_else(|error| panic!("valid ACE5 CD output rejected: {error}\n{valid_ace5_cd}"));

    let mut valid_ace5_empty_cd = valid_ace5_cd.clone();
    valid_ace5_empty_cd["data"]["transfers"][0]["payload"] = json!({});
    validator
        .validate(&valid_ace5_empty_cd)
        .unwrap_or_else(|error| {
            panic!("valid empty ACE5 CD payload rejected: {error}\n{valid_ace5_empty_cd}")
        });

    let mut invalid_ace5_channel = valid_ace5_empty_cd.clone();
    invalid_ace5_channel["data"]["transfers"][0]["channel"] = json!("snp");
    assert!(
        validator.validate(&invalid_ace5_channel).is_err(),
        "ACE5 output must reject unknown channels: {invalid_ace5_channel}"
    );

    let mut invalid_ace5_transfer_profile = valid_ace5_cd.clone();
    invalid_ace5_transfer_profile["data"]["transfers"][0]["profile"] = json!("ace");
    assert!(
        validator.validate(&invalid_ace5_transfer_profile).is_err(),
        "ACE5 data must reject ACE transfer rows: {invalid_ace5_transfer_profile}"
    );

    for barrier in ["awbar", "arbar"] {
        let mut invalid_mapping = valid_ace5_cd.clone();
        invalid_mapping["data"]["mappings"][barrier] = json!({"path": format!("top.{barrier}")});
        assert!(
            validator.validate(&invalid_mapping).is_err(),
            "ACE5 output must reject removed {barrier} mapping: {invalid_mapping}"
        );
    }

    for (channel, barrier) in [("aw", "awbar"), ("ar", "arbar")] {
        let mut valid_empty_channel = valid_ace5_empty_cd.clone();
        valid_empty_channel["data"]["transfers"][0]["channel"] = json!(channel);
        validator
            .validate(&valid_empty_channel)
            .unwrap_or_else(|error| {
                panic!(
                    "valid empty ACE5 {channel} payload rejected: {error}\n{valid_empty_channel}"
                )
            });

        let mut invalid_payload = valid_empty_channel;
        invalid_payload["data"]["transfers"][0]["payload"][barrier] = json!("2'h1");
        assert!(
            validator.validate(&invalid_payload).is_err(),
            "ACE5 {channel} output must reject removed {barrier} payload: {invalid_payload}"
        );
    }
}

#[test]
fn schema_output_validator_enforces_axi5_profile_channels_and_payloads() {
    let validator = output_schema_validator();
    let valid_axi5_ac = json!({
        "$schema": expected_schema_url(),
        "command": "extract axi",
        "data": {
            "name": "axi5",
            "profile": "axi5",
            "issue": "L",
            "mappings": {
                "aclk": {"path": "top.clk"},
                "acvalid": {"path": "top.acvalid"},
                "acready": {"path": "top.acready"},
                "acaddr": {"path": "top.acaddr"}
            },
            "transfers": [{
                "time": "30ns",
                "sample_time": "29ns",
                "profile": "axi5",
                "channel": "ac",
                "payload": {"acaddr": "32'h12345678", "acvmidext": "4'h9"}
            }]
        },
        "diagnostics": []
    });
    validator
        .validate(&valid_axi5_ac)
        .unwrap_or_else(|error| panic!("valid AXI5 AC output rejected: {error}\n{valid_axi5_ac}"));

    let mut invalid_axi5_issue = valid_axi5_ac.clone();
    invalid_axi5_issue["data"]["issue"] = json!("H.c");
    assert!(
        validator.validate(&invalid_axi5_issue).is_err(),
        "AXI5 output must reject Issue H.c metadata: {invalid_axi5_issue}"
    );

    let mut invalid_axi5_channel = valid_axi5_ac.clone();
    invalid_axi5_channel["data"]["transfers"][0]["channel"] = json!("cd");
    assert!(
        validator.validate(&invalid_axi5_channel).is_err(),
        "AXI5 output must reject CD channels: {invalid_axi5_channel}"
    );

    let mut invalid_axi5_transfer_profile = valid_axi5_ac.clone();
    invalid_axi5_transfer_profile["data"]["transfers"][0]["profile"] = json!("ace5");
    assert!(
        validator.validate(&invalid_axi5_transfer_profile).is_err(),
        "AXI5 data must reject ACE5 transfer rows: {invalid_axi5_transfer_profile}"
    );

    let mut invalid_axi5_payload = valid_axi5_ac.clone();
    invalid_axi5_payload["data"]["transfers"][0]["payload"]["acsnoop"] = json!("4'h0");
    assert!(
        validator.validate(&invalid_axi5_payload).is_err(),
        "AXI5 AC output must reject ACE-only ACSNOOP: {invalid_axi5_payload}"
    );

    let mut invalid_axi5_mapping = valid_axi5_ac.clone();
    invalid_axi5_mapping["data"]["mappings"]["awpending"] = json!({"path": "top.awpending"});
    assert!(
        validator.validate(&invalid_axi5_mapping).is_err(),
        "AXI5 output must reject credited-transport mappings: {invalid_axi5_mapping}"
    );

    for standard in ["acsnoop", "cdvalid"] {
        let mut invalid_mapping = valid_axi5_ac.clone();
        invalid_mapping["data"]["mappings"][standard] = json!({"path": format!("top.{standard}")});
        assert!(
            validator.validate(&invalid_mapping).is_err(),
            "AXI5 output must reject ACE-only {standard}: {invalid_mapping}"
        );
    }

    let valid_axi5_lite_w = json!({
        "$schema": expected_schema_url(),
        "command": "extract axi",
        "data": {
            "name": "axi5-lite",
            "profile": "axi5-lite",
            "issue": "L",
            "mappings": {
                "aclk": {"path": "top.clk"},
                "wvalid": {"path": "top.wvalid"},
                "wready": {"path": "top.wready"},
                "wdata": {"path": "top.wdata"},
                "wpoison": {"path": "top.wpoison"}
            },
            "transfers": [{
                "time": "10ns",
                "sample_time": "9ns",
                "profile": "axi5-lite",
                "channel": "w",
                "payload": {"wdata": "8'ha5", "wpoison": "1'h1"}
            }]
        },
        "diagnostics": []
    });
    validator
        .validate(&valid_axi5_lite_w)
        .unwrap_or_else(|error| {
            panic!("valid AXI5-Lite W output rejected: {error}\n{valid_axi5_lite_w}")
        });

    let mut invalid_axi5_lite_transfer_profile = valid_axi5_lite_w.clone();
    invalid_axi5_lite_transfer_profile["data"]["transfers"][0]["profile"] = json!("axi5");
    assert!(
        validator
            .validate(&invalid_axi5_lite_transfer_profile)
            .is_err(),
        "AXI5-Lite data must reject AXI5 transfer rows: {invalid_axi5_lite_transfer_profile}"
    );

    let mut invalid_axi5_lite_payload = valid_axi5_lite_w.clone();
    invalid_axi5_lite_payload["data"]["transfers"][0]["payload"]["wlast"] = json!("1'h1");
    assert!(
        validator.validate(&invalid_axi5_lite_payload).is_err(),
        "AXI5-Lite W output must reject WLAST: {invalid_axi5_lite_payload}"
    );

    let mut invalid_axi5_lite_channel = valid_axi5_lite_w.clone();
    invalid_axi5_lite_channel["data"]["transfers"][0]["channel"] = json!("ac");
    assert!(
        validator.validate(&invalid_axi5_lite_channel).is_err(),
        "AXI5-Lite output must reject AC channels: {invalid_axi5_lite_channel}"
    );

    let mut invalid_axi5_lite_mapping = valid_axi5_lite_w.clone();
    invalid_axi5_lite_mapping["data"]["mappings"]["awlen"] = json!({"path": "top.awlen"});
    assert!(
        validator.validate(&invalid_axi5_lite_mapping).is_err(),
        "AXI5-Lite output must reject AXI5 burst mappings: {invalid_axi5_lite_mapping}"
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
fn schema_input_command_prints_canonical_artifact_bytes() {
    let mut command = wavepeek_cmd();
    let assert = command.args(["schema", "--input"]).assert().success();

    let expected = fs::read(canonical_input_schema_path()).expect("input schema file should read");
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
    assert_eq!(families.len(), 3);
    assert!(families.iter().any(|entry| {
        entry["id"] == "wavepeek.output"
            && entry["version"] == "2.2"
            && entry["path"] == "schema/output.json"
            && entry["url"] == expected_schema_url()
    }));
    assert!(families.iter().any(|entry| {
        entry["id"] == "wavepeek.stream-record"
            && entry["version"] == "2.2"
            && entry["path"] == "schema/stream.json"
            && entry["url"] == expected_stream_schema_url()
    }));
    assert!(families.iter().any(|entry| {
        entry["id"] == "wavepeek.input"
            && entry["version"] == "2.2"
            && entry["path"] == "schema/input.json"
            && entry["url"] == expected_input_schema_url()
    }));
}

#[test]
fn schema_input_command_output_is_valid_json() {
    let value = input_schema_json();

    assert_eq!(
        value["$schema"],
        "https://json-schema.org/draft/2020-12/schema"
    );
    assert_eq!(value["$id"], expected_input_schema_url());
    assert_eq!(value["title"], "wavepeek JSON input documents");
    assert_eq!(
        value["oneOf"],
        json!([
            {"$ref": "#/$defs/extractGenericSourcesInput"},
            {"$ref": "#/$defs/extractAxiSourceInput"}
        ])
    );
    assert_eq!(
        value["$defs"]["extractGenericSourcesInput"]["properties"]["kind"]["const"],
        "extract.generic.sources"
    );
    assert_eq!(
        value["$defs"]["extractAxiSourceInput"]["properties"]["kind"]["const"],
        "extract.axi.source"
    );
    assert_eq!(
        value["$defs"]["axiProfile"]["enum"],
        json!([
            "axi3",
            "axi4",
            "axi4-lite",
            "axi5",
            "axi5-lite",
            "ace",
            "ace-lite",
            "ace5"
        ])
    );
}

#[test]
fn schema_input_validator_accepts_and_rejects_source_documents() {
    let validator = input_schema_validator();
    let valid = json!({
        "$schema": expected_input_schema_url(),
        "kind": "extract.generic.sources",
        "sources": [{
            "name": "rx.beat",
            "on": "posedge clk iff rst_n",
            "when": "valid && ready",
            "payload": ["data", "last"]
        }],
        "x-extension": true
    });
    validator
        .validate(&valid)
        .unwrap_or_else(|error| panic!("valid input document rejected: {error}\n{valid}"));

    let valid_axi = json!({
        "$schema": expected_input_schema_url(),
        "kind": "extract.axi.source",
        "profile": "axi4-lite",
        "name": "ctrl",
        "includes": ["^axi_"],
        "maps": {"aclk": "clk", "aresetn": "rst_n"},
        "x-extension": true
    });
    validator
        .validate(&valid_axi)
        .unwrap_or_else(|error| panic!("valid AXI input document rejected: {error}\n{valid_axi}"));

    for (profile, standard) in [
        ("axi5", "acaddr"),
        ("axi5-lite", "awidunq"),
        ("ace", "acvalid"),
        ("ace-lite", "awunique"),
        ("ace5", "cdpoison"),
    ] {
        let valid_axi_family = json!({
            "$schema": expected_input_schema_url(),
            "kind": "extract.axi.source",
            "profile": profile,
            "maps": {"aclk": "clk", (standard): format!("top.{standard}")}
        });
        validator
            .validate(&valid_axi_family)
            .unwrap_or_else(|error| {
                panic!("valid {profile} input document rejected: {error}\n{valid_axi_family}")
            });
    }

    let valid_default_axi4 = json!({
        "$schema": expected_input_schema_url(),
        "kind": "extract.axi.source",
        "maps": {"awlen": "axi_awlen"}
    });
    validator
        .validate(&valid_default_axi4)
        .unwrap_or_else(|error| {
            panic!("valid default AXI4 input document rejected: {error}\n{valid_default_axi4}")
        });

    for invalid in [
        json!({
            "$schema": expected_input_schema_url(),
            "kind": "wrong",
            "sources": [{"name": "rx", "on": "posedge clk", "when": "1", "payload": ["data"]}]
        }),
        json!({
            "$schema": expected_input_schema_url(),
            "kind": "extract.generic.sources",
            "sources": []
        }),
        json!({
            "$schema": expected_input_schema_url(),
            "kind": "extract.generic.sources",
            "sources": [{"name": "rx", "on": "posedge clk", "when": "1", "payload": []}]
        }),
        json!({
            "$schema": expected_input_schema_url(),
            "kind": "extract.axi.source",
            "profile": "AXI5_LITE"
        }),
        json!({
            "$schema": expected_input_schema_url(),
            "kind": "extract.axi.source",
            "profile": "ACE_LITE"
        }),
        json!({
            "$schema": expected_input_schema_url(),
            "kind": "extract.axi.source",
            "profile": "axi5",
            "maps": {"awpending": "top.awpending"}
        }),
        json!({
            "$schema": expected_input_schema_url(),
            "kind": "extract.axi.source",
            "profile": "axi5",
            "maps": {"acsnoop": "top.acsnoop"}
        }),
        json!({
            "$schema": expected_input_schema_url(),
            "kind": "extract.axi.source",
            "profile": "axi5-lite",
            "maps": {"awlen": "top.awlen"}
        }),
        json!({
            "$schema": expected_input_schema_url(),
            "kind": "extract.axi.source",
            "profile": "ace-lite",
            "maps": {"acvalid": "top.acvalid"}
        }),
        json!({
            "$schema": expected_input_schema_url(),
            "kind": "extract.axi.source",
            "profile": "ace5",
            "maps": {"awbar": "top.awbar"}
        }),
        json!({
            "$schema": expected_input_schema_url(),
            "kind": "extract.axi.source",
            "profile": "ace5",
            "maps": {"arbar": "top.arbar"}
        }),
        json!({
            "$schema": expected_input_schema_url(),
            "kind": "extract.axi.source",
            "profile": null
        }),
        json!({
            "$schema": expected_input_schema_url(),
            "kind": "extract.axi.source",
            "name": null
        }),
        json!({
            "$schema": expected_input_schema_url(),
            "kind": "extract.axi.source",
            "profile": "axi4-lite",
            "maps": {"awlen": "axi_awlen"}
        }),
        json!({
            "$schema": expected_input_schema_url(),
            "kind": "extract.axi.source",
            "maps": {"wid": "axi_wid"}
        }),
    ] {
        assert!(
            validator.validate(&invalid).is_err(),
            "invalid input document should fail validation: {invalid}"
        );
    }
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
        json!([
            "info",
            "scope",
            "signal",
            "value",
            "change",
            "property",
            "extract axi",
            "extract generic"
        ])
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
fn schema_stream_validator_enforces_axi5_context_and_item_isolation() {
    let validator = stream_schema_validator();
    let valid_begin = json!({
        "type": "begin",
        "seq": 0,
        "command": "extract axi",
        "$schema": expected_stream_schema_url(),
        "context": {
            "name": "axi5",
            "profile": "axi5",
            "issue": "L",
            "mappings": {
                "acvalid": {"path": "top.acvalid"},
                "acready": {"path": "top.acready"},
                "acaddr": {"path": "top.acaddr"}
            }
        }
    });
    validator
        .validate(&valid_begin)
        .unwrap_or_else(|error| panic!("valid AXI5 begin rejected: {error}\n{valid_begin}"));

    let valid_item = json!({
        "type": "item",
        "seq": 1,
        "command": "extract axi",
        "item": {
            "time": "30ns",
            "sample_time": "29ns",
            "profile": "axi5",
            "channel": "ac",
            "payload": {"acaddr": "32'h12345678", "acvmidext": "4'h9"}
        }
    });
    validator
        .validate(&valid_item)
        .unwrap_or_else(|error| panic!("valid AXI5 item rejected: {error}\n{valid_item}"));

    let mut invalid_issue = valid_begin.clone();
    invalid_issue["context"]["issue"] = json!("H.c");
    assert!(
        validator.validate(&invalid_issue).is_err(),
        "AXI5 begin must reject Issue H.c metadata: {invalid_issue}"
    );

    for standard in ["acsnoop", "cdvalid"] {
        let mut invalid_mapping = valid_begin.clone();
        invalid_mapping["context"]["mappings"][standard] =
            json!({"path": format!("top.{standard}")});
        assert!(
            validator.validate(&invalid_mapping).is_err(),
            "AXI5 begin must reject ACE-only {standard}: {invalid_mapping}"
        );
    }

    let mut invalid_channel = valid_item.clone();
    invalid_channel["item"]["channel"] = json!("cd");
    assert!(
        validator.validate(&invalid_channel).is_err(),
        "AXI5 item must reject CD channel: {invalid_channel}"
    );

    let mut invalid_payload = valid_item.clone();
    invalid_payload["item"]["payload"]["acsnoop"] = json!("4'h0");
    assert!(
        validator.validate(&invalid_payload).is_err(),
        "AXI5 AC item must reject ACE-only ACSNOOP: {invalid_payload}"
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
            "type": "begin",
            "seq": 0,
            "command": "extract axi",
            "$schema": expected_stream_schema_url(),
            "context": {
                "name": "axi",
                "profile": "axi4-lite",
                "issue": "H.c",
                "mappings": {"aclk": {"path": "top.clk"}}
            }
        }),
        json!({
            "type": "item",
            "seq": 1,
            "command": "extract axi",
            "item": {
                "time": "5ns",
                "sample_time": "4ns",
                "profile": "axi4-lite",
                "channel": "aw",
                "payload": {"awaddr": "32'h40"}
            }
        }),
        json!({
            "type": "begin",
            "seq": 0,
            "command": "extract axi",
            "$schema": expected_stream_schema_url(),
            "context": {
                "name": "ace",
                "profile": "ace",
                "issue": "H.c",
                "mappings": {"acvalid": {"path": "top.acvalid"}}
            }
        }),
        json!({
            "type": "item",
            "seq": 1,
            "command": "extract axi",
            "item": {
                "time": "5ns",
                "sample_time": "4ns",
                "profile": "ace",
                "channel": "ac",
                "payload": {"acaddr": "40'h1234"}
            }
        }),
        json!({
            "type": "begin",
            "seq": 0,
            "command": "extract axi",
            "$schema": expected_stream_schema_url(),
            "context": {
                "name": "ace-lite",
                "profile": "ace-lite",
                "issue": "H.c",
                "mappings": {"awunique": {"path": "top.awunique"}}
            }
        }),
        json!({
            "type": "item",
            "seq": 1,
            "command": "extract axi",
            "item": {
                "time": "5ns",
                "sample_time": "4ns",
                "profile": "ace-lite",
                "channel": "aw",
                "payload": {"awunique": "1'h1"}
            }
        }),
        json!({
            "type": "begin",
            "seq": 0,
            "command": "extract axi",
            "$schema": expected_stream_schema_url(),
            "context": {
                "name": "ace5",
                "profile": "ace5",
                "issue": "H.c",
                "mappings": {"cdpoison": {"path": "top.cdpoison"}}
            }
        }),
        json!({
            "type": "item",
            "seq": 1,
            "command": "extract axi",
            "item": {
                "time": "5ns",
                "sample_time": "4ns",
                "profile": "ace5",
                "channel": "cd",
                "payload": {"cdpoison": "1'h1"}
            }
        }),
        json!({
            "type": "item",
            "seq": 1,
            "command": "extract generic",
            "item": {
                "time": "5ns",
                "sample_time": "4ns",
                "source": "transfer",
                "payload": [{"path": "top.data", "value": "8'haa"}]
            }
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
        json!({
            "type": "item",
            "seq": 1,
            "command": "extract axi",
            "item": {
                "time": "5ns",
                "sample_time": "4ns",
                "profile": "axi4-lite",
                "channel": "aw",
                "payload": {"awlen": "8'h0"}
            }
        }),
        json!({
            "type": "begin",
            "seq": 0,
            "command": "extract axi",
            "$schema": expected_stream_schema_url(),
            "context": {
                "name": "ace-lite",
                "profile": "ace-lite",
                "issue": "H.c",
                "mappings": {"acvalid": {"path": "top.acvalid"}}
            }
        }),
        json!({
            "type": "item",
            "seq": 1,
            "command": "extract axi",
            "item": {
                "time": "5ns",
                "sample_time": "4ns",
                "profile": "ace-lite",
                "channel": "ac",
                "payload": {}
            }
        }),
        json!({
            "type": "begin",
            "seq": 0,
            "command": "extract axi",
            "$schema": expected_stream_schema_url(),
            "context": {
                "name": "ace5",
                "profile": "ace5",
                "issue": "H.c",
                "mappings": {"awbar": {"path": "top.awbar"}}
            }
        }),
        json!({
            "type": "begin",
            "seq": 0,
            "command": "extract axi",
            "$schema": expected_stream_schema_url(),
            "context": {
                "name": "ace5",
                "profile": "ace5",
                "issue": "H.c",
                "mappings": {"arbar": {"path": "top.arbar"}}
            }
        }),
        json!({
            "type": "item",
            "seq": 1,
            "command": "extract axi",
            "item": {
                "time": "5ns",
                "sample_time": "4ns",
                "profile": "ace5",
                "channel": "aw",
                "payload": {"awbar": "2'h1"}
            }
        }),
        json!({
            "type": "item",
            "seq": 1,
            "command": "extract axi",
            "item": {
                "time": "5ns",
                "sample_time": "4ns",
                "profile": "ace5",
                "channel": "ar",
                "payload": {"arbar": "2'h1"}
            }
        }),
    ];

    for case in invalid_cases {
        assert!(
            validator.validate(&case).is_err(),
            "invalid stream record should fail validation: {case}"
        );
    }
}
