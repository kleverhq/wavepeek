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

const RTL_SAMPLING_VCD: &str = concat!(
    "$date\n",
    "  today\n",
    "$end\n",
    "$version\n",
    "  wavepeek-rtl-sampling\n",
    "$end\n",
    "$timescale 1ns $end\n",
    "$scope module top $end\n",
    "$var wire 1 ! clk $end\n",
    "$var wire 1 \" valid $end\n",
    "$var wire 8 # data $end\n",
    "$upscope $end\n",
    "$enddefinitions $end\n",
    "#0\n",
    "0!\n",
    "0\"\n",
    "b00000000 #\n",
    "#5\n",
    "1!\n",
    "1\"\n",
    "b10101010 #\n",
    "#10\n",
    "0!\n",
    "#15\n",
    "1!\n",
    "#20\n",
    "0!\n",
    "#25\n",
    "1!\n",
    "0\"\n",
    "b01010101 #\n",
    "#30\n",
    "0!\n",
    "#35\n",
    "1!\n",
);

fn many_property_matches_vcd(edge_count: u32) -> String {
    let mut vcd = String::from(
        "$date\n  today\n$end\n$version\n  wavepeek-many-property-matches\n$end\n$timescale 1ns $end\n$scope module top $end\n$var wire 1 ! clk $end\n$var wire 1 \" sig $end\n$upscope $end\n$enddefinitions $end\n#0\n0!\n1\"\n",
    );
    for timestamp in 1..=edge_count {
        vcd.push_str(&format!("#{}\n{}!\n", timestamp, timestamp % 2));
    }
    vcd
}

#[test]
fn property_sample_mode_pre_edge_samples_before_trigger_edge() {
    let fixture = write_fixture(RTL_SAMPLING_VCD, "property-rtl-sampling.vcd");
    let fixture = fixture.path().to_string_lossy().into_owned();

    let native_output = wavepeek_cmd()
        .args([
            "property",
            "--waves",
            fixture.as_str(),
            "--from",
            "0ns",
            "--to",
            "20ns",
            "--scope",
            "top",
            "--on",
            "posedge clk",
            "--eval",
            "valid",
            "--capture",
            "assert",
            "--sample-mode",
            "native",
            "--json",
        ])
        .output()
        .expect("property should execute");
    let pre_edge_posedge_output = wavepeek_cmd()
        .args([
            "property",
            "--waves",
            fixture.as_str(),
            "--from",
            "0ns",
            "--to",
            "20ns",
            "--scope",
            "top",
            "--on",
            "posedge clk",
            "--eval",
            "valid",
            "--capture",
            "assert",
            "--sample-mode",
            "pre-edge",
            "--json",
        ])
        .output()
        .expect("property should execute");
    let pre_edge_edge_output = wavepeek_cmd()
        .args([
            "property",
            "--waves",
            fixture.as_str(),
            "--from",
            "0ns",
            "--to",
            "20ns",
            "--scope",
            "top",
            "--on",
            "edge clk",
            "--eval",
            "valid",
            "--capture",
            "assert",
            "--sample-mode",
            "pre-edge",
            "--json",
        ])
        .output()
        .expect("property should execute");
    let pre_edge_human_output = wavepeek_cmd()
        .args([
            "property",
            "--waves",
            fixture.as_str(),
            "--from",
            "0ns",
            "--to",
            "20ns",
            "--scope",
            "top",
            "--on",
            "posedge clk",
            "--eval",
            "valid",
            "--capture",
            "assert",
            "--sample-mode",
            "pre-edge",
        ])
        .output()
        .expect("property should execute");

    assert!(native_output.status.success());
    assert!(pre_edge_posedge_output.status.success());
    assert!(pre_edge_edge_output.status.success());
    assert!(pre_edge_human_output.status.success());
    assert_eq!(
        parse_json(&native_output.stdout)["data"],
        json!([{"time": "5ns", "sample_time": "5ns", "kind": "assert"}])
    );
    assert_eq!(
        parse_json(&pre_edge_posedge_output.stdout)["data"],
        json!([{"time": "15ns", "sample_time": "14ns", "kind": "assert"}])
    );
    assert_eq!(
        parse_json(&pre_edge_edge_output.stdout)["data"],
        json!([{"time": "10ns", "sample_time": "9ns", "kind": "assert"}])
    );
    assert_eq!(
        String::from_utf8(pre_edge_human_output.stdout).expect("human stdout should be UTF-8"),
        "@15ns sample@14ns assert\n"
    );
}

#[test]
fn property_sample_mode_pre_edge_preserves_from_baseline() {
    let fixture = write_fixture(RTL_SAMPLING_VCD, "property-rtl-sampling-boundary.vcd");
    let fixture = fixture.path().to_string_lossy().into_owned();

    let assert_output = wavepeek_cmd()
        .args([
            "property",
            "--waves",
            fixture.as_str(),
            "--from",
            "5ns",
            "--to",
            "20ns",
            "--scope",
            "top",
            "--on",
            "posedge clk",
            "--eval",
            "valid",
            "--capture",
            "assert",
            "--sample-mode",
            "pre-edge",
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
            "5ns",
            "--to",
            "35ns",
            "--scope",
            "top",
            "--on",
            "posedge clk",
            "--eval",
            "valid",
            "--capture",
            "deassert",
            "--sample-mode",
            "pre-edge",
            "--json",
        ])
        .output()
        .expect("property should execute");

    assert!(assert_output.status.success());
    let assert_json = parse_json(&assert_output.stdout);
    assert_eq!(assert_json["data"], json!([]));
    assert_eq!(
        assert_json["diagnostics"],
        json!([{"kind": "warning", "code": "WPK-W0003", "message": "no property matches found in selected time range"}])
    );
    assert!(deassert_output.status.success());
    assert_eq!(
        parse_json(&deassert_output.stdout)["data"],
        json!([{"time": "35ns", "sample_time": "34ns", "kind": "deassert"}])
    );
}

#[test]
fn property_sample_mode_pre_edge_skips_from_boundary_before_eval() {
    let fixture = write_fixture(
        concat!(
            "$date\n",
            "  today\n",
            "$end\n",
            "$version\n",
            "  wavepeek-rtl-sampling-boundary\n",
            "$end\n",
            "$timescale 1ns $end\n",
            "$scope module top $end\n",
            "$var wire 1 ! clk $end\n",
            "$var wire 1 \" sig $end\n",
            "$upscope $end\n",
            "$enddefinitions $end\n",
            "#0\n",
            "0!\n",
            "x\"\n",
            "#5\n",
            "1!\n",
            "1\"\n",
        ),
        "property-rtl-sampling-prewindow-error.vcd",
    );
    let fixture = fixture.path().to_string_lossy().into_owned();

    let output = wavepeek_cmd()
        .args([
            "property",
            "--waves",
            fixture.as_str(),
            "--from",
            "5ns",
            "--to",
            "5ns",
            "--scope",
            "top",
            "--on",
            "posedge clk",
            "--eval",
            "real'(sig) == 1.0",
            "--capture",
            "assert",
            "--sample-mode",
            "pre-edge",
            "--json",
        ])
        .output()
        .expect("property should execute");

    assert!(output.status.success());
    let parsed = parse_json(&output.stdout);
    assert_eq!(parsed["data"], json!([]));
    assert_eq!(
        parsed["diagnostics"],
        json!([{"kind": "warning", "code": "WPK-W0003", "message": "no property matches found in selected time range"}])
    );
}

#[test]
fn property_sample_mode_pre_edge_does_not_replay_previous_raw_event() {
    let fixture = write_fixture(
        concat!(
            "$date\n",
            "  today\n",
            "$end\n",
            "$version\n",
            "  wavepeek-rtl-sampling-event\n",
            "$end\n",
            "$timescale 1ns $end\n",
            "$scope module top $end\n",
            "$var wire 1 ! clk $end\n",
            "$var event 1 # ev $end\n",
            "$upscope $end\n",
            "$enddefinitions $end\n",
            "#0\n",
            "0!\n",
            "#5\n",
            "1#\n",
            "#10\n",
            "1!\n",
            "#15\n",
            "0!\n",
            "#19\n",
            "1#\n",
            "#20\n",
            "1!\n",
        ),
        "property-rtl-sampling-event.vcd",
    );
    let fixture = fixture.path().to_string_lossy().into_owned();

    let output = wavepeek_cmd()
        .args([
            "property",
            "--waves",
            fixture.as_str(),
            "--from",
            "0ns",
            "--to",
            "20ns",
            "--scope",
            "top",
            "--on",
            "posedge clk",
            "--eval",
            "ev.triggered()",
            "--capture",
            "match",
            "--sample-mode",
            "pre-edge",
            "--json",
        ])
        .output()
        .expect("property should execute");

    assert!(output.status.success());
    assert_eq!(
        parse_json(&output.stdout)["data"],
        json!([{"time": "20ns", "sample_time": "19ns", "kind": "match"}])
    );
}

#[test]
fn property_sample_mode_pre_edge_rejects_non_edge_triggers() {
    let fixture = write_fixture(RTL_SAMPLING_VCD, "property-rtl-sampling-invalid.vcd");
    let fixture = fixture.path().to_string_lossy().into_owned();
    let invalid_on_args: &[&[&str]] = &[
        &["--on", "*"],
        &["--on", "valid"],
        &["--on", "valid or posedge clk"],
    ];

    for on_args in invalid_on_args {
        let mut args = vec![
            "property",
            "--waves",
            fixture.as_str(),
            "--scope",
            "top",
            "--eval",
            "valid",
            "--sample-mode",
            "pre-edge",
        ];
        args.extend_from_slice(on_args);

        wavepeek_cmd()
            .args(args)
            .assert()
            .failure()
            .code(1)
            .stdout(predicate::str::is_empty())
            .stderr(predicate::str::starts_with("fatal: args:"))
            .stderr(predicate::str::contains(
                "--sample-mode pre-edge requires --on with only edge event terms",
            ));
    }
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
            "--sample-mode",
            "native",
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
            "--sample-mode",
            "native",
        ])
        .output()
        .expect("property should execute");

    assert!(json_output.status.success());
    assert!(human_output.status.success());
    assert!(json_output.stderr.is_empty());
    assert!(human_output.stderr.is_empty());

    let json = parse_json(&json_output.stdout);
    assert_eq!(json["command"], "property");
    assert_eq!(json["diagnostics"], json!([]));
    assert_eq!(
        json["data"],
        json!([
            {"time": "15ns", "sample_time": "15ns", "kind": "assert"},
            {"time": "25ns", "sample_time": "25ns", "kind": "deassert"}
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
            "--sample-mode",
            "native",
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
            "--sample-mode",
            "native",
            "--json",
        ])
        .output()
        .expect("property should execute");

    assert!(assert_output.status.success());
    assert!(deassert_output.status.success());
    assert_eq!(
        parse_json(&assert_output.stdout)["data"],
        json!([{"time": "15ns", "sample_time": "15ns", "kind": "assert"}])
    );
    assert_eq!(
        parse_json(&deassert_output.stdout)["data"],
        json!([{"time": "25ns", "sample_time": "25ns", "kind": "deassert"}])
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
            "--sample-mode",
            "native",
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
            "--sample-mode",
            "native",
            "--json",
        ])
        .output()
        .expect("property should execute");

    assert!(multibit_output.status.success());
    assert!(real_output.status.success());
    assert_eq!(
        parse_json(&multibit_output.stdout)["data"],
        json!([{"time": "5ns", "sample_time": "5ns", "kind": "match"}])
    );
    assert_eq!(
        parse_json(&real_output.stdout)["data"],
        json!([{"time": "5ns", "sample_time": "5ns", "kind": "match"}])
    );
}

#[test]
fn property_default_max_is_50_with_truncation_warning() {
    let fixture = write_fixture(&many_property_matches_vcd(60), ".property-many-matches.vcd");
    let fixture = fixture.path().to_string_lossy().into_owned();

    let output = wavepeek_cmd()
        .args([
            "property",
            "--waves",
            fixture.as_str(),
            "--scope",
            "top",
            "--on",
            "edge clk",
            "--eval",
            "sig",
            "--capture",
            "match",
            "--sample-mode",
            "native",
            "--json",
        ])
        .output()
        .expect("property should execute");

    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let value = parse_json(&output.stdout);
    let rows = value["data"].as_array().expect("data should be array");
    assert_eq!(rows.len(), 50);
    assert_eq!(rows[0]["time"], "1ns");
    assert_eq!(rows[49]["time"], "50ns");
    assert_eq!(
        value["diagnostics"],
        json!([{"kind": "warning", "code": "WPK-W0002", "message": "truncated output to 50 entries (use --max to increase limit)"}])
    );
}

#[test]
fn property_max_one_truncates_in_human_and_json_modes() {
    let fixture = write_fixture(&many_property_matches_vcd(60), ".property-max-one.vcd");
    let fixture = fixture.path().to_string_lossy().into_owned();

    let json_output = wavepeek_cmd()
        .args([
            "property",
            "--waves",
            fixture.as_str(),
            "--scope",
            "top",
            "--on",
            "edge clk",
            "--eval",
            "sig",
            "--capture",
            "match",
            "--max",
            "1",
            "--sample-mode",
            "native",
            "--json",
        ])
        .output()
        .expect("json property should execute");
    let human_output = wavepeek_cmd()
        .args([
            "property",
            "--waves",
            fixture.as_str(),
            "--scope",
            "top",
            "--on",
            "edge clk",
            "--eval",
            "sig",
            "--capture",
            "match",
            "--max",
            "1",
            "--sample-mode",
            "native",
        ])
        .output()
        .expect("human property should execute");

    assert!(json_output.status.success());
    assert!(human_output.status.success());
    assert_eq!(
        String::from_utf8_lossy(&human_output.stdout).trim(),
        "@1ns match"
    );
    assert_eq!(
        String::from_utf8_lossy(&human_output.stderr).trim(),
        "warning[WPK-W0002]: truncated output to 1 entries (use --max to increase limit)"
    );

    let value = parse_json(&json_output.stdout);
    assert_eq!(
        value["data"]
            .as_array()
            .expect("data should be array")
            .len(),
        1
    );
    assert_eq!(value["data"][0]["time"], "1ns");
    assert_eq!(
        value["diagnostics"],
        json!([{"kind": "warning", "code": "WPK-W0002", "message": "truncated output to 1 entries (use --max to increase limit)"}])
    );
}

#[test]
fn property_unlimited_max_disables_truncation_and_emits_warning_in_both_modes() {
    let fixture = write_fixture(&many_property_matches_vcd(60), ".property-unlimited.vcd");
    let fixture = fixture.path().to_string_lossy().into_owned();

    let json_output = wavepeek_cmd()
        .args([
            "property",
            "--waves",
            fixture.as_str(),
            "--scope",
            "top",
            "--on",
            "edge clk",
            "--eval",
            "sig",
            "--capture",
            "match",
            "--max",
            "unlimited",
            "--sample-mode",
            "native",
            "--json",
        ])
        .output()
        .expect("json property should execute");
    let human_output = wavepeek_cmd()
        .args([
            "property",
            "--waves",
            fixture.as_str(),
            "--scope",
            "top",
            "--on",
            "edge clk",
            "--eval",
            "sig",
            "--capture",
            "match",
            "--max",
            "unlimited",
            "--sample-mode",
            "native",
        ])
        .output()
        .expect("human property should execute");

    assert!(json_output.status.success());
    assert!(human_output.status.success());

    let value = parse_json(&json_output.stdout);
    assert_eq!(
        value["data"]
            .as_array()
            .expect("data should be array")
            .len(),
        60
    );
    assert_eq!(
        value["diagnostics"],
        json!([{"kind": "warning", "code": "WPK-W0001", "message": "limit disabled: --max=unlimited"}])
    );
    assert_eq!(
        String::from_utf8_lossy(&human_output.stdout)
            .lines()
            .count(),
        60
    );
    assert_eq!(
        String::from_utf8_lossy(&human_output.stderr).trim(),
        "warning[WPK-W0001]: limit disabled: --max=unlimited"
    );
}

#[test]
fn property_rejects_zero_max() {
    wavepeek_cmd()
        .args([
            "property",
            "--waves",
            "missing.vcd",
            "--scope",
            "top",
            "--on",
            "edge clk",
            "--eval",
            "sig",
            "--max",
            "0",
            "--sample-mode",
            "native",
        ])
        .assert()
        .failure()
        .code(1)
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::starts_with(
            "fatal: args: --max must be greater than 0.",
        ));
}

#[test]
fn property_empty_result_emits_empty_result_diagnostic() {
    let fixture = fixture_path("m2_core.vcd");
    let fixture = fixture.to_string_lossy().into_owned();

    let json_output = wavepeek_cmd()
        .args([
            "property",
            "--waves",
            fixture.as_str(),
            "--scope",
            "top",
            "--on",
            "posedge clk",
            "--eval",
            "data == 8'hff",
            "--capture",
            "match",
            "--sample-mode",
            "native",
            "--json",
        ])
        .output()
        .expect("json property run should execute");
    let human_output = wavepeek_cmd()
        .args([
            "property",
            "--waves",
            fixture.as_str(),
            "--scope",
            "top",
            "--on",
            "posedge clk",
            "--eval",
            "data == 8'hff",
            "--capture",
            "match",
            "--sample-mode",
            "native",
        ])
        .output()
        .expect("human property run should execute");

    assert!(json_output.status.success());
    assert!(human_output.status.success());
    assert!(human_output.stdout.is_empty());
    let json = parse_json(&json_output.stdout);
    assert_eq!(json["data"], json!([]));
    assert_eq!(
        json["diagnostics"],
        json!([{"kind": "warning", "code": "WPK-W0003", "message": "no property matches found in selected time range"}])
    );
    assert_eq!(
        String::from_utf8_lossy(&human_output.stderr).trim(),
        "warning[WPK-W0003]: no property matches found in selected time range"
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
            "--sample-mode",
            "native",
        ])
        .assert()
        .failure()
        .code(1)
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::starts_with("fatal: expr:"))
        .stderr(predicate::str::contains("EXPR-PARSE-LOGICAL-EXPECTED"));
}

#[test]
fn property_explicit_wildcard_tracks_eval_signal_changes() {
    let fixture = fixture_path("m2_core.vcd");
    let fixture = fixture.to_string_lossy().into_owned();

    let output = wavepeek_cmd()
        .args([
            "property",
            "--on",
            "*",
            "--sample-mode",
            "native",
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
    assert_eq!(json["diagnostics"], json!([]));
    assert_eq!(
        json["data"],
        json!([{"time": "10ns", "sample_time": "10ns", "kind": "match"}])
    );
}

#[test]
fn property_explicit_wildcard_tracks_raw_event_handles_from_eval() {
    let fixture = write_fixture(
        "$date\n  today\n$end\n$version\n  wavepeek-test\n$end\n$timescale 1ns $end\n$scope module top $end\n$var event 1 ! ev $end\n$scope module ev $end\n$var wire 1 \" triggered $end\n$upscope $end\n$upscope $end\n$enddefinitions $end\n#0\n0\"\n#5\n1!\n1\"\n#10\n0\"\n#20\n1!\n1\"\n#25\n0\"\n",
        "property-event-track.vcd",
    );
    let fixture = fixture.path().to_string_lossy().into_owned();

    let output = wavepeek_cmd()
        .args([
            "property",
            "--on",
            "*",
            "--sample-mode",
            "native",
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
    assert_eq!(json["diagnostics"], json!([]));
    assert_eq!(
        json["data"],
        json!([
            {"time": "5ns", "sample_time": "5ns", "kind": "match"},
            {"time": "20ns", "sample_time": "20ns", "kind": "match"}
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
            "--sample-mode",
            "native",
            "--json",
        ])
        .output()
        .expect("property should execute");

    assert!(output.status.success());
    assert_eq!(
        parse_json(&output.stdout)["data"],
        json!([{"time": "5ns", "sample_time": "5ns", "kind": "match"}])
    );
}

#[test]
fn property_requires_on_flag() {
    let fixture = fixture_path("m2_core.vcd");
    let fixture = fixture.to_string_lossy().into_owned();

    wavepeek_cmd()
        .args(["property", "--waves", fixture.as_str(), "--eval", "1"])
        .assert()
        .failure()
        .code(1)
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::starts_with("fatal: args:"))
        .stderr(predicate::str::contains("--on <ON>"))
        .stderr(predicate::str::contains("See 'wavepeek property --help'."));
}

#[test]
fn property_explicit_wildcard_signal_free_eval_reports_tracking_error() {
    let fixture = fixture_path("m2_core.vcd");
    let fixture = fixture.to_string_lossy().into_owned();

    wavepeek_cmd()
        .args([
            "property",
            "--waves",
            fixture.as_str(),
            "--on",
            "*",
            "--sample-mode",
            "native",
            "--eval",
            "1",
        ])
        .assert()
        .failure()
        .code(1)
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::starts_with("fatal: args:"))
        .stderr(predicate::str::contains(
            "wildcard trigger cannot infer tracked signals from --eval",
        ));
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
        .stderr(predicate::str::starts_with("fatal: expr:"))
        .stderr(predicate::str::contains("unknown signal 'top.clk'"));
}

#[test]
fn property_rejects_legacy_surface_flags() {
    let fixture = fixture_path("m2_core.vcd");
    let fixture = fixture.to_string_lossy().into_owned();

    wavepeek_cmd()
        .args([
            "property",
            "--on",
            "*",
            "--sample-mode",
            "native",
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
        .stderr(predicate::str::starts_with("fatal: args:"))
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
            "--sample-mode",
            "native",
        ])
        .assert()
        .failure()
        .code(1)
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::starts_with("fatal: args:"))
        .stderr(predicate::str::contains("unexpected argument '--cond'"))
        .stderr(predicate::str::contains("See 'wavepeek property --help'."));

    wavepeek_cmd()
        .args([
            "property",
            "--on",
            "*",
            "--sample-mode",
            "native",
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
        .stderr(predicate::str::starts_with("fatal: args:"))
        .stderr(predicate::str::contains("unexpected argument '--when'"))
        .stderr(predicate::str::contains("See 'wavepeek property --help'."));
}

#[test]
fn property_requires_eval_flag() {
    let fixture = fixture_path("m2_core.vcd");
    let fixture = fixture.to_string_lossy().into_owned();

    wavepeek_cmd()
        .args([
            "property",
            "--waves",
            fixture.as_str(),
            "--on",
            "*",
            "--sample-mode",
            "native",
        ])
        .assert()
        .failure()
        .code(1)
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::starts_with("fatal: args:"))
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
