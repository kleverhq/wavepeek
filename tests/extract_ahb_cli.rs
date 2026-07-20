use std::fs;

use assert_cmd::prelude::*;
use predicates::prelude::*;
use serde_json::{Value, json};
use tempfile::NamedTempFile;

mod common;
use common::{
    expected_input_schema_url, expected_schema_url, expected_stream_schema_url, fixture_path,
    wavepeek_cmd,
};

fn schema_validator(name: &str) -> jsonschema::Validator {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("schema")
        .join(name);
    let schema: Value =
        serde_json::from_str(&fs::read_to_string(path).expect("schema should read"))
            .expect("schema should parse");
    jsonschema::validator_for(&schema).expect("schema should compile")
}

fn parse_json(stdout: &[u8]) -> Value {
    let value: Value = serde_json::from_slice(stdout).expect("stdout should be valid JSON");
    schema_validator("output.json")
        .validate(&value)
        .unwrap_or_else(|error| panic!("output should validate: {error}\n{value}"));
    value
}

fn parse_stream(stdout: &[u8]) -> Vec<Value> {
    let output = std::str::from_utf8(stdout).expect("stdout should be UTF-8 JSONL");
    assert!(output.ends_with('\n'));
    let validator = schema_validator("stream.json");
    output
        .lines()
        .map(|line| {
            let record: Value = serde_json::from_str(line).expect("record should parse");
            validator
                .validate(&record)
                .unwrap_or_else(|error| panic!("record should validate: {error}\n{record}"));
            record
        })
        .collect()
}

fn fixture(filename: &str) -> String {
    fixture_path(filename).to_string_lossy().into_owned()
}

fn lite_command(extension: &str) -> std::process::Command {
    let waveform = fixture(&format!("extract_ahb_lite.{extension}"));
    let mut command = wavepeek_cmd();
    command.args([
        "extract",
        "ahb",
        "--waves",
        waveform.as_str(),
        "--scope",
        "top",
        "--map",
        "hclk=clk",
        "--include",
        "^ahb_lite_.*",
    ]);
    command
}

fn write_source(value: &Value) -> NamedTempFile {
    let file = NamedTempFile::with_suffix("extract-ahb-source.json").expect("source should create");
    fs::write(
        file.path(),
        serde_json::to_vec_pretty(value).expect("source should serialize"),
    )
    .expect("source should write");
    file
}

fn event_kinds(value: &Value) -> Vec<&str> {
    value["data"]["events"]
        .as_array()
        .expect("events should be an array")
        .iter()
        .map(|event| event["event"].as_str().expect("event kind"))
        .collect()
}

#[test]
fn extract_ahb_lite_defaults_emit_pipeline_events_without_idle_spam() {
    let output = lite_command("vcd")
        .arg("--json")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let value = parse_json(&output);

    assert_eq!(value["$schema"], expected_schema_url());
    assert_eq!(value["command"], "extract ahb");
    assert_eq!(value["data"]["name"], "ahb");
    assert_eq!(value["data"]["profile"], "ahb-lite");
    assert_eq!(value["data"]["issue"], "C");
    assert_eq!(value["data"]["include_stall"], false);
    assert_eq!(value["data"]["include_idle"], false);
    assert_eq!(value["data"]["include_busy"], false);
    assert_eq!(
        value["data"]["initial_data_phase"],
        json!({"state": "desynchronized"})
    );
    assert_eq!(
        event_kinds(&value),
        [
            "reset",
            "address",
            "data-complete",
            "address",
            "data-complete",
            "address",
            "data-complete",
            "desynchronized",
            "address",
            "data-complete",
            "reset",
            "desynchronized",
            "reset",
        ]
    );
    assert_eq!(value["data"]["events"][4]["time"], "40ns");
    assert_eq!(value["data"]["events"][4]["event"], "data-complete");
    assert_eq!(value["data"]["events"][5]["time"], "40ns");
    assert_eq!(value["data"]["events"][5]["event"], "address");
    assert_eq!(value["data"]["events"][5]["transfer"], "seq");
}

#[test]
fn extract_ahb_optional_rows_preserve_event_specific_payload_validity() {
    let output = lite_command("vcd")
        .args([
            "--include-stall",
            "--include-idle",
            "--include-busy",
            "--json",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let value = parse_json(&output);
    let events = value["data"]["events"].as_array().expect("events");

    let stalls = events
        .iter()
        .filter(|event| event["event"] == "data-stall")
        .collect::<Vec<_>>();
    assert_eq!(stalls.len(), 2);
    assert_eq!(stalls[0]["payload"]["hresp"], "1'h0");
    assert!(stalls[0]["payload"].get("hrdata").is_none());
    assert_eq!(stalls[1]["payload"]["hresp"], "1'h1");
    assert!(stalls[1]["payload"].get("hrdata").is_none());

    let error_completion = events
        .iter()
        .find(|event| event["event"] == "data-complete" && event["time"] == "30ns")
        .expect("ERROR completion");
    assert_eq!(error_completion["payload"]["hresp"], "1'h1");
    assert_eq!(error_completion["payload"]["hrdata"], "32'hdeadbeef");
    for success_only in ["hruser", "hbuser"] {
        assert!(error_completion["payload"].get(success_only).is_none());
    }

    let sparse_write = events
        .iter()
        .find(|event| {
            event["event"] == "data-complete"
                && event["time"] == "40ns"
                && event["direction"] == "write"
        })
        .expect("write completion");
    assert_eq!(sparse_write["payload"]["hwstrb"], "4'h5");
    let unknown = events
        .iter()
        .find(|event| event["event"] == "data-complete" && event["time"] == "70ns")
        .expect("unknown-direction completion");
    assert_eq!(unknown["direction"], "unknown");
    assert_eq!(unknown["payload"]["hwdata"], "32'hcafef00d");
    assert_eq!(unknown["payload"]["hrdata"], "32'h0badc0de");

    assert_eq!(
        events
            .iter()
            .filter(|event| event["event"] == "busy")
            .count(),
        1
    );
    assert_eq!(
        events
            .iter()
            .filter(|event| event["event"] == "idle")
            .count(),
        5
    );
}

#[test]
fn extract_ahb5_emits_issue_c_fields_and_ignores_local_or_check_decoys() {
    let waveform = fixture("extract_ahb5.vcd");
    let output = wavepeek_cmd()
        .args([
            "extract",
            "ahb",
            "--waves",
            waveform.as_str(),
            "--scope",
            "top",
            "--profile",
            "ahb5",
            "--map",
            "hclk=clk",
            "--include",
            "^ahb5_.*",
            "--json",
        ])
        .output()
        .expect("AHB5 extraction should execute");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let value = parse_json(&output.stdout);
    let diagnostic_text = value["diagnostics"]
        .as_array()
        .expect("diagnostics")
        .iter()
        .map(|diagnostic| diagnostic["message"].as_str().expect("message"))
        .collect::<Vec<_>>()
        .join("\n");
    for decoy in [
        "ahb5_h_addr_chk_o",
        "ahb5_h_addr_parity_o",
        "ahb5_h_readyout_i",
        "ahb5_h_sel_i",
    ] {
        assert!(
            diagnostic_text.contains(&format!("ignored AHB include candidate '{decoy}'")),
            "missing warning for {decoy}:\n{diagnostic_text}"
        );
    }

    assert_eq!(value["data"]["profile"], "ahb5");
    assert_eq!(value["data"]["issue"], "C");
    let address = &value["data"]["events"][1];
    assert_eq!(address["payload"]["hnonsec"], "1'h1");
    assert_eq!(address["payload"]["hexcl"], "1'h1");
    assert_eq!(address["payload"]["hmaster"], "4'ha");
    let write_complete = &value["data"]["events"][2];
    assert_eq!(write_complete["payload"]["hwstrb"], "4'h5");
    assert_eq!(write_complete["payload"]["hbuser"], "2'h2");
    assert_eq!(write_complete["payload"]["hexokay"], "1'h1");
    let read_complete = &value["data"]["events"][4];
    assert_eq!(read_complete["payload"]["hrdata"], "32'h55667788");
    assert_eq!(read_complete["payload"]["hruser"], "3'h6");
}

#[test]
fn extract_ahb_warms_state_before_inclusive_lower_bound() {
    let output = lite_command("vcd")
        .args(["--from", "20ns", "--include-stall", "--json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let value = parse_json(&output);
    let initial = &value["data"]["initial_data_phase"];
    assert_eq!(initial["state"], "pending");
    assert_eq!(initial["address"]["time"], "15ns");
    assert_eq!(initial["address"]["sample_time"], "14ns");
    assert_eq!(initial["address"]["transfer"], "nonseq");
    assert_eq!(initial["address"]["direction"], "read");
    assert!(initial["address"].get("event").is_none());
    assert!(initial["address"].get("profile").is_none());
    assert_eq!(value["data"]["events"][0]["time"], "20ns");
    assert_eq!(value["data"]["events"][0]["event"], "data-stall");
}

#[test]
fn extract_ahb_limit_can_split_a_same_edge_pair_and_counts_public_rows_only() {
    let output = lite_command("vcd")
        .args(["--from", "40ns", "--max", "1", "--jsonl"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let records = parse_stream(&output);
    assert_eq!(records[0]["$schema"], expected_stream_schema_url());
    assert_eq!(records[0]["command"], "extract ahb");
    assert_eq!(
        records[0]["context"]["initial_data_phase"]["state"],
        "pending"
    );
    assert_eq!(
        records[0]["context"]["initial_data_phase"]["address"]["time"],
        "35ns"
    );
    let items = records
        .iter()
        .filter(|record| record["type"] == "item")
        .collect::<Vec<_>>();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0]["item"]["time"], "40ns");
    assert_eq!(items[0]["item"]["event"], "data-complete");
    assert_eq!(records[2]["diagnostic"]["code"], "WPK-W0002");
    assert_eq!(records[3]["summary"]["items"], 1);
    assert_eq!(records[3]["summary"]["diagnostics"], 1);
    assert_eq!(records[3]["summary"]["truncated"], true);
}

#[test]
fn extract_ahb_upper_bound_can_leave_a_pending_phase_without_false_completion() {
    let output = lite_command("vcd")
        .args(["--to", "15ns", "--json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let value = parse_json(&output);
    assert_eq!(event_kinds(&value), ["reset", "address"]);
    assert_eq!(value["data"]["events"][1]["time"], "15ns");
}

#[test]
fn extract_ahb_human_output_has_context_mappings_and_ordered_rows() {
    lite_command("vcd")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "name: ahb\nprofile: ahb-lite\nissue: C\ninclude_stall: false\ninclude_idle: false\ninclude_busy: false\ninitial_data_phase: desynchronized",
        ))
        .stdout(predicate::str::contains("  hready = ahb_lite_h_ready_i"))
        .stdout(predicate::str::contains(
            "@40ns sample@39ns [data-complete write]",
        ))
        .stdout(predicate::str::contains(
            "@40ns sample@39ns [address seq read]",
        ));
}

#[test]
fn extract_ahb_matches_between_vcd_and_fst() {
    for (profile, stem, include) in [
        ("ahb-lite", "extract_ahb_lite", "^ahb_lite_.*"),
        ("ahb5", "extract_ahb5", "^ahb5_.*"),
    ] {
        let mut outputs = Vec::new();
        for extension in ["vcd", "fst"] {
            let waveform = fixture(&format!("{stem}.{extension}"));
            outputs.push(
                wavepeek_cmd()
                    .args([
                        "extract",
                        "ahb",
                        "--waves",
                        waveform.as_str(),
                        "--scope",
                        "top",
                        "--profile",
                        profile,
                        "--map",
                        "hclk=clk",
                        "--include",
                        include,
                        "--json",
                    ])
                    .output()
                    .expect("cross-format extraction should execute"),
            );
        }
        assert!(
            outputs.iter().all(|output| output.status.success()),
            "{profile}"
        );
        assert_eq!(outputs[0].stdout, outputs[1].stdout, "{profile} stdout");
        assert_eq!(outputs[0].stderr, outputs[1].stderr, "{profile} stderr");
    }
}

#[test]
fn extract_ahb_accepts_profile_aliases_and_rejects_forbidden_mappings() {
    let waveform = fixture("extract_ahb_lite.vcd");
    for alias in ["ahb-lite", "AHB_LITE"] {
        wavepeek_cmd()
            .args([
                "extract",
                "ahb",
                "--waves",
                waveform.as_str(),
                "--scope",
                "top",
                "--profile",
                alias,
                "--map",
                "hclk=clk",
                "--include",
                "^ahb_lite_.*",
            ])
            .assert()
            .success()
            .stdout(predicate::str::contains("profile: ahb-lite\nissue: C"));
    }

    for forbidden in ["hnonsec", "hreadyout", "hsel", "haddrchk"] {
        wavepeek_cmd()
            .args([
                "extract",
                "ahb",
                "--waves",
                waveform.as_str(),
                "--profile",
                "ahb-lite",
                "--map",
                "hclk=top.clk",
                "--map",
                "htrans=top.ahb_lite_h_trans_o",
                "--map",
                "hready=top.ahb_lite_h_ready_i",
                "--map",
                "hwrite=top.ahb_lite_h_write_o",
                "--map",
                &format!("{forbidden}=top.clk"),
            ])
            .assert()
            .failure()
            .stdout(predicate::str::is_empty())
            .stderr(predicate::str::contains(format!(
                "AHB profile ahb-lite has no standard signal '{forbidden}'"
            )));
    }
}

#[test]
fn extract_ahb_requires_manager_progress_and_pipeline_control_mappings() {
    let waveform = fixture("extract_ahb_lite.vcd");
    wavepeek_cmd()
        .args([
            "extract",
            "ahb",
            "--waves",
            waveform.as_str(),
            "--map",
            "hclk=top.clk",
        ])
        .assert()
        .failure()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::contains(
            "AHB extraction requires mappings for htrans, hready, hwrite",
        ));

    wavepeek_cmd()
        .args([
            "extract",
            "ahb",
            "--waves",
            waveform.as_str(),
            "--scope",
            "top",
            "--map",
            "hclk=top.clk",
        ])
        .assert()
        .failure()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::contains(
            "must use a scope-relative signal when --scope is set",
        ));
}

#[test]
fn extract_ahb_source_mode_validates_identity_types_conflicts_and_extensions() {
    let waveform = fixture("extract_ahb_lite.vcd");
    let base = json!({
        "$schema": expected_input_schema_url(),
        "kind": "extract.ahb.source",
        "profile": "ahb-lite",
        "include_stall": true,
        "include_idle": true,
        "include_busy": true,
        "name": "dmem",
        "includes": ["^ahb_lite_.*"],
        "maps": {"hclk": "clk"},
        "extension": {"owner": "test"}
    });
    schema_validator("input.json")
        .validate(&base)
        .expect("extension-friendly source should validate");
    let source = write_source(&base);
    let source_path = source.path().to_string_lossy().into_owned();
    let output = wavepeek_cmd()
        .args([
            "extract",
            "ahb",
            "--waves",
            waveform.as_str(),
            "--scope",
            "top",
            "--source",
            source_path.as_str(),
            "--json",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let value = parse_json(&output);
    assert_eq!(value["data"]["name"], "dmem");
    assert_eq!(value["data"]["profile"], "ahb-lite");
    assert_eq!(value["data"]["include_stall"], true);
    assert_eq!(value["data"]["include_idle"], true);
    assert_eq!(value["data"]["include_busy"], true);

    wavepeek_cmd()
        .args([
            "extract",
            "ahb",
            "--waves",
            waveform.as_str(),
            "--source",
            source_path.as_str(),
            "--include-stall",
        ])
        .assert()
        .failure()
        .stdout(predicate::str::is_empty());

    for (field, value) in [
        ("$schema", json!("https://example.invalid/input.json")),
        ("kind", json!("extract.axi.source")),
        ("profile", Value::Null),
        ("include_stall", Value::Null),
        ("name", Value::Null),
    ] {
        let mut invalid = base.clone();
        invalid[field] = value;
        let invalid = write_source(&invalid);
        wavepeek_cmd()
            .args([
                "extract",
                "ahb",
                "--waves",
                waveform.as_str(),
                "--source",
                invalid.path().to_string_lossy().as_ref(),
            ])
            .assert()
            .failure()
            .stdout(predicate::str::is_empty());
    }
}

#[test]
fn extract_ahb_output_schema_gates_optional_events_by_context_flags() {
    let output = lite_command("vcd")
        .arg("--json")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let mut value = parse_json(&output);
    value["data"]["events"]
        .as_array_mut()
        .expect("events")
        .push(json!({
            "time": "20ns",
            "sample_time": "19ns",
            "profile": "ahb-lite",
            "event": "data-stall",
            "direction": "read",
            "payload": {"hresp": "1'h0"}
        }));
    assert!(schema_validator("output.json").validate(&value).is_err());
}
