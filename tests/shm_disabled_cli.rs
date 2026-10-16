#![cfg(not(feature = "shm"))]

use serde_json::Value;

mod common;
use common::{fixture_path, wavepeek_cmd};

#[test]
fn waveform_commands_explain_disabled_shm_support() {
    let directory = tempfile::Builder::new().suffix(".SHM").tempdir().unwrap();
    for args in [
        vec!["info"],
        vec!["scope"],
        vec!["signal", "--scope", "top"],
        vec!["value", "--signals", "top.clk", "--at", "0ns"],
        vec!["change", "--signals", "top.clk", "--on", "*"],
        vec!["property", "--on", "*", "--eval", "1"],
        vec![
            "extract",
            "generic",
            "--on",
            "posedge top.clk",
            "--when",
            "1",
            "--payload",
            "top.data",
        ],
    ] {
        let output = wavepeek_cmd()
            .args(&args)
            .arg("--waves")
            .arg(directory.path())
            .arg("--json")
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(2), "{args:?}");
        assert!(output.stderr.is_empty());
        let value: Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(value["code"], "WPK-F0002");
        assert!(
            value["message"]
                .as_str()
                .unwrap()
                .contains("--features shm")
        );
        assert!(value["message"].as_str().unwrap().contains("XCELIUM_HOME"));
    }
}

#[test]
fn missing_paths_keep_io_errors_and_content_detection_precedes_suffix() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("missing.shm");
    let output = wavepeek_cmd()
        .args(["info", "--waves"])
        .arg(&path)
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(2));
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("cannot open"));
    assert!(!stderr.contains("--features shm"));
    std::fs::copy(fixture_path("m2_core.vcd"), &path).unwrap();
    let output = wavepeek_cmd()
        .args(["info", "--waves"])
        .arg(&path)
        .output()
        .unwrap();
    assert!(output.status.success());
    assert!(output.stderr.is_empty());
}
