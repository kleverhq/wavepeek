use std::io::Write;

use serde_json::{Value, json};
use tempfile::NamedTempFile;

mod common;
use common::{fixture_path, wavepeek_cmd};

fn parse_json(stdout: &[u8]) -> Value {
    serde_json::from_slice(stdout).expect("stdout should be valid json")
}

fn run_change_json(waves: &str, extra_args: &[&str]) -> Value {
    let mut args = vec!["change", "--waves", waves, "--sample-mode", "native"];
    if !extra_args.contains(&"--row-mode") {
        args.extend_from_slice(&["--row-mode", "sparse"]);
    }
    if !extra_args.contains(&"--on") {
        args.extend_from_slice(&["--on", "*"]);
    }
    args.extend_from_slice(extra_args);
    args.push("--json");
    let output = wavepeek_cmd()
        .args(args)
        .output()
        .expect("change should execute");
    assert!(
        output.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    parse_json(&output.stdout)
}

#[test]
fn change_sparse_compares_with_previous_selected_sample() {
    let fixture = write_fixture(
        "$date\n  today\n$end\n$version\n  wavepeek-test\n$end\n$timescale 1ns $end\n$scope module top $end\n$var wire 1 ! trig $end\n$var wire 1 \" sig $end\n$upscope $end\n$enddefinitions $end\n#0\n0!\n0\"\n#5\n1!\n1\"\n#7\n0\"\n#8\n0!\n#10\n1!\n",
        "change-opt-equivalence.vcd",
    );
    let fixture = fixture.path().to_string_lossy().into_owned();

    let value = run_change_json(
        fixture.as_str(),
        &[
            "--from",
            "0ns",
            "--to",
            "10ns",
            "--signals",
            "top.sig",
            "--on",
            "posedge top.trig",
        ],
    );

    assert_eq!(value["diagnostics"], json!([]));
    assert_eq!(
        value["data"],
        json!([
            {
                "time": "5ns",
                "sample_time": "5ns",
                "signals": [
                    {"path": "top.sig", "value": "1'h1"}
                ]
            },
            {
                "time": "10ns",
                "sample_time": "10ns",
                "signals": [
                    {"path": "top.sig", "value": "1'h0"}
                ]
            }
        ])
    );
}

#[test]
fn change_from_inside_window_compares_with_range_baseline() {
    let fixture = write_fixture(
        "$date\n  today\n$end\n$version\n  wavepeek-test\n$end\n$timescale 1ns $end\n$scope module top $end\n$var wire 1 ! trig $end\n$var wire 1 \" sig $end\n$upscope $end\n$enddefinitions $end\n#0\n0!\n0\"\n#5\n1!\n1\"\n#7\n0\"\n#8\n0!\n#10\n1!\n",
        "change-opt-equivalence.vcd",
    );
    let fixture = fixture.path().to_string_lossy().into_owned();

    let value = run_change_json(
        fixture.as_str(),
        &[
            "--from",
            "6ns",
            "--to",
            "10ns",
            "--signals",
            "top.sig",
            "--on",
            "posedge top.trig",
        ],
    );

    assert_eq!(value["diagnostics"], json!([]));
    assert_eq!(
        value["data"],
        json!([{
            "time": "10ns",
            "sample_time": "10ns",
            "signals": [{"path": "top.sig", "value": "1'h0"}]
        }])
    );
}

#[test]
fn change_empty_window_from_equals_to_remains_empty() {
    let fixture = write_fixture(
        "$date\n  today\n$end\n$version\n  wavepeek-test\n$end\n$timescale 1ns $end\n$scope module top $end\n$var wire 1 ! sig $end\n$upscope $end\n$enddefinitions $end\n#0\n0!\n#5\n1!\n",
        "change-empty-window.vcd",
    );
    let fixture = fixture.path().to_string_lossy().into_owned();

    let value = run_change_json(
        fixture.as_str(),
        &[
            "--from",
            "5ns",
            "--to",
            "5ns",
            "--signals",
            "top.sig",
            "--on",
            "top.sig",
        ],
    );

    assert_eq!(value["data"], json!([]));
    assert_eq!(value["diagnostics"], json!([]));
}

#[test]
fn change_all_candidates_at_or_before_baseline_do_not_emit() {
    let fixture = write_fixture(
        "$date\n  today\n$end\n$version\n  wavepeek-test\n$end\n$timescale 1ns $end\n$scope module top $end\n$var wire 1 ! trig $end\n$var wire 1 \" sig $end\n$upscope $end\n$enddefinitions $end\n#0\n0!\n0\"\n#5\n1!\n1\"\n",
        "change-baseline-window.vcd",
    );
    let fixture = fixture.path().to_string_lossy().into_owned();

    let value = run_change_json(
        fixture.as_str(),
        &[
            "--from",
            "5ns",
            "--to",
            "5ns",
            "--signals",
            "top.sig",
            "--on",
            "posedge top.trig",
        ],
    );

    assert_eq!(value["data"], json!([]));
    assert_eq!(value["diagnostics"], json!([]));
}

#[test]
fn change_max_one_truncation_has_matching_lookahead() {
    let fixture = write_fixture(
        "$date\n  today\n$end\n$version\n  wavepeek-test\n$end\n$timescale 1ns $end\n$scope module top $end\n$var wire 4 ! sig $end\n$upscope $end\n$enddefinitions $end\n#0\nb0000 !\n#5\nb0001 !\n#10\nb0010 !\n#15\nb0011 !\n",
        "change-max-one.vcd",
    );
    let fixture = fixture.path().to_string_lossy().into_owned();

    let value = run_change_json(
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
                "sample_time": "5ns",
                "signals": [
                    {"path": "top.sig", "value": "4'h1"}
                ]
            }
        ])
    );
    assert_eq!(
        value["diagnostics"],
        json!([{"kind": "warning", "code": "WPK-W0002", "message": "truncated output to 1 entries (use --max to increase limit)"}])
    );
}

#[test]
fn change_redundant_same_value_dump_does_not_emit_row() {
    let fixture = write_fixture(
        "$date\n  today\n$end\n$version\n  wavepeek-test\n$end\n$timescale 1ns $end\n$scope module top $end\n$var wire 4 ! sig $end\n$upscope $end\n$enddefinitions $end\n#0\nb0000 !\n#5\nb0000 !\n#10\nb0001 !\n",
        "change-redundant-value.vcd",
    );
    let fixture = fixture.path().to_string_lossy().into_owned();

    let value = run_change_json(
        fixture.as_str(),
        &[
            "--from",
            "0ns",
            "--to",
            "10ns",
            "--signals",
            "top.sig",
            "--on",
            "top.sig",
        ],
    );

    assert_eq!(
        value["data"],
        json!([
            {
                "time": "10ns",
                "sample_time": "10ns",
                "signals": [
                    {"path": "top.sig", "value": "4'h1"}
                ]
            }
        ])
    );
}

#[test]
fn change_typed_iff_has_matching_lookahead() {
    let fixture = fixture_path("m2_core.vcd");
    let fixture = fixture.to_string_lossy().into_owned();

    let value = run_change_json(
        fixture.as_str(),
        &[
            "--scope",
            "top",
            "--signals",
            "data,clk",
            "--on",
            "posedge clk iff data == 8'h00",
        ],
    );

    assert_eq!(value["context"]["scope"], "top");
    assert_eq!(value["diagnostics"], json!([]));
    assert_eq!(
        value["data"],
        json!([
            {
                "time": "5ns",
                "sample_time": "5ns",
                "signals": [
                    {"path": "top.data", "relative_path": "data", "value": "8'h00"},
                    {"path": "top.clk", "relative_path": "clk", "value": "1'h1"}
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

    let value = run_change_json(
        fixture.as_str(),
        &[
            "--from",
            "0ns",
            "--to",
            "5ns",
            "--signals",
            "top.sig",
            "--on",
            "top.trig",
        ],
    );

    assert_eq!(value["diagnostics"], json!([]));
    assert_eq!(
        value["data"],
        json!([
            {
                "time": "5ns",
                "sample_time": "5ns",
                "signals": [
                    {"path": "top.sig", "value": "1'h1"}
                ]
            }
        ])
    );
}

#[test]
fn change_sparse_delta_emits_first_available_value() {
    let fixture = write_fixture(
        "$timescale 1ns $end\n$scope module top $end\n$var wire 1 ! clk $end\n$var wire 1 \" sig $end\n$upscope $end\n$enddefinitions $end\n#0\n0!\n#5\n1!\n1\"\n",
        "change-delayed-value.vcd",
    );
    let fixture = fixture.path().to_string_lossy().into_owned();

    let value = run_change_json(
        fixture.as_str(),
        &[
            "--from",
            "0ns",
            "--to",
            "5ns",
            "--signals",
            "top.sig",
            "--on",
            "posedge top.clk",
            "--row-values",
            "delta",
        ],
    );

    assert_eq!(value["diagnostics"], json!([]));
    assert_eq!(
        value["data"],
        json!([{
            "time": "5ns",
            "sample_time": "5ns",
            "signals": [{"path": "top.sig", "value": "1'h1"}]
        }])
    );
}

#[test]
fn change_dense_posedge_sparse_delta() {
    let fixture = write_fixture(
        "$date\n  today\n$end\n$version\n  wavepeek-test\n$end\n$timescale 1ns $end\n$scope module top $end\n$var wire 1 ! clk $end\n$var wire 1 \" gate $end\n$var wire 1 # data $end\n$upscope $end\n$enddefinitions $end\n#0\n0!\n1\"\n0#\n#1\n1!\n#2\n0!\n#3\n1!\n0\"\n#4\n0!\n#5\n1!\n1#\n#6\n0!\n#7\n1!\n#8\n0!\n1\"\n#9\n1!\n0\"\n0#\n#10\n0!\n",
        "change-dense-edge.vcd",
    );
    let fixture = fixture.path().to_string_lossy().into_owned();

    let value = run_change_json(
        fixture.as_str(),
        &[
            "--from",
            "0ns",
            "--to",
            "10ns",
            "--signals",
            "top.data",
            "--on",
            "posedge top.clk",
        ],
    );

    assert_eq!(value["diagnostics"], json!([]));
    assert_eq!(
        value["data"],
        json!([
            {
                "time": "5ns",
                "sample_time": "5ns",
                "signals": [
                    {"path": "top.data", "value": "1'h1"}
                ]
            },
            {
                "time": "9ns",
                "sample_time": "9ns",
                "signals": [
                    {"path": "top.data", "value": "1'h0"}
                ]
            }
        ])
    );
}

#[test]
fn change_edge_without_requested_delta_remains_empty() {
    let fixture = write_fixture(
        "$date\n  today\n$end\n$version\n  wavepeek-test\n$end\n$timescale 1ns $end\n$scope module top $end\n$var wire 1 ! clk $end\n$var wire 1 \" gate $end\n$var wire 1 # data $end\n$upscope $end\n$enddefinitions $end\n#0\n0!\n1\"\n0#\n#1\n1!\n#2\n0!\n#3\n1!\n0\"\n#4\n0!\n",
        "change-dense-edge-empty.vcd",
    );
    let fixture = fixture.path().to_string_lossy().into_owned();

    let value = run_change_json(
        fixture.as_str(),
        &[
            "--from",
            "0ns",
            "--to",
            "4ns",
            "--signals",
            "top.data",
            "--on",
            "posedge top.clk",
        ],
    );

    assert_eq!(value["data"], json!([]));
    assert_eq!(value["diagnostics"], json!([]));
}

#[test]
fn change_union_edge_trigger_dedups_same_timestamp() {
    let fixture = write_fixture(
        "$date\n  today\n$end\n$version\n  wavepeek-test\n$end\n$timescale 1ns $end\n$scope module top $end\n$var wire 1 ! clk $end\n$var wire 1 \" gate $end\n$var wire 1 # data $end\n$upscope $end\n$enddefinitions $end\n#0\n0!\n1\"\n0#\n#1\n1!\n#2\n0!\n#3\n1!\n0\"\n#4\n0!\n#5\n1!\n1#\n#6\n0!\n#7\n1!\n#8\n0!\n1\"\n#9\n1!\n0\"\n0#\n#10\n0!\n",
        "change-dense-edge-union.vcd",
    );
    let fixture = fixture.path().to_string_lossy().into_owned();

    let value = run_change_json(
        fixture.as_str(),
        &[
            "--from",
            "0ns",
            "--to",
            "10ns",
            "--signals",
            "top.data",
            "--on",
            "posedge top.clk, negedge top.gate",
        ],
    );

    assert_eq!(value["diagnostics"], json!([]));
    assert_eq!(
        value["data"],
        json!([
            {
                "time": "5ns",
                "sample_time": "5ns",
                "signals": [
                    {"path": "top.data", "value": "1'h1"}
                ]
            },
            {
                "time": "9ns",
                "sample_time": "9ns",
                "signals": [
                    {"path": "top.data", "value": "1'h0"}
                ]
            }
        ])
    );
}

#[test]
fn change_dense_edge_max_one_truncation() {
    let fixture = write_fixture(
        "$date\n  today\n$end\n$version\n  wavepeek-test\n$end\n$timescale 1ns $end\n$scope module top $end\n$var wire 1 ! clk $end\n$var wire 1 \" gate $end\n$var wire 1 # data $end\n$upscope $end\n$enddefinitions $end\n#0\n0!\n1\"\n0#\n#1\n1!\n#2\n0!\n#3\n1!\n0\"\n#4\n0!\n#5\n1!\n1#\n#6\n0!\n#7\n1!\n#8\n0!\n1\"\n#9\n1!\n0\"\n0#\n#10\n0!\n",
        "change-dense-edge-max-one.vcd",
    );
    let fixture = fixture.path().to_string_lossy().into_owned();

    let value = run_change_json(
        fixture.as_str(),
        &[
            "--from",
            "0ns",
            "--to",
            "10ns",
            "--signals",
            "top.data",
            "--on",
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
                "sample_time": "5ns",
                "signals": [
                    {"path": "top.data", "value": "1'h1"}
                ]
            }
        ])
    );
    assert_eq!(
        value["diagnostics"],
        json!([{"kind": "warning", "code": "WPK-W0002", "message": "truncated output to 1 entries (use --max to increase limit)"}])
    );
}

#[test]
fn change_wildcard_preserves_repeated_raw_events_with_projections() {
    let fixture = write_fixture(
        "$timescale 1ns $end\n$scope module top $end\n$var event 1 ! tick $end\n$var wire 8 \" data [7:0] $end\n$upscope $end\n$enddefinitions $end\n#0\nb10100101 \"\n#5\n1!\n#10\n1!\n#15\n1!\n#25\n1!\n",
        "change-projected-raw-events.vcd",
    );
    let fixture = fixture.path().to_string_lossy().into_owned();

    for signals in ["top.tick", "top.tick,top.data[3:0]"] {
        let value = run_change_json(
            fixture.as_str(),
            &[
                "--signals",
                signals,
                "--on",
                "*",
                "--row-mode",
                "dense",
                "--row-values",
                "full",
            ],
        );
        assert_eq!(
            value["data"]
                .as_array()
                .expect("change data should be an array")
                .iter()
                .map(|row| row["time"].as_str().expect("row time should be a string"))
                .collect::<Vec<_>>(),
            ["5ns", "10ns", "15ns", "25ns"]
        );
    }
}

#[test]
fn change_rejected_projected_candidates_do_not_emit() {
    let mut source = String::from(
        "$timescale 1ns $end\n$scope module top $end\n$var wire 8 ! data [7:0] $end\n$upscope $end\n$enddefinitions $end\n#0\nb00000000 !\n",
    );
    for timestamp in 1..=100 {
        source.push_str(format!("#{timestamp}\nb{:08b} !\n", timestamp % 16).as_str());
    }
    let fixture = write_fixture(&source, "change-projected-cache-bound.vcd");
    let fixture = fixture.path().to_string_lossy().into_owned();

    let value = run_change_json(
        fixture.as_str(),
        &[
            "--signals",
            "top.data[7:4]",
            "--on",
            "*",
            "--row-mode",
            "dense",
            "--row-values",
            "full",
        ],
    );
    assert_eq!(value["data"], json!([]));
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
