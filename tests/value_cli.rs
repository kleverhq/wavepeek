use std::fs;

use assert_cmd::prelude::*;
use predicates::prelude::*;
use serde_json::{Value, json};
use tempfile::NamedTempFile;

mod common;
use common::{fixture_path, wavepeek_cmd};

fn write_fixture(contents: &str, suffix: &str) -> NamedTempFile {
    let fixture = NamedTempFile::with_suffix(suffix).expect("temp fixture should create");
    fs::write(fixture.path(), contents).expect("fixture should write");
    fixture
}

#[test]
fn value_human_output_with_scope_is_default() {
    let fixture = fixture_path("signal_recursive_depth.vcd");
    let fixture = fixture.to_string_lossy().into_owned();

    let mut command = wavepeek_cmd();
    command
        .args([
            "value",
            "--waves",
            fixture.as_str(),
            "--at",
            "10ns",
            "--scope",
            "top.cpu",
            "--signals",
            "valid,core.execute,top.cpu.valid,top.cpu.core.execute",
        ])
        .assert()
        .success()
        .stdout(predicate::eq(
            "@10ns valid=1'h1 core.execute=1'h1 valid=1'h1 core.execute=1'h1\n",
        ))
        .stderr(predicate::str::is_empty());
}

#[test]
fn value_human_output_with_abs_shows_canonical_paths() {
    let fixture = fixture_path("signal_recursive_depth.vcd");
    let fixture = fixture.to_string_lossy().into_owned();

    let mut command = wavepeek_cmd();
    command
        .args([
            "value",
            "--waves",
            fixture.as_str(),
            "--at",
            "10ns",
            "--scope",
            "top.cpu",
            "--signals",
            "valid,core.execute,top.cpu.valid,top.cpu.core.execute",
            "--abs",
        ])
        .assert()
        .success()
        .stdout(predicate::eq(
            "@10ns top.cpu.valid=1'h1 top.cpu.core.execute=1'h1 top.cpu.valid=1'h1 top.cpu.core.execute=1'h1\n",
        ))
        .stderr(predicate::str::is_empty());
}

#[test]
fn value_requires_signals_flag() {
    let fixture = fixture_path("m2_core.vcd");
    let fixture = fixture.to_string_lossy().into_owned();

    let mut command = wavepeek_cmd();
    command
        .args(["value", "--waves", fixture.as_str(), "--at", "10ns"])
        .assert()
        .failure()
        .code(1)
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::starts_with("fatal: args:"))
        .stderr(predicate::str::contains("--signals <SIGNALS>"))
        .stderr(predicate::str::contains("See 'wavepeek value --help'."));
}

#[test]
fn value_json_shape_with_scope_is_stable_and_ordered() {
    let fixture = fixture_path("m2_core.vcd");
    let fixture = fixture.to_string_lossy().into_owned();

    let mut command = wavepeek_cmd();
    let assert = command
        .args([
            "value",
            "--waves",
            fixture.as_str(),
            "--at",
            "10ns",
            "--scope",
            "top",
            "--signals",
            "clk,data",
            "--json",
        ])
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&assert.get_output().stdout).to_string();
    let value: Value = serde_json::from_str(&stdout).expect("value output should be valid json");

    assert_eq!(value["command"], "value");
    assert_eq!(value["context"]["scope"], "top");
    assert_eq!(value["diagnostics"], Value::Array(vec![]));
    assert_eq!(
        value["data"],
        json!([
            {
                "time": "10ns",
                "signals": [
                    {"path": "top.clk", "relative_path": "clk", "value": "1'h1"},
                    {"path": "top.data", "relative_path": "data", "value": "8'h0f"}
                ]
            }
        ])
    );
}

#[test]
fn value_human_output_accepts_comma_separated_times() {
    let fixture = fixture_path("m2_core.vcd");
    let fixture = fixture.to_string_lossy().into_owned();

    let mut command = wavepeek_cmd();
    command
        .args([
            "value",
            "--waves",
            fixture.as_str(),
            "--at",
            "5ns,10ns",
            "--scope",
            "top",
            "--signals",
            "clk,data",
        ])
        .assert()
        .success()
        .stdout(predicate::eq(
            "@5ns clk=1'h1 data=8'h00\n@10ns clk=1'h1 data=8'h0f\n",
        ))
        .stderr(predicate::str::is_empty());
}

#[test]
fn value_json_flattens_repeated_and_comma_separated_lists_in_order() {
    let fixture = fixture_path("m2_core.vcd");
    let fixture = fixture.to_string_lossy().into_owned();

    let mut command = wavepeek_cmd();
    let assert = command
        .args([
            "value",
            "--waves",
            fixture.as_str(),
            "--at",
            "10ns,5ns",
            "--at",
            "10ns",
            "--scope",
            "top",
            "--signals",
            "clk",
            "--signals",
            "data,clk",
            "--json",
        ])
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&assert.get_output().stdout).to_string();
    let value: Value = serde_json::from_str(&stdout).expect("value output should be valid json");

    assert_eq!(
        value["data"],
        json!([
            {
                "time": "10ns",
                "signals": [
                    {"path": "top.clk", "relative_path": "clk", "value": "1'h1"},
                    {"path": "top.data", "relative_path": "data", "value": "8'h0f"},
                    {"path": "top.clk", "relative_path": "clk", "value": "1'h1"}
                ]
            },
            {
                "time": "5ns",
                "signals": [
                    {"path": "top.clk", "relative_path": "clk", "value": "1'h1"},
                    {"path": "top.data", "relative_path": "data", "value": "8'h00"},
                    {"path": "top.clk", "relative_path": "clk", "value": "1'h1"}
                ]
            },
            {
                "time": "10ns",
                "signals": [
                    {"path": "top.clk", "relative_path": "clk", "value": "1'h1"},
                    {"path": "top.data", "relative_path": "data", "value": "8'h0f"},
                    {"path": "top.clk", "relative_path": "clk", "value": "1'h1"}
                ]
            }
        ])
    );
}

#[test]
fn value_rejects_empty_time_list_entry() {
    let fixture = fixture_path("m2_core.vcd");
    let fixture = fixture.to_string_lossy().into_owned();

    let mut command = wavepeek_cmd();
    command
        .args([
            "value",
            "--waves",
            fixture.as_str(),
            "--at",
            "5ns,,10ns",
            "--scope",
            "top",
            "--signals",
            "clk",
        ])
        .assert()
        .failure()
        .code(1)
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::starts_with("fatal: args:"))
        .stderr(predicate::str::contains(
            "time list in --at must not contain empty entries",
        ))
        .stderr(predicate::str::contains("See 'wavepeek value --help'."));
}

#[test]
fn value_without_scope_treats_signals_as_canonical_paths() {
    let fixture = fixture_path("m2_core.vcd");
    let fixture = fixture.to_string_lossy().into_owned();

    let mut command = wavepeek_cmd();
    let assert = command
        .args([
            "value",
            "--waves",
            fixture.as_str(),
            "--at",
            "10ns",
            "--signals",
            "top.clk,top.data",
            "--json",
        ])
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&assert.get_output().stdout).to_string();
    let value: Value = serde_json::from_str(&stdout).expect("value output should be valid json");

    assert!(value.get("context").is_none());
    assert_eq!(
        value["data"][0]["signals"],
        json!([
            {"path": "top.clk", "value": "1'h1"},
            {"path": "top.data", "value": "8'h0f"}
        ])
    );
}

#[test]
fn value_json_output_is_identical_with_and_without_abs() {
    let fixture = fixture_path("m2_core.vcd");
    let fixture = fixture.to_string_lossy().into_owned();

    let without_abs = wavepeek_cmd()
        .args([
            "value",
            "--waves",
            fixture.as_str(),
            "--at",
            "10ns",
            "--scope",
            "top",
            "--signals",
            "clk,data",
            "--json",
        ])
        .output()
        .expect("run without --abs should execute");
    let with_abs = wavepeek_cmd()
        .args([
            "value",
            "--waves",
            fixture.as_str(),
            "--at",
            "10ns",
            "--scope",
            "top",
            "--signals",
            "clk,data",
            "--json",
            "--abs",
        ])
        .output()
        .expect("run with --abs should execute");

    assert!(without_abs.status.success());
    assert!(with_abs.status.success());
    assert_eq!(without_abs.stdout, with_abs.stdout);
    assert_eq!(without_abs.stderr, with_abs.stderr);
}

#[test]
fn value_invalid_time_token_is_args_error() {
    let fixture = fixture_path("m2_core.vcd");
    let fixture = fixture.to_string_lossy().into_owned();

    let mut command = wavepeek_cmd();
    command
        .args([
            "value",
            "--waves",
            fixture.as_str(),
            "--at",
            "100",
            "--scope",
            "top",
            "--signals",
            "clk",
        ])
        .assert()
        .failure()
        .code(1)
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::starts_with("fatal: args:"))
        .stderr(predicate::str::contains("invalid time token '100'"))
        .stderr(predicate::str::contains("See 'wavepeek value --help'."));
}

#[test]
fn value_decimal_time_token_is_rejected_as_args_error() {
    let fixture = fixture_path("m2_core.vcd");
    let fixture = fixture.to_string_lossy().into_owned();

    let mut command = wavepeek_cmd();
    command
        .args([
            "value",
            "--waves",
            fixture.as_str(),
            "--at",
            "1.5ns",
            "--scope",
            "top",
            "--signals",
            "clk",
        ])
        .assert()
        .failure()
        .code(1)
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::starts_with("fatal: args:"))
        .stderr(predicate::str::contains("invalid time token '1.5ns'"))
        .stderr(predicate::str::contains("See 'wavepeek value --help'."));
}

#[test]
fn value_out_of_range_time_is_args_error_with_bounds() {
    let fixture = fixture_path("m2_core.vcd");
    let fixture = fixture.to_string_lossy().into_owned();

    let mut command = wavepeek_cmd();
    command
        .args([
            "value",
            "--waves",
            fixture.as_str(),
            "--at",
            "11ns",
            "--scope",
            "top",
            "--signals",
            "clk",
        ])
        .assert()
        .failure()
        .code(1)
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::starts_with("fatal: args:"))
        .stderr(predicate::str::contains(
            "time '11ns' is outside dump bounds [0ns, 10ns]",
        ))
        .stderr(predicate::str::contains("See 'wavepeek value --help'."));
}

#[test]
fn value_scope_not_found_is_scope_error() {
    let fixture = fixture_path("m2_core.vcd");
    let fixture = fixture.to_string_lossy().into_owned();

    let mut command = wavepeek_cmd();
    command
        .args([
            "value",
            "--waves",
            fixture.as_str(),
            "--at",
            "10ns",
            "--scope",
            "top.nope",
            "--signals",
            "clk",
        ])
        .assert()
        .failure()
        .code(1)
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::starts_with("fatal: scope:"));
}

#[test]
fn value_missing_signal_is_signal_error_and_fails_fast() {
    let fixture = fixture_path("m2_core.vcd");
    let fixture = fixture.to_string_lossy().into_owned();

    let output = wavepeek_cmd()
        .args([
            "value",
            "--waves",
            fixture.as_str(),
            "--at",
            "10ns",
            "--signals",
            "top.nope,top.clk",
            "--json",
        ])
        .output()
        .expect("value should execute");
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stderr.is_empty());
    let fatal: Value = serde_json::from_slice(&output.stdout).expect("fatal should be JSON");
    assert_eq!(fatal["type"], "fatal");
    assert_eq!(fatal["code"], "WPK-F0004");
    assert!(fatal["message"].as_str().unwrap().contains(
        "no dumped signal with basename 'nope'; the RTL declaration may be optimized, aliased, or not dumped"
    ));

    let output = wavepeek_cmd()
        .args([
            "value",
            "--waves",
            fixture.as_str(),
            "--at",
            "10ns",
            "--signals",
            "top.nope",
            "--jsonl",
        ])
        .output()
        .expect("value should execute");
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stderr.is_empty());
    let fatal: Value = serde_json::from_slice(&output.stdout).expect("fatal should be JSONL");
    assert_eq!(fatal["type"], "fatal");
    assert_eq!(fatal["seq"], 0);
    assert_eq!(fatal["code"], "WPK-F0004");
}

#[test]
fn value_missing_path_suggests_copyable_names_in_current_naming_mode() {
    for fixture_name in ["m2_core.vcd", "m2_core.fst"] {
        let fixture = fixture_path(fixture_name);
        let fixture = fixture.to_string_lossy().into_owned();

        wavepeek_cmd()
            .args([
                "value",
                "--waves",
                fixture.as_str(),
                "--at",
                "10ns",
                "--scope",
                "top",
                "--signals",
                "valid",
            ])
            .assert()
            .failure()
            .code(1)
            .stdout(predicate::str::is_empty())
            .stderr(predicate::str::contains(
                "signal 'valid' not found under scope 'top'\nclosest query names:\n  cpu.valid",
            ));

        wavepeek_cmd()
            .args([
                "value",
                "--waves",
                fixture.as_str(),
                "--at",
                "10ns",
                "--signals",
                "wrong.valid",
            ])
            .assert()
            .failure()
            .code(1)
            .stdout(predicate::str::is_empty())
            .stderr(predicate::str::contains(
                "signal 'wrong.valid' not found in dump\nclosest query names:\n  top.cpu.valid",
            ));
    }
}

#[test]
fn value_signal_typo_suggests_close_basename() {
    let fixture = fixture_path("m2_core.vcd");
    let fixture = fixture.to_string_lossy().into_owned();

    wavepeek_cmd()
        .args([
            "value",
            "--waves",
            fixture.as_str(),
            "--at",
            "10ns",
            "--scope",
            "top",
            "--signals",
            "vlaid",
        ])
        .assert()
        .failure()
        .code(1)
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::contains(
            "closest query names:\n  cpu.valid",
        ));

    let event_fixture = fixture_path("change_property_events.vcd");
    let event_fixture = event_fixture.to_string_lossy().into_owned();
    wavepeek_cmd()
        .args([
            "value",
            "--waves",
            event_fixture.as_str(),
            "--at",
            "5ns",
            "--scope",
            "top",
            "--signals",
            "tik",
        ])
        .assert()
        .failure()
        .code(1)
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::contains("closest query names:\n  tick"));
}

#[test]
fn value_signal_suggestions_are_sorted_and_bounded() {
    let fixture = NamedTempFile::with_suffix(".vcd").expect("fixture should create");
    let scopes = ["foxtrot", "echo", "delta", "charlie", "bravo", "alpha"];
    let mut vcd = String::from(
        "$date\n  today\n$end\n$version\n  suggestions\n$end\n$timescale 1ns $end\n$scope module top $end\n",
    );
    for (index, scope) in scopes.iter().enumerate() {
        vcd.push_str(&format!(
            "$scope module {scope} $end\n$var wire 1 s{index} valid $end\n$upscope $end\n"
        ));
    }
    vcd.push_str("$upscope $end\n$enddefinitions $end\n#0\n");
    for index in 0..scopes.len() {
        vcd.push_str(&format!("0s{index}\n"));
    }
    fs::write(fixture.path(), vcd).expect("fixture should write");
    let fixture = fixture.path().to_string_lossy().into_owned();

    wavepeek_cmd()
        .args([
            "value",
            "--waves",
            fixture.as_str(),
            "--at",
            "0ns",
            "--scope",
            "top",
            "--signals",
            "valid",
        ])
        .assert()
        .failure()
        .code(1)
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::contains(
            "closest query names:\n  alpha.valid\n  bravo.valid\n  charlie.valid\n  delta.valid\n  echo.valid",
        ))
        .stderr(predicate::str::contains("foxtrot.valid").not());
}

#[test]
fn value_preserves_duplicate_signal_order() {
    let fixture = fixture_path("m2_core.vcd");
    let fixture = fixture.to_string_lossy().into_owned();

    let mut command = wavepeek_cmd();
    let assert = command
        .args([
            "value",
            "--waves",
            fixture.as_str(),
            "--at",
            "10ns",
            "--scope",
            "top",
            "--signals",
            "clk,clk,data",
            "--json",
        ])
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&assert.get_output().stdout).to_string();
    let value: Value = serde_json::from_str(&stdout).expect("value output should be valid json");

    assert_eq!(
        value["data"][0]["signals"],
        json!([
            {"path": "top.clk", "relative_path": "clk", "value": "1'h1"},
            {"path": "top.clk", "relative_path": "clk", "value": "1'h1"},
            {"path": "top.data", "relative_path": "data", "value": "8'h0f"}
        ])
    );
}

#[test]
fn value_mixed_mode_names_fail_without_scope() {
    let fixture = fixture_path("m2_core.vcd");
    let fixture = fixture.to_string_lossy().into_owned();

    let mut command = wavepeek_cmd();
    command
        .args([
            "value",
            "--waves",
            fixture.as_str(),
            "--at",
            "10ns",
            "--signals",
            "clk,top.data",
        ])
        .assert()
        .failure()
        .code(1)
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::starts_with("fatal: signal:"));
}

#[test]
fn value_scoped_descendant_name_resolves() {
    let fixture = fixture_path("m2_core.vcd");
    let fixture = fixture.to_string_lossy().into_owned();

    let output = wavepeek_cmd()
        .args([
            "value",
            "--waves",
            fixture.as_str(),
            "--at",
            "5ns",
            "--scope",
            "top",
            "--signals",
            "cpu.valid",
            "--json",
        ])
        .output()
        .expect("value should execute");

    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let value: Value = serde_json::from_slice(&output.stdout).expect("stdout should be json");
    assert_eq!(value["data"][0]["signals"][0]["path"], "top.cpu.valid");
}

#[test]
fn value_scope_accepts_mixed_relative_and_canonical_paths() {
    let fixture = fixture_path("m2_core.vcd");
    let fixture = fixture.to_string_lossy().into_owned();

    let output = wavepeek_cmd()
        .args([
            "value",
            "--waves",
            fixture.as_str(),
            "--at",
            "5ns",
            "--scope",
            "top",
            "--signals",
            "cpu.valid,top.clk",
            "--json",
        ])
        .output()
        .expect("value should execute");

    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let value: Value = serde_json::from_slice(&output.stdout).expect("stdout should be json");
    assert_eq!(
        value["data"][0]["signals"],
        json!([
            {"path": "top.cpu.valid", "relative_path": "cpu.valid", "value": "1'h1"},
            {"path": "top.clk", "relative_path": "clk", "value": "1'h1"}
        ])
    );
}

#[test]
fn value_scope_rejects_canonical_path_outside_scope() {
    let fixture = fixture_path("change_scope_ambiguous.vcd");
    let fixture = fixture.to_string_lossy().into_owned();

    wavepeek_cmd()
        .args([
            "value",
            "--waves",
            fixture.as_str(),
            "--at",
            "5ns",
            "--scope",
            "top.top",
            "--signals",
            "top.clk",
        ])
        .assert()
        .failure()
        .code(1)
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::starts_with("fatal: signal:"));
}

#[test]
fn value_accepts_inclusive_time_bounds() {
    let fixture = fixture_path("m2_core.vcd");
    let fixture = fixture.to_string_lossy().into_owned();

    wavepeek_cmd()
        .args([
            "value",
            "--waves",
            fixture.as_str(),
            "--at",
            "0ns",
            "--scope",
            "top",
            "--signals",
            "clk,data",
            "--json",
        ])
        .assert()
        .success();

    wavepeek_cmd()
        .args([
            "value",
            "--waves",
            fixture.as_str(),
            "--at",
            "10ns",
            "--scope",
            "top",
            "--signals",
            "clk,data",
            "--json",
        ])
        .assert()
        .success();
}

#[test]
fn value_json_is_deterministic_across_identical_runs() {
    let fixture = fixture_path("m2_core.vcd");
    let fixture = fixture.to_string_lossy().into_owned();

    let first = wavepeek_cmd()
        .args([
            "value",
            "--waves",
            fixture.as_str(),
            "--at",
            "10ns",
            "--scope",
            "top",
            "--signals",
            "clk,data",
            "--json",
        ])
        .output()
        .expect("first run should execute");
    let second = wavepeek_cmd()
        .args([
            "value",
            "--waves",
            fixture.as_str(),
            "--at",
            "10ns",
            "--scope",
            "top",
            "--signals",
            "clk,data",
            "--json",
        ])
        .output()
        .expect("second run should execute");

    assert!(first.status.success());
    assert!(second.status.success());
    assert_eq!(first.stdout, second.stdout);
    assert_eq!(first.stderr, second.stderr);
}

#[test]
fn value_vcd_and_fst_payloads_match() {
    let vcd_fixture = fixture_path("m2_core.vcd");
    let vcd_fixture = vcd_fixture.to_string_lossy().into_owned();
    let fst_fixture = fixture_path("m2_core.fst");
    let fst_fixture = fst_fixture.to_string_lossy().into_owned();

    let vcd_output = wavepeek_cmd()
        .args([
            "value",
            "--waves",
            vcd_fixture.as_str(),
            "--at",
            "10ns",
            "--scope",
            "top",
            "--signals",
            "clk,data",
            "--json",
        ])
        .output()
        .expect("vcd run should execute");
    let fst_output = wavepeek_cmd()
        .args([
            "value",
            "--waves",
            fst_fixture.as_str(),
            "--at",
            "10ns",
            "--scope",
            "top",
            "--signals",
            "clk,data",
            "--json",
        ])
        .output()
        .expect("fst run should execute");

    assert!(vcd_output.status.success());
    assert!(fst_output.status.success());

    let vcd_json: Value =
        serde_json::from_slice(&vcd_output.stdout).expect("vcd output should be valid json");
    let fst_json: Value =
        serde_json::from_slice(&fst_output.stdout).expect("fst output should be valid json");

    assert_eq!(vcd_json["data"], fst_json["data"]);
}

fn parse_debug_events(stderr: &[u8]) -> Vec<Value> {
    let stderr = String::from_utf8(stderr.to_vec()).expect("stderr should be utf8");
    let events = stderr
        .lines()
        .map(|line| serde_json::from_str::<Value>(line).expect("debug line should be json"))
        .collect::<Vec<_>>();
    assert!(!events.is_empty(), "expected debug events on stderr");
    for event in &events {
        assert_eq!(event["kind"], "debug");
        assert!(event["message"].is_string());
        assert!(event["timestamp_ns"].is_u64());
        assert!(event["details"].is_object());
        assert_eq!(event["details"]["command"], "value");
    }
    events
}

#[test]
fn value_debug_trace_writes_json_events_to_human_stderr() {
    let fixture = fixture_path("m2_core.vcd");
    let fixture = fixture.to_string_lossy().into_owned();

    let output = wavepeek_cmd()
        .env("DEBUG", "1")
        .args([
            "value",
            "--waves",
            fixture.as_str(),
            "--at",
            "10ns",
            "--scope",
            "top",
            "--signals",
            "clk",
        ])
        .output()
        .expect("value command should execute");

    assert!(output.status.success());
    assert_eq!(String::from_utf8_lossy(&output.stdout), "@10ns clk=1'h1\n");
    let events = parse_debug_events(&output.stderr);
    assert!(
        events
            .iter()
            .any(|event| event["message"] == "backend.open.start")
    );
    assert!(events.iter().any(|event| {
        event["message"] == "backend.open.done"
            && event["details"]["backend"] == "ondas"
            && event["details"]["format"] == "vcd"
    }));
    assert!(
        events
            .iter()
            .any(|event| event["message"] == "value.sample.done")
    );
    assert!(!String::from_utf8_lossy(&output.stderr).contains("1'h1"));
}

#[test]
fn value_debug_trace_keeps_json_stdout_envelope_unchanged() {
    let fixture = fixture_path("m2_core.vcd");
    let fixture = fixture.to_string_lossy().into_owned();

    let output = wavepeek_cmd()
        .env("DEBUG", "1")
        .args([
            "value",
            "--waves",
            fixture.as_str(),
            "--at",
            "10ns",
            "--scope",
            "top",
            "--signals",
            "clk",
            "--json",
        ])
        .output()
        .expect("value command should execute");

    assert!(output.status.success());
    let value: Value = serde_json::from_slice(&output.stdout).expect("stdout should be json");
    assert_eq!(value["command"], "value");
    assert_eq!(value["diagnostics"], json!([]));
    assert_eq!(value["data"][0]["signals"][0]["value"], "1'h1");

    let events = parse_debug_events(&output.stderr);
    assert!(
        events
            .iter()
            .any(|event| event["message"] == "time.parse.done")
    );
}

#[test]
fn value_debug_trace_can_precede_fatal_output() {
    let output = wavepeek_cmd()
        .env("DEBUG", "1")
        .args([
            "value",
            "--waves",
            "tests/fixtures/hand/does-not-exist.vcd",
            "--at",
            "10ns",
            "--signals",
            "top.clk",
        ])
        .output()
        .expect("value command should execute");

    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf8");
    let lines = stderr.lines().collect::<Vec<_>>();
    assert!(lines.len() >= 2, "expected debug event and fatal line");
    let debug: Value = serde_json::from_str(lines[0]).expect("first line should be debug json");
    assert_eq!(debug["kind"], "debug");
    assert_eq!(debug["message"], "backend.open.start");
    assert!(
        lines
            .last()
            .expect("fatal line should exist")
            .starts_with("fatal: ")
    );
}

const FLAT_PROJECTION_VCD: &str = concat!(
    "$timescale 1ns $end\n",
    "$scope module top $end\n",
    "$var wire 8 ! data $end\n",
    "$scope module steps[0] $end\n",
    "$var wire 1 \" flag $end\n",
    "$upscope $end\n",
    "$upscope $end\n",
    "$enddefinitions $end\n",
    "#0\nb00000000 !\n0\"\n",
    "#5\nb00001111 !\n1\"\n",
    "#10\nb11111111 !\n",
);

#[test]
fn value_projects_flat_ranges_and_preserves_request_positions() {
    let fixture = write_fixture(FLAT_PROJECTION_VCD, "value-flat-projection.vcd");
    let output = wavepeek_cmd()
        .args([
            "value",
            "--waves",
            fixture.path().to_str().unwrap(),
            "--at",
            "10ns",
            "--scope",
            "top",
            "--signals",
            "data[7:4],data[5:2],data,data[7:4],data[0:0],steps[0].flag",
            "--json",
        ])
        .output()
        .expect("value should execute");

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let value: Value = serde_json::from_slice(&output.stdout).expect("stdout should be json");
    assert_eq!(
        value["data"][0]["signals"],
        json!([
            {"path": "top.data[7:4]", "relative_path": "data[7:4]", "value": "4'hf"},
            {"path": "top.data[5:2]", "relative_path": "data[5:2]", "value": "4'hf"},
            {"path": "top.data", "relative_path": "data", "value": "8'hff"},
            {"path": "top.data[7:4]", "relative_path": "data[7:4]", "value": "4'hf"},
            {"path": "top.data[0:0]", "relative_path": "data[0:0]", "value": "1'h1"},
            {"path": "top.steps[0].flag", "relative_path": "steps[0].flag", "value": "1'h1"},
        ])
    );
}

#[test]
fn value_rejects_invalid_flat_ranges_before_sampling() {
    let fixture = write_fixture(FLAT_PROJECTION_VCD, "value-invalid-projection.vcd");
    for (projection, message) in [
        ("data[3:4]", "msb must be greater"),
        ("data[8:0]", "outside signal 'top.data' width 8"),
        ("data[WIDTH:0]", "non-negative decimal integers"),
        ("data[7:4][3:2]", "chained and multidimensional"),
        ("data[3]", "no dumped signal with basename 'data[3]'"),
    ] {
        wavepeek_cmd()
            .args([
                "value",
                "--waves",
                fixture.path().to_str().unwrap(),
                "--at",
                "10ns",
                "--scope",
                "top",
                "--signals",
                projection,
            ])
            .assert()
            .failure()
            .code(1)
            .stderr(predicate::str::contains(message));
    }
}

#[test]
fn value_projects_normalized_ascending_and_unknown_bits() {
    let fixture = fixture_path("value_vectors.vcd");
    let output = wavepeek_cmd()
        .args([
            "value",
            "--waves",
            fixture.to_str().unwrap(),
            "--at",
            "0ns,5ns",
            "--scope",
            "top",
            "--signals",
            "asc[3:2],nibble[1:0],status[1:0]",
            "--json",
        ])
        .output()
        .expect("value should execute");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let value: Value = serde_json::from_slice(&output.stdout).expect("stdout should be json");
    assert_eq!(
        value["data"][0]["signals"],
        json!([
            {"path": "top.asc[3:2]", "relative_path": "asc[3:2]", "value": "2'h3"},
            {"path": "top.nibble[1:0]", "relative_path": "nibble[1:0]", "value": "2'h2"},
            {"path": "top.status[1:0]", "relative_path": "status[1:0]", "value": "2'hz"},
        ])
    );
    assert_eq!(
        value["data"][1]["signals"],
        json!([
            {"path": "top.asc[3:2]", "relative_path": "asc[3:2]", "value": "2'h0"},
            {"path": "top.nibble[1:0]", "relative_path": "nibble[1:0]", "value": "2'hx"},
            {"path": "top.status[1:0]", "relative_path": "status[1:0]", "value": "2'h3"},
        ])
    );
}
