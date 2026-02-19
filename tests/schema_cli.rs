use assert_cmd::prelude::*;
use serde_json::Value;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn wavepeek_cmd() -> Command {
    Command::new(env!("CARGO_BIN_EXE_wavepeek"))
}

fn canonical_schema_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("schema")
        .join("wavepeek.json")
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
fn schema_command_output_is_valid_json() {
    let mut command = wavepeek_cmd();
    let assert = command.args(["schema"]).assert().success();

    let stdout = String::from_utf8_lossy(&assert.get_output().stdout).to_string();
    let value: Value = serde_json::from_str(&stdout).expect("schema output should be valid json");

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
