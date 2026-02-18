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
fn modules_human_output_is_default_for_vcd() {
    let fixture = fixture_path("m2_core.vcd");
    let fixture = fixture.to_string_lossy().into_owned();

    let mut command = wavepeek_cmd();
    command
        .args(["modules", "--waves", fixture.as_str(), "--max", "50"])
        .assert()
        .success()
        .stdout(predicate::str::contains("0 top"))
        .stdout(predicate::str::contains("1 top.cpu"))
        .stdout(predicate::str::contains("schema_version").not())
        .stderr(predicate::str::is_empty());
}

#[test]
fn modules_json_order_is_deterministic_for_vcd() {
    let fixture = fixture_path("m2_core.vcd");
    let fixture = fixture.to_string_lossy().into_owned();

    let mut command = wavepeek_cmd();
    let assert = command
        .args([
            "modules",
            "--waves",
            fixture.as_str(),
            "--max",
            "50",
            "--json",
        ])
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&assert.get_output().stdout).to_string();
    let value: Value = serde_json::from_str(&stdout).expect("modules output should be valid json");

    assert_eq!(value["schema_version"], 1);
    assert_eq!(value["command"], "modules");
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
fn modules_json_order_is_deterministic_for_fst() {
    let fixture = fixture_path("m2_core.fst");
    let fixture = fixture.to_string_lossy().into_owned();

    let mut command = wavepeek_cmd();
    let assert = command
        .args([
            "modules",
            "--waves",
            fixture.as_str(),
            "--max",
            "50",
            "--json",
        ])
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&assert.get_output().stdout).to_string();
    let value: Value = serde_json::from_str(&stdout).expect("modules output should be valid json");

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
fn modules_respects_max_depth() {
    let fixture = fixture_path("m2_core.vcd");
    let fixture = fixture.to_string_lossy().into_owned();

    let mut command = wavepeek_cmd();
    let assert = command
        .args([
            "modules",
            "--waves",
            fixture.as_str(),
            "--max",
            "50",
            "--max-depth",
            "0",
            "--json",
        ])
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&assert.get_output().stdout).to_string();
    let value: Value = serde_json::from_str(&stdout).expect("modules output should be valid json");

    assert_eq!(value["data"], json!([{ "path": "top", "depth": 0 }]));
}

#[test]
fn modules_emits_truncation_warning_when_max_is_hit() {
    let fixture = fixture_path("m2_core.vcd");
    let fixture = fixture.to_string_lossy().into_owned();

    let mut command = wavepeek_cmd();
    let assert = command
        .args([
            "modules",
            "--waves",
            fixture.as_str(),
            "--max",
            "2",
            "--json",
        ])
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&assert.get_output().stdout).to_string();
    let value: Value = serde_json::from_str(&stdout).expect("modules output should be valid json");

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
fn modules_invalid_regex_is_args_error() {
    let fixture = fixture_path("m2_core.vcd");
    let fixture = fixture.to_string_lossy().into_owned();

    let mut command = wavepeek_cmd();

    command
        .args([
            "modules",
            "--waves",
            fixture.as_str(),
            "--filter",
            "[unterminated",
        ])
        .assert()
        .failure()
        .code(1)
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::starts_with("error: args: invalid regex"))
        .stderr(predicate::str::contains("See 'wavepeek modules --help'."));
}

#[test]
fn modules_output_is_bit_for_bit_deterministic_across_runs() {
    let fixture = fixture_path("m2_core.vcd");
    let fixture = fixture.to_string_lossy().into_owned();

    let first = wavepeek_cmd()
        .args([
            "modules",
            "--waves",
            fixture.as_str(),
            "--max",
            "50",
            "--json",
        ])
        .output()
        .expect("first run should execute");
    let second = wavepeek_cmd()
        .args([
            "modules",
            "--waves",
            fixture.as_str(),
            "--max",
            "50",
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
fn modules_tree_mode_renders_visual_hierarchy() {
    let fixture = fixture_path("m2_core.vcd");
    let fixture = fixture.to_string_lossy().into_owned();

    let mut command = wavepeek_cmd();

    command
        .args([
            "modules",
            "--waves",
            fixture.as_str(),
            "--tree",
            "--max",
            "50",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("top"))
        .stdout(predicate::str::contains("├── cpu"))
        .stdout(predicate::str::contains("└── mem"))
        .stdout(predicate::str::contains("schema_version").not())
        .stderr(predicate::str::is_empty());
}

#[test]
fn modules_json_ignores_tree_flag_without_extra_warning() {
    let fixture = fixture_path("m2_core.vcd");
    let fixture = fixture.to_string_lossy().into_owned();

    let mut command = wavepeek_cmd();
    let assert = command
        .args([
            "modules",
            "--waves",
            fixture.as_str(),
            "--json",
            "--tree",
            "--max",
            "50",
        ])
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&assert.get_output().stdout).to_string();
    let value: Value = serde_json::from_str(&stdout).expect("modules output should be valid json");

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
fn modules_external_scr1_fixture_keeps_module_hierarchy_semantics() {
    let fixture = rtl_fixture_path("scr1_max_axi_coremark.fst");
    assert!(
        fixture.exists(),
        "required external fixture is missing: {}",
        fixture.display()
    );

    let mut command = wavepeek_cmd();
    let fixture = fixture.to_string_lossy().into_owned();
    let assert = command
        .args([
            "modules",
            "--waves",
            fixture.as_str(),
            "--json",
            "--max",
            "80",
        ])
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&assert.get_output().stdout).to_string();
    let value: Value = serde_json::from_str(&stdout).expect("modules output should be valid json");
    let data = value["data"].as_array().expect("data should be array");

    assert!(!data.is_empty(), "modules list should not be empty");
    assert!(
        data.iter().any(|entry| {
            entry["path"]
                .as_str()
                .map(|path| path == "TOP.scr1_top_tb_axi.i_top.i_core_top.i_pipe_top")
                .unwrap_or(false)
        }),
        "expected core pipeline module path is missing"
    );
    assert!(
        data.iter().all(|entry| {
            entry["path"]
                .as_str()
                .map(|path| !path.ends_with(".clk") && !path.ends_with(".rst_n"))
                .unwrap_or(false)
        }),
        "module listing unexpectedly contains signal-like leaves"
    );
}
