#![cfg(feature = "fsdb")]

use std::path::PathBuf;
use std::process::{Command, Stdio};

use serde_json::Value;
use tempfile::TempDir;

mod common;
use common::{assert_no_wavepeek_diagnostic, fixture_path, wavepeek_cmd};

fn convert(name: &str) -> (TempDir, PathBuf) {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("waveform.fsdb");
    let output = Command::new("vcd2fsdb")
        .current_dir(directory.path())
        .arg(fixture_path(name))
        .arg("-o")
        .arg(&path)
        .output()
        .unwrap();
    assert!(output.status.success(), "{output:?}");
    (directory, path)
}

#[test]
fn fsdb_machine_stdout_covers_metadata_read_limit_and_close() {
    let (_directory, path) = convert("change_property_core.vcd");
    let cases: &[&[&str]] = &[
        &["info"],
        &["scope"],
        &["signal", "--scope", "top"],
        &["value", "--signals", "top.data", "--at", "5ns"],
        &[
            "change",
            "--signals",
            "top.data",
            "--on",
            "*",
            "--sample-mode",
            "native",
            "--max",
            "1",
        ],
        &[
            "property",
            "--on",
            "posedge top.clk",
            "--eval",
            "top.valid",
            "--sample-mode",
            "native",
            "--max",
            "1",
        ],
        &[
            "extract",
            "generic",
            "--on",
            "posedge top.clk",
            "--when",
            "1",
            "--payload",
            "top.data",
            "--max",
            "1",
        ],
    ];
    for args in cases {
        for mode in ["--json", "--jsonl"] {
            let output = wavepeek_cmd()
                .args(*args)
                .arg("--waves")
                .arg(&path)
                .arg(mode)
                .output()
                .unwrap();
            assert!(output.status.success(), "{args:?} {mode}: {output:?}");
            assert_no_wavepeek_diagnostic(&output.stderr);
            if mode == "--json" {
                let value: Value = serde_json::from_slice(&output.stdout).unwrap();
                assert_eq!(value["type"], "result");
                assert!(!value["data"].as_array().unwrap().is_empty(), "{args:?}");
                if args.contains(&"--max") {
                    assert_eq!(value["summary"]["returned"], 1);
                    assert_eq!(value["summary"]["complete"], false);
                }
            } else {
                let text = std::str::from_utf8(&output.stdout).unwrap();
                assert!(text.ends_with('\n'));
                let records: Vec<Value> = text
                    .lines()
                    .map(|line| serde_json::from_str(line).unwrap())
                    .collect();
                assert_eq!(records.first().unwrap()["type"], "begin");
                assert_eq!(records.last().unwrap()["type"], "end");
                for (seq, record) in records.iter().enumerate() {
                    assert_eq!(record["seq"], seq);
                    assert_ne!(record["type"], "fatal");
                }
                if args.contains(&"--max") {
                    assert_eq!(records.last().unwrap()["summary"]["returned"], 1);
                    assert_eq!(records.last().unwrap()["summary"]["complete"], false);
                }
            }
        }
    }
}

#[test]
fn fsdb_machine_failures_preserve_fatal_framing_and_exit_codes() {
    let (directory, path) = convert("change_property_core.vcd");
    let truncated = directory.path().join("truncated.fsdb");
    std::fs::copy(&path, &truncated).unwrap();
    let file = std::fs::OpenOptions::new()
        .write(true)
        .open(&truncated)
        .unwrap();
    file.set_len((file.metadata().unwrap().len() / 3).max(1))
        .unwrap();
    let (_real_directory, real) = convert("change_property_real_output.vcd");
    for (path, args, code, exit) in [
        (&truncated, vec!["info"], "WPK-F0002", 2),
        (
            &truncated,
            vec!["value", "--signals", "top.data", "--at", "0ns"],
            "WPK-F0002",
            2,
        ),
        (
            &real,
            vec!["value", "--signals", "top.temp", "--at", "0ns"],
            "WPK-F0004",
            1,
        ),
    ] {
        for mode in ["--json", "--jsonl"] {
            let output = wavepeek_cmd()
                .args(&args)
                .arg("--waves")
                .arg(path)
                .arg(mode)
                .output()
                .unwrap();
            assert_eq!(output.status.code(), Some(exit), "{output:?}");
            assert_no_wavepeek_diagnostic(&output.stderr);
            let fatal: Value = serde_json::from_slice(&output.stdout).unwrap();
            assert_eq!(fatal["type"], "fatal");
            assert_eq!(fatal["code"], code);
            assert!(fatal.get("command").is_none());
            if mode == "--jsonl" {
                assert_eq!(fatal["seq"], 0);
            } else {
                assert!(fatal.get("seq").is_none());
            }
        }
    }
}

#[test]
fn fsdb_closed_stdout_exits_successfully() {
    let (_directory, path) = convert("change_property_core.vcd");
    let mut child = wavepeek_cmd()
        .args([
            "change",
            "--signals",
            "top.data",
            "--on",
            "*",
            "--sample-mode",
            "native",
            "--max",
            "unlimited",
            "--jsonl",
            "--waves",
        ])
        .arg(path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    drop(child.stdout.take());
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success(), "{output:?}");
    assert_no_wavepeek_diagnostic(&output.stderr);
}
