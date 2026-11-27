use std::fs;
use std::path::Path;
use std::process::Output;

use serde_json::{Value, json};

mod common;
use common::{fixture_path, wavepeek_cmd};

fn run(path: &Path, args: &[&str], mode: &str) -> Output {
    wavepeek_cmd()
        .args(args)
        .arg("--waves")
        .arg(path)
        .arg(mode)
        .output()
        .unwrap()
}

fn data(path: &Path, args: &[&str]) -> Value {
    let output = run(path, args, "--json");
    assert!(output.status.success(), "{output:?}");
    assert!(output.stderr.is_empty(), "{output:?}");
    let result: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(result["diagnostics"], json!([]));
    result["data"].clone()
}

#[test]
fn info_uses_one_femtosecond_ticks_even_without_a_ghw_suffix() {
    let fixture = fixture_path("ghw/ghw_core.ghw");
    let directory = tempfile::tempdir().unwrap();
    let renamed = directory.path().join("trace");
    fs::copy(fixture, &renamed).unwrap();

    assert_eq!(
        data(&renamed, &["info"]),
        json!([{
            "time_unit": "1fs",
            "time_start": "0fs",
            "time_end": "10000000fs"
        }])
    );
}

#[test]
fn info_keeps_file_errors_for_malformed_ghw_and_vcd_without_timescale() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("trace.ghw");
    for (contents, cause) in [
        (b"GHDLwave".as_slice(), "malformed Ghw"),
        (
            b"$var wire 1 ! flag $end $enddefinitions $end #0 0!".as_slice(),
            "waveform is missing timescale metadata",
        ),
    ] {
        fs::write(&path, contents).unwrap();
        let output = run(&path, &["info"], "--json");
        assert_eq!(output.status.code(), Some(2), "{output:?}");
        let fatal: Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(fatal["code"], "WPK-F0002");
        let message = fatal["message"].as_str().unwrap();
        assert!(message.contains(cause), "{message}");
    }
}

#[test]
fn hierarchy_paths_can_read_a_vhdl_record_field() {
    let fixture = fixture_path("ghw/ghw_core.ghw");
    let scopes = data(&fixture, &["scope", "--filter", "^ghw_core[.]dut$"]);
    assert_eq!(scopes.as_array().unwrap().len(), 1);
    assert_eq!(scopes[0]["path"], "ghw_core.dut");
    assert_eq!(scopes[0]["depth"], 1);

    let signals = data(
        &fixture,
        &[
            "signal",
            "--scope",
            scopes[0]["path"].as_str().unwrap(),
            "--recursive",
            "--filter",
            "[.]detail[.]marker$",
        ],
    );
    assert_eq!(signals.as_array().unwrap().len(), 1);
    assert_eq!(signals[0]["path"], "ghw_core.dut.detail.marker");
    assert_eq!(signals[0]["width"], 1);

    let values = data(
        &fixture,
        &[
            "value",
            "--signals",
            signals[0]["path"].as_str().unwrap(),
            "--at",
            "10ns",
        ],
    );
    assert_eq!(values[0]["time"], "10000000fs");
    assert_eq!(values[0]["signals"][0]["value"], "1'h1");
}

#[test]
fn batched_points_preserve_order_and_match_the_vcd_from_the_same_simulation() {
    let ghw = fixture_path("ghw/ghw_core.ghw");
    let vcd = fixture_path("ghw/ghw_core.vcd");
    let args = [
        "value",
        "--signals",
        "ghw_core.dut.data,ghw_core.dut.data[7:4],ghw_core.dut.data",
        "--at",
        "10ns,0ns,5ns,10ns",
    ];
    let rows = data(&ghw, &args);
    assert_eq!(rows, data(&vcd, &args));
    let rows = rows.as_array().unwrap();
    assert_eq!(rows.len(), 4);
    for (row, (time, full, slice)) in rows.iter().zip([
        ("10000000fs", "8'ha5", "4'ha"),
        ("0fs", "8'h00", "4'h0"),
        ("5000000fs", "8'h3c", "4'h3"),
        ("10000000fs", "8'ha5", "4'ha"),
    ]) {
        assert_eq!(row["time"], time);
        assert_eq!(row["signals"][0]["value"], full);
        assert_eq!(row["signals"][1]["value"], slice);
        assert_eq!(row["signals"][2], row["signals"][0]);
    }
}

#[test]
fn change_includes_boundary_edge_and_uses_previous_femtosecond_in_jsonl() {
    let fixture = fixture_path("ghw/ghw_core.ghw");
    let output = run(
        &fixture,
        &[
            "change",
            "--from",
            "5ns",
            "--to",
            "5ns",
            "--on",
            "posedge ghw_core.dut.valid",
            "--signals",
            "ghw_core.dut.data",
            "--sample-mode",
            "pre-edge",
        ],
        "--jsonl",
    );
    assert!(output.status.success(), "{output:?}");
    assert!(output.stderr.is_empty(), "{output:?}");
    let records: Vec<Value> = std::str::from_utf8(&output.stdout)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    assert_eq!(records.len(), 3);
    assert_eq!(records[0]["type"], "begin");
    assert_eq!(records[1]["type"], "data");
    assert_eq!(records[1]["data"]["time"], "5000000fs");
    assert_eq!(records[1]["data"]["sample_time"], "4999999fs");
    assert_eq!(records[1]["data"]["signals"][0]["value"], "8'h00");
    assert_eq!(records[2]["type"], "end");
}

#[test]
fn property_and_extract_share_ghw_event_times() {
    let fixture = fixture_path("ghw/ghw_core.ghw");
    let property = data(
        &fixture,
        &[
            "property",
            "--scope",
            "ghw_core.dut",
            "--on",
            "posedge valid",
            "--eval",
            "data == 8'h3c",
            "--capture",
            "match",
            "--sample-mode",
            "native",
        ],
    );
    assert_eq!(
        property,
        json!([{"time": "5000000fs", "sample_time": "5000000fs", "kind": "match"}])
    );

    let extract = data(
        &fixture,
        &[
            "extract",
            "generic",
            "--scope",
            "ghw_core.dut",
            "--on",
            "posedge valid",
            "--when",
            "data == 8'h00",
            "--payload",
            "data",
        ],
    );
    assert_eq!(extract.as_array().unwrap().len(), 1);
    assert_eq!(extract[0]["time"], "5000000fs");
    assert_eq!(extract[0]["sample_time"], "4999999fs");
    assert_eq!(extract[0]["payload"][0]["value"], "8'h00");
}
