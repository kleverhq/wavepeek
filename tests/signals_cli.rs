use assert_cmd::prelude::*;
use predicates::prelude::*;
use serde_json::{Value, json};
use std::path::PathBuf;
use std::process::Command;

fn wavepeek_cmd() -> Command {
    Command::new(env!("CARGO_BIN_EXE_wavepeek"))
}

fn fixture_path(filename: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("hand")
        .join(filename)
}

#[test]
fn signals_are_sorted_with_stable_shape_for_vcd() {
    let fixture = fixture_path("m2_core.vcd");
    let fixture = fixture.to_string_lossy().into_owned();

    let mut command = wavepeek_cmd();
    let assert = command
        .args([
            "signals",
            "--waves",
            fixture.as_str(),
            "--scope",
            "top",
            "--max",
            "50",
        ])
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&assert.get_output().stdout).to_string();
    let value: Value = serde_json::from_str(&stdout).expect("signals output should be valid json");

    assert_eq!(value["schema_version"], 1);
    assert_eq!(value["command"], "signals");
    assert_eq!(value["warnings"], Value::Array(vec![]));
    assert_eq!(
        value["data"],
        json!([
            {"name": "cfg", "path": "top.cfg", "kind": "unknown", "width": 8},
            {"name": "clk", "path": "top.clk", "kind": "wire", "width": 1},
            {"name": "data", "path": "top.data", "kind": "reg", "width": 8}
        ])
    );
}

#[test]
fn signals_are_sorted_with_stable_shape_for_fst() {
    let fixture = fixture_path("m2_core.fst");
    let fixture = fixture.to_string_lossy().into_owned();

    let mut command = wavepeek_cmd();
    let assert = command
        .args([
            "signals",
            "--waves",
            fixture.as_str(),
            "--scope",
            "top",
            "--max",
            "50",
        ])
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&assert.get_output().stdout).to_string();
    let value: Value = serde_json::from_str(&stdout).expect("signals output should be valid json");

    assert_eq!(
        value["data"],
        json!([
            {"name": "cfg", "path": "top.cfg", "kind": "unknown", "width": 8},
            {"name": "clk", "path": "top.clk", "kind": "wire", "width": 1},
            {"name": "data", "path": "top.data", "kind": "reg", "width": 8}
        ])
    );
}

#[test]
fn signals_filter_applies_to_signal_names() {
    let fixture = fixture_path("m2_core.vcd");
    let fixture = fixture.to_string_lossy().into_owned();

    let mut command = wavepeek_cmd();
    let assert = command
        .args([
            "signals",
            "--waves",
            fixture.as_str(),
            "--scope",
            "top",
            "--filter",
            "^c.*",
        ])
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&assert.get_output().stdout).to_string();
    let value: Value = serde_json::from_str(&stdout).expect("signals output should be valid json");

    assert_eq!(
        value["data"],
        json!([
            {"name": "cfg", "path": "top.cfg", "kind": "unknown", "width": 8},
            {"name": "clk", "path": "top.clk", "kind": "wire", "width": 1}
        ])
    );
}

#[test]
fn signals_emit_truncation_warning_when_max_is_hit() {
    let fixture = fixture_path("m2_core.vcd");
    let fixture = fixture.to_string_lossy().into_owned();

    let mut command = wavepeek_cmd();
    let assert = command
        .args([
            "signals",
            "--waves",
            fixture.as_str(),
            "--scope",
            "top",
            "--max",
            "1",
        ])
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&assert.get_output().stdout).to_string();
    let value: Value = serde_json::from_str(&stdout).expect("signals output should be valid json");

    assert_eq!(
        value["data"]
            .as_array()
            .expect("data should be array")
            .len(),
        1
    );
    assert_eq!(
        value["warnings"]
            .as_array()
            .expect("warnings should be array")
            .len(),
        1
    );
    assert!(
        value["warnings"][0]
            .as_str()
            .expect("warning should be string")
            .contains("truncated output to 1 entries")
    );
}

#[test]
fn signals_scope_not_found_is_scope_error() {
    let fixture = fixture_path("m2_core.vcd");
    let fixture = fixture.to_string_lossy().into_owned();

    let mut command = wavepeek_cmd();

    command
        .args([
            "signals",
            "--waves",
            fixture.as_str(),
            "--scope",
            "top.nope",
        ])
        .assert()
        .failure()
        .code(1)
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::starts_with("error: scope:"));
}

#[test]
fn signals_invalid_regex_is_args_error() {
    let fixture = fixture_path("m2_core.vcd");
    let fixture = fixture.to_string_lossy().into_owned();

    let mut command = wavepeek_cmd();

    command
        .args([
            "signals",
            "--waves",
            fixture.as_str(),
            "--scope",
            "top",
            "--filter",
            "[unterminated",
        ])
        .assert()
        .failure()
        .code(1)
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::starts_with("error: args: invalid regex"));
}
