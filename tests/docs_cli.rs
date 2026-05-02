use assert_cmd::prelude::*;
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::tempdir;

mod common;
use common::{expected_schema_url, wavepeek_cmd};

const TOPIC_IDS: [&str; 9] = [
    "commands/change",
    "commands/docs",
    "commands/help",
    "commands/property",
    "concepts/selectors",
    "concepts/time",
    "intro",
    "troubleshooting/empty-results",
    "workflows/find-first-change",
];

fn docs_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("docs")
        .join("cli")
}

fn canonical_topic_path(topic_id: &str) -> PathBuf {
    docs_root().join("topics").join(format!("{topic_id}.md"))
}

fn canonical_skill_path() -> PathBuf {
    docs_root().join("wavepeek-skill.md")
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

fn successful_stdout_text(args: &[&str]) -> String {
    String::from_utf8(successful_stdout(args)).expect("stdout should be UTF-8")
}

fn successful_json(args: &[&str]) -> Value {
    serde_json::from_slice(&successful_stdout(args)).expect("stdout should be valid json")
}

fn topic_ids_from_listing(output: &str) -> Vec<String> {
    output
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                None
            } else {
                trimmed.split_whitespace().next().map(str::to_string)
            }
        })
        .collect()
}

fn export_to(out_dir: &Path) {
    let out_dir = out_dir.to_string_lossy().into_owned();
    let mut command = wavepeek_cmd();
    command
        .args(["docs", "export", out_dir.as_str()])
        .assert()
        .success();
}

#[test]
fn docs_topics_are_sorted_lexicographically() {
    let output = successful_stdout_text(&["docs", "topics"]);

    assert_eq!(topic_ids_from_listing(&output), TOPIC_IDS);
}

#[test]
fn docs_command_prints_orientation_index() {
    let output = successful_stdout_text(&["docs"]);

    assert!(output.contains("wavepeek local docs"));
    assert!(output.contains("Start here when you need more than command syntax."));
    assert!(output.contains("wavepeek docs topics"));
    assert!(output.contains("wavepeek docs show intro"));
    assert!(output.contains("wavepeek docs search transitions"));
    assert!(output.contains("wavepeek docs skill"));
    assert!(output.contains("wavepeek docs export /tmp/wavepeek-docs"));
}

#[test]
fn docs_topics_json_uses_standard_envelope() {
    let value = successful_json(&["docs", "topics", "--json"]);

    assert_eq!(value["$schema"], expected_schema_url());
    assert_eq!(value["command"], "docs topics");
    assert_eq!(value["warnings"], Value::Array(vec![]));

    let topics = value["data"]["topics"]
        .as_array()
        .expect("docs topics payload should expose a topics array");
    let ids: Vec<String> = topics
        .iter()
        .map(|topic| {
            topic["id"]
                .as_str()
                .expect("topic id should be string")
                .to_string()
        })
        .collect();

    assert_eq!(ids, TOPIC_IDS);
    assert_eq!(topics[0]["id"], "commands/change");
    assert_eq!(topics[0]["title"], "Change command");
    assert_eq!(topics[0]["section"], "commands");
    assert!(topics[0]["summary"].is_string());
    assert!(topics[0]["see_also"].is_array());
}

#[test]
fn docs_search_ranks_matches_deterministically() {
    let value = successful_json(&["docs", "search", "find first change", "--json"]);

    assert_eq!(value["command"], "docs search");
    assert_eq!(value["data"]["query"], "find first change");
    assert_eq!(value["data"]["full_text"], false);

    let matches = value["data"]["matches"]
        .as_array()
        .expect("docs search payload should expose a matches array");
    let ids: Vec<&str> = matches
        .iter()
        .map(|entry| {
            entry["topic"]["id"]
                .as_str()
                .expect("topic id should exist")
        })
        .collect();
    let match_kinds: Vec<&str> = matches
        .iter()
        .map(|entry| {
            entry["match_kind"]
                .as_str()
                .expect("match_kind should be string")
        })
        .collect();

    assert_eq!(
        ids,
        vec![
            "workflows/find-first-change",
            "commands/change",
            "troubleshooting/empty-results"
        ]
    );
    assert_eq!(match_kinds, vec!["title_exact", "id_prefix", "heading"]);
}

#[test]
fn docs_search_matches_topic_id_tokens_by_default() {
    let value = successful_json(&["docs", "search", "empty-results", "--json"]);

    let matches = value["data"]["matches"]
        .as_array()
        .expect("docs search payload should expose a matches array");
    assert_eq!(matches[0]["topic"]["id"], "troubleshooting/empty-results");
    assert_eq!(matches[0]["match_kind"], "id_prefix");
    assert_eq!(matches[0]["matched_tokens"], 1);
}

#[test]
fn docs_search_counts_distinct_query_tokens_only_once() {
    let value = successful_json(&["docs", "search", "change", "change", "--json"]);

    let matches = value["data"]["matches"]
        .as_array()
        .expect("docs search payload should expose a matches array");
    assert_eq!(matches[0]["matched_tokens"], 1);
}

#[test]
fn docs_search_json_normalizes_internal_whitespace() {
    let value = successful_json(&["docs", "search", "find   first change", "--json"]);

    assert_eq!(value["data"]["query"], "find first change");
}

#[test]
fn docs_search_preserves_exact_title_match_kind() {
    let value = successful_json(&["docs", "search", "Change command", "--json"]);

    let matches = value["data"]["matches"]
        .as_array()
        .expect("docs search payload should expose a matches array");
    assert_eq!(matches[0]["topic"]["id"], "commands/change");
    assert_eq!(matches[0]["match_kind"], "title_exact");
}

#[test]
fn docs_search_empty_query_is_argument_error() {
    let output = wavepeek_cmd()
        .args(["docs", "search", "   "])
        .output()
        .expect("docs search should execute");

    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.starts_with("error: args:"));
    assert!(stderr.contains("query must contain at least one non-whitespace token"));
}

#[test]
fn docs_show_unknown_topic_suggests_close_matches() {
    let output = wavepeek_cmd()
        .args(["docs", "show", "commands/cha"])
        .output()
        .expect("docs show should execute");

    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.starts_with("error: args:"));
    assert!(stderr.contains("unknown docs topic 'commands/cha'"));
    assert!(stderr.contains("commands/change"));
}

#[test]
fn docs_show_summary_prints_only_stored_summary_text() {
    let output = successful_stdout_text(&["docs", "show", "commands/change", "--summary"]);

    assert_eq!(
        output.trim(),
        "Inspect value transitions across a bounded time range."
    );
}

#[test]
fn unsupported_docs_json_modes_are_argument_errors() {
    let export_dir = tempdir().expect("tempdir should be created");
    let export_target = export_dir.path().join("wavepeek-docs");
    let export_target = export_target.to_string_lossy().into_owned();

    let cases = [
        vec!["docs", "show", "intro", "--json"],
        vec!["docs", "skill", "--json"],
        vec!["docs", "export", export_target.as_str(), "--json"],
    ];

    for args in cases {
        let output = wavepeek_cmd()
            .args(args.as_slice())
            .output()
            .expect("docs command should execute");

        assert!(!output.status.success(), "args {:?} should fail", args);
        assert!(
            output.stdout.is_empty(),
            "args {:?} should not print stdout",
            args
        );
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr.starts_with("error: args:"),
            "args {:?} stderr was {stderr}",
            args
        );
        assert!(
            stderr.contains("unexpected argument '--json'"),
            "args {:?} stderr was {stderr}",
            args
        );
    }
}

#[test]
fn docs_export_force_requires_managed_root() {
    let temp = tempdir().expect("tempdir should be created");
    let export_dir = temp.path().join("unmanaged-docs");
    fs::create_dir_all(&export_dir).expect("export dir should be created");
    fs::write(export_dir.join("notes.txt"), "keep me").expect("sentinel file should be written");

    let export_dir_string = export_dir.to_string_lossy().into_owned();
    let output = wavepeek_cmd()
        .args(["docs", "export", "--force", export_dir_string.as_str()])
        .output()
        .expect("docs export should execute");

    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.starts_with("error: args:"));
    assert!(stderr.contains("managed export root"));
}

#[test]
fn docs_export_force_rejects_unrecognized_manifest_version() {
    let temp = tempdir().expect("tempdir should be created");
    let export_dir = temp.path().join("managed-docs");
    fs::create_dir_all(&export_dir).expect("export dir should be created");
    fs::write(
        export_dir.join("manifest.json"),
        r#"{"kind":"wavepeek-docs-export","export_format_version":999,"cli_name":"wavepeek","cli_version":"0.4.0","topics":[]}"#,
    )
    .expect("manifest should be written");

    let export_dir_string = export_dir.to_string_lossy().into_owned();
    let output = wavepeek_cmd()
        .args(["docs", "export", "--force", export_dir_string.as_str()])
        .output()
        .expect("docs export should execute");

    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.starts_with("error: args:"));
    assert!(stderr.contains("unrecognized export manifest version"));
}

#[test]
fn docs_export_preserves_front_matter() {
    let temp = tempdir().expect("tempdir should be created");
    let export_dir = temp.path().join("wavepeek-docs");

    export_to(&export_dir);

    let exported = fs::read(export_dir.join("commands").join("change.md"))
        .expect("exported topic should be readable");
    let canonical = fs::read(canonical_topic_path("commands/change"))
        .expect("canonical topic should be readable");

    assert_eq!(exported, canonical);
}

#[test]
fn docs_export_excludes_skill_asset() {
    let temp = tempdir().expect("tempdir should be created");
    let export_dir = temp.path().join("wavepeek-docs");

    export_to(&export_dir);

    assert!(!export_dir.join("wavepeek-skill.md").exists());
}

#[test]
fn docs_export_manifest_matches_contract() {
    let temp = tempdir().expect("tempdir should be created");
    let export_dir = temp.path().join("wavepeek-docs");

    export_to(&export_dir);

    let manifest =
        fs::read_to_string(export_dir.join("manifest.json")).expect("manifest should be readable");
    let value: Value = serde_json::from_str(&manifest).expect("manifest should be valid json");
    let ids: Vec<&str> = value["topics"]
        .as_array()
        .expect("topics should be an array")
        .iter()
        .map(|topic| topic["id"].as_str().expect("topic id should be string"))
        .collect();

    assert_eq!(value["kind"], "wavepeek-docs-export");
    assert_eq!(value["export_format_version"], 1);
    assert_eq!(value["cli_name"], "wavepeek");
    assert_eq!(value["cli_version"], env!("CARGO_PKG_VERSION"));
    assert_eq!(ids, TOPIC_IDS);
}

#[test]
fn docs_export_replaces_stale_managed_files() {
    let temp = tempdir().expect("tempdir should be created");
    let export_dir = temp.path().join("wavepeek-docs");

    export_to(&export_dir);
    fs::write(export_dir.join("stale.txt"), "obsolete").expect("stale file should be written");

    let export_dir_string = export_dir.to_string_lossy().into_owned();
    wavepeek_cmd()
        .args(["docs", "export", "--force", export_dir_string.as_str()])
        .assert()
        .success();

    assert!(!export_dir.join("stale.txt").exists());
    assert!(export_dir.join("commands").join("change.md").exists());
}

#[test]
fn docs_skill_prints_packaged_skill_markdown() {
    let expected = fs::read(canonical_skill_path()).expect("canonical skill should be readable");
    let actual = successful_stdout(&["docs", "skill"]);

    assert_eq!(actual, expected);
}

#[test]
fn packaged_skill_guidance_matches_current_runtime_capabilities() {
    let packaged =
        fs::read_to_string(canonical_skill_path()).expect("packaged skill should be readable");

    assert!(packaged.contains("wavepeek help <command-path...>"));
    assert!(!packaged.contains("parsed but not executed in `change`"));
    assert!(!packaged.contains("parse-level only; runtime execution is not implemented"));
}
