use assert_cmd::prelude::*;
use predicates::prelude::*;
use serde_json::{Value, json};
use std::path::PathBuf;
use std::process::Command;

fn wavepeek_cmd() -> Command {
    Command::new(env!("CARGO_BIN_EXE_wavepeek"))
}

fn expected_schema_url() -> &'static str {
    concat!(
        "https://github.com/kleverhq/wavepeek/blob/v",
        env!("CARGO_PKG_VERSION"),
        "/schema/wavepeek.json"
    )
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
fn scope_human_output_is_default_for_vcd() {
    let fixture = fixture_path("m2_core.vcd");
    let fixture = fixture.to_string_lossy().into_owned();

    let mut command = wavepeek_cmd();
    command
        .args(["scope", "--waves", fixture.as_str(), "--max", "50"])
        .assert()
        .success()
        .stdout(predicate::str::contains("0 top kind=module"))
        .stdout(predicate::str::contains("1 top.cpu kind=module"))
        .stdout(predicate::str::contains("schema_version").not())
        .stderr(predicate::str::is_empty());
}

#[test]
fn scope_json_order_is_deterministic_for_vcd() {
    let fixture = fixture_path("m2_core.vcd");
    let fixture = fixture.to_string_lossy().into_owned();

    let mut command = wavepeek_cmd();
    let assert = command
        .args([
            "scope",
            "--waves",
            fixture.as_str(),
            "--max",
            "50",
            "--json",
        ])
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&assert.get_output().stdout).to_string();
    let value: Value = serde_json::from_str(&stdout).expect("scope output should be valid json");

    assert_eq!(value["$schema"], expected_schema_url());
    assert!(value.get("schema_version").is_none());
    assert_eq!(value["command"], "scope");
    assert_eq!(value["warnings"], Value::Array(vec![]));
    assert_eq!(
        value["data"],
        json!([
            { "path": "top", "depth": 0, "kind": "module" },
            { "path": "top.cpu", "depth": 1, "kind": "module" },
            { "path": "top.mem", "depth": 1, "kind": "module" }
        ])
    );
}

#[test]
fn scope_json_order_is_deterministic_for_fst() {
    let fixture = fixture_path("m2_core.fst");
    let fixture = fixture.to_string_lossy().into_owned();

    let mut command = wavepeek_cmd();
    let assert = command
        .args([
            "scope",
            "--waves",
            fixture.as_str(),
            "--max",
            "50",
            "--json",
        ])
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&assert.get_output().stdout).to_string();
    let value: Value = serde_json::from_str(&stdout).expect("scope output should be valid json");

    assert_eq!(
        value["data"],
        json!([
            { "path": "top", "depth": 0, "kind": "module" },
            { "path": "top.cpu", "depth": 1, "kind": "module" },
            { "path": "top.mem", "depth": 1, "kind": "module" }
        ])
    );
}

#[test]
fn scope_respects_max_depth() {
    let fixture = fixture_path("m2_core.vcd");
    let fixture = fixture.to_string_lossy().into_owned();

    let mut command = wavepeek_cmd();
    let assert = command
        .args([
            "scope",
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
    let value: Value = serde_json::from_str(&stdout).expect("scope output should be valid json");

    assert_eq!(
        value["data"],
        json!([{ "path": "top", "depth": 0, "kind": "module" }])
    );
}

#[test]
fn scope_emits_truncation_warning_when_max_is_hit() {
    let fixture = fixture_path("m2_core.vcd");
    let fixture = fixture.to_string_lossy().into_owned();

    let mut command = wavepeek_cmd();
    let assert = command
        .args(["scope", "--waves", fixture.as_str(), "--max", "2", "--json"])
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&assert.get_output().stdout).to_string();
    let value: Value = serde_json::from_str(&stdout).expect("scope output should be valid json");

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
fn scope_invalid_regex_is_args_error() {
    let fixture = fixture_path("m2_core.vcd");
    let fixture = fixture.to_string_lossy().into_owned();

    let mut command = wavepeek_cmd();

    command
        .args([
            "scope",
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
        .stderr(predicate::str::contains("See 'wavepeek scope --help'."));
}

#[test]
fn scope_output_is_bit_for_bit_deterministic_across_runs() {
    let fixture = fixture_path("m2_core.vcd");
    let fixture = fixture.to_string_lossy().into_owned();

    let first = wavepeek_cmd()
        .args([
            "scope",
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
            "scope",
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
fn scope_tree_mode_renders_visual_hierarchy() {
    let fixture = fixture_path("m2_core.vcd");
    let fixture = fixture.to_string_lossy().into_owned();

    let mut command = wavepeek_cmd();

    command
        .args([
            "scope",
            "--waves",
            fixture.as_str(),
            "--tree",
            "--max",
            "50",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("top kind=module"))
        .stdout(predicate::str::contains("├── cpu kind=module"))
        .stdout(predicate::str::contains("└── mem kind=module"))
        .stdout(predicate::str::contains("schema_version").not())
        .stderr(predicate::str::is_empty());
}

#[test]
fn scope_json_ignores_tree_flag_without_extra_warning() {
    let fixture = fixture_path("m2_core.vcd");
    let fixture = fixture.to_string_lossy().into_owned();

    let mut command = wavepeek_cmd();
    let assert = command
        .args([
            "scope",
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
    let value: Value = serde_json::from_str(&stdout).expect("scope output should be valid json");

    assert_eq!(value["warnings"], Value::Array(vec![]));
    assert_eq!(
        value["data"],
        json!([
            { "path": "top", "depth": 0, "kind": "module" },
            { "path": "top.cpu", "depth": 1, "kind": "module" },
            { "path": "top.mem", "depth": 1, "kind": "module" }
        ])
    );
}

#[test]
fn scope_reports_non_module_kinds_when_fixture_contains_them() {
    let fixture = fixture_path("scope_mixed_kinds.vcd");
    let fixture = fixture.to_string_lossy().into_owned();

    let mut command = wavepeek_cmd();
    let assert = command
        .args([
            "scope",
            "--waves",
            fixture.as_str(),
            "--json",
            "--max",
            "50",
        ])
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&assert.get_output().stdout).to_string();
    let value: Value = serde_json::from_str(&stdout).expect("scope output should be valid json");
    let data = value["data"].as_array().expect("data should be array");

    assert!(
        data.iter().any(|entry| entry["kind"]
            .as_str()
            .is_some_and(|kind| kind == "function")),
        "expected function scope kind is missing"
    );
    assert!(
        data.iter()
            .any(|entry| entry["kind"].as_str().is_some_and(|kind| kind == "task")),
        "expected task scope kind is missing"
    );
}

#[test]
fn scope_tree_mode_is_deterministic_for_mixed_scope_kinds() {
    let fixture = fixture_path("scope_mixed_kinds.vcd");
    let fixture = fixture.to_string_lossy().into_owned();

    let mut command = wavepeek_cmd();
    let expected =
        "top kind=module\n├── cpu kind=module\n├── helper kind=function\n└── worker kind=task\n";

    command
        .args([
            "scope",
            "--waves",
            fixture.as_str(),
            "--tree",
            "--max",
            "50",
        ])
        .assert()
        .success()
        .stdout(predicate::eq(expected))
        .stderr(predicate::str::is_empty());
}

#[test]
fn scope_external_scr1_fixture_keeps_hierarchy_semantics() {
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
            "scope",
            "--waves",
            fixture.as_str(),
            "--json",
            "--max",
            "80",
        ])
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&assert.get_output().stdout).to_string();
    let value: Value = serde_json::from_str(&stdout).expect("scope output should be valid json");
    let data = value["data"].as_array().expect("data should be array");

    assert!(!data.is_empty(), "scope list should not be empty");
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
        data.iter().all(|entry| entry["kind"].as_str().is_some()),
        "all scope entries must expose scope kind"
    );
    assert!(
        data.iter().all(|entry| {
            entry["path"]
                .as_str()
                .map(|path| !path.ends_with(".clk") && !path.ends_with(".rst_n"))
                .unwrap_or(false)
        }),
        "scope listing unexpectedly contains signal-like leaves"
    );
}
