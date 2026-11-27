#![cfg(not(feature = "vwdb"))]

use serde_json::Value;

mod common;
use common::wavepeek_cmd;

#[test]
fn layout_and_suffix_report_disabled_vwdb() {
    let root = tempfile::tempdir().unwrap();
    let legacy = root.path().join("recording.SHM");
    std::fs::create_dir_all(legacy.join("dsn_0")).unwrap();
    std::fs::write(legacy.join("vwdb_info.json"), b"{}").unwrap();
    let suffixed = root.path().join("recording.VWDB");
    std::fs::create_dir(&suffixed).unwrap();

    for path in [&legacy, &suffixed] {
        for command in ["info", "scope"] {
            let output = wavepeek_cmd()
                .args([command, "--json", "--waves"])
                .arg(path)
                .output()
                .unwrap();
            assert_eq!(output.status.code(), Some(2));
            assert!(output.stderr.is_empty());
            let error: Value = serde_json::from_slice(&output.stdout).unwrap();
            assert_eq!(error["code"], "WPK-F0002");
            let message = error["message"].as_str().unwrap();
            assert!(message.contains("--features vwdb"), "{message}");
            assert!(message.contains("XCELIUM_HOME"), "{message}");
        }
    }
}

#[test]
fn missing_path_and_open_format_keep_their_own_behavior() {
    let root = tempfile::tempdir().unwrap();
    let input = root.path().join("recording.vwdb");
    let info = || {
        wavepeek_cmd()
            .args(["info", "--json", "--waves"])
            .arg(&input)
            .output()
            .unwrap()
    };
    let output = info();
    assert_eq!(output.status.code(), Some(2));
    let error: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(error["code"], "WPK-F0002");
    assert!(error["message"].as_str().unwrap().contains("I/O error"));

    std::fs::write(
        &input,
        b"$timescale 1ns $end\n$var wire 1 ! ready $end\n$enddefinitions $end\n#0\n0!\n#5\n1!\n",
    )
    .unwrap();
    let output = info();
    assert!(output.status.success(), "{output:?}");
    let result: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(result["data"][0]["time_end"], "5ns");
}
