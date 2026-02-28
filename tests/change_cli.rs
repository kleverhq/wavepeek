use assert_cmd::prelude::*;
use predicates::prelude::*;
use serde_json::{Value, json};

mod common;
use common::{expected_schema_url, fixture_path, wavepeek_cmd};

fn parse_json(stdout: &[u8]) -> Value {
    serde_json::from_slice(stdout).expect("stdout should be valid json")
}

#[test]
fn change_requires_signals_flag() {
    let fixture = fixture_path("m2_core.vcd");
    let fixture = fixture.to_string_lossy().into_owned();

    wavepeek_cmd()
        .args(["change", "--waves", fixture.as_str()])
        .assert()
        .failure()
        .code(1)
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::starts_with("error: args:"))
        .stderr(predicate::str::contains("--signals <SIGNALS>"))
        .stderr(predicate::str::contains("See 'wavepeek change --help'."));
}

#[test]
fn change_default_when_matches_expected_json_payload() {
    let fixture = fixture_path("m2_core.vcd");
    let fixture = fixture.to_string_lossy().into_owned();

    let output = wavepeek_cmd()
        .args([
            "change",
            "--waves",
            fixture.as_str(),
            "--from",
            "1ns",
            "--to",
            "10ns",
            "--signals",
            "top.clk,top.data",
            "--json",
        ])
        .output()
        .expect("change should execute");

    assert!(output.status.success());
    assert!(output.stderr.is_empty());

    let value = parse_json(&output.stdout);
    assert_eq!(value["$schema"], expected_schema_url());
    assert_eq!(value["command"], "change");
    assert_eq!(value["warnings"], json!([]));
    assert_eq!(
        value["data"],
        json!([
            {
                "time": "5ns",
                "signals": [
                    {"path": "top.clk", "value": "1'h1"},
                    {"path": "top.data", "value": "8'h00"}
                ]
            },
            {
                "time": "10ns",
                "signals": [
                    {"path": "top.clk", "value": "1'h1"},
                    {"path": "top.data", "value": "8'h0f"}
                ]
            }
        ])
    );
}

#[test]
fn change_omitted_when_matches_explicit_wildcard() {
    let fixture = fixture_path("m2_core.vcd");
    let fixture = fixture.to_string_lossy().into_owned();

    let default_output = wavepeek_cmd()
        .args([
            "change",
            "--waves",
            fixture.as_str(),
            "--from",
            "1ns",
            "--to",
            "10ns",
            "--signals",
            "top.clk,top.data",
            "--json",
        ])
        .output()
        .expect("default --when run should execute");
    let star_output = wavepeek_cmd()
        .args([
            "change",
            "--waves",
            fixture.as_str(),
            "--from",
            "1ns",
            "--to",
            "10ns",
            "--signals",
            "top.clk,top.data",
            "--when",
            "*",
            "--json",
        ])
        .output()
        .expect("explicit wildcard run should execute");

    assert!(default_output.status.success());
    assert!(star_output.status.success());
    assert_eq!(default_output.stdout, star_output.stdout);
    assert_eq!(default_output.stderr, star_output.stderr);
}

#[test]
fn change_named_non_edge_trigger_emits_expected_single_row() {
    let fixture = fixture_path("m2_core.vcd");
    let fixture = fixture.to_string_lossy().into_owned();

    let output = wavepeek_cmd()
        .args([
            "change",
            "--waves",
            fixture.as_str(),
            "--from",
            "0ns",
            "--to",
            "10ns",
            "--scope",
            "top",
            "--signals",
            "data,clk",
            "--when",
            "data",
            "--json",
        ])
        .output()
        .expect("change should execute");

    assert!(output.status.success());
    assert!(output.stderr.is_empty());

    let value = parse_json(&output.stdout);
    assert_eq!(value["warnings"], json!([]));
    assert_eq!(
        value["data"],
        json!([
            {
                "time": "10ns",
                "signals": [
                    {"path": "top.data", "value": "8'h0f"},
                    {"path": "top.clk", "value": "1'h1"}
                ]
            }
        ])
    );
}

#[test]
fn change_zero_delta_path_returns_empty_data_with_warning() {
    let fixture = fixture_path("m2_core.vcd");
    let fixture = fixture.to_string_lossy().into_owned();

    let output = wavepeek_cmd()
        .args([
            "change",
            "--waves",
            fixture.as_str(),
            "--from",
            "0ns",
            "--to",
            "10ns",
            "--scope",
            "top",
            "--signals",
            "data",
            "--when",
            "posedge clk",
            "--json",
        ])
        .output()
        .expect("change should execute");

    assert!(output.status.success());
    let value = parse_json(&output.stdout);
    assert_eq!(value["data"], json!([]));
    assert_eq!(
        value["warnings"],
        json!(["no signal changes found in selected time range"])
    );
}

#[test]
fn change_zero_delta_warning_matches_between_json_and_human_modes() {
    let fixture = fixture_path("m2_core.vcd");
    let fixture = fixture.to_string_lossy().into_owned();

    let json_output = wavepeek_cmd()
        .args([
            "change",
            "--waves",
            fixture.as_str(),
            "--from",
            "0ns",
            "--to",
            "10ns",
            "--scope",
            "top",
            "--signals",
            "data",
            "--when",
            "posedge clk",
            "--json",
        ])
        .output()
        .expect("json change run should execute");
    let human_output = wavepeek_cmd()
        .args([
            "change",
            "--waves",
            fixture.as_str(),
            "--from",
            "0ns",
            "--to",
            "10ns",
            "--scope",
            "top",
            "--signals",
            "data",
            "--when",
            "posedge clk",
        ])
        .output()
        .expect("human change run should execute");

    assert!(json_output.status.success());
    assert!(human_output.status.success());
    assert!(human_output.stdout.is_empty());
    let json = parse_json(&json_output.stdout);
    assert_eq!(json["data"], json!([]));
    assert_eq!(
        json["warnings"],
        json!(["no signal changes found in selected time range"])
    );
    assert_eq!(
        String::from_utf8_lossy(&human_output.stderr).trim(),
        "warning: no signal changes found in selected time range"
    );
}

#[test]
fn change_unlimited_warning_precedes_empty_result_warning_in_json_and_human_modes() {
    let fixture = fixture_path("m2_core.vcd");
    let fixture = fixture.to_string_lossy().into_owned();

    let json_output = wavepeek_cmd()
        .args([
            "change",
            "--waves",
            fixture.as_str(),
            "--from",
            "9ns",
            "--to",
            "9ns",
            "--signals",
            "top.clk,top.data",
            "--max",
            "unlimited",
            "--json",
        ])
        .output()
        .expect("json change run should execute");
    let human_output = wavepeek_cmd()
        .args([
            "change",
            "--waves",
            fixture.as_str(),
            "--from",
            "9ns",
            "--to",
            "9ns",
            "--signals",
            "top.clk,top.data",
            "--max",
            "unlimited",
        ])
        .output()
        .expect("human change run should execute");

    assert!(json_output.status.success());
    assert!(human_output.status.success());

    let value = parse_json(&json_output.stdout);
    assert_eq!(value["data"], json!([]));
    assert_eq!(
        value["warnings"],
        json!([
            "limit disabled: --max=unlimited",
            "no signal changes found in selected time range"
        ])
    );
    assert_eq!(
        String::from_utf8_lossy(&human_output.stderr)
            .lines()
            .collect::<Vec<_>>(),
        vec![
            "warning: limit disabled: --max=unlimited",
            "warning: no signal changes found in selected time range"
        ]
    );
}

#[test]
fn change_omitted_from_uses_dump_start_baseline_checkpoint() {
    let fixture = fixture_path("change_from_boundary.vcd");
    let fixture = fixture.to_string_lossy().into_owned();

    let output = wavepeek_cmd()
        .args([
            "change",
            "--waves",
            fixture.as_str(),
            "--to",
            "5ns",
            "--scope",
            "top",
            "--signals",
            "sig",
            "--when",
            "*",
            "--json",
        ])
        .output()
        .expect("change should execute");

    assert!(output.status.success());
    let value = parse_json(&output.stdout);
    assert_eq!(value["warnings"], json!([]));
    assert_eq!(
        value["data"],
        json!([
            {
                "time": "5ns",
                "signals": [
                    {"path": "top.sig", "value": "1'h0"}
                ]
            }
        ])
    );
}

#[test]
fn change_from_timestamp_is_baseline_only_for_emission() {
    let fixture = fixture_path("m2_core.vcd");
    let fixture = fixture.to_string_lossy().into_owned();

    let output = wavepeek_cmd()
        .args([
            "change",
            "--waves",
            fixture.as_str(),
            "--from",
            "5ns",
            "--to",
            "10ns",
            "--scope",
            "top",
            "--signals",
            "clk",
            "--when",
            "posedge clk",
            "--json",
        ])
        .output()
        .expect("change should execute");

    assert!(output.status.success());
    let value = parse_json(&output.stdout);
    assert_eq!(value["data"], json!([]));
    assert_eq!(
        value["warnings"],
        json!(["no signal changes found in selected time range"])
    );
}

#[test]
fn change_equal_from_and_to_never_emits_baseline_row() {
    let fixture = fixture_path("m2_core.vcd");
    let fixture = fixture.to_string_lossy().into_owned();

    let output = wavepeek_cmd()
        .args([
            "change",
            "--waves",
            fixture.as_str(),
            "--from",
            "5ns",
            "--to",
            "5ns",
            "--scope",
            "top",
            "--signals",
            "clk",
            "--when",
            "posedge clk",
            "--json",
        ])
        .output()
        .expect("change should execute");

    assert!(output.status.success());
    let value = parse_json(&output.stdout);
    assert_eq!(value["data"], json!([]));
    assert_eq!(
        value["warnings"],
        json!(["no signal changes found in selected time range"])
    );
}

#[test]
fn change_union_or_and_comma_forms_are_exact_synonyms() {
    let fixture = fixture_path("change_edge_cases.vcd");
    let fixture = fixture.to_string_lossy().into_owned();

    let comma_output = wavepeek_cmd()
        .args([
            "change",
            "--waves",
            fixture.as_str(),
            "--from",
            "0ns",
            "--to",
            "30ns",
            "--scope",
            "top",
            "--signals",
            "clk1",
            "--when",
            "posedge clk1, posedge clk2",
            "--json",
        ])
        .output()
        .expect("comma form should execute");
    let or_output = wavepeek_cmd()
        .args([
            "change",
            "--waves",
            fixture.as_str(),
            "--from",
            "0ns",
            "--to",
            "30ns",
            "--scope",
            "top",
            "--signals",
            "clk1",
            "--when",
            "posedge clk1 or posedge clk2",
            "--json",
        ])
        .output()
        .expect("or form should execute");

    assert!(comma_output.status.success());
    assert!(or_output.status.success());
    assert_eq!(comma_output.stdout, or_output.stdout);
    assert_eq!(comma_output.stderr, or_output.stderr);
}

#[test]
fn change_union_overlap_timestamp_is_deduplicated() {
    let fixture = fixture_path("change_edge_cases.vcd");
    let fixture = fixture.to_string_lossy().into_owned();

    let output = wavepeek_cmd()
        .args([
            "change",
            "--waves",
            fixture.as_str(),
            "--from",
            "0ns",
            "--to",
            "30ns",
            "--scope",
            "top",
            "--signals",
            "clk1",
            "--when",
            "posedge clk1, posedge clk2",
            "--json",
        ])
        .output()
        .expect("change should execute");

    assert!(output.status.success());
    let value = parse_json(&output.stdout);
    let data = value["data"].as_array().expect("data should be array");
    let at_30ns = data.iter().filter(|entry| entry["time"] == "30ns").count();
    assert_eq!(at_30ns, 1, "30ns snapshot must appear exactly once");
}

#[test]
fn change_negedge_wiring_is_end_to_end() {
    let fixture = fixture_path("change_edge_cases.vcd");
    let fixture = fixture.to_string_lossy().into_owned();

    let output = wavepeek_cmd()
        .args([
            "change",
            "--waves",
            fixture.as_str(),
            "--from",
            "0ns",
            "--to",
            "30ns",
            "--scope",
            "top",
            "--signals",
            "clk",
            "--when",
            "negedge clk",
            "--json",
        ])
        .output()
        .expect("change should execute");

    assert!(output.status.success());
    let value = parse_json(&output.stdout);
    assert_eq!(value["warnings"], json!([]));
    assert_eq!(
        value["data"]
            .as_array()
            .expect("data should be array")
            .iter()
            .map(|entry| entry["time"].as_str().expect("time should be string"))
            .collect::<Vec<_>>(),
        vec!["10ns", "20ns", "25ns"]
    );
}

#[test]
fn change_edge_wiring_is_end_to_end() {
    let fixture = fixture_path("change_edge_cases.vcd");
    let fixture = fixture.to_string_lossy().into_owned();

    let output = wavepeek_cmd()
        .args([
            "change",
            "--waves",
            fixture.as_str(),
            "--from",
            "0ns",
            "--to",
            "30ns",
            "--scope",
            "top",
            "--signals",
            "clk",
            "--when",
            "edge clk",
            "--json",
        ])
        .output()
        .expect("change should execute");

    assert!(output.status.success());
    let value = parse_json(&output.stdout);
    assert_eq!(value["warnings"], json!([]));
    assert_eq!(
        value["data"]
            .as_array()
            .expect("data should be array")
            .iter()
            .map(|entry| entry["time"].as_str().expect("time should be string"))
            .collect::<Vec<_>>(),
        vec!["5ns", "10ns", "15ns", "20ns", "25ns", "30ns"]
    );
}

#[test]
fn change_iff_is_recognized_but_runtime_is_deferred() {
    let fixture = fixture_path("m2_core.vcd");
    let fixture = fixture.to_string_lossy().into_owned();

    wavepeek_cmd()
        .args([
            "change",
            "--waves",
            fixture.as_str(),
            "--scope",
            "top",
            "--signals",
            "data",
            "--when",
            "negedge clk iff rstn",
        ])
        .assert()
        .failure()
        .code(1)
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::starts_with("error: args:"))
        .stderr(predicate::str::contains(
            "iff logical expressions are not implemented yet",
        ));

    wavepeek_cmd()
        .args([
            "change",
            "--waves",
            fixture.as_str(),
            "--scope",
            "top",
            "--signals",
            "data",
            "--when",
            "posedge clk iff (",
        ])
        .assert()
        .failure()
        .code(1)
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::starts_with("error: args:"))
        .stderr(predicate::str::contains(
            "iff logical expressions are not implemented yet",
        ));
}

#[test]
fn change_empty_result_warning_matches_between_json_and_human_modes() {
    let fixture = fixture_path("m2_core.vcd");
    let fixture = fixture.to_string_lossy().into_owned();

    let json_output = wavepeek_cmd()
        .args([
            "change",
            "--waves",
            fixture.as_str(),
            "--from",
            "6ns",
            "--to",
            "9ns",
            "--signals",
            "top.clk,top.data",
            "--json",
        ])
        .output()
        .expect("json change run should execute");
    let human_output = wavepeek_cmd()
        .args([
            "change",
            "--waves",
            fixture.as_str(),
            "--from",
            "6ns",
            "--to",
            "9ns",
            "--signals",
            "top.clk,top.data",
        ])
        .output()
        .expect("human change run should execute");

    assert!(json_output.status.success());
    assert!(human_output.status.success());
    assert!(human_output.stdout.is_empty());

    let value = parse_json(&json_output.stdout);
    assert_eq!(value["data"], json!([]));
    assert_eq!(
        value["warnings"],
        json!(["no signal changes found in selected time range"])
    );
    assert_eq!(
        String::from_utf8_lossy(&human_output.stderr).trim(),
        "warning: no signal changes found in selected time range"
    );
}

#[test]
fn change_truncation_warning_matches_between_json_and_human_modes() {
    let fixture = fixture_path("m2_core.vcd");
    let fixture = fixture.to_string_lossy().into_owned();

    let json_output = wavepeek_cmd()
        .args([
            "change",
            "--waves",
            fixture.as_str(),
            "--from",
            "1ns",
            "--to",
            "10ns",
            "--signals",
            "top.clk,top.data",
            "--max",
            "1",
            "--json",
        ])
        .output()
        .expect("json change run should execute");
    let human_output = wavepeek_cmd()
        .args([
            "change",
            "--waves",
            fixture.as_str(),
            "--from",
            "1ns",
            "--to",
            "10ns",
            "--signals",
            "top.clk,top.data",
            "--max",
            "1",
        ])
        .output()
        .expect("human change run should execute");

    assert!(json_output.status.success());
    assert!(human_output.status.success());
    assert_eq!(
        String::from_utf8_lossy(&human_output.stdout).trim(),
        "@5ns top.clk=1'h1 top.data=8'h00"
    );
    assert_eq!(
        String::from_utf8_lossy(&human_output.stderr).trim(),
        "warning: truncated output to 1 entries (use --max to increase limit)"
    );

    let value = parse_json(&json_output.stdout);
    assert_eq!(
        value["data"]
            .as_array()
            .expect("data should be array")
            .len(),
        1
    );
    assert_eq!(
        value["warnings"],
        json!(["truncated output to 1 entries (use --max to increase limit)"])
    );
}

#[test]
fn change_abs_only_affects_human_labels_not_json_payload() {
    let fixture = fixture_path("m2_core.vcd");
    let fixture = fixture.to_string_lossy().into_owned();

    let json_default = wavepeek_cmd()
        .args([
            "change",
            "--waves",
            fixture.as_str(),
            "--from",
            "1ns",
            "--to",
            "10ns",
            "--scope",
            "top",
            "--signals",
            "data,clk",
            "--json",
        ])
        .output()
        .expect("json default run should execute");
    let json_abs = wavepeek_cmd()
        .args([
            "change",
            "--waves",
            fixture.as_str(),
            "--from",
            "1ns",
            "--to",
            "10ns",
            "--scope",
            "top",
            "--signals",
            "data,clk",
            "--json",
            "--abs",
        ])
        .output()
        .expect("json --abs run should execute");

    assert!(json_default.status.success());
    assert!(json_abs.status.success());
    assert_eq!(json_default.stdout, json_abs.stdout);

    let human_default = wavepeek_cmd()
        .args([
            "change",
            "--waves",
            fixture.as_str(),
            "--from",
            "1ns",
            "--to",
            "10ns",
            "--scope",
            "top",
            "--signals",
            "data,clk",
        ])
        .output()
        .expect("human default run should execute");
    let human_abs = wavepeek_cmd()
        .args([
            "change",
            "--waves",
            fixture.as_str(),
            "--from",
            "1ns",
            "--to",
            "10ns",
            "--scope",
            "top",
            "--signals",
            "data,clk",
            "--abs",
        ])
        .output()
        .expect("human abs run should execute");

    assert!(human_default.status.success());
    assert!(human_abs.status.success());
    let default_lines = String::from_utf8_lossy(&human_default.stdout);
    let abs_lines = String::from_utf8_lossy(&human_abs.stdout);
    assert!(default_lines.lines().next().is_some());
    assert!(abs_lines.lines().next().is_some());
    assert_eq!(
        default_lines
            .lines()
            .next()
            .expect("first line should exist"),
        "@5ns data=8'h00 clk=1'h1"
    );
    assert_eq!(
        abs_lines.lines().next().expect("first line should exist"),
        "@5ns top.data=8'h00 top.clk=1'h1"
    );
}

#[test]
fn change_preserves_duplicate_signal_order() {
    let fixture = fixture_path("m2_core.vcd");
    let fixture = fixture.to_string_lossy().into_owned();

    let output = wavepeek_cmd()
        .args([
            "change",
            "--waves",
            fixture.as_str(),
            "--from",
            "1ns",
            "--to",
            "10ns",
            "--signals",
            "top.data,top.clk,top.data",
            "--json",
        ])
        .output()
        .expect("change should execute");

    assert!(output.status.success());
    let value = parse_json(&output.stdout);
    let first_row = &value["data"][0]["signals"];
    assert_eq!(
        first_row,
        &json!([
            {"path": "top.data", "value": "8'h00"},
            {"path": "top.clk", "value": "1'h1"},
            {"path": "top.data", "value": "8'h00"}
        ])
    );
}

#[test]
fn change_default_max_is_50_with_truncation_warning() {
    let fixture = fixture_path("change_many_events.vcd");
    let fixture = fixture.to_string_lossy().into_owned();

    let output = wavepeek_cmd()
        .args([
            "change",
            "--waves",
            fixture.as_str(),
            "--signals",
            "top.sig",
            "--json",
        ])
        .output()
        .expect("change should execute");

    assert!(output.status.success());
    let value = parse_json(&output.stdout);
    assert_eq!(
        value["data"]
            .as_array()
            .expect("data should be array")
            .len(),
        50
    );
    assert_eq!(
        value["warnings"],
        json!(["truncated output to 50 entries (use --max to increase limit)"])
    );
}

#[test]
fn change_unlimited_max_disables_truncation_and_emits_warning_in_both_modes() {
    let fixture = fixture_path("change_many_events.vcd");
    let fixture = fixture.to_string_lossy().into_owned();

    let json_output = wavepeek_cmd()
        .args([
            "change",
            "--waves",
            fixture.as_str(),
            "--signals",
            "top.sig",
            "--max",
            "unlimited",
            "--json",
        ])
        .output()
        .expect("json run should execute");
    let human_output = wavepeek_cmd()
        .args([
            "change",
            "--waves",
            fixture.as_str(),
            "--signals",
            "top.sig",
            "--max",
            "unlimited",
        ])
        .output()
        .expect("human run should execute");

    assert!(json_output.status.success());
    assert!(human_output.status.success());

    let value = parse_json(&json_output.stdout);
    assert!(
        value["data"]
            .as_array()
            .expect("data should be array")
            .len()
            > 50
    );
    assert_eq!(
        value["warnings"],
        json!(["limit disabled: --max=unlimited"])
    );

    assert_eq!(
        String::from_utf8_lossy(&human_output.stderr).trim(),
        "warning: limit disabled: --max=unlimited"
    );
    assert!(
        String::from_utf8_lossy(&human_output.stdout)
            .lines()
            .count()
            > 50,
        "human output should include more than the default 50 rows"
    );
}

#[test]
fn change_rejects_zero_max_with_args_error() {
    let fixture = fixture_path("m2_core.vcd");
    let fixture = fixture.to_string_lossy().into_owned();

    wavepeek_cmd()
        .args([
            "change",
            "--waves",
            fixture.as_str(),
            "--signals",
            "top.clk",
            "--max",
            "0",
        ])
        .assert()
        .failure()
        .code(1)
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::starts_with(
            "error: args: --max must be greater than 0.",
        ));
}

#[test]
fn change_validates_error_paths_for_args_scope_and_signal_resolution() {
    let fixture = fixture_path("m2_core.vcd");
    let fixture = fixture.to_string_lossy().into_owned();

    wavepeek_cmd()
        .args([
            "change",
            "--waves",
            fixture.as_str(),
            "--signals",
            "top.clk",
            "--max",
            "0",
        ])
        .assert()
        .failure()
        .code(1)
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::starts_with(
            "error: args: --max must be greater than 0.",
        ));

    wavepeek_cmd()
        .args([
            "change",
            "--waves",
            fixture.as_str(),
            "--from",
            "10ns",
            "--to",
            "1ns",
            "--signals",
            "top.clk",
        ])
        .assert()
        .failure()
        .code(1)
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::starts_with("error: args:"))
        .stderr(predicate::str::contains(
            "--from must be less than or equal to --to",
        ));

    wavepeek_cmd()
        .args([
            "change",
            "--waves",
            fixture.as_str(),
            "--from",
            "10",
            "--signals",
            "top.clk",
        ])
        .assert()
        .failure()
        .code(1)
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::starts_with("error: args:"))
        .stderr(predicate::str::contains("requires units"));

    wavepeek_cmd()
        .args([
            "change",
            "--waves",
            fixture.as_str(),
            "--scope",
            "top",
            "--signals",
            "top.clk",
            "--when",
            "posedge top.clk",
        ])
        .assert()
        .failure()
        .code(1)
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::starts_with("error: signal:"));

    wavepeek_cmd()
        .args([
            "change",
            "--waves",
            fixture.as_str(),
            "--when",
            "posedge nope",
            "--signals",
            "top.clk",
        ])
        .assert()
        .failure()
        .code(1)
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::starts_with("error: signal:"));

    wavepeek_cmd()
        .args([
            "change",
            "--waves",
            fixture.as_str(),
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

    wavepeek_cmd()
        .args([
            "change",
            "--waves",
            fixture.as_str(),
            "--from",
            "15ps",
            "--signals",
            "top.clk",
        ])
        .assert()
        .failure()
        .code(1)
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::starts_with("error: args:"))
        .stderr(predicate::str::contains(
            "cannot be represented exactly in dump precision",
        ));
}

#[test]
fn change_invalid_when_signal_fails_even_without_in_range_timestamps() {
    let fixture = fixture_path("m2_core.vcd");
    let fixture = fixture.to_string_lossy().into_owned();

    wavepeek_cmd()
        .args([
            "change",
            "--waves",
            fixture.as_str(),
            "--from",
            "1ns",
            "--to",
            "4ns",
            "--signals",
            "top.clk",
            "--when",
            "posedge top.nope",
        ])
        .assert()
        .failure()
        .code(1)
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::starts_with("error: signal:"));

    wavepeek_cmd()
        .args([
            "change",
            "--waves",
            fixture.as_str(),
            "--from",
            "1ns",
            "--to",
            "4ns",
            "--scope",
            "top",
            "--signals",
            "clk",
            "--when",
            "posedge nope",
        ])
        .assert()
        .failure()
        .code(1)
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::starts_with("error: signal:"));
}

#[test]
fn change_scoped_mode_rejects_canonical_tokens_even_if_prefixed_path_exists() {
    let fixture = fixture_path("change_scope_ambiguous.vcd");
    let fixture = fixture.to_string_lossy().into_owned();

    wavepeek_cmd()
        .args([
            "change",
            "--waves",
            fixture.as_str(),
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

    wavepeek_cmd()
        .args([
            "change",
            "--waves",
            fixture.as_str(),
            "--scope",
            "top",
            "--signals",
            "clk",
            "--when",
            "posedge top.clk",
        ])
        .assert()
        .failure()
        .code(1)
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::starts_with("error: signal:"));
}
