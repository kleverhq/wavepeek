use std::fs;

use assert_cmd::prelude::*;
use predicates::prelude::*;
use serde_json::{Value, json};
use tempfile::NamedTempFile;

mod common;
use common::{fixture_path, wavepeek_cmd};

fn parse_json(stdout: &[u8]) -> Value {
    serde_json::from_slice(stdout).expect("stdout should be valid json")
}

fn write_fixture(contents: &str, suffix: &str) -> NamedTempFile {
    let fixture = NamedTempFile::with_suffix(suffix).expect("temp fixture should create");
    fs::write(fixture.path(), contents).expect("fixture should write");
    fixture
}

#[test]
fn property_switch_capture_reports_transitions() {
    let fixture = write_fixture(
        "$date\n  today\n$end\n$version\n  wavepeek-test\n$end\n$timescale 1ns $end\n$scope module top $end\n$var wire 1 ! clk $end\n$var wire 1 \" sig $end\n$upscope $end\n$enddefinitions $end\n#0\n0!\n0\"\n#5\n1!\n#10\n0!\n1\"\n#15\n1!\n#20\n0!\n0\"\n#25\n1!\n",
        "property-switch.vcd",
    );
    let fixture = fixture.path().to_string_lossy().into_owned();

    let json_output = wavepeek_cmd()
        .args([
            "property",
            "--waves",
            fixture.as_str(),
            "--from",
            "0ns",
            "--to",
            "25ns",
            "--scope",
            "top",
            "--on",
            "posedge clk",
            "--eval",
            "sig",
            "--capture",
            "switch",
            "--json",
        ])
        .output()
        .expect("property should execute");
    let human_output = wavepeek_cmd()
        .args([
            "property",
            "--waves",
            fixture.as_str(),
            "--from",
            "0ns",
            "--to",
            "25ns",
            "--scope",
            "top",
            "--on",
            "posedge clk",
            "--eval",
            "sig",
            "--capture",
            "switch",
        ])
        .output()
        .expect("property should execute");

    assert!(json_output.status.success());
    assert!(human_output.status.success());
    assert!(json_output.stderr.is_empty());
    assert!(human_output.stderr.is_empty());

    let json = parse_json(&json_output.stdout);
    assert_eq!(json["command"], "property");
    assert_eq!(json["warnings"], json!([]));
    assert_eq!(
        json["data"],
        json!([
            {"time": "15ns", "kind": "assert"},
            {"time": "25ns", "kind": "deassert"}
        ])
    );
    assert_eq!(
        String::from_utf8_lossy(&human_output.stdout).trim(),
        "@15ns assert\n@25ns deassert"
    );
}

#[test]
fn property_assert_and_deassert_capture_filters() {
    let fixture = write_fixture(
        "$date\n  today\n$end\n$version\n  wavepeek-test\n$end\n$timescale 1ns $end\n$scope module top $end\n$var wire 1 ! clk $end\n$var wire 1 \" sig $end\n$upscope $end\n$enddefinitions $end\n#0\n0!\n0\"\n#5\n1!\n#10\n0!\n1\"\n#15\n1!\n#20\n0!\n0\"\n#25\n1!\n",
        "property-capture-filter.vcd",
    );
    let fixture = fixture.path().to_string_lossy().into_owned();

    let assert_output = wavepeek_cmd()
        .args([
            "property",
            "--waves",
            fixture.as_str(),
            "--from",
            "0ns",
            "--to",
            "25ns",
            "--scope",
            "top",
            "--on",
            "posedge clk",
            "--eval",
            "sig",
            "--capture",
            "assert",
            "--json",
        ])
        .output()
        .expect("property should execute");
    let deassert_output = wavepeek_cmd()
        .args([
            "property",
            "--waves",
            fixture.as_str(),
            "--from",
            "0ns",
            "--to",
            "25ns",
            "--scope",
            "top",
            "--on",
            "posedge clk",
            "--eval",
            "sig",
            "--capture",
            "deassert",
            "--json",
        ])
        .output()
        .expect("property should execute");

    assert!(assert_output.status.success());
    assert!(deassert_output.status.success());
    assert_eq!(
        parse_json(&assert_output.stdout)["data"],
        json!([{"time": "15ns", "kind": "assert"}])
    );
    assert_eq!(
        parse_json(&deassert_output.stdout)["data"],
        json!([{"time": "25ns", "kind": "deassert"}])
    );
}

#[test]
fn property_boolean_context_accepts_multibit_and_real_truthy_results() {
    let fixture = write_fixture(
        "$date\n  today\n$end\n$version\n  wavepeek-test\n$end\n$timescale 1ns $end\n$scope module top $end\n$var wire 1 ! clk $end\n$var wire 2 \" data $end\n$upscope $end\n$enddefinitions $end\n#0\n0!\nb00 \"\n#5\n1!\nb10 \"\n",
        "property-truthiness.vcd",
    );
    let fixture = fixture.path().to_string_lossy().into_owned();

    let multibit_output = wavepeek_cmd()
        .args([
            "property",
            "--waves",
            fixture.as_str(),
            "--scope",
            "top",
            "--on",
            "posedge clk",
            "--eval",
            "data",
            "--capture",
            "match",
            "--json",
        ])
        .output()
        .expect("property should execute");
    let real_output = wavepeek_cmd()
        .args([
            "property",
            "--waves",
            fixture.as_str(),
            "--scope",
            "top",
            "--on",
            "posedge clk",
            "--eval",
            "real'(1)",
            "--capture",
            "match",
            "--json",
        ])
        .output()
        .expect("property should execute");

    assert!(multibit_output.status.success());
    assert!(real_output.status.success());
    assert_eq!(
        parse_json(&multibit_output.stdout)["data"],
        json!([{"time": "5ns", "kind": "match"}])
    );
    assert_eq!(
        parse_json(&real_output.stdout)["data"],
        json!([{"time": "5ns", "kind": "match"}])
    );
}

#[test]
fn property_invalid_eval_reports_expr_error() {
    let fixture = fixture_path("m2_core.vcd");
    let fixture = fixture.to_string_lossy().into_owned();

    wavepeek_cmd()
        .args([
            "property",
            "--waves",
            fixture.as_str(),
            "--on",
            "posedge top.clk",
            "--eval",
            "(",
        ])
        .assert()
        .failure()
        .code(1)
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::starts_with("error: expr:"))
        .stderr(predicate::str::contains("EXPR-PARSE-LOGICAL-EXPECTED"));
}

#[test]
fn property_omitted_on_tracks_eval_signal_changes() {
    let fixture = fixture_path("m2_core.vcd");
    let fixture = fixture.to_string_lossy().into_owned();

    let output = wavepeek_cmd()
        .args([
            "property",
            "--waves",
            fixture.as_str(),
            "--scope",
            "top",
            "--eval",
            "data == 8'h0f",
            "--capture",
            "match",
            "--json",
        ])
        .output()
        .expect("property should execute");

    assert!(output.status.success());
    assert!(output.stderr.is_empty());

    let json = parse_json(&output.stdout);
    assert_eq!(json["warnings"], json!([]));
    assert_eq!(json["data"], json!([{"time": "10ns", "kind": "match"}]));
}

#[test]
fn property_omitted_on_tracks_raw_event_handles_from_eval() {
    let fixture = write_fixture(
        "$date\n  today\n$end\n$version\n  wavepeek-test\n$end\n$timescale 1ns $end\n$scope module top $end\n$var event 1 ! ev $end\n$scope module ev $end\n$var wire 1 \" triggered $end\n$upscope $end\n$upscope $end\n$enddefinitions $end\n#0\n0\"\n#5\n1!\n1\"\n#10\n0\"\n#20\n1!\n1\"\n#25\n0\"\n",
        "property-event-track.vcd",
    );
    let fixture = fixture.path().to_string_lossy().into_owned();

    let output = wavepeek_cmd()
        .args([
            "property",
            "--waves",
            fixture.as_str(),
            "--scope",
            "top",
            "--eval",
            "ev.triggered()",
            "--capture",
            "match",
            "--json",
        ])
        .output()
        .expect("property should execute");

    assert!(output.status.success());
    assert!(output.stderr.is_empty());

    let json = parse_json(&output.stdout);
    assert_eq!(json["warnings"], json!([]));
    assert_eq!(
        json["data"],
        json!([
            {"time": "5ns", "kind": "match"},
            {"time": "20ns", "kind": "match"}
        ])
    );
}

#[test]
fn property_mixed_wildcard_union_runs_with_signal_free_eval() {
    let fixture = fixture_path("m2_core.vcd");
    let fixture = fixture.to_string_lossy().into_owned();

    let output = wavepeek_cmd()
        .args([
            "property",
            "--waves",
            fixture.as_str(),
            "--scope",
            "top",
            "--on",
            "* or posedge clk",
            "--eval",
            "1",
            "--capture",
            "match",
            "--json",
        ])
        .output()
        .expect("property should execute");

    assert!(output.status.success());
    assert_eq!(
        parse_json(&output.stdout)["data"],
        json!([{"time": "5ns", "kind": "match"}])
    );
}

#[test]
fn property_wildcard_without_referenced_signals_requires_explicit_on() {
    let fixture = fixture_path("m2_core.vcd");
    let fixture = fixture.to_string_lossy().into_owned();

    wavepeek_cmd()
        .args(["property", "--waves", fixture.as_str(), "--eval", "1"])
        .assert()
        .failure()
        .code(1)
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::starts_with("error: args:"))
        .stderr(predicate::str::contains("pass --on explicitly"));
}

#[test]
fn property_scope_rejects_dotted_names_in_scoped_mode() {
    let fixture = fixture_path("m2_core.vcd");
    let fixture = fixture.to_string_lossy().into_owned();

    wavepeek_cmd()
        .args([
            "property",
            "--waves",
            fixture.as_str(),
            "--scope",
            "top",
            "--on",
            "posedge top.clk",
            "--eval",
            "data == 8'h00",
        ])
        .assert()
        .failure()
        .code(1)
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::starts_with("error: expr:"))
        .stderr(predicate::str::contains("unknown signal 'top.clk'"));
}

#[test]
fn property_rejects_legacy_surface_flags() {
    let fixture = fixture_path("m2_core.vcd");
    let fixture = fixture.to_string_lossy().into_owned();

    wavepeek_cmd()
        .args([
            "property",
            "--waves",
            fixture.as_str(),
            "--clk",
            "top.clk",
            "--eval",
            "1",
        ])
        .assert()
        .failure()
        .code(1)
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::starts_with("error: args:"))
        .stderr(predicate::str::contains("unexpected argument '--clk'"))
        .stderr(predicate::str::contains("See 'wavepeek property --help'."));

    wavepeek_cmd()
        .args([
            "property",
            "--waves",
            fixture.as_str(),
            "--on",
            "posedge top.clk",
            "--cond",
            "1",
        ])
        .assert()
        .failure()
        .code(1)
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::starts_with("error: args:"))
        .stderr(predicate::str::contains("unexpected argument '--cond'"))
        .stderr(predicate::str::contains("See 'wavepeek property --help'."));

    wavepeek_cmd()
        .args([
            "property",
            "--waves",
            fixture.as_str(),
            "--when",
            "posedge top.clk",
            "--eval",
            "1",
        ])
        .assert()
        .failure()
        .code(1)
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::starts_with("error: args:"))
        .stderr(predicate::str::contains("unexpected argument '--when'"))
        .stderr(predicate::str::contains("See 'wavepeek property --help'."));
}

#[test]
fn property_requires_eval_flag() {
    let fixture = fixture_path("m2_core.vcd");
    let fixture = fixture.to_string_lossy().into_owned();

    wavepeek_cmd()
        .args(["property", "--waves", fixture.as_str(), "--on", "*"])
        .assert()
        .failure()
        .code(1)
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::starts_with("error: args:"))
        .stderr(predicate::str::contains(
            "required arguments were not provided",
        ))
        .stderr(predicate::str::contains("--eval <EVAL>"))
        .stderr(predicate::str::contains("See 'wavepeek property --help'."));
}

#[test]
fn property_runtime_manifest_cases_pass() {
    let positive =
        common::command_cases::load_positive_manifest("command_runtime_positive_manifest.json");
    for case in &positive.cases {
        if case.command == common::command_cases::CommandCaseCommand::Property {
            common::command_cases::assert_positive_case(case);
        }
    }

    let negative =
        common::command_cases::load_negative_manifest("command_runtime_negative_manifest.json");
    for case in &negative.cases {
        if case.command == common::command_cases::CommandCaseCommand::Property {
            common::command_cases::assert_negative_case(case);
        }
    }
}
