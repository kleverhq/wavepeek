use assert_cmd::prelude::*;
use predicates::prelude::*;
use serde_json::Value;
use std::io::Write;
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

fn rtl_fixture_path(filename: &str) -> PathBuf {
    let base = std::env::var("WAVEPEEK_RTL_ARTIFACTS_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/opt/rtl-artifacts"));
    base.join(filename)
}

#[test]
fn info_human_output_is_default_for_vcd_fixture() {
    let fixture = fixture_path("m2_core.vcd");
    let fixture = fixture.to_string_lossy().into_owned();

    let mut command = wavepeek_cmd();
    command
        .args(["info", "--waves", fixture.as_str()])
        .assert()
        .success()
        .stdout(predicate::str::contains("time_unit: 1ns"))
        .stdout(predicate::str::contains("time_precision").not())
        .stdout(predicate::str::contains("time_start: 0ns"))
        .stdout(predicate::str::contains("time_end: 10ns"))
        .stderr(predicate::str::is_empty());
}

#[test]
fn info_json_contract_for_vcd_fixture() {
    let fixture = fixture_path("m2_core.vcd");
    let fixture = fixture.to_string_lossy().into_owned();

    let mut command = wavepeek_cmd();
    let assert = command
        .args(["info", "--waves", fixture.as_str(), "--json"])
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&assert.get_output().stdout).to_string();
    let value: Value = serde_json::from_str(&stdout).expect("info output should be valid json");

    assert_eq!(value["schema_version"], 2);
    assert_eq!(value["command"], "info");
    assert_eq!(value["warnings"], Value::Array(vec![]));
    assert_eq!(value["data"]["time_unit"], "1ns");
    assert!(value["data"].get("time_precision").is_none());
    assert_eq!(value["data"]["time_start"], "0ns");
    assert_eq!(value["data"]["time_end"], "10ns");
}

#[test]
fn info_json_contract_for_fst_fixture() {
    let fixture = fixture_path("m2_core.fst");
    let fixture = fixture.to_string_lossy().into_owned();

    let mut command = wavepeek_cmd();
    let assert = command
        .args(["info", "--waves", fixture.as_str(), "--json"])
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&assert.get_output().stdout).to_string();
    let value: Value = serde_json::from_str(&stdout).expect("info output should be valid json");

    assert_eq!(value["schema_version"], 2);
    assert_eq!(value["command"], "info");
    assert_eq!(value["warnings"], Value::Array(vec![]));
    assert_eq!(value["data"]["time_unit"], "1ns");
    assert!(value["data"].get("time_precision").is_none());
    assert_eq!(value["data"]["time_start"], "0ns");
    assert_eq!(value["data"]["time_end"], "10ns");
}

#[test]
fn info_json_output_is_deterministic_across_runs() {
    let fixture = fixture_path("m2_core.vcd");
    let fixture = fixture.to_string_lossy().into_owned();

    let first = wavepeek_cmd()
        .args(["info", "--waves", fixture.as_str(), "--json"])
        .output()
        .expect("first run should execute");
    let second = wavepeek_cmd()
        .args(["info", "--waves", fixture.as_str(), "--json"])
        .output()
        .expect("second run should execute");

    assert!(first.status.success());
    assert!(second.status.success());
    assert_eq!(first.stdout, second.stdout);
    assert_eq!(first.stderr, second.stderr);
}

#[test]
fn info_json_contract_for_external_picorv32_fixture() {
    let fixture = rtl_fixture_path("picorv32_test_vcd.fst");
    assert!(
        fixture.exists(),
        "required external fixture is missing: {}",
        fixture.display()
    );

    let mut command = wavepeek_cmd();
    let fixture = fixture.to_string_lossy().into_owned();
    let assert = command
        .args(["info", "--waves", fixture.as_str(), "--json"])
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&assert.get_output().stdout).to_string();
    let value: Value = serde_json::from_str(&stdout).expect("info output should be valid json");

    assert_eq!(value["command"], "info");
    assert!(value["data"]["time_unit"].as_str().is_some());
    assert!(value["data"]["time_start"].as_str().is_some());
    assert!(value["data"]["time_end"].as_str().is_some());
    assert!(value["data"].get("time_precision").is_none());
}

#[test]
fn info_missing_file_is_file_error_with_exit_code_two() {
    let mut command = wavepeek_cmd();

    command
        .args(["info", "--waves", "/tmp/wavepeek-does-not-exist.vcd"])
        .assert()
        .failure()
        .code(2)
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::starts_with("error: file: cannot open"));
}

#[test]
fn info_invalid_file_is_parse_error_with_exit_code_two() {
    let mut fixture = tempfile::NamedTempFile::new().expect("temp file should be created");
    fixture
        .write_all(b"not-a-waveform")
        .expect("temp file should be writable");

    let mut command = wavepeek_cmd();
    let path = fixture.path().to_string_lossy().into_owned();

    command
        .args(["info", "--waves", path.as_str()])
        .assert()
        .failure()
        .code(2)
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::starts_with("error: file: cannot parse"));
}
