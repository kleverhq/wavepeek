use std::io::Write;

use serde_json::{Value, json};
use tempfile::NamedTempFile;

mod common;
use common::wavepeek_cmd;

fn parse_json(stdout: &[u8]) -> Value {
    serde_json::from_slice(stdout).expect("stdout should be valid json")
}

#[test]
fn change_uses_strict_previous_timestamp_not_previous_candidate() {
    let fixture = write_fixture(
        "$date\n  today\n$end\n$version\n  wavepeek-test\n$end\n$timescale 1ns $end\n$scope module top $end\n$var wire 1 ! trig $end\n$var wire 1 \" sig $end\n$upscope $end\n$enddefinitions $end\n#0\n0!\n0\"\n#5\n1!\n1\"\n#7\n0\"\n#8\n0!\n#10\n1!\n",
        "change-opt-equivalence.vcd",
    );
    let fixture = fixture.path().to_string_lossy().into_owned();

    let output = wavepeek_cmd()
        .args([
            "change",
            "--waves",
            fixture.as_str(),
            "--from",
            "0ns",
            "--to",
            "10ns",
            "--signals",
            "top.sig",
            "--when",
            "posedge top.trig",
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

    let output = wavepeek_cmd()
        .args([
            "change",
            "--waves",
            fixture.as_str(),
            "--from",
            "6ns",
            "--to",
            "10ns",
            "--signals",
            "top.sig",
            "--when",
            "posedge top.trig",
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

fn write_fixture(contents: &str, filename: &str) -> NamedTempFile {
    let mut file = tempfile::Builder::new()
        .suffix(filename)
        .tempfile()
        .expect("tempfile should be created");
    file.write_all(contents.as_bytes())
        .expect("fixture should be written");
    file
}
