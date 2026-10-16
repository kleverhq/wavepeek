#![cfg(feature = "shm")]

use std::path::PathBuf;

use serde_json::Value;

mod common;
use common::{fixture_path, wavepeek_cmd};

fn shm_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/shm")
        .join(format!("{name}.shm"))
}

fn run_json(path: &std::path::Path, args: &[&str]) -> Value {
    let output = wavepeek_cmd()
        .args(args)
        .arg("--waves")
        .arg(path)
        .arg("--json")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{args:?}: {} {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        output.stderr.is_empty(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).unwrap()
}

fn assert_parity(name: &str, args: &[&str]) {
    let vcd = run_json(&fixture_path(&format!("{name}.vcd")), args);
    let shm = run_json(&shm_path(name), args);
    assert_eq!(shm["data"], vcd["data"], "{name}: {args:?}");
    assert_eq!(shm["diagnostics"], vcd["diagnostics"], "{name}: {args:?}");
}

#[test]
fn metadata_and_hierarchy_match_vcd() {
    for args in [
        vec!["info"],
        vec!["scope", "--max-depth", "0"],
        vec!["scope", "--max-depth", "3"],
    ] {
        assert_parity("m2_core", &args);
    }
    let args = ["signal", "--scope", "top", "--recursive"];
    let mut vcd = run_json(&fixture_path("m2_core.vcd"), &args);
    let mut shm = run_json(&shm_path("m2_core"), &args);
    // VCD conversion retains values and paths, but SHM exposes generic bit kinds.
    for row in shm["data"].as_array().unwrap() {
        assert_eq!(row["kind"], "bit_vector");
    }
    for result in [&mut vcd, &mut shm] {
        for row in result["data"].as_array_mut().unwrap() {
            row.as_object_mut().unwrap().remove("kind");
        }
    }
    assert_eq!(shm["data"], vcd["data"]);
}

#[test]
fn projections_duplicates_unknowns_and_boundary_samples_match_vcd() {
    for at in ["0ns", "4ns", "5ns", "10ns"] {
        assert_parity(
            "value_vectors",
            &[
                "value",
                "--scope",
                "top",
                "--signals",
                "data[7:4],nibble,status,asc,data[7:4]",
                "--at",
                at,
            ],
        );
    }
}

#[test]
fn same_tick_final_values_and_triggers_match_vcd() {
    assert_parity(
        "same_time_updates",
        &["value", "--signals", "top.glitch,top.bus", "--at", "5ns"],
    );
    assert_parity(
        "same_time_updates",
        &[
            "change",
            "--signals",
            "top.glitch,top.bus",
            "--on",
            "posedge top.clk",
        ],
    );
    assert_parity(
        "same_time_updates",
        &[
            "property",
            "--on",
            "posedge top.clk",
            "--eval",
            "top.bus == 2'b10",
        ],
    );
}

#[test]
fn changes_properties_and_generic_extraction_match_vcd() {
    for args in [
        vec![
            "change",
            "--scope",
            "top",
            "--signals",
            "data,state",
            "--on",
            "*",
            "--sample-mode",
            "native",
        ],
        vec![
            "change",
            "--scope",
            "top",
            "--signals",
            "data",
            "--on",
            "posedge clk",
            "--from",
            "7ns",
            "--to",
            "25ns",
        ],
        vec![
            "property",
            "--scope",
            "top",
            "--on",
            "posedge clk",
            "--eval",
            "valid && ready",
        ],
        vec![
            "extract",
            "generic",
            "--scope",
            "top",
            "--on",
            "posedge clk",
            "--when",
            "valid",
            "--payload",
            "data,state,data[7:4]",
        ],
    ] {
        assert_parity("change_property_core", &args);
    }
}

#[test]
fn real_expressions_match_vcd() {
    assert_parity(
        "change_property_real_output",
        &[
            "property",
            "--on",
            "posedge top.clk",
            "--eval",
            "top.temp > 1.0",
        ],
    );
}

#[test]
fn escaped_names_and_array_paths_match_vcd() {
    assert_parity(
        "rich_names",
        &[
            "value",
            "--signals",
            "top.\\block.dot.\\data.bus,top.memory.[0]",
            "--at",
            "5ns",
        ],
    );
}

#[test]
fn native_string_real_and_event_values_are_available() {
    let path = shm_path("shm_rich");
    for expression in ["top.message == \"stop\"", "top.temperature > 1.0"] {
        let result = run_json(
            &path,
            &[
                "property",
                "--on",
                "top.tick",
                "--sample-mode",
                "native",
                "--eval",
                expression,
            ],
        );
        assert_eq!(result["data"].as_array().unwrap().len(), 1);
        assert_eq!(result["data"][0]["time"], "5ns");
    }
}

#[test]
fn human_and_jsonl_value_output_match_vcd() {
    for mode in [None, Some("--jsonl")] {
        let mut outputs = Vec::new();
        for path in [fixture_path("m2_core.vcd"), shm_path("m2_core")] {
            let mut command = wavepeek_cmd();
            command
                .args(["value", "--signals", "top.data", "--at", "10ns", "--waves"])
                .arg(path);
            if let Some(mode) = mode {
                command.arg(mode);
            }
            let output = command.output().unwrap();
            assert!(output.status.success());
            assert!(output.stderr.is_empty());
            outputs.push(output.stdout);
        }
        assert_eq!(outputs[0], outputs[1]);
    }
}

#[test]
fn raw_event_occurrences_match_vcd() {
    assert_parity(
        "change_property_events",
        &[
            "property",
            "--on",
            "top.tick",
            "--eval",
            "1",
            "--sample-mode",
            "native",
        ],
    );
}

#[test]
fn protocol_extractors_match_vcd() {
    for (name, args) in [
        (
            "extract_apb3",
            vec![
                "extract",
                "apb",
                "--scope",
                "top",
                "--profile",
                "apb3",
                "--include",
                "^apb3_",
            ],
        ),
        (
            "extract_ahb_lite",
            vec![
                "extract",
                "ahb",
                "--scope",
                "top",
                "--profile",
                "ahb-lite",
                "--map",
                "hclk=clk",
                "--include",
                "^ahb_lite_",
            ],
        ),
        (
            "extract_atb",
            vec!["extract", "atb", "--scope", "top", "--include", "^trace_"],
        ),
        (
            "extract_axistream",
            vec![
                "extract",
                "axistream",
                "--scope",
                "top",
                "--map",
                "aclk=clk",
                "--map",
                "aresetn=rst_n",
                "--include",
                "^s_axis_",
            ],
        ),
        (
            "extract_axi5_lite",
            vec![
                "extract",
                "axi",
                "--scope",
                "top",
                "--profile",
                "axi5-lite",
                "--map",
                "aclk=clk",
                "--include",
                "^axi5_lite_",
            ],
        ),
    ] {
        assert_parity(name, &args);
    }
}

#[test]
fn directory_detection_does_not_require_a_suffix() {
    let directory = tempfile::tempdir().unwrap();
    let renamed = directory.path().join("recording");
    std::os::unix::fs::symlink(shm_path("m2_core"), &renamed).unwrap();
    assert_eq!(
        run_json(&renamed, &["info"]),
        run_json(&shm_path("m2_core"), &["info"])
    );
}

#[test]
fn incomplete_databases_and_individual_components_fail_cleanly() {
    let directory = tempfile::tempdir().unwrap();
    let incomplete = directory.path().join("incomplete.shm");
    std::fs::create_dir(&incomplete).unwrap();
    let component = std::fs::read_dir(shm_path("m2_core"))
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .find(|path| path.extension().is_some_and(|s| s == "trn"))
        .unwrap();
    for path in [incomplete, component] {
        for args in [
            vec!["info"],
            vec!["value", "--signals", "top.clk", "--at", "0ns"],
        ] {
            let output = wavepeek_cmd()
                .args(args)
                .arg("--waves")
                .arg(path.clone())
                .arg("--json")
                .output()
                .unwrap();
            assert_eq!(output.status.code(), Some(2));
            assert!(output.stderr.is_empty());
            let value: Value = serde_json::from_slice(&output.stdout).unwrap();
            assert_eq!(value["type"], "fatal");
            assert_eq!(value["code"], "WPK-F0002");
        }
    }
}
