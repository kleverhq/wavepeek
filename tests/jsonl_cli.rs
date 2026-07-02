use std::fs;
use std::io::{BufRead, BufReader};
use std::process::Stdio;

use assert_cmd::prelude::*;
use predicates::prelude::*;
use serde_json::{Value, json};
use tempfile::NamedTempFile;

mod common;
use common::{expected_stream_schema_url, fixture_path, wavepeek_cmd};

fn stream_schema_validator() -> jsonschema::Validator {
    let schema_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("schema")
        .join("stream.json");
    let schema: Value =
        serde_json::from_str(&fs::read_to_string(schema_path).expect("stream schema should read"))
            .expect("stream schema should parse");
    jsonschema::validator_for(&schema).expect("stream schema should compile")
}

fn parse_stream(stdout: &[u8], expected_command: &str) -> Vec<Value> {
    let output = std::str::from_utf8(stdout).expect("stdout should be UTF-8 JSONL");
    assert!(
        !output.is_empty(),
        "successful JSONL stdout should not be empty"
    );
    assert!(
        output.ends_with('\n'),
        "JSONL stream should end each record with a newline"
    );

    let validator = stream_schema_validator();
    let records = output
        .lines()
        .map(|line| {
            let record: Value = serde_json::from_str(line).expect("line should parse as JSON");
            validator.validate(&record).unwrap_or_else(|error| {
                panic!("JSONL record should match schema: {error}\n{record}")
            });
            record
        })
        .collect::<Vec<_>>();

    assert!(
        records.len() >= 2,
        "stream should contain begin and end records"
    );
    let mut items = 0usize;
    let mut diagnostics = 0usize;
    let mut seen_diagnostic = false;
    let mut saw_truncation = false;

    for (seq, record) in records.iter().enumerate() {
        assert_eq!(record["seq"], seq, "sequence numbers should be contiguous");
        assert_eq!(record["command"], expected_command);
        match record["type"]
            .as_str()
            .expect("record type should be string")
        {
            "begin" => {
                assert_eq!(seq, 0, "begin must be the first record");
                assert_eq!(record["$schema"], expected_stream_schema_url());
            }
            "item" => {
                assert!(!seen_diagnostic, "items should precede diagnostics");
                items += 1;
            }
            "diagnostic" => {
                seen_diagnostic = true;
                diagnostics += 1;
                if record["diagnostic"]["code"] == "WPK-W0002" {
                    saw_truncation = true;
                }
            }
            "end" => {
                assert_eq!(seq, records.len() - 1, "end must be the final record");
                assert_eq!(record["summary"]["status"], "ok");
                assert_eq!(record["summary"]["items"], items);
                assert_eq!(record["summary"]["diagnostics"], diagnostics);
                assert_eq!(record["summary"]["truncated"], saw_truncation);
            }
            other => panic!("unexpected JSONL record type {other}"),
        }
    }

    assert_eq!(records.first().unwrap()["type"], "begin");
    assert_eq!(records.last().unwrap()["type"], "end");
    records
}

fn write_fixture(contents: &str, suffix: &str) -> NamedTempFile {
    let fixture = NamedTempFile::with_suffix(suffix).expect("temp fixture should create");
    fs::write(fixture.path(), contents).expect("fixture should write");
    fixture
}

const PROPERTY_VCD: &str = concat!(
    "$date\n  today\n$end\n",
    "$version\n  jsonl-property-test\n$end\n",
    "$timescale 1ns $end\n",
    "$scope module top $end\n",
    "$var wire 1 ! sig $end\n",
    "$upscope $end\n",
    "$enddefinitions $end\n",
    "#0\n",
    "0!\n",
    "#5\n",
    "1!\n",
    "#10\n",
    "0!\n"
);

#[test]
fn change_jsonl_streams_items_and_validates_against_schema() {
    let fixture = fixture_path("m2_core.vcd");
    let fixture = fixture.to_string_lossy().into_owned();

    let output = wavepeek_cmd()
        .args([
            "change",
            "--waves",
            fixture.as_str(),
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
            "--jsonl",
        ])
        .output()
        .expect("change --jsonl should execute");

    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let records = parse_stream(&output.stdout, "change");
    let items = records
        .iter()
        .filter(|record| record["type"] == "item")
        .collect::<Vec<_>>();
    assert_eq!(items.len(), 2);
    assert_eq!(items[0]["item"]["time"], "5ns");
    assert_eq!(
        items[0]["item"]["signals"][0],
        json!({"path": "top.clk", "value": "1'h1"})
    );
}

#[test]
fn change_jsonl_reports_truncation_in_summary() {
    let fixture = fixture_path("m2_core.vcd");
    let fixture = fixture.to_string_lossy().into_owned();

    let output = wavepeek_cmd()
        .args([
            "change",
            "--waves",
            fixture.as_str(),
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
            "1",
            "--jsonl",
        ])
        .output()
        .expect("change --jsonl should execute");

    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let records = parse_stream(&output.stdout, "change");
    assert_eq!(
        records
            .iter()
            .filter(|record| record["type"] == "item")
            .count(),
        1
    );
    assert!(records.iter().any(|record| {
        record["type"] == "diagnostic" && record["diagnostic"]["code"] == "WPK-W0002"
    }));
    assert_eq!(records.last().unwrap()["summary"]["truncated"], true);
}

#[test]
fn change_jsonl_reports_empty_result_before_end() {
    let fixture = fixture_path("m2_core.vcd");
    let fixture = fixture.to_string_lossy().into_owned();

    let output = wavepeek_cmd()
        .args([
            "change",
            "--waves",
            fixture.as_str(),
            "--from",
            "1ns",
            "--to",
            "10ns",
            "--scope",
            "top",
            "--signals",
            "clk",
            "--on",
            "negedge clk",
            "--jsonl",
        ])
        .output()
        .expect("empty change --jsonl should execute");

    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let records = parse_stream(&output.stdout, "change");
    assert_eq!(
        records
            .iter()
            .filter(|record| record["type"] == "item")
            .count(),
        0
    );
    assert!(records.iter().any(|record| {
        record["type"] == "diagnostic" && record["diagnostic"]["code"] == "WPK-W0003"
    }));
    assert_eq!(records.last().unwrap()["summary"]["truncated"], false);
}

#[test]
fn extract_generic_jsonl_streams_rows_and_validates_against_schema() {
    let fixture = fixture_path("m2_core.vcd");
    let fixture = fixture.to_string_lossy().into_owned();

    let output = wavepeek_cmd()
        .args([
            "extract",
            "generic",
            "--waves",
            fixture.as_str(),
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
            "--jsonl",
        ])
        .output()
        .expect("extract generic should execute");

    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let records = parse_stream(&output.stdout, "extract generic");
    let items = records
        .iter()
        .filter(|record| record["type"] == "item")
        .collect::<Vec<_>>();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0]["item"]["source"], "transfer");
}

#[test]
fn property_jsonl_streams_capture_rows() {
    let fixture = write_fixture(PROPERTY_VCD, ".property-jsonl.vcd");
    let fixture = fixture.path().to_string_lossy().into_owned();

    let output = wavepeek_cmd()
        .args([
            "property",
            "--waves",
            fixture.as_str(),
            "--scope",
            "top",
            "--on",
            "*",
            "--sample-mode",
            "native",
            "--eval",
            "sig",
            "--capture",
            "switch",
            "--jsonl",
        ])
        .output()
        .expect("property --jsonl should execute");

    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let records = parse_stream(&output.stdout, "property");
    let kinds = records
        .iter()
        .filter(|record| record["type"] == "item")
        .map(|record| record["item"]["kind"].as_str().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(kinds, vec!["assert", "deassert"]);
}

#[test]
fn property_jsonl_reports_truncation_in_summary() {
    let fixture = write_fixture(PROPERTY_VCD, ".property-jsonl-truncated.vcd");
    let fixture = fixture.path().to_string_lossy().into_owned();

    let output = wavepeek_cmd()
        .args([
            "property",
            "--waves",
            fixture.as_str(),
            "--scope",
            "top",
            "--on",
            "*",
            "--sample-mode",
            "native",
            "--eval",
            "sig",
            "--capture",
            "switch",
            "--max",
            "1",
            "--jsonl",
        ])
        .output()
        .expect("property --jsonl should execute");

    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let records = parse_stream(&output.stdout, "property");
    assert_eq!(
        records
            .iter()
            .filter(|record| record["type"] == "item")
            .count(),
        1
    );
    assert!(records.iter().any(|record| {
        record["type"] == "diagnostic" && record["diagnostic"]["code"] == "WPK-W0002"
    }));
    assert_eq!(records.last().unwrap()["summary"]["truncated"], true);
}

#[test]
fn info_scope_signal_and_value_jsonl_emit_representative_items() {
    let fixture = fixture_path("m2_core.vcd");
    let fixture = fixture.to_string_lossy().into_owned();

    let info = wavepeek_cmd()
        .args(["info", "--waves", fixture.as_str(), "--jsonl"])
        .output()
        .expect("info --jsonl should execute");
    assert!(info.status.success());
    let info_records = parse_stream(&info.stdout, "info");
    assert_eq!(info_records[1]["item"]["time_unit"], "1ns");

    let scope = wavepeek_cmd()
        .args(["scope", "--waves", fixture.as_str(), "--jsonl"])
        .output()
        .expect("scope --jsonl should execute");
    assert!(scope.status.success());
    let scope_records = parse_stream(&scope.stdout, "scope");
    assert!(
        scope_records
            .iter()
            .any(|record| { record["type"] == "item" && record["item"]["path"] == "top" })
    );

    let signal = wavepeek_cmd()
        .args([
            "signal",
            "--waves",
            fixture.as_str(),
            "--scope",
            "top",
            "--jsonl",
        ])
        .output()
        .expect("signal --jsonl should execute");
    assert!(signal.status.success());
    let signal_records = parse_stream(&signal.stdout, "signal");
    assert!(
        signal_records
            .iter()
            .any(|record| { record["type"] == "item" && record["item"]["path"] == "top.clk" })
    );

    let value = wavepeek_cmd()
        .args([
            "value",
            "--waves",
            fixture.as_str(),
            "--at",
            "5ns",
            "--signals",
            "top.clk,top.data",
            "--jsonl",
        ])
        .output()
        .expect("value --jsonl should execute");
    assert!(value.status.success());
    let value_records = parse_stream(&value.stdout, "value");
    assert_eq!(value_records[1]["item"]["time"], "5ns");
    assert_eq!(
        value_records[1]["item"]["signals"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
}

#[test]
fn json_and_jsonl_flags_conflict_on_waveform_commands() {
    let fixture = fixture_path("m2_core.vcd");
    let fixture = fixture.to_string_lossy().into_owned();

    wavepeek_cmd()
        .args([
            "change",
            "--waves",
            fixture.as_str(),
            "--signals",
            "top.clk",
            "--json",
            "--jsonl",
        ])
        .assert()
        .failure()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::contains("cannot be used with"))
        .stderr(predicate::str::contains("--json"))
        .stderr(predicate::str::contains("--jsonl"));
}

#[test]
fn helper_commands_do_not_accept_jsonl_output_mode() {
    wavepeek_cmd()
        .args(["docs", "topics", "--jsonl"])
        .assert()
        .failure()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::contains("unexpected argument '--jsonl'"));
}

#[test]
fn jsonl_broken_pipe_from_early_consumer_is_silent_success() {
    let mut vcd = String::from(
        "$date\n  today\n$end\n$version\n  jsonl-broken-pipe-test\n$end\n$timescale 1ns $end\n$scope module top $end\n$var wire 1 ! sig $end\n$upscope $end\n$enddefinitions $end\n#0\n0!\n",
    );
    for index in 1..=20_000 {
        vcd.push_str(&format!("#{}\n{}!\n", index, index % 2));
    }
    let fixture = write_fixture(&vcd, ".jsonl-broken-pipe.vcd");
    let fixture = fixture.path().to_string_lossy().into_owned();

    let mut child = wavepeek_cmd()
        .args([
            "change",
            "--waves",
            fixture.as_str(),
            "--signals",
            "top.sig",
            "--on",
            "*",
            "--sample-mode",
            "native",
            "--max",
            "unlimited",
            "--jsonl",
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("change --jsonl should spawn");

    let stdout = child.stdout.take().expect("child stdout should be piped");
    let mut reader = BufReader::new(stdout);
    let mut line = String::new();
    for _ in 0..4 {
        line.clear();
        let bytes = reader
            .read_line(&mut line)
            .expect("should read early JSONL line");
        assert!(bytes > 0, "expected an early JSONL line");
    }
    drop(reader);

    let output = child.wait_with_output().expect("child should finish");
    assert!(output.status.success());
    assert!(
        output.stderr.is_empty(),
        "broken pipe should not print stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}
