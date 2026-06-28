use assert_cmd::prelude::*;
use std::fs;
use std::path::PathBuf;

mod common;
use common::wavepeek_cmd;

fn canonical_skill_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("docs")
        .join("skills")
        .join("wavepeek.md")
}

fn successful_stdout(args: &[&str]) -> Vec<u8> {
    let mut command = wavepeek_cmd();
    let assert = command.args(args).assert().success();
    let output = assert.get_output();
    assert!(
        output.stderr.is_empty(),
        "expected empty stderr for args {:?}, got: {}",
        args,
        String::from_utf8_lossy(&output.stderr)
    );
    output.stdout.clone()
}

#[test]
fn skill_prints_packaged_skill_markdown() {
    let expected = fs::read(canonical_skill_path()).expect("canonical skill should be readable");
    let actual = successful_stdout(&["skill"]);

    assert_eq!(actual, expected);
}

#[test]
fn skill_json_mode_is_an_argument_error() {
    let output = wavepeek_cmd()
        .args(["skill", "--json"])
        .output()
        .expect("skill command should execute");

    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.starts_with("fatal: args:"));
    assert!(stderr.contains("unexpected argument '--json'"));
    assert!(stderr.contains("See 'wavepeek skill --help'."));
}

#[test]
fn packaged_skill_guidance_matches_current_runtime_capabilities() {
    let packaged =
        fs::read_to_string(canonical_skill_path()).expect("packaged skill should be readable");

    assert!(packaged.contains("wavepeek help <command-path...>"));
    assert!(packaged.contains(".fsdb"));
    assert!(packaged.contains("wavepeek docs show commands/extract-generic"));
    assert!(packaged.contains("Event/transaction rows, handshakes, beats, and counts with payload values: `extract generic`."));
    assert!(packaged.contains("It does not decode protocol-specific transactions"));
    assert!(!packaged.contains("protocol transaction enumeration"));
    assert!(!packaged.contains("Event/transaction enumeration and counting: `property --capture match`, then `value --at <sample_time>`"));
    assert!(!packaged.contains("parsed but not executed in `change`"));
    assert!(!packaged.contains("parse-level only; runtime execution is not implemented"));
}
