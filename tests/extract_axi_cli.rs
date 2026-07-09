use std::fs;

use assert_cmd::prelude::*;
use predicates::prelude::*;
use serde_json::{Value, json};
use tempfile::NamedTempFile;

mod common;
use common::{
    expected_input_schema_url, expected_schema_url, expected_stream_schema_url, fixture_path,
    wavepeek_cmd,
};

fn output_schema_validator() -> jsonschema::Validator {
    let schema_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("schema")
        .join("output.json");
    let schema: Value =
        serde_json::from_str(&fs::read_to_string(schema_path).expect("output schema should read"))
            .expect("output schema should parse");
    jsonschema::validator_for(&schema).expect("output schema should compile")
}

fn stream_schema_validator() -> jsonschema::Validator {
    let schema_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("schema")
        .join("stream.json");
    let schema: Value =
        serde_json::from_str(&fs::read_to_string(schema_path).expect("stream schema should read"))
            .expect("stream schema should parse");
    jsonschema::validator_for(&schema).expect("stream schema should compile")
}

fn parse_json(stdout: &[u8]) -> Value {
    let value: Value = serde_json::from_slice(stdout).expect("stdout should be valid json");
    output_schema_validator()
        .validate(&value)
        .unwrap_or_else(|error| panic!("output should validate: {error}\n{value}"));
    value
}

fn waveform_fixture(filename: &str) -> String {
    fixture_path(filename).to_string_lossy().into_owned()
}

fn write_source(contents: &str) -> NamedTempFile {
    let source =
        NamedTempFile::with_suffix("extract-axi-source.json").expect("source should create");
    fs::write(source.path(), contents).expect("source should write");
    source
}

fn parse_stream(stdout: &[u8]) -> Vec<Value> {
    let output = std::str::from_utf8(stdout).expect("stdout should be UTF-8 JSONL");
    assert!(output.ends_with('\n'));
    let validator = stream_schema_validator();
    output
        .lines()
        .map(|line| {
            let record: Value = serde_json::from_str(line).expect("JSONL line should parse");
            validator
                .validate(&record)
                .unwrap_or_else(|error| panic!("record should validate: {error}\n{record}"));
            record
        })
        .collect()
}

#[test]
fn extract_axi_json_automaps_axi4_lite_and_gates_reset() {
    let fixture = waveform_fixture("extract_axi_lite.vcd");

    let output = wavepeek_cmd()
        .args([
            "extract",
            "axi",
            "--waves",
            fixture.as_str(),
            "--scope",
            "top",
            "--profile",
            "axi4-lite",
            "--map",
            "aclk=clk",
            "--map",
            "aresetn=aresetn",
            "--include",
            "^axi_(aw|w|b|ar|r)_",
            "--json",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let value = parse_json(&output);
    assert_eq!(value["$schema"], expected_schema_url());
    assert_eq!(value["command"], "extract axi");
    assert_eq!(value["diagnostics"], json!([]));
    assert_eq!(value["data"]["name"], "axi");
    assert_eq!(value["data"]["profile"], "axi4-lite");
    assert_eq!(value["data"]["issue"], "H.c");
    assert_eq!(
        value["data"]["mappings"]["awvalid"]["path"],
        "top.axi_aw_valid_o"
    );

    let transfers = value["data"]["transfers"].as_array().unwrap();
    assert_eq!(transfers.len(), 5);
    assert_eq!(
        transfers
            .iter()
            .map(|row| row["channel"].as_str().unwrap())
            .collect::<Vec<_>>(),
        vec!["aw", "w", "b", "ar", "r"]
    );
    assert!(transfers.iter().all(|row| row["profile"] == "axi4-lite"));
    assert_eq!(transfers[0]["time"], "5ns");
    assert_eq!(transfers[0]["sample_time"], "4ns");
    assert_eq!(transfers[0]["payload"]["awaddr"], "8'h12");
    assert_eq!(transfers[1]["payload"]["wdata"], "8'haa");
    assert_eq!(transfers[4]["payload"]["rresp"], "2'h2");
}

#[test]
fn extract_axi_human_defaults_to_axi4() {
    let fixture = waveform_fixture("extract_axi_lite.vcd");

    wavepeek_cmd()
        .args([
            "extract",
            "axi",
            "--waves",
            fixture.as_str(),
            "--scope",
            "top",
            "--map",
            "aclk=clk",
            "--map",
            "aresetn=aresetn",
            "--include",
            "^axi_(aw|w|b|ar|r)_",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("profile: axi4"))
        .stdout(predicate::str::contains("mappings:\n  aclk = clk"))
        .stdout(predicate::str::contains(
            "@5ns sample@4ns [aw] awaddr=8'h12 awprot=3'h2",
        ));
}

#[test]
fn extract_axi3_profile_extracts_wid() {
    let fixture = waveform_fixture("extract_axi3_w.vcd");

    let output = wavepeek_cmd()
        .args([
            "extract",
            "axi",
            "--waves",
            fixture.as_str(),
            "--scope",
            "top",
            "--profile",
            "axi3",
            "--map",
            "aclk=clk",
            "--include",
            "^axi_w",
            "--json",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let value = parse_json(&output);
    assert_eq!(value["data"]["profile"], "axi3");
    assert_eq!(value["data"]["transfers"].as_array().unwrap().len(), 1);
    assert_eq!(value["data"]["transfers"][0]["profile"], "axi3");
    assert_eq!(value["data"]["transfers"][0]["channel"], "w");
    assert_eq!(value["data"]["transfers"][0]["payload"]["wid"], "4'ha");
    assert_eq!(value["data"]["transfers"][0]["payload"]["wdata"], "8'hcc");
}

#[test]
fn extract_axi_source_jsonl_includes_begin_context() {
    let fixture_path = waveform_fixture("extract_axi_lite.vcd");
    let source = write_source(&format!(
        r#"{{
  "$schema": "{}",
  "kind": "extract.axi.source",
  "profile": "axi4-lite",
  "name": "cfg",
  "includes": ["^axi_(aw|w|b|ar|r)_"],
  "maps": {{"aclk": "clk", "aresetn": "aresetn"}}
}}"#,
        expected_input_schema_url()
    ));
    let source_path = source.path().to_string_lossy().into_owned();

    let output = wavepeek_cmd()
        .args([
            "extract",
            "axi",
            "--waves",
            fixture_path.as_str(),
            "--scope",
            "top",
            "--source",
            source_path.as_str(),
            "--jsonl",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let records = parse_stream(&output);
    assert_eq!(records.first().unwrap()["type"], "begin");
    assert_eq!(
        records.first().unwrap()["$schema"],
        expected_stream_schema_url()
    );
    assert_eq!(records.first().unwrap()["context"]["name"], "cfg");
    assert_eq!(records.first().unwrap()["context"]["profile"], "axi4-lite");
    assert_eq!(records[1]["type"], "item");
    assert_eq!(records[1]["item"]["profile"], "axi4-lite");
    assert_eq!(records[1]["item"]["channel"], "aw");
    assert_eq!(records.last().unwrap()["type"], "end");
    assert_eq!(records.last().unwrap()["summary"]["items"], 5);
}

#[test]
fn extract_axi_profile_flag_accepts_case_insensitive_alias() {
    let fixture = waveform_fixture("extract_axi3_w.vcd");

    let output = wavepeek_cmd()
        .args([
            "extract",
            "axi",
            "--waves",
            fixture.as_str(),
            "--scope",
            "top",
            "--profile",
            "AXI4_LITE",
            "--map",
            "aclk=clk",
            "--include",
            "^axi_w",
            "--json",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let value = parse_json(&output);
    assert_eq!(value["data"]["profile"], "axi4-lite");
}

#[test]
fn extract_axi_reuses_mapping_waveform_for_execution() {
    let fixture = waveform_fixture("extract_axi_lite.vcd");

    let output = wavepeek_cmd()
        .env("DEBUG", "1")
        .args([
            "extract",
            "axi",
            "--waves",
            fixture.as_str(),
            "--scope",
            "top",
            "--profile",
            "axi4-lite",
            "--map",
            "aclk=clk",
            "--map",
            "aresetn=aresetn",
            "--include",
            "^axi_(aw|w|b|ar|r)_",
            "--max",
            "1",
            "--json",
        ])
        .assert()
        .success()
        .get_output()
        .clone();

    let stderr = std::str::from_utf8(&output.stderr).expect("debug stderr should be UTF-8");
    assert_eq!(stderr.matches("backend.open.start").count(), 1);
    assert_eq!(stderr.matches("backend.open.done").count(), 1);
}

#[test]
fn extract_axi_source_rejects_explicit_null_strings() {
    let fixture = waveform_fixture("extract_axi_lite.vcd");

    for contents in [
        r#"{
  "$schema": "https://kleverhq.github.io/wavepeek/schema-input-v2.2.json",
  "kind": "extract.axi.source",
  "profile": null
}
"#,
        r#"{
  "$schema": "https://kleverhq.github.io/wavepeek/schema-input-v2.2.json",
  "kind": "extract.axi.source",
  "name": null
}
"#,
    ] {
        let source = write_source(contents);
        let source = source.path().to_string_lossy().into_owned();

        wavepeek_cmd()
            .args([
                "extract",
                "axi",
                "--waves",
                fixture.as_str(),
                "--scope",
                "top",
                "--source",
                source.as_str(),
            ])
            .assert()
            .failure()
            .stderr(predicate::str::contains("expected string, got null"));
    }
}

#[test]
fn extract_axi_source_rejects_legacy_generic_schema_url() {
    let fixture = waveform_fixture("extract_axi_lite.vcd");
    let source = write_source(
        r#"{
  "$schema": "https://kleverhq.github.io/wavepeek/schema-input-v2.1.json",
  "kind": "extract.axi.source",
  "profile": "axi4-lite",
  "maps": {"aclk": "clk"}
}
"#,
    );
    let source = source.path().to_string_lossy().into_owned();

    wavepeek_cmd()
        .args([
            "extract",
            "axi",
            "--waves",
            fixture.as_str(),
            "--scope",
            "top",
            "--source",
            source.as_str(),
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("uses unsupported $schema"));
}

#[test]
fn extract_axi_source_conflicts_with_explicit_profile() {
    let fixture_path = waveform_fixture("extract_axi_lite.vcd");
    let source = write_source(&format!(
        r#"{{"$schema":"{}","kind":"extract.axi.source","maps":{{"aclk":"clk"}}}}"#,
        expected_input_schema_url()
    ));
    let source_path = source.path().to_string_lossy().into_owned();

    wavepeek_cmd()
        .args([
            "extract",
            "axi",
            "--waves",
            fixture_path.as_str(),
            "--scope",
            "top",
            "--source",
            source_path.as_str(),
            "--profile",
            "axi3",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("cannot be used with"));
}

#[test]
fn extract_axi_warns_for_unmatched_include_candidates() {
    let fixture = waveform_fixture("extract_axi_lite.vcd");

    let output = wavepeek_cmd()
        .args([
            "extract",
            "axi",
            "--waves",
            fixture.as_str(),
            "--scope",
            "top",
            "--map",
            "aclk=clk",
            "--map",
            "aresetn=aresetn",
            "--map",
            "awvalid=axi_aw_valid_o",
            "--map",
            "awready=axi_aw_ready_i",
            "--include",
            "^axi_misc_o$",
            "--json",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let value = parse_json(&output);
    assert_eq!(value["data"]["transfers"].as_array().unwrap().len(), 1);
    assert_eq!(value["data"]["transfers"][0]["payload"], json!({}));
    assert_eq!(value["diagnostics"][0]["code"], "WPK-W0004");
}

#[test]
fn extract_axi_does_not_warn_for_explicitly_mapped_include_path() {
    let fixture = waveform_fixture("extract_axi3_w.vcd");

    let output = wavepeek_cmd()
        .args([
            "extract",
            "axi",
            "--waves",
            fixture.as_str(),
            "--scope",
            "top",
            "--profile",
            "axi3",
            "--map",
            "aclk=clk",
            "--include",
            "^(clk|axi_w.*)$",
            "--json",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let value = parse_json(&output);
    assert_eq!(value["diagnostics"], json!([]));
}

#[test]
fn extract_axi_rejects_single_candidate_matching_multiple_standards() {
    let fixture = waveform_fixture("extract_axi_multi_match.vcd");

    wavepeek_cmd()
        .args([
            "extract",
            "axi",
            "--waves",
            fixture.as_str(),
            "--scope",
            "top",
            "--map",
            "aclk=clk",
            "--include",
            ".*awvalid_awready.*",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "ambiguous AXI auto-mapping for 'axi_awvalid_awready'",
        ));
}

#[test]
fn extract_axi_rejects_ambiguous_auto_mapping() {
    let fixture = waveform_fixture("extract_axi_ambiguous.vcd");

    wavepeek_cmd()
        .args([
            "extract",
            "axi",
            "--waves",
            fixture.as_str(),
            "--scope",
            "top",
            "--map",
            "aclk=clk",
            "--include",
            ".*awvalid.*",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "ambiguous AXI auto-mapping for 'awvalid'",
        ));
}

#[test]
fn extract_axi_reports_ambiguous_auto_mapping_in_standard_order() {
    let fixture = waveform_fixture("extract_axi_multi_ambiguous.vcd");

    wavepeek_cmd()
        .args([
            "extract",
            "axi",
            "--waves",
            fixture.as_str(),
            "--scope",
            "top",
            "--map",
            "aclk=clk",
            "--include",
            ".*valid.*",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "ambiguous AXI auto-mapping for 'awvalid'",
        ));
}

#[test]
fn extract_axi_rejects_partial_ready_valid_pairs() {
    let fixture = waveform_fixture("extract_axi_lite.vcd");

    wavepeek_cmd()
        .args([
            "extract",
            "axi",
            "--waves",
            fixture.as_str(),
            "--scope",
            "top",
            "--map",
            "aclk=clk",
            "--map",
            "awvalid=axi_aw_valid_o",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "AXI channel 'aw' must map both awvalid and awready",
        ));
}
