use serde_json::{Value, json};

mod common;
use common::{fixture_path, wavepeek_cmd};

fn run(args: &[String], mode: &str, summary_only: bool) -> std::process::Output {
    let mut command = wavepeek_cmd();
    command.args(args).arg(mode);
    if summary_only {
        command.arg("--summary");
    }
    let output = command.output().expect("wavepeek should execute");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    output
}

fn run_json(args: &[String], summary_only: bool) -> Value {
    let output = run(args, "--json", summary_only);
    assert!(output.stderr.is_empty());
    serde_json::from_slice(&output.stdout).expect("valid JSON")
}

fn run_jsonl(args: &[String], summary_only: bool) -> Vec<Value> {
    let output = run(args, "--jsonl", summary_only);
    assert!(output.stderr.is_empty());
    String::from_utf8(output.stdout)
        .expect("UTF-8 JSONL")
        .lines()
        .map(|line| serde_json::from_str(line).expect("valid JSONL record"))
        .collect()
}

fn assert_summary(value: &Value, complete: bool, returned: usize, limit: Value, total: Value) {
    assert_eq!(
        value,
        &json!({
            "complete": complete,
            "returned": returned,
            "limit": limit,
            "total": total,
        })
    );
}

#[test]
fn list_summary_distinguishes_selection_depth_and_count_truncation() {
    let waves = fixture_path("m2_core.vcd").to_string_lossy().into_owned();
    let args = |max: &str| {
        vec![
            "scope".to_string(),
            "--waves".to_string(),
            waves.clone(),
            "--max".to_string(),
            max.to_string(),
        ]
    };

    assert_summary(
        &run_json(&args("4"), false)["summary"],
        true,
        3,
        json!(4),
        json!(3),
    );
    assert_summary(
        &run_json(&args("3"), false)["summary"],
        true,
        3,
        json!(3),
        json!(3),
    );
    let truncated = run_json(&args("1"), false);
    assert_summary(&truncated["summary"], false, 1, json!(1), json!(3));
    assert_eq!(truncated["diagnostics"][0]["code"], "WPK-W0002");
    assert_summary(
        &run_json(&args("unlimited"), false)["summary"],
        true,
        3,
        Value::Null,
        json!(3),
    );

    let mut depth_limited = args("50");
    depth_limited.extend(["--max-depth".to_string(), "0".to_string()]);
    assert_summary(
        &run_json(&depth_limited, false)["summary"],
        true,
        1,
        json!(50),
        json!(1),
    );
}

#[test]
fn row_summary_distinguishes_exact_limit_from_truncation() {
    let waves = fixture_path("m2_core.vcd").to_string_lossy().into_owned();
    let args = |max: &str| {
        vec![
            "change".to_string(),
            "--waves".to_string(),
            waves.clone(),
            "--from".to_string(),
            "1ns".to_string(),
            "--to".to_string(),
            "10ns".to_string(),
            "--signals".to_string(),
            "top.clk,top.data".to_string(),
            "--on".to_string(),
            "*".to_string(),
            "--sample-mode".to_string(),
            "native".to_string(),
            "--max".to_string(),
            max.to_string(),
        ]
    };

    assert_summary(
        &run_json(&args("3"), false)["summary"],
        true,
        2,
        json!(3),
        json!(2),
    );
    assert_summary(
        &run_json(&args("2"), false)["summary"],
        true,
        2,
        json!(2),
        json!(2),
    );
    assert_summary(
        &run_json(&args("1"), false)["summary"],
        false,
        1,
        json!(1),
        Value::Null,
    );
    assert_summary(
        &run_json(&args("unlimited"), false)["summary"],
        true,
        2,
        Value::Null,
        json!(2),
    );
}

#[test]
fn event_summary_distinguishes_exact_limit_from_truncation() {
    let waves = fixture_path("extract_ahb_lite.vcd")
        .to_string_lossy()
        .into_owned();
    let args = |max: &str| {
        vec![
            "extract".to_string(),
            "ahb".to_string(),
            "--waves".to_string(),
            waves.clone(),
            "--scope".to_string(),
            "top".to_string(),
            "--map".to_string(),
            "hclk=clk".to_string(),
            "--include".to_string(),
            "^ahb_lite_.*".to_string(),
            "--max".to_string(),
            max.to_string(),
        ]
    };

    assert_summary(
        &run_json(&args("14"), false)["summary"],
        true,
        13,
        json!(14),
        json!(13),
    );
    assert_summary(
        &run_json(&args("13"), false)["summary"],
        true,
        13,
        json!(13),
        json!(13),
    );
    assert_summary(
        &run_json(&args("1"), false)["summary"],
        false,
        1,
        json!(1),
        Value::Null,
    );
    assert_summary(
        &run_json(&args("unlimited"), false)["summary"],
        true,
        13,
        Value::Null,
        json!(13),
    );
}

#[test]
fn summary_only_suppresses_data_but_preserves_query_results() {
    let m2 = fixture_path("m2_core.vcd").to_string_lossy().into_owned();
    let ahb = fixture_path("extract_ahb_lite.vcd")
        .to_string_lossy()
        .into_owned();
    let apb = fixture_path("extract_apb3.vcd")
        .to_string_lossy()
        .into_owned();
    let cases = vec![
        vec!["scope", "--waves", &m2, "--max", "1"],
        vec!["signal", "--waves", &m2, "--scope", "top", "--max", "1"],
        vec![
            "change",
            "--waves",
            &m2,
            "--from",
            "1ns",
            "--to",
            "10ns",
            "--scope",
            "top",
            "--signals",
            "clk,data",
            "--on",
            "*",
            "--sample-mode",
            "native",
            "--max",
            "1",
        ],
        vec![
            "property",
            "--waves",
            &m2,
            "--scope",
            "top",
            "--on",
            "*",
            "--sample-mode",
            "native",
            "--eval",
            "data == 8'h0f",
            "--capture",
            "match",
            "--max",
            "1",
        ],
        vec![
            "extract",
            "generic",
            "--waves",
            &m2,
            "--scope",
            "top",
            "--on",
            "posedge clk",
            "--when",
            "1",
            "--payload",
            "data",
            "--max",
            "1",
        ],
        vec![
            "extract",
            "ahb",
            "--waves",
            &ahb,
            "--scope",
            "top",
            "--map",
            "hclk=clk",
            "--include",
            "^ahb_lite_.*",
            "--max",
            "1",
        ],
        vec![
            "extract",
            "apb",
            "--waves",
            &apb,
            "--scope",
            "top",
            "--profile",
            "apb3",
            "--include",
            "^apb3_",
            "--max",
            "1",
        ],
    ];

    for args in cases {
        let args = args.into_iter().map(String::from).collect::<Vec<_>>();
        let ordinary = run_json(&args, false);
        let summary_only = run_json(&args, true);
        assert!(summary_only.get("data").is_none());
        assert_eq!(summary_only["summary"], ordinary["summary"]);
        assert_eq!(summary_only["diagnostics"], ordinary["diagnostics"]);
        assert_eq!(summary_only.get("context"), ordinary.get("context"));

        let records = run_jsonl(&args, true);
        assert_eq!(records.first().unwrap()["type"], "begin");
        assert!(records.iter().all(|record| record["type"] != "data"));
        let end = records.last().unwrap();
        assert_eq!(end["type"], "end");
        assert_eq!(end["records"]["data"], 0);
        assert_eq!(end["summary"], ordinary["summary"]);
        assert_eq!(
            records.first().unwrap().get("context"),
            ordinary.get("context")
        );
    }
}

#[test]
fn human_empty_summary_suppresses_empty_message() {
    let waves = fixture_path("m2_core.vcd").to_string_lossy().into_owned();
    let args = vec![
        "scope".to_string(),
        "--waves".to_string(),
        waves,
        "--filter".to_string(),
        "^missing$".to_string(),
    ];
    let output = run(&args, "--summary", false);

    assert_eq!(
        String::from_utf8(output.stdout).expect("UTF-8 stdout"),
        "complete: true\nreturned: 0\nlimit: 50\ntotal: 0\n"
    );
    assert_eq!(
        output.stderr,
        b"info: opening waveform\ninfo: listing scopes\n"
    );
}

#[test]
fn human_summary_only_preserves_protocol_context_and_diagnostics() {
    let waves = fixture_path("extract_ahb_lite.vcd")
        .to_string_lossy()
        .into_owned();
    let args = vec![
        "extract".to_string(),
        "ahb".to_string(),
        "--waves".to_string(),
        waves,
        "--scope".to_string(),
        "top".to_string(),
        "--map".to_string(),
        "hclk=clk".to_string(),
        "--include".to_string(),
        "^ahb_lite_.*".to_string(),
        "--max".to_string(),
        "1".to_string(),
    ];
    let output = run(&args, "--summary", false);
    let stdout = String::from_utf8(output.stdout).expect("UTF-8 stdout");
    let stderr = String::from_utf8(output.stderr).expect("UTF-8 stderr");

    assert!(stdout.contains("name: ahb\nprofile: ahb-lite"));
    assert!(stdout.contains("mappings:\n"));
    assert!(!stdout.contains("events:"));
    assert!(stdout.ends_with("complete: false\nreturned: 1\nlimit: 1\ntotal: null\n"));
    assert!(stderr.contains("warning[WPK-W0002]"));
}

#[test]
fn summary_flag_is_limited_to_commands_with_max() {
    let affected = [
        &["scope"][..],
        &["signal"][..],
        &["change"][..],
        &["property"][..],
        &["extract", "ahb"][..],
        &["extract", "apb"][..],
        &["extract", "atb"][..],
        &["extract", "axi"][..],
        &["extract", "axistream"][..],
        &["extract", "generic"][..],
    ];
    for command in affected {
        let output = wavepeek_cmd()
            .args(command)
            .arg("--help")
            .output()
            .expect("help should execute");
        assert!(output.status.success());
        assert!(String::from_utf8_lossy(&output.stdout).contains("--summary"));
    }

    for command in [&["info"][..], &["value"][..]] {
        let output = wavepeek_cmd()
            .args(command)
            .arg("--help")
            .output()
            .expect("help should execute");
        assert!(output.status.success());
        assert!(!String::from_utf8_lossy(&output.stdout).contains("--summary"));
    }
}
