#![cfg(feature = "wlf")]

use std::path::{Path, PathBuf};
use std::process::Output;

use serde_json::Value;

mod common;
use common::{fixture_path, wavepeek_cmd};

fn converted(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(format!("tests/fixtures/wlf/{name}.wlf"))
}

fn native(name: &str) -> PathBuf {
    PathBuf::from(std::env::var_os("ONDAS_FIXTURES_DIR").expect("ONDAS_FIXTURES_DIR is required"))
        .join(format!("wlf/{name}/waveform.wlf"))
}

fn run(path: &Path, args: &[&str], mode: &str) -> Output {
    wavepeek_cmd()
        .args(args)
        .arg("--waves")
        .arg(path)
        .arg(mode)
        .output()
        .unwrap()
}

fn json(path: &Path, args: &[&str]) -> Value {
    let output = run(path, args, "--json");
    assert!(output.status.success(), "{args:?}: {output:?}");
    assert!(output.stderr.is_empty(), "{args:?}: {output:?}");
    serde_json::from_slice(&output.stdout).unwrap()
}

fn assert_parity(name: &str, args: &[&str]) -> Value {
    let vcd = json(&fixture_path(&format!("{name}.vcd")), args);
    let wlf = json(&converted(name), args);
    assert_eq!(vcd, wlf, "{args:?}");
    wlf
}

#[test]
fn converted_waveforms_preserve_hierarchy_and_point_and_range_queries() {
    assert_parity("m2_core", &["info"]);
    assert_parity("m2_core", &["scope"]);
    let listing = assert_parity(
        "m2_core",
        &[
            "signal",
            "--scope",
            "top",
            "--recursive",
            "--filter",
            "^top[.]cpu[.]valid$",
        ],
    );
    assert_parity(
        "m2_core",
        &[
            "value",
            "--signals",
            listing["data"][0]["path"].as_str().unwrap(),
            "--at",
            "5ns",
        ],
    );
    assert_parity(
        "value_vectors",
        &[
            "value",
            "--signals",
            "top.data[7:4],top.status,top.data[7:4]",
            "--at",
            "10ns,0ns,5ns,5ns",
        ],
    );
    for command in [
        vec![
            "change",
            "--signals",
            "top.data",
            "--on",
            "*",
            "--sample-mode",
            "native",
            "--from",
            "5ns",
            "--to",
            "7ns",
        ],
        vec!["property", "--on", "posedge top.clk", "--eval", "top.valid"],
        vec![
            "extract",
            "generic",
            "--on",
            "posedge top.clk",
            "--when",
            "top.valid",
            "--payload",
            "top.data",
        ],
    ] {
        assert_parity("change_property_core", &command);
    }
    assert_parity(
        "extract_global_include",
        &[
            "extract",
            "axi",
            "--profile",
            "axi4-lite",
            "--include",
            r"^top\.(aclk|arvalid|arready|araddr)$",
            "--max",
            "1",
        ],
    );
}

#[test]
fn native_samples_preserve_aliases_deltas_and_expression_types() {
    let waveform = native("wlf0070-typed-deltas-aliases");
    let samples = json(
        &waveform,
        &[
            "value",
            "--at",
            "70ps,80ps",
            "--signals",
            "top.bus,top.net_alias,top.logic_value",
        ],
    );
    for row in samples["data"].as_array().unwrap() {
        assert_eq!(row["signals"][0]["value"], "9'h005");
        assert_eq!(row["signals"][0]["value"], row["signals"][1]["value"]);
    }
    assert_eq!(samples["data"][0]["signals"][2]["value"], "1'h1");
    let real = json(
        &waveform,
        &[
            "property",
            "--on",
            "top.bus",
            "--sample-mode",
            "native",
            "--eval",
            "top.real_value < -3.0",
            "--from",
            "0ps",
            "--to",
            "110ps",
        ],
    );
    assert_eq!(real["data"][0]["time"], "70ps");
    assert_eq!(real["data"][0]["kind"], "assert");
    assert_eq!(real["data"][1]["time"], "110ps");
    assert_eq!(real["data"][1]["kind"], "deassert");

    let event = json(
        &native("wlf0071-event-occurrences"),
        &[
            "property",
            "--on",
            "top.pulse",
            "--sample-mode",
            "native",
            "--eval",
            "top.count == 5",
        ],
    );
    assert_eq!(event["data"][0]["time"], "3ns");
    assert_eq!(event["data"][0]["kind"], "assert");
    assert_eq!(event["data"][1]["time"], "5ns");

    let waveform = native("wlf0072-strings-and-names");
    let string = json(
        &waveform,
        &[
            "property",
            "--on",
            "top.message",
            "--sample-mode",
            "native",
            "--eval",
            r#"top.message == "hello""#,
            "--to",
            "2ns",
        ],
    );
    assert_eq!(string["data"][0]["time"], "1ns");
    assert_eq!(string["data"][0]["kind"], "assert");
    assert_eq!(string["data"][1]["time"], "2ns");

    let listing = json(&waveform, &["signal", "--scope", "top"]);
    let escaped = &listing["data"][0];
    assert_eq!(escaped["name"], r"\name.with.dot ");
    let selected = json(
        &waveform,
        &[
            "value",
            "--signals",
            escaped["path"].as_str().unwrap(),
            "--at",
            "0ns",
        ],
    );
    assert_eq!(selected["data"][0]["signals"][0]["value"], "8'h01");
}

#[test]
fn machine_streams_remain_parseable_on_success_and_file_errors() {
    let sample = [
        "change",
        "--signals",
        "top.cpu.valid",
        "--on",
        "*",
        "--sample-mode",
        "native",
    ];
    let json = json(&converted("m2_core"), &sample);
    let output = run(&converted("m2_core"), &sample, "--jsonl");
    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let records = String::from_utf8(output.stdout).unwrap();
    let records = records
        .lines()
        .map(|line| serde_json::from_str::<Value>(line).unwrap())
        .collect::<Vec<_>>();
    assert_eq!(records.first().unwrap()["type"], "begin");
    assert_eq!(records.last().unwrap()["type"], "end");
    assert_eq!(
        records
            .iter()
            .filter(|row| row["type"] == "data")
            .map(|row| &row["data"])
            .collect::<Vec<_>>(),
        json["data"].as_array().unwrap().iter().collect::<Vec<_>>(),
    );

    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("invalid.wlf");
    std::fs::write(&path, b"invalid waveform").unwrap();
    for mode in ["--json", "--jsonl"] {
        let output = run(&path, &["info"], mode);
        assert_eq!(output.status.code(), Some(2), "{output:?}");
        let fatal: Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(fatal["type"], "fatal");
        assert_eq!(fatal["code"], "WPK-F0002");
    }
}
