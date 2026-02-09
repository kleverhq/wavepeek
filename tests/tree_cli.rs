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
fn tree_json_order_is_deterministic_for_vcd() {
    let fixture = fixture_path("m2_core.vcd");
    let fixture = fixture.to_string_lossy().into_owned();

    let mut command = wavepeek_cmd();
    let assert = command
        .args(["tree", "--waves", fixture.as_str(), "--max", "50"])
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&assert.get_output().stdout).to_string();
    let value: Value = serde_json::from_str(&stdout).expect("tree output should be valid json");

    assert_eq!(value["schema_version"], 1);
    assert_eq!(value["command"], "tree");
    assert_eq!(value["warnings"], Value::Array(vec![]));
    assert_eq!(
        value["data"],
        json!([
            { "path": "top", "depth": 0 },
            { "path": "top.cpu", "depth": 1 },
            { "path": "top.mem", "depth": 1 }
        ])
    );
}

#[test]
fn tree_json_order_is_deterministic_for_fst() {
    let fixture = fixture_path("m2_core.fst");
    let fixture = fixture.to_string_lossy().into_owned();

    let mut command = wavepeek_cmd();
    let assert = command
        .args(["tree", "--waves", fixture.as_str(), "--max", "50"])
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&assert.get_output().stdout).to_string();
    let value: Value = serde_json::from_str(&stdout).expect("tree output should be valid json");

    assert_eq!(
        value["data"],
        json!([
            { "path": "top", "depth": 0 },
            { "path": "top.cpu", "depth": 1 },
            { "path": "top.mem", "depth": 1 }
        ])
    );
}

#[test]
fn tree_respects_max_depth() {
    let fixture = fixture_path("m2_core.vcd");
    let fixture = fixture.to_string_lossy().into_owned();

    let mut command = wavepeek_cmd();
    let assert = command
        .args([
            "tree",
            "--waves",
            fixture.as_str(),
            "--max",
            "50",
            "--max-depth",
            "0",
        ])
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&assert.get_output().stdout).to_string();
    let value: Value = serde_json::from_str(&stdout).expect("tree output should be valid json");

    assert_eq!(value["data"], json!([{ "path": "top", "depth": 0 }]));
}

#[test]
fn tree_emits_truncation_warning_when_max_is_hit() {
    let fixture = fixture_path("m2_core.vcd");
    let fixture = fixture.to_string_lossy().into_owned();

    let mut command = wavepeek_cmd();
    let assert = command
        .args(["tree", "--waves", fixture.as_str(), "--max", "2"])
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&assert.get_output().stdout).to_string();
    let value: Value = serde_json::from_str(&stdout).expect("tree output should be valid json");

    assert_eq!(
        value["data"]
            .as_array()
            .expect("data should be array")
            .len(),
        2
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
            .contains("truncated output to 2 entries")
    );
}

#[test]
fn tree_invalid_regex_is_args_error() {
    let fixture = fixture_path("m2_core.vcd");
    let fixture = fixture.to_string_lossy().into_owned();

    let mut command = wavepeek_cmd();

    command
        .args([
            "tree",
            "--waves",
            fixture.as_str(),
            "--filter",
            "[unterminated",
        ])
        .assert()
        .failure()
        .code(1)
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::starts_with("error: args: invalid regex"));
}

#[test]
fn tree_output_is_bit_for_bit_deterministic_across_runs() {
    let fixture = fixture_path("m2_core.vcd");
    let fixture = fixture.to_string_lossy().into_owned();

    let first = wavepeek_cmd()
        .args(["tree", "--waves", fixture.as_str(), "--max", "50"])
        .output()
        .expect("first run should execute");
    let second = wavepeek_cmd()
        .args(["tree", "--waves", fixture.as_str(), "--max", "50"])
        .output()
        .expect("second run should execute");

    assert!(first.status.success());
    assert!(second.status.success());
    assert_eq!(first.stdout, second.stdout);
    assert_eq!(first.stderr, second.stderr);
}

#[test]
fn tree_human_mode_routes_truncation_warning_to_stderr() {
    let fixture = fixture_path("m2_core.vcd");
    let fixture = fixture.to_string_lossy().into_owned();

    let mut command = wavepeek_cmd();

    command
        .args(["tree", "--waves", fixture.as_str(), "--max", "1", "--human"])
        .assert()
        .success()
        .stdout(predicate::str::contains("0 top"))
        .stdout(predicate::str::contains("schema_version").not())
        .stdout(predicate::str::contains("warning: truncated output").not())
        .stderr(predicate::str::contains(
            "warning: truncated output to 1 entries",
        ));
}
