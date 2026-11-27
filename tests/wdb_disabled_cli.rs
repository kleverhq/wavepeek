#![cfg(not(feature = "wdb"))]

use serde_json::Value;

mod common;
use common::{fixture_path, wavepeek_cmd};

#[test]
fn wdb_disabled_diagnostic_covers_metadata_and_waveform_open() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("recording");
    std::fs::write(&path, b"Xilinx WAVE DATABASE ").unwrap();

    for command in ["info", "scope"] {
        let output = wavepeek_cmd()
            .args([command, "--waves"])
            .arg(&path)
            .arg("--json")
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(2), "{command}");
        assert!(output.stderr.is_empty(), "{command}");
        let value: Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(value["code"], "WPK-F0002", "{command}");
        let message = value["message"].as_str().unwrap();
        assert!(
            message.contains("no available backend for Wdb"),
            "{message}"
        );
        assert!(message.contains("--features wdb"), "{message}");
        assert!(message.contains("XILINX_VIVADO"), "{message}");
    }
}

#[test]
fn wdb_suffix_preserves_io_errors_and_content_detection() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("recording.wdb");
    let output = wavepeek_cmd()
        .args(["info", "--waves"])
        .arg(&path)
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(2));
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("I/O error"), "{stderr}");

    std::fs::copy(fixture_path("m2_core.vcd"), &path).unwrap();
    let output = wavepeek_cmd()
        .args(["info", "--waves"])
        .arg(&path)
        .arg("--json")
        .output()
        .unwrap();
    assert!(output.status.success(), "{:?}", output);
    let value: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value["data"][0]["time_end"], "10ns");
}
