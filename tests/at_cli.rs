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

#[test]
fn at_human_output_with_scope_is_default() {
    let fixture = fixture_path("m2_core.vcd");
    let fixture = fixture.to_string_lossy().into_owned();

    let mut command = wavepeek_cmd();
    command
        .args([
            "at",
            "--waves",
            fixture.as_str(),
            "--time",
            "10ns",
            "--scope",
            "top",
            "--signals",
            "clk,data",
        ])
        .assert()
        .success()
        .stdout(predicate::eq("@10ns\nclk 1'h1\ndata 8'h0f\n"))
        .stderr(predicate::str::is_empty());
}

#[test]
fn at_human_output_with_abs_shows_canonical_paths() {
    let fixture = fixture_path("m2_core.vcd");
    let fixture = fixture.to_string_lossy().into_owned();

    let mut command = wavepeek_cmd();
    command
        .args([
            "at",
            "--waves",
            fixture.as_str(),
            "--time",
            "10ns",
            "--scope",
            "top",
            "--signals",
            "clk,data",
            "--abs",
        ])
        .assert()
        .success()
        .stdout(predicate::eq("@10ns\ntop.clk 1'h1\ntop.data 8'h0f\n"))
        .stderr(predicate::str::is_empty());
}

#[test]
fn at_requires_signals_flag() {
    let fixture = fixture_path("m2_core.vcd");
    let fixture = fixture.to_string_lossy().into_owned();

    let mut command = wavepeek_cmd();
    command
        .args(["at", "--waves", fixture.as_str(), "--time", "10ns"])
        .assert()
        .failure()
        .code(1)
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::starts_with("error: args:"))
        .stderr(predicate::str::contains("--signals <SIGNALS>"))
        .stderr(predicate::str::contains("See 'wavepeek at --help'."));
}

#[test]
fn at_json_shape_with_scope_is_stable_and_ordered() {
    let fixture = fixture_path("m2_core.vcd");
    let fixture = fixture.to_string_lossy().into_owned();

    let mut command = wavepeek_cmd();
    let assert = command
        .args([
            "at",
            "--waves",
            fixture.as_str(),
            "--time",
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
    let value: Value = serde_json::from_str(&stdout).expect("at output should be valid json");

    assert_eq!(value["$schema"], expected_schema_url());
    assert_eq!(value["command"], "at");
    assert_eq!(value["warnings"], Value::Array(vec![]));
    assert_eq!(
        value["data"],
        json!({
            "time": "10ns",
            "signals": [
                {"path": "top.clk", "value": "1'h1"},
                {"path": "top.data", "value": "8'h0f"}
            ]
        })
    );
}

#[test]
fn at_without_scope_treats_signals_as_canonical_paths() {
    let fixture = fixture_path("m2_core.vcd");
    let fixture = fixture.to_string_lossy().into_owned();

    let mut command = wavepeek_cmd();
    let assert = command
        .args([
            "at",
            "--waves",
            fixture.as_str(),
            "--time",
            "10ns",
            "--signals",
            "top.clk,top.data",
            "--json",
        ])
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&assert.get_output().stdout).to_string();
    let value: Value = serde_json::from_str(&stdout).expect("at output should be valid json");

    assert_eq!(
        value["data"]["signals"],
        json!([
            {"path": "top.clk", "value": "1'h1"},
            {"path": "top.data", "value": "8'h0f"}
        ])
    );
}

#[test]
fn at_json_output_is_identical_with_and_without_abs() {
    let fixture = fixture_path("m2_core.vcd");
    let fixture = fixture.to_string_lossy().into_owned();

    let without_abs = wavepeek_cmd()
        .args([
            "at",
            "--waves",
            fixture.as_str(),
            "--time",
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
            "at",
            "--waves",
            fixture.as_str(),
            "--time",
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
fn at_invalid_time_token_is_args_error() {
    let fixture = fixture_path("m2_core.vcd");
    let fixture = fixture.to_string_lossy().into_owned();

    let mut command = wavepeek_cmd();
    command
        .args([
            "at",
            "--waves",
            fixture.as_str(),
            "--time",
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
        .stderr(predicate::str::starts_with("error: args:"))
        .stderr(predicate::str::contains("invalid time token '100'"))
        .stderr(predicate::str::contains("See 'wavepeek at --help'."));
}

#[test]
fn at_decimal_time_token_is_rejected_as_args_error() {
    let fixture = fixture_path("m2_core.vcd");
    let fixture = fixture.to_string_lossy().into_owned();

    let mut command = wavepeek_cmd();
    command
        .args([
            "at",
            "--waves",
            fixture.as_str(),
            "--time",
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
        .stderr(predicate::str::starts_with("error: args:"))
        .stderr(predicate::str::contains("invalid time token '1.5ns'"))
        .stderr(predicate::str::contains("See 'wavepeek at --help'."));
}

#[test]
fn at_out_of_range_time_is_args_error_with_bounds() {
    let fixture = fixture_path("m2_core.vcd");
    let fixture = fixture.to_string_lossy().into_owned();

    let mut command = wavepeek_cmd();
    command
        .args([
            "at",
            "--waves",
            fixture.as_str(),
            "--time",
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
        .stderr(predicate::str::starts_with("error: args:"))
        .stderr(predicate::str::contains(
            "time '11ns' is outside dump bounds [0ns, 10ns]",
        ))
        .stderr(predicate::str::contains("See 'wavepeek at --help'."));
}

#[test]
fn at_scope_not_found_is_scope_error() {
    let fixture = fixture_path("m2_core.vcd");
    let fixture = fixture.to_string_lossy().into_owned();

    let mut command = wavepeek_cmd();
    command
        .args([
            "at",
            "--waves",
            fixture.as_str(),
            "--time",
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
        .stderr(predicate::str::starts_with("error: scope:"));
}

#[test]
fn at_missing_signal_is_signal_error_and_fails_fast() {
    let fixture = fixture_path("m2_core.vcd");
    let fixture = fixture.to_string_lossy().into_owned();

    let mut command = wavepeek_cmd();
    command
        .args([
            "at",
            "--waves",
            fixture.as_str(),
            "--time",
            "10ns",
            "--signals",
            "top.nope,top.clk",
        ])
        .assert()
        .failure()
        .code(1)
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::starts_with("error: signal:"));
}

#[test]
fn at_preserves_duplicate_signal_order() {
    let fixture = fixture_path("m2_core.vcd");
    let fixture = fixture.to_string_lossy().into_owned();

    let mut command = wavepeek_cmd();
    let assert = command
        .args([
            "at",
            "--waves",
            fixture.as_str(),
            "--time",
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
    let value: Value = serde_json::from_str(&stdout).expect("at output should be valid json");

    assert_eq!(
        value["data"]["signals"],
        json!([
            {"path": "top.clk", "value": "1'h1"},
            {"path": "top.clk", "value": "1'h1"},
            {"path": "top.data", "value": "8'h0f"}
        ])
    );
}

#[test]
fn at_mixed_mode_names_fail_without_scope() {
    let fixture = fixture_path("m2_core.vcd");
    let fixture = fixture.to_string_lossy().into_owned();

    let mut command = wavepeek_cmd();
    command
        .args([
            "at",
            "--waves",
            fixture.as_str(),
            "--time",
            "10ns",
            "--signals",
            "clk,top.data",
        ])
        .assert()
        .failure()
        .code(1)
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::starts_with("error: signal:"));
}

#[test]
fn at_mixed_mode_names_resolve_with_scope() {
    let fixture = fixture_path("m2_core.vcd");
    let fixture = fixture.to_string_lossy().into_owned();

    let mut command = wavepeek_cmd();
    command
        .args([
            "at",
            "--waves",
            fixture.as_str(),
            "--time",
            "10ns",
            "--scope",
            "top",
            "--signals",
            "clk,data",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("clk 1'h1"))
        .stdout(predicate::str::contains("data 8'h0f"))
        .stderr(predicate::str::is_empty());
}

#[test]
fn at_full_paths_are_not_accepted_when_scope_is_set() {
    let fixture = fixture_path("m2_core.vcd");
    let fixture = fixture.to_string_lossy().into_owned();

    let mut command = wavepeek_cmd();
    command
        .args([
            "at",
            "--waves",
            fixture.as_str(),
            "--time",
            "10ns",
            "--scope",
            "top",
            "--signals",
            "top.clk",
        ])
        .assert()
        .failure()
        .code(1)
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::starts_with("error: signal:"));
}

#[test]
fn at_accepts_inclusive_time_bounds() {
    let fixture = fixture_path("m2_core.vcd");
    let fixture = fixture.to_string_lossy().into_owned();

    wavepeek_cmd()
        .args([
            "at",
            "--waves",
            fixture.as_str(),
            "--time",
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
            "at",
            "--waves",
            fixture.as_str(),
            "--time",
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
fn at_json_is_deterministic_across_identical_runs() {
    let fixture = fixture_path("m2_core.vcd");
    let fixture = fixture.to_string_lossy().into_owned();

    let first = wavepeek_cmd()
        .args([
            "at",
            "--waves",
            fixture.as_str(),
            "--time",
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
            "at",
            "--waves",
            fixture.as_str(),
            "--time",
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
fn at_vcd_and_fst_payloads_match() {
    let vcd_fixture = fixture_path("m2_core.vcd");
    let vcd_fixture = vcd_fixture.to_string_lossy().into_owned();
    let fst_fixture = fixture_path("m2_core.fst");
    let fst_fixture = fst_fixture.to_string_lossy().into_owned();

    let vcd_output = wavepeek_cmd()
        .args([
            "at",
            "--waves",
            vcd_fixture.as_str(),
            "--time",
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
            "at",
            "--waves",
            fst_fixture.as_str(),
            "--time",
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
