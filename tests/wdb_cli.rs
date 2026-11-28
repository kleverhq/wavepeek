#![cfg(feature = "wdb")]

use std::path::{Path, PathBuf};

use serde_json::{Value, json};

mod common;
use common::{fixture_path, wavepeek_cmd};

fn wdb_fixture(path: &str) -> PathBuf {
    PathBuf::from(std::env::var_os("WAVEPEEK_WDB_FIXTURES").expect("run just test-wdb")).join(path)
}

fn run(waves: &Path, args: &[&str], mode: &str) -> String {
    let output = wavepeek_cmd()
        .args(args)
        .arg("--waves")
        .arg(waves)
        .arg(mode)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{args:?} ({waves:?}): {:?}",
        output
    );
    String::from_utf8(output.stdout).unwrap()
}

fn run_data(waves: &Path, args: &[&str]) -> Value {
    let value: Value = serde_json::from_str(&run(waves, args, "--json")).unwrap();
    assert_eq!(value["diagnostics"], json!([]), "{args:?} ({waves:?})");
    value["data"].clone()
}

#[test]
fn help_reports_wdb_enabled() {
    let output = wavepeek_cmd().arg("--help").output().unwrap();
    assert!(output.status.success(), "{output:?}");
    let help = String::from_utf8(output.stdout).unwrap();
    assert!(help.lines().any(|line| line == "- WDB - enabled"), "{help}");
}

#[test]
fn missing_signal_reports_a_fatal_diagnostic_on_wdb() {
    let output = wavepeek_cmd()
        .args(["value", "--waves"])
        .arg(wdb_fixture("m2_core/waveform.wdb"))
        .args(["--signals", "top.missing", "--at", "5ns", "--json"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1), "{output:?}");
    let response: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(response["type"], "fatal");
    assert_eq!(response["code"], "WPK-F0004");
    assert!(
        response["message"]
            .as_str()
            .unwrap()
            .contains("top.missing")
    );
}

#[test]
fn metadata_bounds_and_hierarchy_match_the_rtl() {
    let wdb = wdb_fixture("m2_core/waveform.wdb");
    let vcd = fixture_path("m2_core.vcd");
    let info = json!([{"time_unit": "1ns", "time_start": "0ns", "time_end": "10ns"}]);
    let scope = ["scope", "--filter", "^top[.]cpu$"];
    let signal = [
        "signal",
        "--scope",
        "top",
        "--recursive",
        "--filter",
        "^top[.]cpu[.]valid$",
    ];

    for waves in [&wdb, &vcd] {
        assert_eq!(run_data(waves, &["info"]), info);
        // Native WDB exposes compiler scopes and can classify HDL scope kinds differently.
        let scopes = run_data(waves, &scope);
        let scopes = scopes.as_array().unwrap();
        assert_eq!(scopes.len(), 1);
        assert_eq!(scopes[0]["path"], "top.cpu");
        assert_eq!(scopes[0]["depth"], 1);
        let signals = run_data(waves, &signal);
        let signals = signals.as_array().unwrap();
        assert_eq!(signals.len(), 1);
        assert_eq!(signals[0]["path"], "top.cpu.valid");
        assert_eq!(signals[0]["width"], 1);
    }
}

#[test]
fn values_and_events_match_at_the_boundaries() {
    let wdb = wdb_fixture("m2_core/waveform.wdb");
    let vcd = fixture_path("m2_core.vcd");
    let selected = "top.data[3:0],top.cpu.valid,top.data[3:0]";
    let value_args = ["value", "--signals", selected, "--at", "4ns,5ns,10ns"];
    let change_args = [
        "change",
        "--signals",
        "top.cpu.valid",
        "--from",
        "0ns",
        "--to",
        "5ns",
        "--on",
        "*",
        "--sample-mode",
        "native",
    ];
    let property_args = [
        "property",
        "--on",
        "posedge top.clk",
        "--eval",
        "top.cpu.valid",
        "--sample-mode",
        "native",
    ];
    let generic_args = [
        "extract",
        "generic",
        "--on",
        "posedge top.clk",
        "--when",
        "!top.cpu.valid",
        "--payload",
        "top.data",
    ];

    let values = run_data(&wdb, &value_args);
    assert_eq!(values, run_data(&vcd, &value_args));
    let rows = values.as_array().unwrap();
    assert_eq!(rows.len(), 3);
    for (row, (time, valid, nibble)) in rows.iter().zip([
        ("4ns", "1'h0", "4'h0"),
        ("5ns", "1'h1", "4'h0"),
        ("10ns", "1'h1", "4'hf"),
    ]) {
        assert_eq!(
            row,
            &json!({"time": time, "signals": [
                {"path": "top.data[3:0]", "value": nibble},
                {"path": "top.cpu.valid", "value": valid},
                {"path": "top.data[3:0]", "value": nibble},
            ]})
        );
    }
    for waves in [&wdb, &vcd] {
        assert_eq!(
            run_data(waves, &change_args),
            json!([{"time": "5ns", "sample_time": "5ns", "signals": [
                {"path": "top.cpu.valid", "value": "1'h1"}
            ]}])
        );
        assert_eq!(
            run_data(waves, &property_args),
            json!([{"time": "5ns", "sample_time": "5ns", "kind": "assert"}])
        );
    }
    let generic = run_data(&wdb, &generic_args);
    assert_eq!(generic, run_data(&vcd, &generic_args));
    assert_eq!(generic.as_array().unwrap().len(), 1);
    assert_eq!(generic[0]["time"], "5ns");
    assert_eq!(generic[0]["sample_time"], "4ns");
}

#[test]
fn apb_transfers_match_the_rtl_and_vcd() {
    let args = [
        "extract",
        "apb",
        "--scope",
        "top",
        "--profile",
        "apb3",
        "--include",
        "^apb3_",
    ];
    let expected = json!([
        {
            "time": "5ns", "sample_time": "4ns", "profile": "apb3",
            "event": "setup", "direction": "write",
            "payload": {"paddr": "8'h10", "pwdata": "8'h3a", "pwrite": "1'h1"},
        },
        {
            "time": "10ns", "sample_time": "9ns", "profile": "apb3",
            "event": "access-complete", "direction": "write",
            "payload": {"paddr": "8'h10", "pslverr": "1'h0", "pwdata": "8'h3a", "pwrite": "1'h1"},
        },
    ]);
    assert_eq!(
        run_data(&wdb_fixture("extract_apb3/waveform.wdb"), &args),
        expected
    );
    assert_eq!(run_data(&fixture_path("extract_apb3.vcd"), &args), expected);
}

#[test]
fn enum_labels_and_native_paths_from_the_published_fixture() {
    let waves = wdb_fixture("enum-core.wdb");
    let data = run_data(
        &waves,
        &[
            "property",
            "--from",
            "1ns",
            "--to",
            "10ns",
            "--on",
            "*",
            "--sample-mode",
            "native",
            "--eval",
            "top.sparse8 == type(top.sparse8)::SPARSE8_RUN",
        ],
    );
    assert_eq!(
        data,
        json!([
            {"time": "4ns", "sample_time": "4ns", "kind": "assert"},
            {"time": "9ns", "sample_time": "9ns", "kind": "deassert"},
        ])
    );

    let signals = run_data(
        &waves,
        &[
            "signal",
            "--scope",
            "top",
            "--recursive",
            "--filter",
            r"alternate[.]value|array_values\[0\]",
        ],
    );
    let mut paths: Vec<&str> = signals
        .as_array()
        .unwrap()
        .iter()
        .map(|signal| {
            assert_eq!(signal["kind"], "enum");
            signal["path"].as_str().unwrap()
        })
        .collect();
    paths.sort_unstable();
    assert_eq!(paths, [r"top.\alternate.value ", "top.array_values[0]"]);

    let output = run(
        &waves,
        &["value", "--signals", &paths.join(","), "--at", "9ns"],
        "--jsonl",
    );
    let records: Vec<Value> = output
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    assert_eq!(
        records,
        [
            json!({"type": "begin", "seq": 0, "command": "value"}),
            json!({"type": "data", "seq": 1, "data": {"time": "9ns", "signals": [
                {"path": paths[0], "value": "8'h01"},
                {"path": paths[1], "value": "8'h2a"},
            ]}}),
            json!({"type": "end", "seq": 2, "records": {"data": 1, "diagnostics": 0}}),
        ]
    );
}
