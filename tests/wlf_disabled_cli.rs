#![cfg(not(feature = "wlf"))]

use serde_json::Value;

mod common;
use common::{fixture_path, wavepeek_cmd};

#[test]
fn missing_paths_and_content_detection_keep_priority_over_the_wlf_hint() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("test.WLF");
    let run = |command: &str| {
        wavepeek_cmd()
            .args([command, "--waves"])
            .arg(&path)
            .arg("--json")
            .output()
            .unwrap()
    };

    let output = run("info");
    assert_eq!(output.status.code(), Some(2));
    let payload: Value = serde_json::from_slice(&output.stdout).unwrap();
    let message = payload["message"].as_str().unwrap();
    assert!(message.contains("I/O error"));
    assert!(!message.contains("--features wlf"));

    std::fs::write(&path, b"waveform").unwrap();
    for command in ["info", "scope"] {
        let output = run(command);
        assert_eq!(output.status.code(), Some(2));
        assert!(output.stderr.is_empty());
        let payload: Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(payload["code"], "WPK-F0002");
        let message = payload["message"].as_str().unwrap();
        assert!(message.contains("--features wlf") && message.contains("QUESTA_HOME"));
    }

    std::fs::copy(fixture_path("m2_core.vcd"), &path).unwrap();
    assert!(run("info").status.success());
}
