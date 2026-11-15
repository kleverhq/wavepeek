use std::fs;
use std::path::Path;
use std::process::Command;

use serde_json::{Value, json};
use tempfile::NamedTempFile;

mod common;
use common::wavepeek_cmd;

fn waveform(body: &str) -> NamedTempFile {
    let file = NamedTempFile::with_suffix(".vcd").unwrap();
    fs::write(file.path(), format!("$timescale 1us $end\n$scope module top $end\n$var wire 1 c clk $end\n$var wire 1 r reset_n $end\n$var wire 2 t trans $end\n$var wire 1 y ready $end\n$var wire 1 w write $end\n$var wire 8 a addr $end\n$var wire 8 d data $end\n$var wire 1 p resp $end\n$upscope $end\n$enddefinitions $end\n{body}")).unwrap();
    file
}

fn command(path: &Path) -> Command {
    let mut cmd = wavepeek_cmd();
    cmd.args(["extract", "ahb", "--waves"]).arg(path).args([
        "--scope",
        "top",
        "--map",
        "hclk=clk",
        "--map",
        "hresetn=reset_n",
        "--map",
        "htrans=trans",
        "--map",
        "hready=ready",
        "--map",
        "hwrite=write",
        "--map",
        "haddr=addr",
        "--map",
        "hrdata=data",
        "--map",
        "hresp=resp",
    ]);
    cmd
}

fn result(path: &Path, args: &[&str]) -> Value {
    let output = command(path).args(args).arg("--json").output().unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stdout)
    );
    assert!(output.stderr.is_empty());
    serde_json::from_slice(&output.stdout).unwrap()
}

fn boundary_waveform() -> NamedTempFile {
    waveform(
        "#0\n0c\n0r\nb00 t\n1y\n0w\nb00000000 a\nb00000000 d\n0p\n#5\n1c\n#6\n0c\n1r\nb10 t\nb00001010 a\n#995\n1c\n#996\n0c\n0y\nb00 t\n#1000\nb10100101 d\n#1005\n1c\n#1006\n0c\n1y\nb01011010 d\n#1015\n1c\n#1016\n0c\nb10 t\nb00001100 a\n1w\n#1025\n1c\n#1026\n0c\n0r\n#1035\n1c\n#1036\n0c\n1r\nb10 t\nb00001101 a\n0w\n#1045\n1c\n#1046\n0c\nb00 t\n#1055\n1c\n#1100\n",
    )
}

#[test]
fn pending_wait_and_reset_survive_the_former_chunk_boundary() {
    let file = boundary_waveform();
    let value = result(
        file.path(),
        &[
            "--from",
            "996us",
            "--to",
            "1055us",
            "--include-stall",
            "--max",
            "unlimited",
        ],
    );
    let initial = &value["context"]["initial_data_phase"];
    assert_eq!(initial["state"], "pending");
    assert_eq!(initial["address"]["time"], "995us");
    assert_eq!(initial["address"]["payload"]["haddr"], "8'h0a");
    let rows = value["data"].as_array().unwrap();
    assert_eq!(
        rows.iter()
            .map(|row| (
                row["time"].as_str().unwrap(),
                row["event"].as_str().unwrap()
            ))
            .collect::<Vec<_>>(),
        [
            ("1005us", "data-stall"),
            ("1015us", "data-complete"),
            ("1025us", "address"),
            ("1035us", "reset"),
            ("1045us", "address"),
            ("1055us", "data-complete")
        ]
    );
    assert_eq!(rows[0]["sample_time"], "1004us");
    assert_eq!(rows[1]["payload"]["hrdata"], "8'h5a");
    assert_eq!(
        value["summary"],
        json!({"complete":true,"returned":6,"limit":null,"total":6})
    );
    assert_eq!(value["diagnostics"], json!([]));
}

#[test]
fn quiet_window_keeps_warmed_context_without_a_synthetic_edge() {
    let file = boundary_waveform();
    for (from, to) in [("997us", "1004us"), ("1000us", "1000us")] {
        let value = result(
            file.path(),
            &["--from", from, "--to", to, "--include-stall"],
        );
        assert_eq!(value["data"], json!([]));
        assert_eq!(value["context"]["initial_data_phase"]["state"], "pending");
        assert_eq!(
            value["context"]["initial_data_phase"]["address"]["time"],
            "995us"
        );
        assert_eq!(value["summary"]["total"], 0);
        let output = command(file.path())
            .args(["--from", from, "--to", to, "--include-stall", "--jsonl"])
            .output()
            .unwrap();
        assert!(output.status.success());
        let records = String::from_utf8(output.stdout)
            .unwrap()
            .lines()
            .map(|line| serde_json::from_str::<Value>(line).unwrap())
            .collect::<Vec<_>>();
        assert_eq!(records.len(), 2);
        assert_eq!(records[0]["type"], "begin");
        assert_eq!(records[0]["context"], value["context"]);
        assert_eq!(records[1]["type"], "end");
    }
}

#[test]
fn inclusive_right_edge_retains_an_unfinished_address_and_exact_limit() {
    let file = boundary_waveform();
    let value = result(file.path(), &["--to", "995us", "--max", "2"]);
    assert_eq!(value["data"][1]["event"], "address");
    assert_eq!(value["data"][1]["time"], "995us");
    assert_eq!(
        value["summary"],
        json!({"complete":true,"returned":2,"limit":2,"total":2})
    );
    assert_eq!(value["diagnostics"], json!([]));
    let truncated = result(file.path(), &["--to", "1015us", "--max", "2"]);
    assert_eq!(
        truncated["summary"],
        json!({"complete":false,"returned":2,"limit":2,"total":null})
    );
    assert_eq!(truncated["diagnostics"][0]["code"], "WPK-W0002");
}

#[test]
fn completed_tick_filters_clock_noise_and_samples_absolute_previous_tick() {
    let file = waveform(
        "#0\n0c\n1r\nb10 t\n1y\n0w\nb00001010 a\nb01010101 d\n0p\n#5\n1c\n0c\n#10\n1c\nb11111111 a\n#20\n1c\n#25\n0c\nb00 t\n#30\n1c\n#40\n",
    );
    let value = result(file.path(), &["--max", "unlimited"]);
    assert_eq!(value["data"].as_array().unwrap().len(), 2);
    assert_eq!(value["data"][0]["event"], "address");
    assert_eq!(value["data"][0]["time"], "10us");
    assert_eq!(value["data"][0]["sample_time"], "9us");
    assert_eq!(value["data"][0]["payload"]["haddr"], "8'h0a");
    assert_eq!(value["data"][1]["time"], "30us");
}

#[test]
fn missing_payload_stops_jsonl_after_begin_without_an_end_record() {
    let file = waveform(
        "#0\n0c\n1r\nb10 t\n1y\n0w\nb00000000 d\n0p\n#5\n1c\n#6\n0c\nb00001010 a\n#10\n1c\n",
    );
    let output = command(file.path()).arg("--jsonl").output().unwrap();
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stderr.is_empty());
    let records = String::from_utf8(output.stdout)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str::<Value>(line).unwrap())
        .collect::<Vec<_>>();
    assert_eq!(
        records
            .iter()
            .map(|row| row["type"].as_str().unwrap())
            .collect::<Vec<_>>(),
        ["begin", "fatal"]
    );
    assert_eq!(records[1]["code"], "WPK-F0004");
    assert!(
        records[1]["message"]
            .as_str()
            .unwrap()
            .contains("no value at or before requested time")
    );
}

#[test]
fn zero_limit_fails_before_opening_the_waveform() {
    let output = command(Path::new("missing.vcd"))
        .args(["--max", "0", "--json"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    let value: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value["code"], "WPK-F0001");
}

#[test]
fn first_falling_candidate_does_not_advance_the_warmed_pipeline() {
    let file = boundary_waveform();
    for from in ["995us", "996us"] {
        let value = result(
            file.path(),
            &["--from", from, "--to", "1015us", "--max", "unlimited"],
        );
        let rows = value["data"].as_array().unwrap();
        let completions = rows
            .iter()
            .filter(|row| row["event"] == "data-complete")
            .collect::<Vec<_>>();
        assert_eq!(completions.len(), 1);
        assert_eq!(completions[0]["time"], "1015us");
        if from == "995us" {
            assert_eq!(value["context"]["initial_data_phase"]["state"], "empty");
            assert_eq!(rows[0]["event"], "address");
            assert_eq!(rows[0]["time"], "995us");
        } else {
            assert_eq!(value["context"]["initial_data_phase"]["state"], "pending");
            assert_eq!(rows.len(), 1);
        }
    }
}

#[test]
fn real_and_string_mappings_fail_even_in_a_quiet_window() {
    for declaration in ["$var real 1 a addr $end", "$var string 1 a addr $end"] {
        let file = waveform("#0\n0c\n1r\nb00 t\n1y\n0w\nb00000000 d\n0p\n#10\n");
        let content = fs::read_to_string(file.path())
            .unwrap()
            .replace("$var wire 8 a addr $end", declaration);
        fs::write(file.path(), content).unwrap();
        let output = command(file.path()).arg("--jsonl").output().unwrap();
        assert_eq!(output.status.code(), Some(1));
        let records = String::from_utf8(output.stdout)
            .unwrap()
            .lines()
            .map(|line| serde_json::from_str::<Value>(line).unwrap())
            .collect::<Vec<_>>();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0]["type"], "fatal");
        assert_eq!(records[0]["code"], "WPK-F0004");
    }
}

#[test]
fn shared_payload_history_keeps_each_standard_mapping() {
    let file = boundary_waveform();
    let content = fs::read_to_string(file.path())
        .unwrap()
        .replacen("\n0w\n", "\nxw\n", 1);
    fs::write(file.path(), content).unwrap();
    let output = command(file.path())
        .args([
            "--map",
            "hwdata=data",
            "--map",
            "hwuser=data",
            "--from",
            "996us",
            "--to",
            "1025us",
            "--json",
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let value: Value = serde_json::from_slice(&output.stdout).unwrap();
    for standard in ["hrdata", "hwdata", "hwuser"] {
        assert_eq!(value["context"]["mappings"][standard]["path"], "top.data");
    }
    assert_eq!(value["data"][0]["direction"], "unknown");
    for standard in ["hrdata", "hwdata", "hwuser"] {
        assert_eq!(value["data"][0]["payload"][standard], "8'h5a");
    }
}

#[test]
fn boundary_queries_match_between_vcd_and_fst_for_both_profiles() {
    let file = boundary_waveform();
    let fst = NamedTempFile::with_suffix(".fst").unwrap();
    let conversion = Command::new("vcd2fst")
        .arg(file.path())
        .arg(fst.path())
        .output()
        .unwrap();
    assert!(conversion.status.success(), "{conversion:?}");
    for profile in ["ahb-lite", "ahb5"] {
        for (from, to) in [
            ("0us", "1055us"),
            ("995us", "995us"),
            ("996us", "1015us"),
            ("997us", "1004us"),
        ] {
            for max in ["1", "2", "unlimited"] {
                let args = [
                    "--profile",
                    profile,
                    "--from",
                    from,
                    "--to",
                    to,
                    "--max",
                    max,
                    "--include-stall",
                ];
                assert_eq!(
                    result(file.path(), &args),
                    result(fst.path(), &args),
                    "{profile} {from}..{to} max={max}"
                );
            }
        }
    }
}

#[test]
fn first_dump_tick_cannot_supply_a_pre_edge_sample() {
    let file = waveform(
        "#100\n1c\n1r\nb10 t\n1y\n0w\nb00001010 a\nb01010101 d\n0p\n#200\nb11110000 a\n#300\n0c\n#400\n1c\n#500\n",
    );
    let value = result(file.path(), &["--max", "unlimited"]);
    assert_eq!(value["data"].as_array().unwrap().len(), 1);
    assert_eq!(value["data"][0]["time"], "400us");
    assert_eq!(value["data"][0]["sample_time"], "399us");
}

#[test]
fn raw_event_payload_is_zero_width_before_its_first_occurrence() {
    let file =
        waveform("#0\n0c\n1r\nb10 t\n1y\n0w\nb00000000 d\n0p\n#5\n1c\n#6\n0c\n1a\n#15\n1c\n#20\n");
    let content = fs::read_to_string(file.path())
        .unwrap()
        .replace("$var wire 8 a addr $end", "$var event 1 a addr $end");
    fs::write(file.path(), content).unwrap();
    let fst = NamedTempFile::with_suffix(".fst").unwrap();
    let conversion = Command::new("vcd2fst")
        .arg(file.path())
        .arg(fst.path())
        .output()
        .unwrap();
    assert!(conversion.status.success(), "{conversion:?}");
    let vcd = result(file.path(), &["--max", "unlimited"]);
    assert_eq!(vcd["data"][0]["time"], "5us");
    assert_eq!(vcd["data"][0]["payload"]["haddr"], "0'h0");
    assert_eq!(vcd["data"][2]["time"], "15us");
    assert_eq!(vcd["data"][2]["payload"]["haddr"], "0'h0");
    assert_eq!(vcd, result(fst.path(), &["--max", "unlimited"]));
}
