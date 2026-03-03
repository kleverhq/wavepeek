use std::io::Write;

use serde_json::{Value, json};
use tempfile::NamedTempFile;

mod common;
use common::wavepeek_cmd;

fn parse_json(stdout: &[u8]) -> Value {
    serde_json::from_slice(stdout).expect("stdout should be valid json")
}

fn run_change_json_with_modes(waves: &str, extra_args: &[&str]) -> Value {
    let pre_fusion = run_change_json_with_mode(waves, "pre-fusion", "random", extra_args);
    let fused = run_change_json_with_mode(waves, "fused", "random", extra_args);

    assert_eq!(pre_fusion["data"], fused["data"]);
    assert_eq!(pre_fusion["warnings"], fused["warnings"]);
    fused
}

fn run_change_json_with_edge_modes(waves: &str, extra_args: &[&str]) -> Value {
    let pre_fusion = run_change_json_with_mode(waves, "pre-fusion", "random", extra_args);
    let fused = run_change_json_with_mode(waves, "fused", "random", extra_args);
    let edge_fast = run_change_json_with_mode(waves, "edge-fast", "random", extra_args);

    assert_eq!(pre_fusion["data"], fused["data"]);
    assert_eq!(pre_fusion["warnings"], fused["warnings"]);
    assert_eq!(pre_fusion["data"], edge_fast["data"]);
    assert_eq!(pre_fusion["warnings"], edge_fast["warnings"]);

    edge_fast
}

fn run_change_json_with_mode(
    waves: &str,
    engine_mode: &str,
    candidate_mode: &str,
    extra_args: &[&str],
) -> Value {
    let mut args = vec![
        "change",
        "--waves",
        waves,
        "--internal-change-engine",
        engine_mode,
        "--internal-change-candidates",
        candidate_mode,
    ];
    if engine_mode == "edge-fast" {
        args.push("--internal-change-edge-fast-force");
    }
    args.extend_from_slice(extra_args);
    args.push("--json");

    let output = wavepeek_cmd()
        .args(args)
        .output()
        .expect("change should execute");

    assert!(output.status.success());
    parse_json(&output.stdout)
}

#[test]
fn change_uses_strict_previous_timestamp_not_previous_candidate() {
    let fixture = write_fixture(
        "$date\n  today\n$end\n$version\n  wavepeek-test\n$end\n$timescale 1ns $end\n$scope module top $end\n$var wire 1 ! trig $end\n$var wire 1 \" sig $end\n$upscope $end\n$enddefinitions $end\n#0\n0!\n0\"\n#5\n1!\n1\"\n#7\n0\"\n#8\n0!\n#10\n1!\n",
        "change-opt-equivalence.vcd",
    );
    let fixture = fixture.path().to_string_lossy().into_owned();

    let value = run_change_json_with_modes(
        fixture.as_str(),
        &[
            "--from",
            "0ns",
            "--to",
            "10ns",
            "--signals",
            "top.sig",
            "--when",
            "posedge top.trig",
        ],
    );

    assert_eq!(value["warnings"], json!([]));
    assert_eq!(
        value["data"],
        json!([
            {
                "time": "5ns",
                "signals": [
                    {"path": "top.sig", "value": "1'h1"}
                ]
            }
        ])
    );
}

#[test]
fn change_from_inside_window_respects_intermediate_non_candidate_updates() {
    let fixture = write_fixture(
        "$date\n  today\n$end\n$version\n  wavepeek-test\n$end\n$timescale 1ns $end\n$scope module top $end\n$var wire 1 ! trig $end\n$var wire 1 \" sig $end\n$upscope $end\n$enddefinitions $end\n#0\n0!\n0\"\n#5\n1!\n1\"\n#7\n0\"\n#8\n0!\n#10\n1!\n",
        "change-opt-equivalence.vcd",
    );
    let fixture = fixture.path().to_string_lossy().into_owned();

    let value = run_change_json_with_modes(
        fixture.as_str(),
        &[
            "--from",
            "6ns",
            "--to",
            "10ns",
            "--signals",
            "top.sig",
            "--when",
            "posedge top.trig",
        ],
    );

    assert_eq!(value["data"], json!([]));
    assert_eq!(
        value["warnings"],
        json!(["no signal changes found in selected time range"])
    );
}

#[test]
fn change_empty_window_from_equals_to_remains_empty() {
    let fixture = write_fixture(
        "$date\n  today\n$end\n$version\n  wavepeek-test\n$end\n$timescale 1ns $end\n$scope module top $end\n$var wire 1 ! sig $end\n$upscope $end\n$enddefinitions $end\n#0\n0!\n#5\n1!\n",
        "change-empty-window.vcd",
    );
    let fixture = fixture.path().to_string_lossy().into_owned();

    let value = run_change_json_with_modes(
        fixture.as_str(),
        &[
            "--from",
            "5ns",
            "--to",
            "5ns",
            "--signals",
            "top.sig",
            "--when",
            "top.sig",
        ],
    );

    assert_eq!(value["data"], json!([]));
    assert_eq!(
        value["warnings"],
        json!(["no signal changes found in selected time range"])
    );
}

#[test]
fn change_all_candidates_at_or_before_baseline_do_not_emit() {
    let fixture = write_fixture(
        "$date\n  today\n$end\n$version\n  wavepeek-test\n$end\n$timescale 1ns $end\n$scope module top $end\n$var wire 1 ! trig $end\n$var wire 1 \" sig $end\n$upscope $end\n$enddefinitions $end\n#0\n0!\n0\"\n#5\n1!\n1\"\n",
        "change-baseline-window.vcd",
    );
    let fixture = fixture.path().to_string_lossy().into_owned();

    let value = run_change_json_with_modes(
        fixture.as_str(),
        &[
            "--from",
            "5ns",
            "--to",
            "5ns",
            "--signals",
            "top.sig",
            "--when",
            "posedge top.trig",
        ],
    );

    assert_eq!(value["data"], json!([]));
    assert_eq!(
        value["warnings"],
        json!(["no signal changes found in selected time range"])
    );
}

#[test]
fn change_max_one_truncation_matches_between_modes() {
    let fixture = write_fixture(
        "$date\n  today\n$end\n$version\n  wavepeek-test\n$end\n$timescale 1ns $end\n$scope module top $end\n$var wire 4 ! sig $end\n$upscope $end\n$enddefinitions $end\n#0\nb0000 !\n#5\nb0001 !\n#10\nb0010 !\n#15\nb0011 !\n",
        "change-max-one.vcd",
    );
    let fixture = fixture.path().to_string_lossy().into_owned();

    let value = run_change_json_with_modes(
        fixture.as_str(),
        &[
            "--from",
            "0ns",
            "--to",
            "15ns",
            "--signals",
            "top.sig",
            "--max",
            "1",
        ],
    );

    assert_eq!(
        value["data"],
        json!([
            {
                "time": "5ns",
                "signals": [
                    {"path": "top.sig", "value": "4'h1"}
                ]
            }
        ])
    );
    assert_eq!(
        value["warnings"],
        json!(["truncated output to 1 entries (use --max to increase limit)"])
    );
}

#[test]
fn change_redundant_same_value_dump_does_not_emit_row() {
    let fixture = write_fixture(
        "$date\n  today\n$end\n$version\n  wavepeek-test\n$end\n$timescale 1ns $end\n$scope module top $end\n$var wire 4 ! sig $end\n$upscope $end\n$enddefinitions $end\n#0\nb0000 !\n#5\nb0000 !\n#10\nb0001 !\n",
        "change-redundant-value.vcd",
    );
    let fixture = fixture.path().to_string_lossy().into_owned();

    let value = run_change_json_with_modes(
        fixture.as_str(),
        &[
            "--from",
            "0ns",
            "--to",
            "10ns",
            "--signals",
            "top.sig",
            "--when",
            "top.sig",
        ],
    );

    assert_eq!(
        value["data"],
        json!([
            {
                "time": "10ns",
                "signals": [
                    {"path": "top.sig", "value": "4'h1"}
                ]
            }
        ])
    );
}

#[test]
fn change_anychange_trigger_detects_none_to_some_transition() {
    let fixture = write_fixture(
        "$date\n  today\n$end\n$version\n  wavepeek-test\n$end\n$timescale 1ns $end\n$scope module top $end\n$var wire 1 ! trig $end\n$var wire 1 \" sig $end\n$upscope $end\n$enddefinitions $end\n#0\n0\"\n#5\n1!\n1\"\n",
        "change-anychange-none-to-some.vcd",
    );
    let fixture = fixture.path().to_string_lossy().into_owned();

    let value = run_change_json_with_modes(
        fixture.as_str(),
        &[
            "--from",
            "0ns",
            "--to",
            "5ns",
            "--signals",
            "top.sig",
            "--when",
            "top.trig",
        ],
    );

    assert_eq!(value["warnings"], json!([]));
    assert_eq!(
        value["data"],
        json!([
            {
                "time": "5ns",
                "signals": [
                    {"path": "top.sig", "value": "1'h1"}
                ]
            }
        ])
    );
}

#[test]
fn change_forced_edge_fast_falls_back_for_non_edge_triggers() {
    let fixture = write_fixture(
        "$date\n  today\n$end\n$version\n  wavepeek-test\n$end\n$timescale 1ns $end\n$scope module top $end\n$var wire 1 ! sig $end\n$upscope $end\n$enddefinitions $end\n#0\n0!\n#5\n1!\n#10\n0!\n",
        "change-edge-fast-fallback.vcd",
    );
    let fixture = fixture.path().to_string_lossy().into_owned();

    let args = [
        "--from",
        "0ns",
        "--to",
        "10ns",
        "--signals",
        "top.sig",
        "--when",
        "top.sig",
    ];

    let pre_fusion = run_change_json_with_mode(fixture.as_str(), "pre-fusion", "random", &args);
    let edge_fast = run_change_json_with_mode(fixture.as_str(), "edge-fast", "random", &args);

    assert_eq!(pre_fusion["data"], edge_fast["data"]);
    assert_eq!(pre_fusion["warnings"], edge_fast["warnings"]);
}

#[test]
fn change_trigger_matrix_matches_between_prefusion_and_fused() {
    let fixture = write_fixture(
        "$date\n  today\n$end\n$version\n  wavepeek-test\n$end\n$timescale 1ns $end\n$scope module top $end\n$var wire 1 ! clk $end\n$var wire 1 \" data $end\n$var wire 1 # gate $end\n$upscope $end\n$enddefinitions $end\n#0\n0!\n0\"\n0#\n#5\n1!\n0\"\n1#\n#10\n0!\n1\"\n1#\n#15\n1!\n1\"\n0#\n#20\n0!\n0\"\n0#\n",
        "change-trigger-matrix.vcd",
    );
    let fixture = fixture.path().to_string_lossy().into_owned();

    for args in [
        vec![
            "--from",
            "0ns",
            "--to",
            "20ns",
            "--signals",
            "top.data",
            "--when",
            "negedge top.clk",
        ],
        vec![
            "--from",
            "0ns",
            "--to",
            "20ns",
            "--signals",
            "top.data",
            "--when",
            "edge top.clk",
        ],
        vec![
            "--from",
            "0ns",
            "--to",
            "20ns",
            "--signals",
            "top.data",
            "--when",
            "posedge top.clk, negedge top.gate",
        ],
        vec![
            "--from",
            "0ns",
            "--to",
            "20ns",
            "--signals",
            "top.data",
            "--when",
            "top.gate",
        ],
    ] {
        let value = run_change_json_with_modes(fixture.as_str(), args.as_slice());
        assert!(value["data"].is_array());
        assert!(value["warnings"].is_array());
    }
}

#[test]
fn change_dense_posedge_sparse_delta_matches_all_modes() {
    let fixture = write_fixture(
        "$date\n  today\n$end\n$version\n  wavepeek-test\n$end\n$timescale 1ns $end\n$scope module top $end\n$var wire 1 ! clk $end\n$var wire 1 \" gate $end\n$var wire 1 # data $end\n$upscope $end\n$enddefinitions $end\n#0\n0!\n1\"\n0#\n#1\n1!\n#2\n0!\n#3\n1!\n0\"\n#4\n0!\n#5\n1!\n1#\n#6\n0!\n#7\n1!\n#8\n0!\n1\"\n#9\n1!\n0\"\n0#\n#10\n0!\n",
        "change-dense-edge.vcd",
    );
    let fixture = fixture.path().to_string_lossy().into_owned();

    let value = run_change_json_with_edge_modes(
        fixture.as_str(),
        &[
            "--from",
            "0ns",
            "--to",
            "10ns",
            "--signals",
            "top.data",
            "--when",
            "posedge top.clk",
        ],
    );

    assert_eq!(value["warnings"], json!([]));
    assert_eq!(
        value["data"],
        json!([
            {
                "time": "5ns",
                "signals": [
                    {"path": "top.data", "value": "1'h1"}
                ]
            },
            {
                "time": "9ns",
                "signals": [
                    {"path": "top.data", "value": "1'h0"}
                ]
            }
        ])
    );
}

#[test]
fn change_edge_without_requested_delta_remains_empty_in_all_modes() {
    let fixture = write_fixture(
        "$date\n  today\n$end\n$version\n  wavepeek-test\n$end\n$timescale 1ns $end\n$scope module top $end\n$var wire 1 ! clk $end\n$var wire 1 \" gate $end\n$var wire 1 # data $end\n$upscope $end\n$enddefinitions $end\n#0\n0!\n1\"\n0#\n#1\n1!\n#2\n0!\n#3\n1!\n0\"\n#4\n0!\n",
        "change-dense-edge-empty.vcd",
    );
    let fixture = fixture.path().to_string_lossy().into_owned();

    let value = run_change_json_with_edge_modes(
        fixture.as_str(),
        &[
            "--from",
            "0ns",
            "--to",
            "4ns",
            "--signals",
            "top.data",
            "--when",
            "posedge top.clk",
        ],
    );

    assert_eq!(value["data"], json!([]));
    assert_eq!(
        value["warnings"],
        json!(["no signal changes found in selected time range"])
    );
}

#[test]
fn change_union_edge_trigger_dedups_same_timestamp_in_all_modes() {
    let fixture = write_fixture(
        "$date\n  today\n$end\n$version\n  wavepeek-test\n$end\n$timescale 1ns $end\n$scope module top $end\n$var wire 1 ! clk $end\n$var wire 1 \" gate $end\n$var wire 1 # data $end\n$upscope $end\n$enddefinitions $end\n#0\n0!\n1\"\n0#\n#1\n1!\n#2\n0!\n#3\n1!\n0\"\n#4\n0!\n#5\n1!\n1#\n#6\n0!\n#7\n1!\n#8\n0!\n1\"\n#9\n1!\n0\"\n0#\n#10\n0!\n",
        "change-dense-edge-union.vcd",
    );
    let fixture = fixture.path().to_string_lossy().into_owned();

    let value = run_change_json_with_edge_modes(
        fixture.as_str(),
        &[
            "--from",
            "0ns",
            "--to",
            "10ns",
            "--signals",
            "top.data",
            "--when",
            "posedge top.clk, negedge top.gate",
        ],
    );

    assert_eq!(value["warnings"], json!([]));
    assert_eq!(
        value["data"],
        json!([
            {
                "time": "5ns",
                "signals": [
                    {"path": "top.data", "value": "1'h1"}
                ]
            },
            {
                "time": "9ns",
                "signals": [
                    {"path": "top.data", "value": "1'h0"}
                ]
            }
        ])
    );
}

#[test]
fn change_dense_edge_max_one_truncation_matches_all_modes() {
    let fixture = write_fixture(
        "$date\n  today\n$end\n$version\n  wavepeek-test\n$end\n$timescale 1ns $end\n$scope module top $end\n$var wire 1 ! clk $end\n$var wire 1 \" gate $end\n$var wire 1 # data $end\n$upscope $end\n$enddefinitions $end\n#0\n0!\n1\"\n0#\n#1\n1!\n#2\n0!\n#3\n1!\n0\"\n#4\n0!\n#5\n1!\n1#\n#6\n0!\n#7\n1!\n#8\n0!\n1\"\n#9\n1!\n0\"\n0#\n#10\n0!\n",
        "change-dense-edge-max-one.vcd",
    );
    let fixture = fixture.path().to_string_lossy().into_owned();

    let value = run_change_json_with_edge_modes(
        fixture.as_str(),
        &[
            "--from",
            "0ns",
            "--to",
            "10ns",
            "--signals",
            "top.data",
            "--when",
            "posedge top.clk",
            "--max",
            "1",
        ],
    );

    assert_eq!(
        value["data"],
        json!([
            {
                "time": "5ns",
                "signals": [
                    {"path": "top.data", "value": "1'h1"}
                ]
            }
        ])
    );
    assert_eq!(
        value["warnings"],
        json!(["truncated output to 1 entries (use --max to increase limit)"])
    );
}

fn write_fixture(contents: &str, filename: &str) -> NamedTempFile {
    let mut file = tempfile::Builder::new()
        .suffix(filename)
        .tempfile()
        .expect("tempfile should be created");
    file.write_all(contents.as_bytes())
        .expect("fixture should be written");
    file
}
