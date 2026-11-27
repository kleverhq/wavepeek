#![cfg(feature = "vwdb")]

use std::path::{Path, PathBuf};

use serde_json::{Value, json};

mod common;
use common::{fixture_path, wavepeek_cmd};

fn recording(name: &str) -> PathBuf {
    PathBuf::from(std::env::var_os("WAVEPEEK_VWDB_TEST_DIR").expect("run just test-vwdb"))
        .join(name)
        .join("ida.db/ida.vwdb")
}

fn corpus(name: &str) -> PathBuf {
    PathBuf::from(std::env::var_os("ONDAS_FIXTURES_DIR").expect("run inside ./dev"))
        .join("vwdb")
        .join(name)
        .join("waveform.vwdb")
}

fn run(path: &Path, args: &[&str], output_mode: &str) -> String {
    let output = wavepeek_cmd()
        .args(args)
        .arg("--waves")
        .arg(path)
        .arg(output_mode)
        .output()
        .unwrap();
    assert!(output.status.success(), "{path:?} {args:?}: {output:?}");
    assert!(output.stderr.is_empty(), "{path:?} {args:?}: {output:?}");
    String::from_utf8(output.stdout).unwrap()
}

fn run_json(path: &Path, args: &[&str]) -> Value {
    serde_json::from_str(&run(path, args, "--json")).unwrap()
}

fn assert_parity(name: &str, args: &[&str]) -> Value {
    let vcd = run_json(&fixture_path(&format!("{name}.vcd")), args);
    let vwdb = run_json(&recording(name), args);
    assert_eq!(vwdb, vcd, "{name}: {args:?}");
    assert_eq!(vwdb["diagnostics"], json!([]));
    assert!(!vwdb["data"].as_array().unwrap().is_empty());
    vwdb
}

#[test]
fn metadata_hierarchy_and_points() {
    let info = assert_parity("m2_core", &["info"]);
    assert_eq!(info["data"][0]["time_end"], "10ns");
    let scope = assert_parity("m2_core", &["scope", "--filter", "^top[.]cpu$"]);
    let cpu = scope["data"][0]["path"].as_str().unwrap();
    let signal = run_json(
        &recording("m2_core"),
        &["signal", "--scope", cpu, "--filter", "^valid$"],
    );
    let row = &signal["data"][0];
    let path = row["path"].as_str().unwrap();
    assert_eq!(path, "top.cpu.valid");
    assert_eq!(row["width"], 1);
    assert_parity("m2_core", &["value", "--signals", path, "--at", "5ns"]);
    let result = assert_parity(
        "m2_core",
        &[
            "value",
            "--signals",
            &format!("{path},top.data,{path}"),
            "--at",
            "5ns",
            "--at",
            "0ns",
            "--at",
            "5ns",
        ],
    );
    assert_eq!(result["data"].as_array().unwrap().len(), 3);
    assert_eq!(result["data"][0]["signals"][2]["value"], "1'h1");
    assert_eq!(result["data"][1]["signals"][0]["value"], "1'h0");
}

#[test]
fn inclusive_boundary_property_generic_and_jsonl_match_vcd() {
    let name = "m2_core";
    let change = [
        "change",
        "--signals",
        "top.cpu.valid",
        "--from",
        "5ns",
        "--to",
        "5ns",
        "--on",
        "*",
        "--sample-mode",
        "native",
    ];
    let result = assert_parity(name, &change);
    assert_eq!(result["data"][0]["time"], "5ns");
    let vcd = run(&fixture_path("m2_core.vcd"), &change, "--jsonl");
    let vwdb = run(&recording(name), &change, "--jsonl");
    assert_eq!(vwdb, vcd);
    assert_parity(
        name,
        &[
            "property",
            "--on",
            "top.cpu.valid",
            "--sample-mode",
            "native",
            "--eval",
            "top.cpu.valid",
        ],
    );
    assert_parity(
        name,
        &[
            "extract",
            "generic",
            "--on",
            "posedge top.clk",
            "--when",
            "1",
            "--payload",
            "top.data",
        ],
    );
}

#[test]
fn axi_transfer_matches_vcd() {
    let result = assert_parity(
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
    assert_eq!(result["data"][0]["payload"]["araddr"], "32'h00001234");
}

#[test]
fn native_enum_label_selects_the_recorded_value() {
    let input = corpus("vwdb0000-enum-core");
    let result = run_json(
        &input,
        &[
            "property",
            "--from",
            "0ns",
            "--to",
            "9ns",
            "--on",
            "top.alternate.value",
            "--sample-mode",
            "native",
            "--eval",
            "top.alternate.value == type(top.alternate.value)::RUN",
        ],
    );
    assert_eq!(result["diagnostics"], json!([]));
    assert_eq!(
        result["data"],
        json!([
            {"time": "4ns", "sample_time": "4ns", "kind": "assert"},
            {"time": "9ns", "sample_time": "9ns", "kind": "deassert"},
        ])
    );
}
