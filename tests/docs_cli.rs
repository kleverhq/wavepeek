use assert_cmd::prelude::*;
use serde_json::{Value, json};
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::tempdir;

mod common;
use common::{expected_schema_url, wavepeek_cmd};

const TOPIC_IDS: [&str; 24] = [
    "intro",
    "commands/change",
    "commands/docs",
    "commands/extract",
    "commands/help",
    "commands/info",
    "commands/overview",
    "commands/property",
    "commands/schema",
    "commands/scope",
    "commands/signal",
    "commands/skill",
    "commands/value",
    "workflows/extract-handshake",
    "workflows/find-first-change",
    "troubleshooting/clock-edge-sampling",
    "troubleshooting/empty-results",
    "troubleshooting/scoped-vs-canonical-names",
    "troubleshooting/time-tokens-and-alignment",
    "troubleshooting/unsupported-signal-encodings",
    "reference/command-model",
    "reference/expression-language",
    "reference/machine-output",
    "reference/waveform-performance",
];

fn docs_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("docs")
        .join("public")
}

fn canonical_topic_path(topic_id: &str) -> PathBuf {
    docs_root().join(format!("{topic_id}.md"))
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
fn docs_topics_use_logical_section_order() {
    let output = successful_stdout_text(&["docs", "topics"]);

    assert_eq!(topic_ids_from_listing(&output), TOPIC_IDS);
}

#[test]
fn docs_command_without_subcommand_prints_help() {
    let output = successful_stdout_text(&["docs"]);

    assert!(output.starts_with("Browse the embedded documentation packaged with this build."));
    assert!(output.contains("Usage: wavepeek docs"));
    assert!(output.contains("Commands:"));
    assert!(output.contains("topics"));
    assert!(output.contains("show"));
    assert!(output.contains("search"));
    assert!(output.contains("export"));
    assert!(!output.contains("skill"));
    assert!(!output.contains("wavepeek local docs"));
    assert!(!output.contains("Try:"));
}

#[test]
fn docs_topics_json_uses_standard_envelope() {
    let value = successful_json(&["docs", "topics", "--json"]);

    assert_eq!(value["$schema"], expected_schema_url());
    assert_eq!(value["command"], "docs topics");
    assert_eq!(value["diagnostics"], Value::Array(vec![]));

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
    assert_eq!(topics[0]["id"], "intro");
    assert_eq!(topics[0]["title"], "Introduction");
    assert_eq!(topics[0]["section"], "intro");
    assert!(topics[0]["description"].is_string());
    let topic_keys = topics[0]
        .as_object()
        .expect("topic should be object")
        .keys()
        .cloned()
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(
        topic_keys,
        std::collections::BTreeSet::from([
            "description".to_string(),
            "id".to_string(),
            "section".to_string(),
            "see_also".to_string(),
            "title".to_string(),
        ])
    );
    assert!(topics[0]["see_also"].is_array());
}

#[test]
fn docs_search_ranks_matches_deterministically() {
    let value = successful_json(&["docs", "search", "find first change", "--json"]);

    assert_eq!(value["command"], "docs search");
    assert_eq!(value["data"]["query"], "find first change");
    assert!(value["data"].get("full_text").is_none());

    let matches = value["data"]["matches"]
        .as_array()
        .expect("docs search payload should expose a matches array");

    assert_eq!(matches[0]["topic"]["id"], "workflows/find-first-change");
    assert_eq!(matches[0]["match_kind"], "title_exact");

    let heading_idx = matches
        .iter()
        .position(|entry| entry["topic"]["id"] == "troubleshooting/empty-results")
        .expect("troubleshooting/empty-results should match");
    assert_eq!(matches[heading_idx]["match_kind"], "heading");

    let title_or_description_idx = matches
        .iter()
        .position(|entry| entry["topic"]["id"] == "reference/expression-language")
        .expect("reference/expression-language should match");
    assert_eq!(
        matches[title_or_description_idx]["match_kind"],
        "title_or_description"
    );

    let body_idx = matches
        .iter()
        .position(|entry| entry["match_kind"] == "body")
        .expect("query should produce at least one body match");

    let id_prefix_idx = matches
        .iter()
        .position(|entry| entry["topic"]["id"] == "commands/change")
        .expect("commands/change should match");
    assert_eq!(matches[id_prefix_idx]["match_kind"], "id_prefix");

    assert!(heading_idx > 0);
    assert!(title_or_description_idx > heading_idx);
    assert!(body_idx > heading_idx);
    assert!(id_prefix_idx > heading_idx);
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
    let change_match = matches
        .iter()
        .find(|entry| entry["topic"]["id"] == "commands/change")
        .expect("commands/change should match exact title query");
    assert_eq!(change_match["match_kind"], "title_exact");
}

#[test]
fn docs_search_empty_result_emits_empty_result_diagnostic() {
    let json_output = wavepeek_cmd()
        .args(["docs", "search", "zzzzzzzzzzzz", "--json"])
        .output()
        .expect("json docs search should execute");
    let human_output = wavepeek_cmd()
        .args(["docs", "search", "zzzzzzzzzzzz"])
        .output()
        .expect("human docs search should execute");

    assert!(json_output.status.success());
    assert!(human_output.status.success());
    assert!(human_output.stdout.is_empty());
    let value: Value =
        serde_json::from_slice(&json_output.stdout).expect("stdout should be valid json");
    assert_eq!(value["data"]["matches"], json!([]));
    assert_eq!(
        value["diagnostics"],
        json!([{"kind": "warning", "code": "WPK-W0003", "message": "no docs topics matched query"}])
    );
    assert_eq!(
        String::from_utf8_lossy(&human_output.stderr).trim(),
        "warning[WPK-W0003]: no docs topics matched query"
    );
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
    assert!(stderr.starts_with("fatal: args:"));
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
    assert!(stderr.starts_with("fatal: args:"));
    assert!(stderr.contains("unknown docs topic 'commands/cha'"));
    assert!(stderr.contains("commands/change"));
}

#[test]
fn docs_show_description_prints_only_stored_description_text() {
    let output = successful_stdout_text(&["docs", "show", "commands/change", "--description"]);

    assert_eq!(
        output.trim(),
        "Inspect value transitions across a bounded time range."
    );
}

#[test]
fn public_extract_docs_cover_axi5_and_ace_family_profiles() {
    for topic_id in [
        "commands/extract",
        "commands/overview",
        "workflows/extract-handshake",
        "reference/machine-output",
    ] {
        let output = successful_stdout_text(&["docs", "show", topic_id]);
        assert!(
            output.contains("AXI5"),
            "topic {topic_id} should cover AXI5 extraction profiles"
        );
        assert!(
            output.contains("ACE5"),
            "topic {topic_id} should cover ACE-family extraction profiles"
        );
        assert!(
            output.contains("ACE5, ACE5-Lite, ACE5-LiteDVM, and ACE5-LiteACP"),
            "topic {topic_id} should list every ACE5 family profile"
        );
    }

    let extract = successful_stdout_text(&["docs", "show", "commands/extract"]);
    assert!(extract.contains("ACE-Lite"));
    assert!(extract.contains("Issue L"));
    assert!(extract.contains("AXI5 and ACE5-LiteDVM add the `ac` and `cr`"));
    assert!(extract.contains("ready/valid channel transfers"));
    assert!(extract.contains("credited transport"));
    assert!(extract.contains(
        "ACE5-LiteDVM additionally accepts `ace5-litedvm`, `ace5_litedvm`, and `ace5_lite_dvm`"
    ));
    assert!(extract.contains(
        "ACE5-LiteACP additionally accepts `ace5-liteacp`, `ace5_liteacp`, and `ace5_lite_acp`"
    ));
    assert!(extract.contains("Generated schemas accept canonical hyphenated profile names only."));

    let machine_output = successful_stdout_text(&["docs", "show", "reference/machine-output"]);
    assert!(machine_output.contains("Issue H.c"));
    assert!(machine_output.contains("Issue L"));
}

#[test]
fn public_extract_docs_cover_atb_profiles_and_stateless_boundaries() {
    let extract = successful_stdout_text(&["docs", "show", "commands/extract"]);
    for fragment in [
        "`extract atb` emits stateless AMBA ATB interface events",
        "`atb-a`, `atb-b`, and `atb-c`",
        "Arm IHI 0032C Issue C",
        "atbv1.0",
        "atbv1.1",
        "atclken",
        "atwakeup",
        "`transfer`, `flush`, then `sync-request` order",
        "does not reconstruct trace packets",
        "Appendix A Table A-1",
    ] {
        assert!(
            extract.contains(fragment),
            "extract docs should contain `{fragment}`"
        );
    }

    let machine_output = successful_stdout_text(&["docs", "show", "reference/machine-output"]);
    assert!(machine_output.contains("`extract atb` data is an object"));
    assert!(machine_output.contains("`extract.atb.source`"));
}

#[test]
fn public_docs_describe_fsdb_target_restriction() {
    for topic_id in ["intro", "reference/command-model"] {
        let output = successful_stdout_text(&["docs", "show", topic_id]);

        assert!(
            output.contains("FSDB support is currently Linux x86_64 only"),
            "topic {topic_id} should describe the FSDB target restriction"
        );
    }

    for topic_id in ["commands/change", "commands/property"] {
        let output = successful_stdout_text(&["docs", "show", topic_id]);

        assert!(
            output.contains("FSDB works only in binaries built with the `fsdb` Cargo feature"),
            "topic {topic_id} should describe FSDB build requirements"
        );
    }
}

#[test]
fn expression_reference_describes_fsdb_metadata_support() {
    let output = successful_stdout_text(&["docs", "show", "reference/expression-language"]);

    assert!(
        output.contains("FSDB waveform dumps in FSDB-enabled builds"),
        "expression reference should include FSDB-enabled expression metadata support"
    );
    assert!(
        !output.contains("recoverable from\nVCD/FST waveform dumps"),
        "expression reference should not describe expression metadata as VCD/FST-only"
    );
}

#[test]
fn waveform_performance_guide_describes_current_format_tradeoffs() {
    let output = successful_stdout_text(&["docs", "show", "reference/waveform-performance"]);

    for expected in [
        "VCD is a textual waveform format",
        "FST is a compact, indexed waveform format",
        "FSDB is a compact proprietary waveform format",
        "Converting a dump to FST can help",
    ] {
        assert!(
            output.contains(expected),
            "performance guide should contain {expected:?}"
        );
    }

    let lower = output.to_ascii_lowercase();
    assert!(
        !lower.contains("batch") && !lower.contains("session workflow"),
        "performance guide should describe current behavior, not future batch/session workflow"
    );
}

#[test]
fn unsupported_docs_json_modes_are_argument_errors() {
    let export_dir = tempdir().expect("tempdir should be created");
    let export_target = export_dir.path().join("wavepeek-docs");
    let export_target = export_target.to_string_lossy().into_owned();

    let cases = [
        vec!["docs", "show", "intro", "--json"],
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
            stderr.starts_with("fatal: args:"),
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
    assert!(stderr.starts_with("fatal: args:"));
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
    assert!(stderr.starts_with("fatal: args:"));
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

    assert!(!export_dir.join("wavepeek.md").exists());
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
fn commands_skill_topic_explains_top_level_entrypoint() {
    let output = successful_stdout_text(&["docs", "show", "commands/skill"]);

    assert!(output.contains("wavepeek skill"));
    assert!(output.contains("wavepeek help skill"));
    assert!(!output.contains("wavepeek docs skill"));
}

#[test]
fn commands_docs_topic_no_longer_mentions_skill_printing() {
    let output = successful_stdout_text(&["docs", "show", "commands/docs"]);

    assert!(output.contains("wavepeek docs topics"));
    assert!(!output.contains("wavepeek docs skill"));
    assert!(!output.contains("docs skill --json"));
}
