mod common;

use serde_json::Value;

use common::{fixture_path, wavepeek_cmd};

fn run(args: &[&str]) -> std::process::Output {
    wavepeek_cmd()
        .args(args)
        .output()
        .expect("wavepeek should execute")
}

fn json_stdout(output: &std::process::Output) -> Value {
    serde_json::from_slice(&output.stdout).expect("stdout should contain one JSON value")
}

#[test]
fn json_serializes_parse_and_file_failures_without_stderr() {
    let parse = run(&["--json", "unknown"]);
    assert_eq!(parse.status.code(), Some(1));
    assert!(parse.stderr.is_empty());
    let fatal = json_stdout(&parse);
    assert_eq!(fatal["type"], "fatal");
    assert_eq!(fatal["code"], "WPK-F0001");
    assert!(fatal["message"].as_str().unwrap().contains("unrecognized"));
    assert!(fatal.get("seq").is_none());

    let file = run(&[
        "info",
        "--waves",
        "definitely-missing-wavepeek.vcd",
        "--json",
    ]);
    assert_eq!(file.status.code(), Some(2));
    assert!(file.stderr.is_empty());
    let fatal = json_stdout(&file);
    assert_eq!(fatal["type"], "fatal");
    assert_eq!(fatal["code"], "WPK-F0002");
    assert!(fatal["message"].as_str().unwrap().contains("I/O error"));
}

#[test]
fn selector_after_unrelated_or_missing_value_option_still_selects_machine_output() {
    let output = run(&["info", "--profile", "--json"]);
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stderr.is_empty());
    let fatal = json_stdout(&output);
    assert_eq!(fatal["type"], "fatal");
    assert_eq!(fatal["code"], "WPK-F0001");
    assert!(fatal["message"].as_str().unwrap().contains("--profile"));

    let output = run(&["info", "--waves", "--json"]);
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stderr.is_empty());
    let fatal = json_stdout(&output);
    assert_eq!(fatal["type"], "fatal");
    assert_eq!(fatal["code"], "WPK-F0001");
    assert!(fatal["message"].as_str().unwrap().contains("--waves"));
}

#[test]
fn jsonl_serializes_pre_begin_failure_at_sequence_zero() {
    let output = run(&[
        "--jsonl",
        "info",
        "--waves",
        "definitely-missing-wavepeek.vcd",
    ]);
    assert_eq!(output.status.code(), Some(2));
    assert!(output.stderr.is_empty());
    let fatal = json_stdout(&output);
    assert_eq!(fatal["type"], "fatal");
    assert_eq!(fatal["seq"], 0);
    assert_eq!(fatal["code"], "WPK-F0002");
    assert!(fatal.get("command").is_none());
    assert_eq!(String::from_utf8_lossy(&output.stdout).lines().count(), 1);
}

#[test]
fn jsonl_appends_runtime_fatal_after_begin_without_end() {
    let output = wavepeek_cmd()
        .env("DEBUG", "1")
        .args([
            "change",
            "--waves",
            "tests/fixtures/hand/change_property_events.vcd",
            "--signals",
            "top.armed",
            "--on",
            "*",
            "--sample-mode",
            "native",
            "--tune-engine",
            "baseline",
            "--tune-candidates",
            "stream",
            "--jsonl",
        ])
        .output()
        .expect("wavepeek should execute");
    assert_eq!(output.status.code(), Some(1));
    let records = String::from_utf8(output.stdout)
        .expect("stdout should be UTF-8")
        .lines()
        .map(|line| serde_json::from_str::<Value>(line).expect("record should be JSON"))
        .collect::<Vec<_>>();
    assert_eq!(records.len(), 2);
    assert_eq!(records[0]["type"], "begin");
    assert_eq!(records[0]["seq"], 0);
    assert_eq!(records[1]["type"], "fatal");
    assert_eq!(records[1]["seq"], 1);
    assert_eq!(records[1]["code"], "WPK-F0006");
    assert!(records[1].get("command").is_none());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr
            .lines()
            .all(|line| serde_json::from_str::<Value>(line).is_ok())
    );
    assert!(!stderr.contains("fatal:"));
}

#[test]
fn mixed_selectors_use_jsonl_for_the_conflict_fatal() {
    let output = run(&["info", "--json", "--jsonl"]);
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stderr.is_empty());
    let fatal = json_stdout(&output);
    assert_eq!(fatal["type"], "fatal");
    assert_eq!(fatal["seq"], 0);
    assert_eq!(fatal["code"], "WPK-F0001");
    assert!(
        fatal["message"]
            .as_str()
            .unwrap()
            .contains("cannot be used with")
    );
}

#[test]
fn machine_selector_without_complete_command_is_a_machine_fatal() {
    for args in [&["--json"][..], &["extract", "--json"][..]] {
        let output = run(args);
        assert_eq!(output.status.code(), Some(1));
        assert!(output.stderr.is_empty());
        let fatal = json_stdout(&output);
        assert_eq!(fatal["type"], "fatal");
        assert_eq!(fatal["code"], "WPK-F0001");
    }

    let output = run(&["extract", "--jsonl"]);
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stderr.is_empty());
    let fatal = json_stdout(&output);
    assert_eq!(fatal["type"], "fatal");
    assert_eq!(fatal["seq"], 0);
    assert_eq!(fatal["code"], "WPK-F0001");
}

#[test]
fn unsupported_helper_mode_uses_the_requested_machine_format() {
    let output = run(&["--jsonl", "skill", "unused"]);
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stderr.is_empty());
    let fatal = json_stdout(&output);
    assert_eq!(fatal["type"], "fatal");
    assert_eq!(fatal["seq"], 0);
    assert_eq!(fatal["code"], "WPK-F0001");
}

#[test]
fn help_and_version_ignore_machine_selectors_when_successful() {
    for args in [
        &["--json", "--help"][..],
        &["info", "--jsonl", "--help"][..],
        &["--json", "--version"][..],
    ] {
        let output = run(args);
        assert!(output.status.success());
        assert!(output.stderr.is_empty());
        assert!(serde_json::from_slice::<Value>(&output.stdout).is_err());
    }

    let invalid = run(&["info", "--version", "--json"]);
    assert_eq!(invalid.status.code(), Some(1));
    assert!(invalid.stderr.is_empty());
    assert_eq!(json_stdout(&invalid)["code"], "WPK-F0001");
}

#[test]
fn reachable_query_failures_use_stable_fatal_codes() {
    let fixture = fixture_path("m2_core.vcd");
    let fixture = fixture.to_string_lossy().into_owned();
    let cases = [
        (
            vec![
                "value",
                "--waves",
                fixture.as_str(),
                "--at",
                "10ns",
                "--scope",
                "missing",
                "--signals",
                "clk",
                "--json",
            ],
            "WPK-F0003",
        ),
        (
            vec![
                "value",
                "--waves",
                fixture.as_str(),
                "--at",
                "10ns",
                "--signals",
                "top.missing",
                "--json",
            ],
            "WPK-F0004",
        ),
        (
            vec![
                "property",
                "--waves",
                fixture.as_str(),
                "--on",
                "posedge top.clk",
                "--eval",
                "(",
                "--json",
            ],
            "WPK-F0005",
        ),
    ];

    for (args, code) in cases {
        let output = run(&args);
        assert_eq!(output.status.code(), Some(1));
        assert!(output.stderr.is_empty());
        let fatal = json_stdout(&output);
        assert_eq!(fatal["type"], "fatal");
        assert_eq!(fatal["code"], code);
        assert!(fatal.get("seq").is_none());
    }
}

#[test]
fn selector_like_values_and_tokens_after_separator_stay_human() {
    for args in [
        &[
            "property",
            "--waves",
            "missing.vcd",
            "--on",
            "top.valid",
            "--eval=--json",
        ][..],
        &[
            "property",
            "--waves",
            "missing.vcd",
            "--on",
            "top.valid",
            "--eval",
            "--json",
        ][..],
        &["info", "--", "--json"][..],
    ] {
        let output = run(args);
        assert!(!output.status.success());
        assert!(output.stdout.is_empty());
        assert!(String::from_utf8_lossy(&output.stderr).contains("fatal:"));
    }
}

#[test]
fn human_and_debug_failure_channels_are_preserved() {
    let human = run(&["info", "--waves", "definitely-missing-wavepeek.vcd"]);
    assert_eq!(human.status.code(), Some(2));
    assert!(human.stdout.is_empty());
    assert!(String::from_utf8_lossy(&human.stderr).contains("fatal: file:"));

    let debug = wavepeek_cmd()
        .env("DEBUG", "1")
        .args([
            "info",
            "--waves",
            "definitely-missing-wavepeek.vcd",
            "--json",
        ])
        .output()
        .expect("wavepeek should execute");
    assert_eq!(debug.status.code(), Some(2));
    assert_eq!(json_stdout(&debug)["code"], "WPK-F0002");
    let stderr = String::from_utf8_lossy(&debug.stderr);
    assert!(stderr.contains("\"kind\":\"debug\""));
    assert!(!stderr.contains("fatal: file:"));
}
