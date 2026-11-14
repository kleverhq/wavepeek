use serde_json::{Value, json};
use tempfile::NamedTempFile;

mod common;
use common::wavepeek_cmd;

fn fixture(source: &str) -> NamedTempFile {
    let file = tempfile::Builder::new().suffix(".vcd").tempfile().unwrap();
    std::fs::write(file.path(), source).unwrap();
    file
}

const QUERY_VCD: &str = concat!(
    "$timescale 1ns $end\n$scope module top $end\n",
    "$var wire 1 c clk $end\n$var wire 1 g gate $end\n",
    "$var wire 4 b bus $end\n$var wire 4 b alias $end\n",
    "$upscope $end\n$enddefinitions $end\n",
    "#0\n0c\n1g\nb0000 b\n#5\nb0001 b\n",
    "#6\nb1101 b\nb0001 b\n#7\n0g\n",
    "#10\n1c\n0c\n1c\n1g\nb0101 b\n",
    "#11\nb0010 b\n#12\n0c\n#15\n1c\n0g\nb1100 b\n",
    "#17\n1g\n#20\n0c\n#25\n1c\n#30\n0c\n",
);

fn run(file: &NamedTempFile, args: &[&str]) -> Value {
    let output = wavepeek_cmd()
        .arg("change")
        .arg("--waves")
        .arg(file.path())
        .args(args)
        .arg("--json")
        .output()
        .unwrap();
    assert!(output.status.success(), "{output:?}");
    assert!(output.stderr.is_empty(), "{output:?}");
    serde_json::from_slice(&output.stdout).unwrap()
}

fn times(value: &Value) -> Vec<&str> {
    value["data"]
        .as_array()
        .unwrap()
        .iter()
        .map(|row| row["time"].as_str().unwrap())
        .collect()
}

#[test]
fn wildcard_filters_projection_noise_and_same_tick_net_updates() {
    let file = fixture(QUERY_VCD);
    let value = run(
        &file,
        &[
            "--signals",
            "top.bus[3:2]",
            "--on",
            "*",
            "--sample-mode",
            "native",
        ],
    );
    assert_eq!(times(&value), ["10ns", "11ns", "15ns"]);
    assert_eq!(value["data"][0]["signals"][0]["value"], "2'h1");
    assert_eq!(value["data"][1]["signals"][0]["value"], "2'h0");
    assert_eq!(value["data"][2]["signals"][0]["value"], "2'h3");
}

#[test]
fn union_triggers_use_final_tick_and_preserve_alias_columns() {
    let file = fixture(QUERY_VCD);
    let value = run(
        &file,
        &[
            "--scope",
            "top",
            "--signals",
            "alias[3:2],bus[3:2],alias[3:2]",
            "--on",
            "posedge clk iff gate or *",
            "--sample-mode",
            "native",
        ],
    );
    assert_eq!(times(&value), ["10ns", "11ns", "15ns", "25ns"]);
    assert_eq!(
        value["data"][0]["signals"],
        json!([
            {"path":"top.alias[3:2]", "relative_path":"alias[3:2]", "value":"2'h1"},
            {"path":"top.bus[3:2]", "relative_path":"bus[3:2]", "value":"2'h1"},
            {"path":"top.alias[3:2]", "relative_path":"alias[3:2]", "value":"2'h1"},
        ])
    );
}

#[test]
fn pre_edge_uses_absolute_previous_tick_and_iff_uses_event_tick() {
    let file = fixture(QUERY_VCD);
    let value = run(
        &file,
        &[
            "--from",
            "10ns",
            "--signals",
            "top.bus[3:2]",
            "--on",
            "posedge top.clk iff top.gate",
            "--sample-mode",
            "pre-edge",
        ],
    );
    assert_eq!(times(&value), ["10ns", "25ns"]);
    assert_eq!(value["data"][0]["sample_time"], "9ns");
    assert_eq!(value["data"][0]["signals"][0]["value"], "2'h0");
    assert_eq!(value["data"][1]["sample_time"], "24ns");
    assert_eq!(value["data"][1]["signals"][0]["value"], "2'h3");
}

#[test]
fn sparse_lookahead_counts_changed_matching_rows_and_reaches_eof() {
    let file = fixture(QUERY_VCD);
    for (mode, end, complete, time) in [
        ("native", "20ns", true, "10ns"),
        ("native", "30ns", false, "10ns"),
        ("pre-edge", "30ns", true, "25ns"),
    ] {
        let value = run(
            &file,
            &[
                "--signals",
                "top.bus[3:2]",
                "--on",
                "posedge top.clk iff top.gate",
                "--sample-mode",
                mode,
                "--to",
                end,
                "--row-mode",
                "sparse",
                "--row-values",
                "delta",
                "--max",
                "1",
            ],
        );
        assert_eq!(times(&value), [time], "{value}");
        assert_eq!(value["summary"]["complete"], complete, "{value}");
        assert_eq!(value["summary"]["returned"], 1);
        assert_eq!(
            value["summary"]["total"],
            if complete { json!(1) } else { Value::Null }
        );
    }
}

#[test]
fn quiet_sparse_baseline_and_first_tick_do_not_create_rows() {
    let file = fixture(QUERY_VCD);
    let value = run(
        &file,
        &[
            "--signals",
            "top.bus[3:2]",
            "--on",
            "*",
            "--sample-mode",
            "native",
            "--row-mode",
            "sparse",
            "--from",
            "17ns",
            "--to",
            "20ns",
        ],
    );
    assert_eq!(times(&value), Vec::<&str>::new());
    assert_eq!(value["summary"]["complete"], true);
    let file = fixture(
        "$timescale 1ns $end\n$scope module top $end\n$var wire 1 c clk $end\n$upscope $end\n$enddefinitions $end\n#0\n1c\n#1\n0c\n#2\n1c\n",
    );
    let value = run(
        &file,
        &[
            "--signals",
            "top.clk",
            "--on",
            "posedge top.clk",
            "--sample-mode",
            "pre-edge",
        ],
    );
    assert_eq!(times(&value), ["2ns"]);
    assert_eq!(value["data"][0]["sample_time"], "1ns");
}

#[test]
fn lookahead_evaluation_error_preserves_owned_and_partial_jsonl_contracts() {
    let file = fixture(
        "$timescale 1ns $end\n$scope module top $end\n$var wire 1 c clk $end\n$var wire 1 v valid $end\n$var real 1 r real_value $end\n$upscope $end\n$enddefinitions $end\n#0\n0c\n1v\n#5\n1c\n#10\n0c\n0v\n#15\n1c\n",
    );
    for mode in ["--json", "--jsonl"] {
        let output = wavepeek_cmd()
            .arg("change")
            .arg("--waves")
            .arg(file.path())
            .args([
                "--signals",
                "top.valid",
                "--on",
                "posedge top.clk iff (top.valid || (top.real_value > 0))",
                "--sample-mode",
                "native",
                "--max",
                "1",
                mode,
            ])
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(1), "{output:?}");
        assert!(output.stderr.is_empty());
        let records: Vec<Value> = std::str::from_utf8(&output.stdout)
            .unwrap()
            .lines()
            .map(|line| serde_json::from_str(line).unwrap())
            .collect();
        let fatal = records.last().unwrap();
        assert_eq!(fatal["type"], "fatal");
        assert!(fatal["message"].as_str().unwrap().contains("real_value"));
        if mode == "--jsonl" {
            assert_eq!(records.len(), 3);
            assert_eq!(records[0]["type"], "begin");
            assert_eq!(records[1]["type"], "data");
            assert_eq!(records[1]["data"]["time"], "5ns");
            for (seq, record) in records.iter().enumerate() {
                assert_eq!(record["seq"], seq);
            }
        } else {
            assert_eq!(records.len(), 1);
        }
    }
}

#[test]
fn zero_limit_and_empty_windows_preserve_validation() {
    for mode in ["--json", "--jsonl"] {
        let output = wavepeek_cmd()
            .args([
                "change",
                "--waves",
                "missing.vcd",
                "--signals",
                "missing",
                "--on",
                "???",
                "--max",
                "0",
                mode,
            ])
            .output()
            .unwrap();
        let value: Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(output.status.code(), Some(1));
        assert_eq!(value["code"], "WPK-F0001");
        assert!(value["message"].as_str().unwrap().contains("--max"));
    }
    let file = fixture(QUERY_VCD);
    let output = wavepeek_cmd()
        .arg("change")
        .arg("--waves")
        .arg(file.path())
        .args([
            "--signals",
            "top.bus",
            "--on",
            "posedge missing",
            "--from",
            "18ns",
            "--to",
            "19ns",
            "--json",
        ])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    let value: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value["code"], "WPK-F0005");
}

#[cfg(feature = "fsdb")]
#[test]
fn nine_state_payload_is_normalized_before_sparse_delta_comparison() {
    let path = std::path::PathBuf::from(std::env::var_os("ONDAS_FIXTURES_DIR").unwrap())
        .join("fsdb/fsdb0020-nine-state-ranges/waveform.fsdb");
    let output = wavepeek_cmd()
        .arg("change")
        .arg("--waves")
        .arg(path)
        .args([
            "--signals",
            "RICH_LOGIC.RESOLVED_BIT",
            "--on",
            "*",
            "--sample-mode",
            "native",
            "--from",
            "60ns",
            "--to",
            "80ns",
            "--row-mode",
            "sparse",
            "--row-values",
            "delta",
            "--json",
        ])
        .output()
        .unwrap();
    assert!(output.status.success(), "{output:?}");
    let value: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(times(&value), ["70ns", "80ns"]);
    assert_eq!(value["data"][0]["signals"][0]["value"], "1'h1");
    assert_eq!(value["data"][1]["signals"][0]["value"], "1'hx");
}

#[test]
fn matching_lookahead_stops_outer_query_before_later_evaluation_failure() {
    let file = fixture(
        "$timescale 1ns $end\n$scope module top $end\n$var wire 1 c clk $end\n$var wire 1 v valid $end\n$var real 1 r real_value $end\n$upscope $end\n$enddefinitions $end\n#0\n0c\n1v\n#5\n1c\n#10\n0c\n#15\n1c\n#20\n0c\n0v\n#25\n1c\n",
    );
    let value = run(
        &file,
        &[
            "--signals",
            "top.valid",
            "--on",
            "posedge top.clk iff (top.valid || (top.real_value > 0))",
            "--sample-mode",
            "native",
            "--max",
            "1",
        ],
    );
    assert_eq!(times(&value), ["5ns"]);
    assert_eq!(value["summary"]["complete"], false);
    assert_eq!(value["summary"]["returned"], 1);
}
