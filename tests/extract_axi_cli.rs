use std::fs;

use assert_cmd::prelude::*;
use predicates::prelude::*;
use serde_json::{Value, json};
use tempfile::NamedTempFile;

mod common;
use common::{
    expected_input_schema_url, expected_schema_url, expected_stream_schema_url, wavepeek_cmd,
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

fn write_fixture(contents: &str, suffix: &str) -> NamedTempFile {
    let fixture = NamedTempFile::with_suffix(suffix).expect("temp fixture should create");
    fs::write(fixture.path(), contents).expect("fixture should write");
    fixture
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

const AXI_LITE_VCD: &str = concat!(
    "$date\n  today\n$end\n",
    "$version\n  wavepeek-extract-axi-test\n$end\n",
    "$timescale 1ns $end\n",
    "$scope module top $end\n",
    "$var wire 1 ! clk $end\n",
    "$var wire 1 \" aresetn $end\n",
    "$var wire 1 # axi_aw_valid_o $end\n",
    "$var wire 1 $ axi_aw_ready_i $end\n",
    "$var wire 8 % axi_aw_addr_o $end\n",
    "$var wire 3 & axi_aw_prot_o $end\n",
    "$var wire 1 ' axi_w_valid_o $end\n",
    "$var wire 1 ( axi_w_ready_i $end\n",
    "$var wire 8 ) axi_w_data_o $end\n",
    "$var wire 1 * axi_w_strb_o $end\n",
    "$var wire 1 + axi_b_valid_i $end\n",
    "$var wire 1 , axi_b_ready_o $end\n",
    "$var wire 2 - axi_b_resp_i $end\n",
    "$var wire 1 . axi_ar_valid_o $end\n",
    "$var wire 1 / axi_ar_ready_i $end\n",
    "$var wire 8 : axi_ar_addr_o $end\n",
    "$var wire 3 ; axi_ar_prot_o $end\n",
    "$var wire 1 < axi_r_valid_i $end\n",
    "$var wire 1 = axi_r_ready_o $end\n",
    "$var wire 8 > axi_r_data_i $end\n",
    "$var wire 2 ? axi_r_resp_i $end\n",
    "$var wire 1 @ axi_misc_o $end\n",
    "$upscope $end\n",
    "$enddefinitions $end\n",
    "#0\n",
    "0!\n",
    "1\"\n",
    "0#\n",
    "1$\n",
    "b00000000 %\n",
    "b000 &\n",
    "0'\n",
    "1(\n",
    "b00000000 )\n",
    "1*\n",
    "0+\n",
    "1,\n",
    "b00 -\n",
    "0.\n",
    "1/\n",
    "b00000000 :\n",
    "b000 ;\n",
    "0<\n",
    "1=\n",
    "b00000000 >\n",
    "b00 ?\n",
    "1@\n",
    "#4\n",
    "1#\n",
    "b00010010 %\n",
    "b010 &\n",
    "1'\n",
    "b10101010 )\n",
    "1+\n",
    "b01 -\n",
    "1.\n",
    "b00110100 :\n",
    "b011 ;\n",
    "1<\n",
    "b01010101 >\n",
    "b10 ?\n",
    "#5\n",
    "1!\n",
    "#6\n",
    "0!\n",
    "#9\n",
    "0\"\n",
    "b11111111 %\n",
    "#10\n",
    "1!\n"
);

const AXI3_W_VCD: &str = concat!(
    "$date\n  today\n$end\n",
    "$version\n  wavepeek-extract-axi3-w\n$end\n",
    "$timescale 1ns $end\n",
    "$scope module top $end\n",
    "$var wire 1 ! clk $end\n",
    "$var wire 1 \" axi_wvalid $end\n",
    "$var wire 1 # axi_wready $end\n",
    "$var wire 4 $ axi_wid $end\n",
    "$var wire 8 % axi_wdata $end\n",
    "$upscope $end\n",
    "$enddefinitions $end\n",
    "#0\n",
    "0!\n",
    "0\"\n",
    "1#\n",
    "b0000 $\n",
    "b00000000 %\n",
    "#4\n",
    "1\"\n",
    "b1010 $\n",
    "b11001100 %\n",
    "#5\n",
    "1!\n"
);

const AMBIGUOUS_VCD: &str = concat!(
    "$date\n  today\n$end\n",
    "$version\n  wavepeek-extract-axi-ambiguous\n$end\n",
    "$timescale 1ns $end\n",
    "$scope module top $end\n",
    "$var wire 1 ! clk $end\n",
    "$var wire 1 \" axi_awvalid_o $end\n",
    "$var wire 1 # other_awvalid_o $end\n",
    "$upscope $end\n",
    "$enddefinitions $end\n",
    "#0\n0!\n0\"\n0#\n"
);

const MULTI_MATCH_VCD: &str = concat!(
    "$date\n  today\n$end\n",
    "$version\n  wavepeek-extract-axi-multi-match\n$end\n",
    "$timescale 1ns $end\n",
    "$scope module top $end\n",
    "$var wire 1 ! clk $end\n",
    "$var wire 1 \" axi_awvalid_awready $end\n",
    "$upscope $end\n",
    "$enddefinitions $end\n",
    "#0\n0!\n0\"\n"
);

#[test]
fn extract_axi_json_automaps_axi4_lite_and_gates_reset() {
    let fixture = write_fixture(AXI_LITE_VCD, "extract-axi-lite.vcd");
    let fixture = fixture.path().to_string_lossy().into_owned();

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
    assert_eq!(transfers[0]["time"], "5ns");
    assert_eq!(transfers[0]["sample_time"], "4ns");
    assert_eq!(transfers[0]["payload"]["awaddr"], "8'h12");
    assert_eq!(transfers[1]["payload"]["wdata"], "8'haa");
    assert_eq!(transfers[4]["payload"]["rresp"], "2'h2");
}

#[test]
fn extract_axi_human_defaults_to_axi4() {
    let fixture = write_fixture(AXI_LITE_VCD, "extract-axi-default.vcd");
    let fixture = fixture.path().to_string_lossy().into_owned();

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
    let fixture = write_fixture(AXI3_W_VCD, "extract-axi3-w.vcd");
    let fixture = fixture.path().to_string_lossy().into_owned();

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
    assert_eq!(value["data"]["transfers"][0]["channel"], "w");
    assert_eq!(value["data"]["transfers"][0]["payload"]["wid"], "4'ha");
    assert_eq!(value["data"]["transfers"][0]["payload"]["wdata"], "8'hcc");
}

#[test]
fn extract_axi_source_jsonl_includes_begin_context() {
    let fixture = write_fixture(AXI_LITE_VCD, "extract-axi-source.vcd");
    let fixture_path = fixture.path().to_string_lossy().into_owned();
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
    assert_eq!(records[1]["item"]["channel"], "aw");
    assert_eq!(records.last().unwrap()["type"], "end");
    assert_eq!(records.last().unwrap()["summary"]["items"], 5);
}

#[test]
fn extract_axi_warns_for_unmatched_include_candidates() {
    let fixture = write_fixture(AXI_LITE_VCD, "extract-axi-warning.vcd");
    let fixture = fixture.path().to_string_lossy().into_owned();

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
    let fixture = write_fixture(AXI3_W_VCD, "extract-axi-explicit-include.vcd");
    let fixture = fixture.path().to_string_lossy().into_owned();

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
    let fixture = write_fixture(MULTI_MATCH_VCD, "extract-axi-multi-match.vcd");
    let fixture = fixture.path().to_string_lossy().into_owned();

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
    let fixture = write_fixture(AMBIGUOUS_VCD, "extract-axi-ambiguous.vcd");
    let fixture = fixture.path().to_string_lossy().into_owned();

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
fn extract_axi_rejects_partial_ready_valid_pairs() {
    let fixture = write_fixture(AXI_LITE_VCD, "extract-axi-partial.vcd");
    let fixture = fixture.path().to_string_lossy().into_owned();

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
