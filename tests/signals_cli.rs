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

fn rtl_fixture_path(filename: &str) -> PathBuf {
    let base = std::env::var("WAVEPEEK_RTL_ARTIFACTS_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/opt/rtl-artifacts"));
    base.join(filename)
}

#[test]
fn signals_human_mode_uses_short_names_by_default() {
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
            "--max",
            "50",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("cfg kind=parameter width=8"))
        .stdout(predicate::str::contains("clk kind=wire width=1"))
        .stdout(predicate::str::contains("top.cfg").not())
        .stdout(predicate::str::contains("schema_version").not())
        .stderr(predicate::str::is_empty());
}

#[test]
fn signals_human_mode_supports_absolute_paths_with_abs_flag() {
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
            "--abs",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("top.cfg kind=parameter width=8"))
        .stdout(predicate::str::contains("top.clk kind=wire width=1"));
}

#[test]
fn signals_json_shape_for_vcd_keeps_full_paths() {
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
            "--json",
        ])
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&assert.get_output().stdout).to_string();
    let value: Value = serde_json::from_str(&stdout).expect("signals output should be valid json");

    assert_eq!(value["schema_version"], 2);
    assert_eq!(value["command"], "signals");
    assert_eq!(value["warnings"], Value::Array(vec![]));
    assert_eq!(
        value["data"],
        json!([
            {"name": "cfg", "path": "top.cfg", "kind": "parameter", "width": 8},
            {"name": "clk", "path": "top.clk", "kind": "wire", "width": 1},
            {"name": "data", "path": "top.data", "kind": "reg", "width": 8}
        ])
    );
}

#[test]
fn signals_json_shape_for_fst_keeps_full_paths() {
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
            "--json",
        ])
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&assert.get_output().stdout).to_string();
    let value: Value = serde_json::from_str(&stdout).expect("signals output should be valid json");

    assert_eq!(
        value["data"],
        json!([
            {"name": "cfg", "path": "top.cfg", "kind": "parameter", "width": 8},
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
            "--json",
        ])
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&assert.get_output().stdout).to_string();
    let value: Value = serde_json::from_str(&stdout).expect("signals output should be valid json");

    assert_eq!(
        value["data"],
        json!([
            {"name": "cfg", "path": "top.cfg", "kind": "parameter", "width": 8},
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
            "--json",
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
        .stderr(predicate::str::starts_with("error: args: invalid regex"))
        .stderr(predicate::str::contains("See 'wavepeek signals --help'."));
}

#[test]
fn signals_human_mode_routes_truncation_warning_to_stderr() {
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
            "--max",
            "1",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("cfg kind=parameter width=8"))
        .stdout(predicate::str::contains("schema_version").not())
        .stdout(predicate::str::contains("warning: truncated output").not())
        .stderr(predicate::str::contains(
            "warning: truncated output to 1 entries",
        ));
}

#[test]
fn signals_json_output_is_bit_for_bit_deterministic_across_runs() {
    let fixture = fixture_path("m2_core.vcd");
    let fixture = fixture.to_string_lossy().into_owned();

    let first = wavepeek_cmd()
        .args([
            "signals",
            "--waves",
            fixture.as_str(),
            "--scope",
            "top",
            "--json",
        ])
        .output()
        .expect("first run should execute");
    let second = wavepeek_cmd()
        .args([
            "signals",
            "--waves",
            fixture.as_str(),
            "--scope",
            "top",
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
fn signals_external_picorv32_fixture_uses_short_names_by_default() {
    let fixture = rtl_fixture_path("picorv32_test_vcd.fst");
    assert!(
        fixture.exists(),
        "required external fixture is missing: {}",
        fixture.display()
    );

    let mut command = wavepeek_cmd();
    let fixture = fixture.to_string_lossy().into_owned();
    command
        .args([
            "signals",
            "--waves",
            fixture.as_str(),
            "--scope",
            "testbench.top.uut",
            "--max",
            "8",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "BARREL_SHIFTER kind=parameter width=1",
        ))
        .stdout(predicate::str::contains("testbench.top.uut.BARREL_SHIFTER").not());
}
