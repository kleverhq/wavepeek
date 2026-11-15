use std::fs;

use assert_cmd::prelude::*;
use predicates::prelude::*;
use serde_json::{Value, json};
use tempfile::NamedTempFile;

mod common;
use common::{fixture_path, wavepeek_cmd};

fn parse_json(stdout: &[u8]) -> Value {
    serde_json::from_slice(stdout).expect("stdout should be valid json")
}

fn write_fixture(contents: &str, suffix: &str) -> NamedTempFile {
    let fixture = NamedTempFile::with_suffix(suffix).expect("temp fixture should create");
    fs::write(fixture.path(), contents).expect("fixture should write");
    fixture
}

const ROW_MODES_VCD: &str = concat!(
    "$timescale 1ns $end\n",
    "$scope module top $end\n",
    "$var wire 1 ! clk $end\n",
    "$var wire 1 \" a $end\n",
    "$var wire 1 # b $end\n",
    "$upscope $end\n",
    "$enddefinitions $end\n",
    "#0\n0!\n0\"\n0#\n",
    "#5\n1!\n",
    "#10\n0!\n",
    "#15\n1!\n1\"\n",
    "#20\n0!\n",
    "#25\n1!\n",
    "#30\n0!\n",
    "#35\n1!\n1#\n",
);

#[test]
fn change_row_mode_and_values_matrix() {
    let fixture = write_fixture(ROW_MODES_VCD, "change-row-modes.vcd");
    let fixture = fixture.path().to_string_lossy().into_owned();
    let run = |mode: &str, values: &str| {
        let output = wavepeek_cmd()
            .args([
                "change",
                "--waves",
                fixture.as_str(),
                "--from",
                "5ns",
                "--to",
                "35ns",
                "--signals",
                "top.a,top.b",
                "--on",
                "posedge top.clk",
                "--sample-mode",
                "native",
                "--row-mode",
                mode,
                "--row-values",
                values,
                "--json",
            ])
            .output()
            .expect("change should execute");
        assert!(output.status.success());
        let value = parse_json(&output.stdout);
        assert!(value.get("context").is_none());
        value["data"].clone()
    };

    assert_eq!(
        run("dense", "full"),
        json!([
            {"time":"5ns","sample_time":"5ns","signals":[{"path":"top.a","value":"1'h0"},{"path":"top.b","value":"1'h0"}]},
            {"time":"15ns","sample_time":"15ns","signals":[{"path":"top.a","value":"1'h1"},{"path":"top.b","value":"1'h0"}]},
            {"time":"25ns","sample_time":"25ns","signals":[{"path":"top.a","value":"1'h1"},{"path":"top.b","value":"1'h0"}]},
            {"time":"35ns","sample_time":"35ns","signals":[{"path":"top.a","value":"1'h1"},{"path":"top.b","value":"1'h1"}]}
        ])
    );
    assert_eq!(
        run("dense", "delta"),
        json!([
            {"time":"5ns","sample_time":"5ns","signals":[{"path":"top.a","value":"1'h0"},{"path":"top.b","value":"1'h0"}]},
            {"time":"15ns","sample_time":"15ns","signals":[{"path":"top.a","value":"1'h1"}]},
            {"time":"25ns","sample_time":"25ns","signals":[]},
            {"time":"35ns","sample_time":"35ns","signals":[{"path":"top.b","value":"1'h1"}]}
        ])
    );
    assert_eq!(
        run("sparse", "full"),
        json!([
            {"time":"15ns","sample_time":"15ns","signals":[{"path":"top.a","value":"1'h1"},{"path":"top.b","value":"1'h0"}]},
            {"time":"35ns","sample_time":"35ns","signals":[{"path":"top.a","value":"1'h1"},{"path":"top.b","value":"1'h1"}]}
        ])
    );
    assert_eq!(
        run("sparse", "delta"),
        json!([
            {"time":"15ns","sample_time":"15ns","signals":[{"path":"top.a","value":"1'h1"},{"path":"top.b","value":"1'h0"}]},
            {"time":"35ns","sample_time":"35ns","signals":[{"path":"top.b","value":"1'h1"}]}
        ])
    );

    let limited = wavepeek_cmd()
        .args([
            "change",
            "--waves",
            fixture.as_str(),
            "--from",
            "5ns",
            "--to",
            "35ns",
            "--signals",
            "top.a,top.b",
            "--on",
            "posedge top.clk",
            "--sample-mode",
            "native",
            "--row-mode",
            "sparse",
            "--max",
            "1",
            "--json",
        ])
        .output()
        .expect("limited change should execute");
    let limited = parse_json(&limited.stdout);
    assert_eq!(limited["data"][0]["time"], "15ns");
    assert_eq!(limited["diagnostics"][0]["code"], "WPK-W0002");
}

#[test]
fn change_dense_pre_edge_delta_keeps_empty_rows() {
    let fixture = write_fixture(RTL_SAMPLING_VCD, "change-dense-pre-edge.vcd");
    let fixture = fixture.path().to_string_lossy().into_owned();
    let output = wavepeek_cmd()
        .args([
            "change",
            "--waves",
            fixture.as_str(),
            "--from",
            "5ns",
            "--to",
            "25ns",
            "--scope",
            "top",
            "--signals",
            "data",
            "--on",
            "posedge clk",
            "--row-values",
            "delta",
            "--json",
        ])
        .output()
        .expect("change should execute");

    assert!(output.status.success());
    assert_eq!(
        parse_json(&output.stdout)["data"],
        json!([
            {"time":"5ns","sample_time":"4ns","signals":[{"path":"top.data", "relative_path": "data","value":"8'h00"}]},
            {"time":"15ns","sample_time":"14ns","signals":[{"path":"top.data", "relative_path": "data","value":"8'haa"}]},
            {"time":"25ns","sample_time":"24ns","signals":[]}
        ])
    );
}

const RTL_SAMPLING_VCD: &str = concat!(
    "$date\n",
    "  today\n",
    "$end\n",
    "$version\n",
    "  wavepeek-rtl-sampling\n",
    "$end\n",
    "$timescale 1ns $end\n",
    "$scope module top $end\n",
    "$var wire 1 ! clk $end\n",
    "$var wire 1 \" valid $end\n",
    "$var wire 8 # data $end\n",
    "$upscope $end\n",
    "$enddefinitions $end\n",
    "#0\n",
    "0!\n",
    "0\"\n",
    "b00000000 #\n",
    "#5\n",
    "1!\n",
    "1\"\n",
    "b10101010 #\n",
    "#10\n",
    "0!\n",
    "#15\n",
    "1!\n",
    "#20\n",
    "0!\n",
    "#25\n",
    "1!\n",
    "0\"\n",
    "b01010101 #\n",
    "#30\n",
    "0!\n",
    "#35\n",
    "1!\n",
);

#[test]
fn change_sample_mode_pre_edge_samples_before_trigger_edge() {
    let fixture = write_fixture(RTL_SAMPLING_VCD, "change-rtl-sampling.vcd");
    let fixture = fixture.path().to_string_lossy().into_owned();

    let native_output = wavepeek_cmd()
        .args([
            "change",
            "--waves",
            fixture.as_str(),
            "--from",
            "0ns",
            "--to",
            "15ns",
            "--scope",
            "top",
            "--signals",
            "data",
            "--on",
            "posedge clk",
            "--sample-mode",
            "native",
            "--row-mode",
            "sparse",
            "--json",
        ])
        .output()
        .expect("change should execute");
    let pre_edge_output = wavepeek_cmd()
        .args([
            "change",
            "--waves",
            fixture.as_str(),
            "--from",
            "0ns",
            "--to",
            "15ns",
            "--scope",
            "top",
            "--signals",
            "data",
            "--on",
            "posedge clk",
            "--row-mode",
            "sparse",
            "--json",
        ])
        .output()
        .expect("change should execute");
    let pre_edge_human_output = wavepeek_cmd()
        .args([
            "change",
            "--waves",
            fixture.as_str(),
            "--from",
            "0ns",
            "--to",
            "15ns",
            "--scope",
            "top",
            "--signals",
            "data",
            "--on",
            "posedge clk",
            "--row-mode",
            "sparse",
        ])
        .output()
        .expect("change should execute");

    assert!(native_output.status.success());
    assert!(pre_edge_output.status.success());
    assert!(pre_edge_human_output.status.success());
    assert_eq!(
        parse_json(&native_output.stdout)["data"],
        json!([{
            "time": "5ns",
            "sample_time": "5ns",
            "signals": [{"path": "top.data", "relative_path": "data", "value": "8'haa"}]
        }])
    );
    assert_eq!(
        parse_json(&pre_edge_output.stdout)["data"],
        json!([{
            "time": "15ns",
            "sample_time": "14ns",
            "signals": [{"path": "top.data", "relative_path": "data", "value": "8'haa"}]
        }])
    );
    assert_eq!(
        String::from_utf8(pre_edge_human_output.stdout).expect("human stdout should be UTF-8"),
        "@15ns sample@14ns data=8'haa\n"
    );
}

#[test]
fn change_sample_mode_pre_edge_preserves_from_baseline() {
    let fixture = write_fixture(RTL_SAMPLING_VCD, "change-rtl-sampling-boundary.vcd");
    let fixture = fixture.path().to_string_lossy().into_owned();

    let short_output = wavepeek_cmd()
        .args([
            "change",
            "--waves",
            fixture.as_str(),
            "--from",
            "5ns",
            "--to",
            "15ns",
            "--scope",
            "top",
            "--signals",
            "data",
            "--on",
            "posedge clk",
            "--sample-mode",
            "pre-edge",
            "--row-mode",
            "sparse",
            "--json",
        ])
        .output()
        .expect("change should execute");
    let long_output = wavepeek_cmd()
        .args([
            "change",
            "--waves",
            fixture.as_str(),
            "--from",
            "5ns",
            "--to",
            "35ns",
            "--scope",
            "top",
            "--signals",
            "data",
            "--on",
            "posedge clk",
            "--sample-mode",
            "pre-edge",
            "--row-mode",
            "sparse",
            "--json",
        ])
        .output()
        .expect("change should execute");

    assert!(short_output.status.success());
    let short_json = parse_json(&short_output.stdout);
    assert_eq!(short_json["data"], json!([]));
    assert_eq!(short_json["diagnostics"], json!([]));
    assert!(long_output.status.success());
    assert_eq!(
        parse_json(&long_output.stdout)["data"],
        json!([{
            "time": "35ns",
            "sample_time": "34ns",
            "signals": [{"path": "top.data", "relative_path": "data", "value": "8'h55"}]
        }])
    );
}

#[test]
fn change_default_pre_edge_rejects_non_edge_triggers() {
    let fixture = write_fixture(RTL_SAMPLING_VCD, "change-rtl-sampling-invalid.vcd");
    let fixture = fixture.path().to_string_lossy().into_owned();

    for trigger in ["*", "data", "data or posedge clk"] {
        wavepeek_cmd()
            .args([
                "change",
                "--waves",
                fixture.as_str(),
                "--scope",
                "top",
                "--signals",
                "data",
                "--on",
                trigger,
            ])
            .assert()
            .failure()
            .code(1)
            .stdout(predicate::str::is_empty())
            .stderr(predicate::str::starts_with("fatal: args:"))
            .stderr(predicate::str::contains(
                "--sample-mode pre-edge requires --on with only edge event terms",
            ));
    }
}

#[test]
fn change_requires_signals_flag() {
    let fixture = fixture_path("m2_core.vcd");
    let fixture = fixture.to_string_lossy().into_owned();

    wavepeek_cmd()
        .args([
            "change",
            "--waves",
            fixture.as_str(),
            "--on",
            "*",
            "--sample-mode",
            "native",
        ])
        .assert()
        .failure()
        .code(1)
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::starts_with("fatal: args:"))
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
            "--on",
            "*",
            "--sample-mode",
            "native",
            "--json",
        ])
        .output()
        .expect("change should execute");

    assert!(output.status.success());
    assert!(output.stderr.is_empty());

    let value = parse_json(&output.stdout);
    assert_eq!(value["command"], "change");
    assert_eq!(value["diagnostics"], json!([]));
    assert_eq!(
        value["data"],
        json!([
            {
                "time": "5ns",
                "sample_time": "5ns",
                "signals": [
                    {"path": "top.clk", "value": "1'h1"},
                    {"path": "top.data", "value": "8'h00"}
                ]
            },
            {
                "time": "10ns",
                "sample_time": "10ns",
                "signals": [
                    {"path": "top.clk", "value": "1'h1"},
                    {"path": "top.data", "value": "8'h0f"}
                ]
            }
        ])
    );
}

#[test]
fn change_flattens_repeated_and_comma_separated_signals_in_order() {
    let fixture = fixture_path("m2_core.vcd");
    let fixture = fixture.to_string_lossy().into_owned();

    wavepeek_cmd()
        .args([
            "change",
            "--waves",
            fixture.as_str(),
            "--from",
            "10ns",
            "--to",
            "10ns",
            "--signals",
            "top.clk",
            "--signals",
            "top.data,top.clk",
            "--on",
            "*",
            "--sample-mode",
            "native",
        ])
        .assert()
        .success()
        .stdout(predicate::eq(
            "@10ns top.clk=1'h1 top.data=8'h0f top.clk=1'h1\n",
        ))
        .stderr(predicate::str::is_empty());
}

#[test]
fn change_requires_on_flag() {
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
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stderr.is_empty());
    let fatal = parse_json(&output.stdout);
    assert_eq!(fatal["type"], "fatal");
    assert_eq!(fatal["code"], "WPK-F0001");
    assert!(fatal["message"].as_str().unwrap().contains("--on <ON>"));
    assert!(
        fatal["message"]
            .as_str()
            .unwrap()
            .contains("See 'wavepeek change --help'.")
    );
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
            "--on",
            "data",
            "--sample-mode",
            "native",
            "--json",
        ])
        .output()
        .expect("change should execute");

    assert!(output.status.success());
    assert!(output.stderr.is_empty());

    let value = parse_json(&output.stdout);
    assert_eq!(value["diagnostics"], json!([]));
    assert_eq!(
        value["data"],
        json!([
            {
                "time": "10ns",
                "sample_time": "10ns",
                "signals": [
                    {"path": "top.data", "relative_path": "data", "value": "8'h0f"},
                    {"path": "top.clk", "relative_path": "clk", "value": "1'h1"}
                ]
            }
        ])
    );
}

#[test]
fn change_scope_mixes_relative_and_canonical_names() {
    let fixture = fixture_path("m2_core.vcd");
    let fixture = fixture.to_string_lossy().into_owned();

    let output = wavepeek_cmd()
        .args([
            "change",
            "--waves",
            fixture.as_str(),
            "--scope",
            "top",
            "--signals",
            "cpu.valid,top.clk",
            "--on",
            "posedge top.cpu.valid",
            "--sample-mode",
            "native",
            "--json",
        ])
        .output()
        .expect("change should execute");

    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let value = parse_json(&output.stdout);
    assert_eq!(value["context"]["scope"], "top");
    assert_eq!(value["data"][0]["time"], "5ns");
    assert_eq!(
        value["data"][0]["signals"],
        json!([
            {"path": "top.cpu.valid", "relative_path": "cpu.valid", "value": "1'h1"},
            {"path": "top.clk", "relative_path": "clk", "value": "1'h1"}
        ])
    );
}

#[test]
fn change_zero_delta_path_returns_empty_data_without_diagnostic() {
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
            "--on",
            "posedge clk",
            "--sample-mode",
            "native",
            "--row-mode",
            "sparse",
            "--json",
        ])
        .output()
        .expect("change should execute");

    assert!(output.status.success());
    let value = parse_json(&output.stdout);
    assert_eq!(value["data"], json!([]));
    assert_eq!(value["diagnostics"], json!([]));
}

#[test]
fn change_zero_delta_uses_normal_human_output_and_empty_machine_diagnostics() {
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
            "--on",
            "posedge clk",
            "--sample-mode",
            "native",
            "--row-mode",
            "sparse",
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
            "--on",
            "posedge clk",
            "--sample-mode",
            "native",
            "--row-mode",
            "sparse",
        ])
        .output()
        .expect("human change run should execute");

    assert!(json_output.status.success());
    assert!(human_output.status.success());
    assert_eq!(
        String::from_utf8_lossy(&human_output.stdout).trim(),
        "no change rows found in selected time range"
    );
    assert!(human_output.stderr.is_empty());
    let json = parse_json(&json_output.stdout);
    assert_eq!(json["data"], json!([]));
    assert_eq!(json["diagnostics"], json!([]));
}

#[test]
fn change_unlimited_empty_result_has_no_diagnostics_in_json_and_human_modes() {
    let fixture = fixture_path("m2_core.vcd");
    let fixture = fixture.to_string_lossy().into_owned();

    let json_output = wavepeek_cmd()
        .args([
            "change",
            "--on",
            "*",
            "--sample-mode",
            "native",
            "--waves",
            fixture.as_str(),
            "--from",
            "9ns",
            "--to",
            "9ns",
            "--signals",
            "top.clk,top.data",
            "--row-mode",
            "sparse",
            "--max",
            "unlimited",
            "--json",
        ])
        .output()
        .expect("json change run should execute");
    let human_output = wavepeek_cmd()
        .args([
            "change",
            "--on",
            "*",
            "--sample-mode",
            "native",
            "--waves",
            fixture.as_str(),
            "--from",
            "9ns",
            "--to",
            "9ns",
            "--signals",
            "top.clk,top.data",
            "--row-mode",
            "sparse",
            "--max",
            "unlimited",
        ])
        .output()
        .expect("human change run should execute");

    assert!(json_output.status.success());
    assert!(human_output.status.success());

    let value = parse_json(&json_output.stdout);
    assert_eq!(value["data"], json!([]));
    assert_eq!(value["diagnostics"], json!([]));
    assert_eq!(value["summary"]["limit"], Value::Null);
    assert_eq!(
        String::from_utf8_lossy(&human_output.stdout).trim(),
        "no change rows found in selected time range"
    );
    assert!(human_output.stderr.is_empty());
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
            "--on",
            "*",
            "--sample-mode",
            "native",
            "--json",
        ])
        .output()
        .expect("change should execute");

    assert!(output.status.success());
    let value = parse_json(&output.stdout);
    assert_eq!(value["diagnostics"], json!([]));
    assert_eq!(
        value["data"],
        json!([
            {
                "time": "5ns",
                "sample_time": "5ns",
                "signals": [
                    {"path": "top.sig", "relative_path": "sig", "value": "1'h0"}
                ]
            }
        ])
    );
}

#[test]
fn change_sparse_from_timestamp_is_baseline_only_for_emission() {
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
            "--on",
            "posedge clk",
            "--sample-mode",
            "native",
            "--row-mode",
            "sparse",
            "--json",
        ])
        .output()
        .expect("change should execute");

    assert!(output.status.success());
    let value = parse_json(&output.stdout);
    assert_eq!(value["data"], json!([]));
    assert_eq!(value["diagnostics"], json!([]));
}

#[test]
fn change_sparse_equal_from_and_to_never_emits_baseline_row() {
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
            "--on",
            "posedge clk",
            "--sample-mode",
            "native",
            "--row-mode",
            "sparse",
            "--json",
        ])
        .output()
        .expect("change should execute");

    assert!(output.status.success());
    let value = parse_json(&output.stdout);
    assert_eq!(value["data"], json!([]));
    assert_eq!(value["diagnostics"], json!([]));
}

#[test]
fn change_accepts_inclusive_dump_time_bounds() {
    let fixture = fixture_path("m2_core.vcd");
    let fixture = fixture.to_string_lossy().into_owned();

    wavepeek_cmd()
        .args([
            "change",
            "--on",
            "*",
            "--sample-mode",
            "native",
            "--waves",
            fixture.as_str(),
            "--from",
            "0ns",
            "--to",
            "10ns",
            "--signals",
            "top.clk",
            "--json",
        ])
        .assert()
        .success();

    wavepeek_cmd()
        .args([
            "change",
            "--on",
            "*",
            "--sample-mode",
            "native",
            "--waves",
            fixture.as_str(),
            "--from",
            "10ns",
            "--to",
            "10ns",
            "--signals",
            "top.clk",
            "--json",
        ])
        .assert()
        .success();
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
            "--on",
            "posedge clk1, posedge clk2",
            "--sample-mode",
            "native",
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
            "--on",
            "posedge clk1 or posedge clk2",
            "--sample-mode",
            "native",
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
            "--on",
            "posedge clk1, posedge clk2",
            "--sample-mode",
            "native",
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
            "--on",
            "negedge clk",
            "--sample-mode",
            "native",
            "--json",
        ])
        .output()
        .expect("change should execute");

    assert!(output.status.success());
    let value = parse_json(&output.stdout);
    assert_eq!(value["diagnostics"], json!([]));
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
            "--on",
            "edge clk",
            "--sample-mode",
            "native",
            "--json",
        ])
        .output()
        .expect("change should execute");

    assert!(output.status.success());
    let value = parse_json(&output.stdout);
    assert_eq!(value["diagnostics"], json!([]));
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
fn change_iff_executes_end_to_end() {
    let fixture = fixture_path("m2_core.vcd");
    let fixture = fixture.to_string_lossy().into_owned();

    let json_output = wavepeek_cmd()
        .args([
            "change",
            "--waves",
            fixture.as_str(),
            "--scope",
            "top",
            "--signals",
            "data,clk",
            "--on",
            "posedge clk iff data == 8'h00",
            "--sample-mode",
            "native",
            "--json",
        ])
        .output()
        .expect("json change run should execute");
    let human_output = wavepeek_cmd()
        .args([
            "change",
            "--waves",
            fixture.as_str(),
            "--scope",
            "top",
            "--signals",
            "data,clk",
            "--on",
            "posedge clk iff data == 8'h00",
            "--sample-mode",
            "native",
        ])
        .output()
        .expect("human change run should execute");

    assert!(json_output.status.success());
    assert!(human_output.status.success());
    assert!(json_output.stderr.is_empty());
    assert!(human_output.stderr.is_empty());

    let json = parse_json(&json_output.stdout);
    assert_eq!(json["diagnostics"], json!([]));
    assert_eq!(
        json["data"],
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
    assert_eq!(
        String::from_utf8_lossy(&human_output.stdout).trim(),
        "@5ns data=8'h00 clk=1'h1"
    );
}

#[test]
fn change_invalid_on_reports_expr_error() {
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
            "--on",
            "posedge (",
            "--sample-mode",
            "native",
        ])
        .assert()
        .failure()
        .code(1)
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::starts_with("fatal: expr:"))
        .stderr(predicate::str::contains("EXPR-PARSE-EVENT-UNMATCHED-OPEN"))
        .stderr(predicate::str::contains("source: posedge ("));
}

#[test]
fn change_unmatched_close_parenthesis_reports_expr_error() {
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
            "--on",
            "posedge clk)",
            "--sample-mode",
            "native",
        ])
        .assert()
        .failure()
        .code(1)
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::starts_with("fatal: expr:"))
        .stderr(predicate::str::contains("EXPR-PARSE-EVENT-UNMATCHED-CLOSE"))
        .stderr(predicate::str::contains("source: posedge clk)"));
}

#[test]
fn change_broken_union_reports_expr_error() {
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
            "--on",
            "posedge clk or , clk",
            "--sample-mode",
            "native",
        ])
        .assert()
        .failure()
        .code(1)
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::starts_with("fatal: expr:"))
        .stderr(predicate::str::contains("EXPR-PARSE-EVENT-BROKEN-UNION"))
        .stderr(predicate::str::contains("source: posedge clk or , clk"));
}

#[test]
fn change_empty_iff_reports_expr_error() {
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
            "--on",
            "posedge clk iff",
            "--sample-mode",
            "native",
        ])
        .assert()
        .failure()
        .code(1)
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::starts_with("fatal: expr:"))
        .stderr(predicate::str::contains("EXPR-PARSE-EVENT-EMPTY-IFF"))
        .stderr(predicate::str::contains("source: posedge clk iff"));
}

#[test]
fn change_rich_type_iff_payload_executes() {
    let fixture = fixture_path("m2_core.vcd");
    let fixture = fixture.to_string_lossy().into_owned();

    let output = wavepeek_cmd()
        .args([
            "change",
            "--waves",
            fixture.as_str(),
            "--scope",
            "top",
            "--signals",
            "data,clk",
            "--on",
            "posedge clk iff (type(data)'(0) == 8'h00 && real'(1) > 0.5 && \"go\" == \"go\")",
            "--sample-mode",
            "native",
            "--json",
        ])
        .output()
        .expect("change should execute");

    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let json = parse_json(&output.stdout);
    assert_eq!(json["diagnostics"], json!([]));
    assert_eq!(
        json["data"],
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
fn change_triggered_iff_payload_executes() {
    let fixture = write_fixture(
        "$date\n  today\n$end\n$version\n  wavepeek-test\n$end\n$timescale 1ns $end\n$scope module top $end\n$var wire 1 ! clk $end\n$var wire 1 \" sig $end\n$var event 1 # ev $end\n$upscope $end\n$enddefinitions $end\n#0\n0!\n0\"\n#5\n1!\n1#\n#10\n0!\n#15\n1!\n",
        "change-triggered-iff.vcd",
    );
    let fixture = fixture.path().to_string_lossy().into_owned();

    let output = wavepeek_cmd()
        .args([
            "change",
            "--waves",
            fixture.as_str(),
            "--scope",
            "top",
            "--signals",
            "sig,clk",
            "--on",
            "posedge clk iff ev.triggered()",
            "--sample-mode",
            "native",
            "--json",
        ])
        .output()
        .expect("change should execute");

    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let json = parse_json(&output.stdout);
    assert_eq!(json["diagnostics"], json!([]));
    assert_eq!(
        json["data"],
        json!([
            {
                "time": "5ns",
                "sample_time": "5ns",
                "signals": [
                    {"path": "top.sig", "relative_path": "sig", "value": "1'h0"},
                    {"path": "top.clk", "relative_path": "clk", "value": "1'h1"}
                ]
            }
        ])
    );
}

#[test]
fn change_dense_empty_result_has_no_machine_diagnostic() {
    let fixture = fixture_path("m2_core.vcd");
    let fixture = fixture.to_string_lossy().into_owned();
    let output = wavepeek_cmd()
        .args([
            "change",
            "--waves",
            fixture.as_str(),
            "--from",
            "6ns",
            "--to",
            "9ns",
            "--scope",
            "top",
            "--signals",
            "data",
            "--on",
            "posedge clk",
            "--sample-mode",
            "native",
            "--json",
        ])
        .output()
        .expect("change should execute");

    assert!(output.status.success());
    let value = parse_json(&output.stdout);
    assert_eq!(value["context"]["scope"], "top");
    assert_eq!(value["diagnostics"], json!([]));
}

#[test]
fn change_empty_result_uses_normal_human_output_and_empty_machine_diagnostics() {
    let fixture = fixture_path("m2_core.vcd");
    let fixture = fixture.to_string_lossy().into_owned();

    let json_output = wavepeek_cmd()
        .args([
            "change",
            "--on",
            "*",
            "--sample-mode",
            "native",
            "--waves",
            fixture.as_str(),
            "--from",
            "6ns",
            "--to",
            "9ns",
            "--signals",
            "top.clk,top.data",
            "--row-mode",
            "sparse",
            "--json",
        ])
        .output()
        .expect("json change run should execute");
    let human_output = wavepeek_cmd()
        .args([
            "change",
            "--on",
            "*",
            "--sample-mode",
            "native",
            "--waves",
            fixture.as_str(),
            "--from",
            "6ns",
            "--to",
            "9ns",
            "--signals",
            "top.clk,top.data",
            "--row-mode",
            "sparse",
        ])
        .output()
        .expect("human change run should execute");

    assert!(json_output.status.success());
    assert!(human_output.status.success());
    assert_eq!(
        String::from_utf8_lossy(&human_output.stdout).trim(),
        "no change rows found in selected time range"
    );
    assert!(human_output.stderr.is_empty());

    let value = parse_json(&json_output.stdout);
    assert_eq!(value["data"], json!([]));
    assert_eq!(value["diagnostics"], json!([]));
}

#[test]
fn change_truncation_warning_matches_between_json_and_human_modes() {
    let fixture = fixture_path("m2_core.vcd");
    let fixture = fixture.to_string_lossy().into_owned();

    let json_output = wavepeek_cmd()
        .args([
            "change",
            "--on",
            "*",
            "--sample-mode",
            "native",
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
            "--on",
            "*",
            "--sample-mode",
            "native",
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
        "warning[WPK-W0002]: truncated output to 1 entries (use --max to increase limit)"
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
        value["diagnostics"],
        json!([{"kind": "warning", "code": "WPK-W0002", "message": "truncated output to 1 entries (use --max to increase limit)"}])
    );
}

#[test]
fn change_abs_only_affects_human_labels_not_json_payload() {
    let fixture = fixture_path("signal_recursive_depth.vcd");
    let fixture = fixture.to_string_lossy().into_owned();

    let json_default = wavepeek_cmd()
        .args([
            "change",
            "--on",
            "*",
            "--sample-mode",
            "native",
            "--waves",
            fixture.as_str(),
            "--from",
            "1ns",
            "--to",
            "10ns",
            "--scope",
            "top.cpu",
            "--signals",
            "valid,core.execute,top.cpu.valid,top.cpu.core.execute",
            "--json",
        ])
        .output()
        .expect("json default run should execute");
    let json_abs = wavepeek_cmd()
        .args([
            "change",
            "--on",
            "*",
            "--sample-mode",
            "native",
            "--waves",
            fixture.as_str(),
            "--from",
            "1ns",
            "--to",
            "10ns",
            "--scope",
            "top.cpu",
            "--signals",
            "valid,core.execute,top.cpu.valid,top.cpu.core.execute",
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
            "--on",
            "*",
            "--sample-mode",
            "native",
            "--waves",
            fixture.as_str(),
            "--from",
            "1ns",
            "--to",
            "10ns",
            "--scope",
            "top.cpu",
            "--signals",
            "valid,core.execute,top.cpu.valid,top.cpu.core.execute",
        ])
        .output()
        .expect("human default run should execute");
    let human_abs = wavepeek_cmd()
        .args([
            "change",
            "--on",
            "*",
            "--sample-mode",
            "native",
            "--waves",
            fixture.as_str(),
            "--from",
            "1ns",
            "--to",
            "10ns",
            "--scope",
            "top.cpu",
            "--signals",
            "valid,core.execute,top.cpu.valid,top.cpu.core.execute",
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
        "@5ns valid=1'h1 core.execute=1'h1 valid=1'h1 core.execute=1'h1"
    );
    assert_eq!(
        abs_lines.lines().next().expect("first line should exist"),
        "@5ns top.cpu.valid=1'h1 top.cpu.core.execute=1'h1 top.cpu.valid=1'h1 top.cpu.core.execute=1'h1"
    );
}

#[test]
fn change_preserves_duplicate_signal_order() {
    let fixture = fixture_path("m2_core.vcd");
    let fixture = fixture.to_string_lossy().into_owned();

    let output = wavepeek_cmd()
        .args([
            "change",
            "--on",
            "*",
            "--sample-mode",
            "native",
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
            "--on",
            "*",
            "--sample-mode",
            "native",
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
        value["diagnostics"],
        json!([{"kind": "warning", "code": "WPK-W0002", "message": "truncated output to 50 entries (use --max to increase limit)"}])
    );
}

#[test]
fn change_unlimited_max_disables_truncation_without_diagnostics() {
    let fixture = fixture_path("change_many_events.vcd");
    let fixture = fixture.to_string_lossy().into_owned();

    let json_output = wavepeek_cmd()
        .args([
            "change",
            "--on",
            "*",
            "--sample-mode",
            "native",
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
            "--on",
            "*",
            "--sample-mode",
            "native",
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
    assert_eq!(value["diagnostics"], json!([]));
    assert_eq!(value["summary"]["limit"], Value::Null);
    assert!(human_output.stderr.is_empty());
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
            "--on",
            "*",
            "--sample-mode",
            "native",
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
            "fatal: args: --max must be greater than 0.",
        ));
}

#[test]
fn change_rejects_legacy_internal_and_perf_change_flags() {
    let fixture = fixture_path("m2_core.vcd");
    let fixture = fixture.to_string_lossy().into_owned();

    for legacy_args in [
        vec!["--internal-change-engine", "fused"],
        vec!["--internal-change-candidates", "stream"],
        vec!["--internal-change-edge-fast-force"],
        vec!["--perf-engine", "fused"],
        vec!["--perf-candidates", "stream"],
        vec!["--perf-edge-fast-force"],
    ] {
        wavepeek_cmd()
            .args([
                "change",
                "--on",
                "*",
                "--sample-mode",
                "native",
                "--waves",
                fixture.as_str(),
                "--signals",
                "top.clk",
            ])
            .args(legacy_args)
            .assert()
            .failure()
            .code(1)
            .stdout(predicate::str::is_empty())
            .stderr(predicate::str::starts_with("fatal: args:"));
    }
}

#[test]
fn change_validates_error_paths_for_args_scope_and_signal_resolution() {
    let fixture = fixture_path("m2_core.vcd");
    let fixture = fixture.to_string_lossy().into_owned();

    wavepeek_cmd()
        .args([
            "change",
            "--on",
            "*",
            "--sample-mode",
            "native",
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
            "fatal: args: --max must be greater than 0.",
        ));

    wavepeek_cmd()
        .args([
            "change",
            "--on",
            "*",
            "--sample-mode",
            "native",
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
        .stderr(predicate::str::starts_with("fatal: args:"))
        .stderr(predicate::str::contains(
            "--from must be less than or equal to --to",
        ));

    wavepeek_cmd()
        .args([
            "change",
            "--on",
            "*",
            "--sample-mode",
            "native",
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
        .stderr(predicate::str::starts_with("fatal: args:"))
        .stderr(predicate::str::contains("requires units"));

    wavepeek_cmd()
        .args([
            "change",
            "--waves",
            fixture.as_str(),
            "--on",
            "posedge nope",
            "--signals",
            "top.clk",
            "--sample-mode",
            "native",
        ])
        .assert()
        .failure()
        .code(1)
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::starts_with("fatal: expr:"))
        .stderr(predicate::str::contains("unknown signal 'nope'"));

    wavepeek_cmd()
        .args([
            "change",
            "--on",
            "*",
            "--sample-mode",
            "native",
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
        .stderr(predicate::str::starts_with("fatal: scope:"));

    wavepeek_cmd()
        .args([
            "change",
            "--on",
            "*",
            "--sample-mode",
            "native",
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
        .stderr(predicate::str::starts_with("fatal: args:"))
        .stderr(predicate::str::contains(
            "cannot be represented exactly in dump precision",
        ));
}

#[test]
fn change_decimal_time_token_is_rejected_as_args_error() {
    let fixture = fixture_path("m2_core.vcd");
    let fixture = fixture.to_string_lossy().into_owned();

    wavepeek_cmd()
        .args([
            "change",
            "--on",
            "*",
            "--sample-mode",
            "native",
            "--waves",
            fixture.as_str(),
            "--from",
            "1.5ns",
            "--signals",
            "top.clk",
        ])
        .assert()
        .failure()
        .code(1)
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::starts_with("fatal: args:"))
        .stderr(predicate::str::contains("invalid time token '1.5ns'"))
        .stderr(predicate::str::contains("See 'wavepeek change --help'."));
}

#[test]
fn change_empty_on_expression_reports_expr_error() {
    let fixture = fixture_path("m2_core.vcd");
    let fixture = fixture.to_string_lossy().into_owned();

    wavepeek_cmd()
        .args([
            "change",
            "--waves",
            fixture.as_str(),
            "--signals",
            "top.clk",
            "--on",
            "",
            "--sample-mode",
            "native",
        ])
        .assert()
        .failure()
        .code(1)
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::starts_with("fatal: expr:"))
        .stderr(predicate::str::contains("EXPR-PARSE-EVENT-EMPTY"));
}

#[test]
fn change_out_of_range_from_time_is_args_error_with_bounds() {
    let fixture = fixture_path("m2_core.vcd");
    let fixture = fixture.to_string_lossy().into_owned();

    wavepeek_cmd()
        .args([
            "change",
            "--on",
            "*",
            "--sample-mode",
            "native",
            "--waves",
            fixture.as_str(),
            "--from",
            "11ns",
            "--signals",
            "top.clk",
        ])
        .assert()
        .failure()
        .code(1)
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::starts_with("fatal: args:"))
        .stderr(predicate::str::contains(
            "time '11ns' for --from is outside dump bounds [0ns, 10ns]",
        ))
        .stderr(predicate::str::contains("See 'wavepeek change --help'."));
}

#[test]
fn change_misaligned_time_includes_help_hint() {
    let fixture = fixture_path("m2_core.vcd");
    let fixture = fixture.to_string_lossy().into_owned();

    wavepeek_cmd()
        .args([
            "change",
            "--on",
            "*",
            "--sample-mode",
            "native",
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
        .stderr(predicate::str::starts_with("fatal: args:"))
        .stderr(predicate::str::contains(
            "cannot be represented exactly in dump precision",
        ))
        .stderr(predicate::str::contains("See 'wavepeek change --help'."));
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
            "--on",
            "posedge top.nope",
            "--sample-mode",
            "native",
        ])
        .assert()
        .failure()
        .code(1)
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::starts_with("fatal: expr:"))
        .stderr(predicate::str::contains("unknown signal 'top.nope'"));

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
            "--on",
            "posedge vlaid",
            "--sample-mode",
            "native",
        ])
        .assert()
        .failure()
        .code(1)
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::starts_with("fatal: expr:"))
        .stderr(predicate::str::contains("unknown signal 'vlaid'"))
        .stderr(predicate::str::contains(
            "signal 'vlaid' not found under scope 'top'\nclosest query names:\n  cpu.valid",
        ));
}

#[test]
fn change_scope_accepts_canonical_signal_and_trigger_paths() {
    let fixture = fixture_path("change_scope_ambiguous.vcd");
    let fixture = fixture.to_string_lossy().into_owned();

    let output = wavepeek_cmd()
        .args([
            "change",
            "--waves",
            fixture.as_str(),
            "--scope",
            "top",
            "--signals",
            "top.clk",
            "--on",
            "posedge top.clk",
            "--sample-mode",
            "native",
            "--json",
        ])
        .output()
        .expect("change should execute");

    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    assert_eq!(
        parse_json(&output.stdout)["data"],
        json!([{
            "time": "5ns",
            "sample_time": "5ns",
            "signals": [{"path": "top.clk", "relative_path": "clk", "value": "1'h1"}]
        }])
    );
}

#[test]
fn change_runtime_manifest_cases_pass() {
    let positive =
        common::command_cases::load_positive_manifest("command_runtime_positive_manifest.json");
    for case in &positive.cases {
        if case.command == common::command_cases::CommandCaseCommand::Change {
            common::command_cases::assert_positive_case(case);
        }
    }

    let negative =
        common::command_cases::load_negative_manifest("command_runtime_negative_manifest.json");
    for case in &negative.cases {
        if case.command == common::command_cases::CommandCaseCommand::Change {
            common::command_cases::assert_negative_case(case);
        }
    }
}

const FLAT_PROJECTION_VCD: &str = concat!(
    "$timescale 1ns $end\n",
    "$scope module top $end\n",
    "$var wire 8 ! data $end\n",
    "$upscope $end\n",
    "$enddefinitions $end\n",
    "#0\nb00000000 !\n",
    "#5\nb00001111 !\n",
    "#10\nb11111111 !\n",
);

#[test]
fn change_compares_projected_values_for_wildcard_sparse_and_delta_rows() {
    let fixture = write_fixture(FLAT_PROJECTION_VCD, "change-flat-projection.vcd");
    let run = || {
        let output = wavepeek_cmd()
            .env("DEBUG", "1")
            .args([
                "change",
                "--waves",
                fixture.path().to_str().unwrap(),
                "--scope",
                "top",
                "--signals",
                "data[7:4],data[5:2],data,data[7:4]",
                "--on",
                "*",
                "--sample-mode",
                "native",
                "--row-mode",
                "sparse",
                "--row-values",
                "delta",
                "--json",
            ])
            .output()
            .expect("change should execute");
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        parse_json(&output.stdout)["data"].clone()
    };

    let expected = json!([
        {
            "time": "5ns",
            "sample_time": "5ns",
            "signals": [
                {"path": "top.data[7:4]", "relative_path": "data[7:4]", "value": "4'h0"},
                {"path": "top.data[5:2]", "relative_path": "data[5:2]", "value": "4'h3"},
                {"path": "top.data", "relative_path": "data", "value": "8'h0f"},
                {"path": "top.data[7:4]", "relative_path": "data[7:4]", "value": "4'h0"},
            ]
        },
        {
            "time": "10ns",
            "sample_time": "10ns",
            "signals": [
                {"path": "top.data[7:4]", "relative_path": "data[7:4]", "value": "4'hf"},
                {"path": "top.data[5:2]", "relative_path": "data[5:2]", "value": "4'hf"},
                {"path": "top.data", "relative_path": "data", "value": "8'hff"},
                {"path": "top.data[7:4]", "relative_path": "data[7:4]", "value": "4'hf"},
            ]
        }
    ]);
    assert_eq!(run(), expected);
}

#[test]
fn change_wildcard_ignores_changes_outside_the_only_projection() {
    let fixture = write_fixture(FLAT_PROJECTION_VCD, "change-projection-wildcard.vcd");
    let output = wavepeek_cmd()
        .env("DEBUG", "1")
        .args([
            "change",
            "--waves",
            fixture.path().to_str().unwrap(),
            "--scope",
            "top",
            "--signals",
            "data[7:4]",
            "--on",
            "*",
            "--sample-mode",
            "native",
            "--json",
        ])
        .output()
        .expect("change should execute");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let data = parse_json(&output.stdout)["data"].clone();
    assert_eq!(data.as_array().unwrap().len(), 1);
    assert_eq!(data[0]["time"], "10ns");
}

#[test]
fn change_rejects_removed_tuning_controls_even_in_debug_mode() {
    for flag in [
        "--tune-engine",
        "--tune-candidates",
        "--tune-edge-fast-force",
    ] {
        wavepeek_cmd()
            .env("DEBUG", "1")
            .args([
                "change",
                "--waves",
                "missing.vcd",
                "--signals",
                "top.clk",
                "--on",
                "posedge top.clk",
                flag,
            ])
            .assert()
            .failure()
            .code(1)
            .stderr(predicate::str::contains("unexpected argument"));
    }
}
