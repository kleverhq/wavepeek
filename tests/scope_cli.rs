use assert_cmd::prelude::*;
use predicates::prelude::*;
use serde_json::{Value, json};

mod common;
use common::{fixture_path, ondas_fixture_path, wavepeek_cmd};

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

    assert_eq!(value["command"], "scope");
    assert_eq!(value["diagnostics"], Value::Array(vec![]));
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
        value["diagnostics"]
            .as_array()
            .expect("diagnostics should be array")
            .len(),
        1
    );
    assert!(
        value["diagnostics"][0]["message"]
            .as_str()
            .expect("diagnostic message should be string")
            .contains("truncated output to 2 entries")
    );
}

#[test]
fn scope_unlimited_max_disables_limit_without_diagnostics() {
    let fixture = fixture_path("m2_core.vcd");
    let fixture = fixture.to_string_lossy().into_owned();

    let json_output = wavepeek_cmd()
        .args([
            "scope",
            "--waves",
            fixture.as_str(),
            "--max",
            "unlimited",
            "--json",
        ])
        .output()
        .expect("json run should execute");
    let human_output = wavepeek_cmd()
        .args(["scope", "--waves", fixture.as_str(), "--max", "unlimited"])
        .output()
        .expect("human run should execute");

    assert!(json_output.status.success());
    assert!(human_output.status.success());

    let value = parse_scope_json(&json_output.stdout);
    assert_eq!(value["diagnostics"], json!([]));
    assert_eq!(
        value["data"]
            .as_array()
            .expect("data should be array")
            .len(),
        3
    );
    assert_eq!(value["summary"]["limit"], Value::Null);
    assert!(human_output.stderr.is_empty());
}

#[test]
fn scope_unlimited_max_depth_disables_depth_bound_without_diagnostic() {
    let fixture = fixture_path("signal_recursive_depth.vcd");
    let fixture = fixture.to_string_lossy().into_owned();

    let depth_unlimited_output = wavepeek_cmd()
        .args([
            "scope",
            "--waves",
            fixture.as_str(),
            "--max",
            "50",
            "--max-depth",
            "unlimited",
            "--json",
        ])
        .output()
        .expect("unlimited depth run should execute");
    let depth_one_output = wavepeek_cmd()
        .args([
            "scope",
            "--waves",
            fixture.as_str(),
            "--max",
            "50",
            "--max-depth",
            "1",
            "--json",
        ])
        .output()
        .expect("depth 1 run should execute");

    assert!(depth_unlimited_output.status.success());
    assert!(depth_one_output.status.success());

    let depth_unlimited = parse_scope_json(&depth_unlimited_output.stdout);
    let depth_one = parse_scope_json(&depth_one_output.stdout);

    assert_eq!(depth_unlimited["diagnostics"], json!([]));
    assert_eq!(depth_one["diagnostics"], json!([]));
    assert!(
        depth_unlimited["data"]
            .as_array()
            .expect("unlimited data should be array")
            .len()
            > depth_one["data"]
                .as_array()
                .expect("depth-1 data should be array")
                .len()
    );
}

#[test]
fn scope_dual_unlimited_limits_have_no_diagnostics() {
    let fixture = fixture_path("signal_recursive_depth.vcd");
    let fixture = fixture.to_string_lossy().into_owned();

    let json_output = wavepeek_cmd()
        .args([
            "scope",
            "--waves",
            fixture.as_str(),
            "--max",
            "unlimited",
            "--max-depth",
            "unlimited",
            "--json",
        ])
        .output()
        .expect("json run should execute");
    let human_output = wavepeek_cmd()
        .args([
            "scope",
            "--waves",
            fixture.as_str(),
            "--max",
            "unlimited",
            "--max-depth",
            "unlimited",
        ])
        .output()
        .expect("human run should execute");

    assert!(json_output.status.success());
    assert!(human_output.status.success());

    let value = parse_scope_json(&json_output.stdout);
    assert_eq!(value["diagnostics"], json!([]));
    assert_eq!(value["summary"]["limit"], Value::Null);
    assert!(human_output.stderr.is_empty());
}

#[test]
fn scope_unlimited_depth_keeps_truncation_warning_only() {
    let fixture = fixture_path("signal_recursive_depth.vcd");
    let fixture = fixture.to_string_lossy().into_owned();

    let json_output = wavepeek_cmd()
        .args([
            "scope",
            "--waves",
            fixture.as_str(),
            "--max",
            "1",
            "--max-depth",
            "unlimited",
            "--json",
        ])
        .output()
        .expect("json run should execute");
    let human_output = wavepeek_cmd()
        .args([
            "scope",
            "--waves",
            fixture.as_str(),
            "--max",
            "1",
            "--max-depth",
            "unlimited",
        ])
        .output()
        .expect("human run should execute");

    assert!(json_output.status.success());
    assert!(human_output.status.success());

    let value = parse_scope_json(&json_output.stdout);
    assert_eq!(
        value["diagnostics"],
        json!([{"kind": "warning", "code": "WPK-W0002", "message": "truncated output to 1 entries (use --max to increase limit)"}])
    );
    assert_eq!(
        String::from_utf8_lossy(&human_output.stderr)
            .lines()
            .collect::<Vec<_>>(),
        vec!["warning[WPK-W0002]: truncated output to 1 entries (use --max to increase limit)"]
    );
}

#[test]
fn scope_rejects_zero_max_with_args_error() {
    let fixture = fixture_path("m2_core.vcd");
    let fixture = fixture.to_string_lossy().into_owned();

    wavepeek_cmd()
        .args(["scope", "--waves", fixture.as_str(), "--max", "0"])
        .assert()
        .failure()
        .code(1)
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::starts_with(
            "fatal: args: --max must be greater than 0.",
        ))
        .stderr(predicate::str::contains("See 'wavepeek scope --help'."));
}

fn parse_scope_json(stdout: &[u8]) -> Value {
    serde_json::from_slice(stdout).expect("scope output should be valid json")
}

#[test]
fn scope_empty_filter_uses_normal_human_output_and_empty_machine_diagnostics() {
    let fixture = fixture_path("m2_core.vcd");
    let fixture = fixture.to_string_lossy().into_owned();

    let json_output = wavepeek_cmd()
        .args([
            "scope",
            "--waves",
            fixture.as_str(),
            "--filter",
            "DOES_NOT_MATCH",
            "--json",
        ])
        .output()
        .expect("json scope run should execute");
    let human_output = wavepeek_cmd()
        .args([
            "scope",
            "--waves",
            fixture.as_str(),
            "--filter",
            "DOES_NOT_MATCH",
        ])
        .output()
        .expect("human scope run should execute");

    assert!(json_output.status.success());
    assert!(human_output.status.success());
    assert_eq!(
        String::from_utf8_lossy(&human_output.stdout).trim(),
        "no scopes found"
    );
    assert!(human_output.stderr.is_empty());
    let value = parse_scope_json(&json_output.stdout);
    assert_eq!(value["data"], json!([]));
    assert_eq!(value["diagnostics"], json!([]));
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
        .stderr(predicate::str::starts_with("fatal: args: invalid regex"))
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
        .stderr(predicate::str::is_empty());
}

#[test]
fn scope_filtered_tree_includes_ancestors_to_root() {
    let fixture = fixture_path("m2_core.vcd");
    let fixture = fixture.to_string_lossy().into_owned();

    wavepeek_cmd()
        .args([
            "scope",
            "--waves",
            fixture.as_str(),
            "--filter",
            "^top\\.cpu$",
            "--tree",
            "--max",
            "1",
        ])
        .assert()
        .success()
        .stdout(predicate::eq("top kind=module\n└── cpu kind=module\n"))
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
            "--filter",
            "^top\\.cpu$",
            "--max",
            "50",
        ])
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&assert.get_output().stdout).to_string();
    let value: Value = serde_json::from_str(&stdout).expect("scope output should be valid json");

    assert_eq!(value["diagnostics"], Value::Array(vec![]));
    assert_eq!(
        value["data"],
        json!([{ "path": "top.cpu", "depth": 1, "kind": "module" }])
    );
    assert_eq!(value["summary"]["returned"], 1);
    assert_eq!(value["summary"]["total"], 1);
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
    let fixture = ondas_fixture_path("fst/fst0022-scr1-max-axi-coremark");
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
