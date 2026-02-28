use assert_cmd::prelude::*;
use predicates::prelude::*;
use serde_json::{Value, json};

mod common;
use common::{expected_schema_url, fixture_path, rtl_fixture_path, wavepeek_cmd};

#[test]
fn signal_human_mode_uses_short_names_by_default() {
    let fixture = fixture_path("m2_core.vcd");
    let fixture = fixture.to_string_lossy().into_owned();

    let mut command = wavepeek_cmd();
    command
        .args([
            "signal",
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
fn signal_human_mode_supports_absolute_paths_with_abs_flag() {
    let fixture = fixture_path("m2_core.vcd");
    let fixture = fixture.to_string_lossy().into_owned();

    let mut command = wavepeek_cmd();

    command
        .args([
            "signal",
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
fn signal_json_shape_for_vcd_keeps_full_paths() {
    let fixture = fixture_path("m2_core.vcd");
    let fixture = fixture.to_string_lossy().into_owned();

    let mut command = wavepeek_cmd();
    let assert = command
        .args([
            "signal",
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
    let value: Value = serde_json::from_str(&stdout).expect("signal output should be valid json");

    assert_eq!(value["$schema"], expected_schema_url());
    assert!(value.get("schema_version").is_none());
    assert_eq!(value["command"], "signal");
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
fn signal_json_shape_for_fst_keeps_full_paths() {
    let fixture = fixture_path("m2_core.fst");
    let fixture = fixture.to_string_lossy().into_owned();

    let mut command = wavepeek_cmd();
    let assert = command
        .args([
            "signal",
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
    let value: Value = serde_json::from_str(&stdout).expect("signal output should be valid json");

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
fn signal_filter_applies_to_signal_names() {
    let fixture = fixture_path("m2_core.vcd");
    let fixture = fixture.to_string_lossy().into_owned();

    let mut command = wavepeek_cmd();
    let assert = command
        .args([
            "signal",
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
    let value: Value = serde_json::from_str(&stdout).expect("signal output should be valid json");

    assert_eq!(
        value["data"],
        json!([
            {"name": "cfg", "path": "top.cfg", "kind": "parameter", "width": 8},
            {"name": "clk", "path": "top.clk", "kind": "wire", "width": 1}
        ])
    );
}

#[test]
fn signal_emits_truncation_warning_when_max_is_hit() {
    let fixture = fixture_path("m2_core.vcd");
    let fixture = fixture.to_string_lossy().into_owned();

    let mut command = wavepeek_cmd();
    let assert = command
        .args([
            "signal",
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
    let value: Value = serde_json::from_str(&stdout).expect("signal output should be valid json");

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
fn signal_scope_not_found_is_scope_error() {
    let fixture = fixture_path("m2_core.vcd");
    let fixture = fixture.to_string_lossy().into_owned();

    let mut command = wavepeek_cmd();

    command
        .args(["signal", "--waves", fixture.as_str(), "--scope", "top.nope"])
        .assert()
        .failure()
        .code(1)
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::starts_with("error: scope:"));
}

#[test]
fn signal_invalid_regex_is_args_error() {
    let fixture = fixture_path("m2_core.vcd");
    let fixture = fixture.to_string_lossy().into_owned();

    let mut command = wavepeek_cmd();

    command
        .args([
            "signal",
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
        .stderr(predicate::str::contains("See 'wavepeek signal --help'."));
}

#[test]
fn signal_human_mode_routes_truncation_warning_to_stderr() {
    let fixture = fixture_path("m2_core.vcd");
    let fixture = fixture.to_string_lossy().into_owned();

    let mut command = wavepeek_cmd();

    command
        .args([
            "signal",
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
fn signal_recursive_human_mode_uses_relative_paths_from_scope() {
    let fixture = fixture_path("m2_core.vcd");
    let fixture = fixture.to_string_lossy().into_owned();

    let mut command = wavepeek_cmd();
    command
        .args([
            "signal",
            "--waves",
            fixture.as_str(),
            "--scope",
            "top",
            "--recursive",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("cfg kind=parameter width=8"))
        .stdout(predicate::str::contains("clk kind=wire width=1"))
        .stdout(predicate::str::contains("cpu.valid kind=wire width=1"))
        .stdout(predicate::str::contains("mem.ready kind=wire width=1"))
        .stdout(predicate::str::contains("top.cpu.valid").not())
        .stderr(predicate::str::is_empty());
}

#[test]
fn signal_recursive_abs_mode_uses_canonical_paths() {
    let fixture = fixture_path("m2_core.vcd");
    let fixture = fixture.to_string_lossy().into_owned();

    let mut command = wavepeek_cmd();
    command
        .args([
            "signal",
            "--waves",
            fixture.as_str(),
            "--scope",
            "top",
            "--recursive",
            "--abs",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("top.cpu.valid kind=wire width=1"))
        .stdout(predicate::str::contains("top.mem.ready kind=wire width=1"));
}

#[test]
fn signal_recursive_max_depth_zero_matches_non_recursive() {
    let fixture = fixture_path("signal_recursive_depth.vcd");
    let fixture = fixture.to_string_lossy().into_owned();

    let non_recursive = wavepeek_cmd()
        .args([
            "signal",
            "--waves",
            fixture.as_str(),
            "--scope",
            "top",
            "--json",
        ])
        .output()
        .expect("non-recursive run should execute");
    let recursive_depth_zero = wavepeek_cmd()
        .args([
            "signal",
            "--waves",
            fixture.as_str(),
            "--scope",
            "top",
            "--recursive",
            "--max-depth",
            "0",
            "--json",
        ])
        .output()
        .expect("recursive run should execute");

    assert!(non_recursive.status.success());
    assert!(recursive_depth_zero.status.success());

    let non_recursive_json: Value = serde_json::from_slice(&non_recursive.stdout)
        .expect("non-recursive output should be valid json");
    let recursive_depth_zero_json: Value = serde_json::from_slice(&recursive_depth_zero.stdout)
        .expect("recursive depth-0 output should be valid json");

    assert_eq!(
        non_recursive_json["data"],
        recursive_depth_zero_json["data"]
    );
    assert_eq!(
        non_recursive_json["warnings"],
        recursive_depth_zero_json["warnings"]
    );
}

#[test]
fn signal_recursive_max_depth_respects_grandchild_boundary() {
    let fixture = fixture_path("signal_recursive_depth.vcd");
    let fixture = fixture.to_string_lossy().into_owned();

    let depth_0 = wavepeek_cmd()
        .args([
            "signal",
            "--waves",
            fixture.as_str(),
            "--scope",
            "top",
            "--recursive",
            "--max-depth",
            "0",
            "--json",
        ])
        .output()
        .expect("depth 0 run should execute");
    let depth_1 = wavepeek_cmd()
        .args([
            "signal",
            "--waves",
            fixture.as_str(),
            "--scope",
            "top",
            "--recursive",
            "--max-depth",
            "1",
            "--json",
        ])
        .output()
        .expect("depth 1 run should execute");
    let depth_2 = wavepeek_cmd()
        .args([
            "signal",
            "--waves",
            fixture.as_str(),
            "--scope",
            "top",
            "--recursive",
            "--max-depth",
            "2",
            "--json",
        ])
        .output()
        .expect("depth 2 run should execute");

    assert!(depth_0.status.success());
    assert!(depth_1.status.success());
    assert!(depth_2.status.success());

    let depth_0_json: Value = serde_json::from_slice(&depth_0.stdout).expect("depth 0 json");
    let depth_1_json: Value = serde_json::from_slice(&depth_1.stdout).expect("depth 1 json");
    let depth_2_json: Value = serde_json::from_slice(&depth_2.stdout).expect("depth 2 json");

    let depth_0_paths = depth_0_json["data"]
        .as_array()
        .expect("depth 0 data should be array")
        .iter()
        .map(|entry| {
            entry["path"]
                .as_str()
                .expect("signal path should be string")
                .to_string()
        })
        .collect::<Vec<_>>();
    let depth_1_paths = depth_1_json["data"]
        .as_array()
        .expect("depth 1 data should be array")
        .iter()
        .map(|entry| {
            entry["path"]
                .as_str()
                .expect("signal path should be string")
                .to_string()
        })
        .collect::<Vec<_>>();
    let depth_2_paths = depth_2_json["data"]
        .as_array()
        .expect("depth 2 data should be array")
        .iter()
        .map(|entry| {
            entry["path"]
                .as_str()
                .expect("signal path should be string")
                .to_string()
        })
        .collect::<Vec<_>>();

    assert_eq!(depth_0_paths, vec!["top.clk", "top.reset_n"]);
    assert_eq!(
        depth_1_paths,
        vec!["top.clk", "top.reset_n", "top.cpu.valid", "top.mem.ready"]
    );
    assert_eq!(
        depth_2_paths,
        vec![
            "top.clk",
            "top.reset_n",
            "top.cpu.valid",
            "top.cpu.core.execute",
            "top.mem.ready",
        ]
    );
}

#[test]
fn signal_max_depth_requires_recursive_flag() {
    let fixture = fixture_path("m2_core.vcd");
    let fixture = fixture.to_string_lossy().into_owned();

    let mut command = wavepeek_cmd();
    command
        .args([
            "signal",
            "--waves",
            fixture.as_str(),
            "--scope",
            "top",
            "--max-depth",
            "1",
        ])
        .assert()
        .failure()
        .code(1)
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::starts_with(
            "error: args: --max-depth requires --recursive",
        ))
        .stderr(predicate::str::contains("See 'wavepeek signal --help'."));
}

#[test]
fn signal_recursive_filter_and_max_preserve_truncation_warning() {
    let fixture = fixture_path("signal_recursive_depth.vcd");
    let fixture = fixture.to_string_lossy().into_owned();

    let mut command = wavepeek_cmd();
    let assert = command
        .args([
            "signal",
            "--waves",
            fixture.as_str(),
            "--scope",
            "top",
            "--recursive",
            "--filter",
            ".*",
            "--max",
            "2",
            "--json",
        ])
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&assert.get_output().stdout).to_string();
    let value: Value = serde_json::from_str(&stdout).expect("signal output should be valid json");

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
fn signal_recursive_filter_matches_name_not_relative_path() {
    let fixture = fixture_path("m2_core.vcd");
    let fixture = fixture.to_string_lossy().into_owned();

    let mut command = wavepeek_cmd();
    let assert = command
        .args([
            "signal",
            "--waves",
            fixture.as_str(),
            "--scope",
            "top",
            "--recursive",
            "--filter",
            "^cpu\\..*",
            "--json",
        ])
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&assert.get_output().stdout).to_string();
    let value: Value = serde_json::from_str(&stdout).expect("signal output should be valid json");

    assert_eq!(value["data"], Value::Array(vec![]));
}

#[test]
fn signal_recursive_json_output_is_bit_for_bit_deterministic_across_runs() {
    let fixture = fixture_path("m2_core.vcd");
    let fixture = fixture.to_string_lossy().into_owned();

    let first = wavepeek_cmd()
        .args([
            "signal",
            "--waves",
            fixture.as_str(),
            "--scope",
            "top",
            "--recursive",
            "--json",
        ])
        .output()
        .expect("first run should execute");
    let second = wavepeek_cmd()
        .args([
            "signal",
            "--waves",
            fixture.as_str(),
            "--scope",
            "top",
            "--recursive",
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
fn signal_external_picorv32_fixture_uses_short_names_by_default() {
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
            "signal",
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
