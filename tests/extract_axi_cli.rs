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

fn output_schema_validator() -> jsonschema::Validator {
    let schema_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("schema")
        .join("output.json");
    let schema: Value =
        serde_json::from_str(&fs::read_to_string(schema_path).expect("output schema should read"))
            .expect("output schema should parse");
    jsonschema::validator_for(&schema).expect("output schema should compile")
}

fn stream_schema_validator() -> jsonschema::Validator {
    let schema_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("schema")
        .join("stream.json");
    let schema: Value =
        serde_json::from_str(&fs::read_to_string(schema_path).expect("stream schema should read"))
            .expect("stream schema should parse");
    jsonschema::validator_for(&schema).expect("stream schema should compile")
}

fn parse_json(stdout: &[u8]) -> Value {
    let value: Value = serde_json::from_slice(stdout).expect("stdout should be valid json");
    output_schema_validator()
        .validate(&value)
        .unwrap_or_else(|error| panic!("output should validate: {error}\n{value}"));
    value
}

fn waveform_fixture(filename: &str) -> String {
    fixture_path(filename).to_string_lossy().into_owned()
}

fn write_source(contents: &str) -> NamedTempFile {
    let source =
        NamedTempFile::with_suffix("extract-axi-source.json").expect("source should create");
    fs::write(source.path(), contents).expect("source should write");
    source
}

fn human_transfer_channels(output: &str) -> Vec<&str> {
    output
        .lines()
        .filter(|line| line.starts_with('@'))
        .map(|line| {
            line.split_once('[')
                .and_then(|(_, rest)| rest.split_once(']'))
                .map(|(channel, _)| channel)
                .expect("transfer row should contain a channel")
        })
        .collect()
}

fn parse_stream(stdout: &[u8]) -> Vec<Value> {
    let output = std::str::from_utf8(stdout).expect("stdout should be UTF-8 JSONL");
    assert!(output.ends_with('\n'));
    let validator = stream_schema_validator();
    output
        .lines()
        .map(|line| {
            let record: Value = serde_json::from_str(line).expect("JSONL line should parse");
            validator
                .validate(&record)
                .unwrap_or_else(|error| panic!("record should validate: {error}\n{record}"));
            record
        })
        .collect()
}

#[test]
fn extract_axi5_human_automaps_issue_l_base_and_dvm_channels() {
    let fixture = waveform_fixture("extract_axi5.vcd");
    let output = wavepeek_cmd()
        .args([
            "extract",
            "axi",
            "--waves",
            fixture.as_str(),
            "--scope",
            "top",
            "--profile",
            "axi5",
            "--map",
            "aclk=clk",
            "--include",
            "^axi5_.*",
        ])
        .output()
        .expect("extract axi5 should execute");

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8(output.stderr).expect("diagnostics should be UTF-8");
    for decoy in [
        "axi5_aw_pending_o",
        "axi5_aw_valid_chk_o",
        "axi5_cd_valid_o",
        "axi5_awakeup_o",
        "axi5_varqosaccept_i",
        "axi5_syscoreq_o",
        "axi5_broadcastatomic_i",
        "axi5_activatereq_o",
    ] {
        assert!(
            stderr.contains(&format!("ignored AXI include candidate '{decoy}'")),
            "missing decoy diagnostic for {decoy}:\n{stderr}"
        );
    }

    let stdout = String::from_utf8(output.stdout).expect("human output should be UTF-8");
    assert!(stdout.contains("profile: axi5\nissue: L"));
    assert_eq!(
        human_transfer_channels(&stdout),
        ["aw", "w", "b", "ar", "r", "ac", "cr"]
    );
    for expected in [
        "@5ns sample@4ns [aw] awprot=3'h5",
        "awnse=1'h1",
        "awmmuvalid=1'h1",
        "awmecid=16'ha55a",
        "awactv=1'h1",
        "@10ns sample@9ns [w] wtagupdate=4'hc",
        "@15ns sample@14ns [b] btagmatch=2'h2",
        "@20ns sample@19ns [ar] armecid=16'hb66b archunken=1'h1",
        "@25ns sample@24ns [r] rchunknum=5'h07",
        "@30ns sample@29ns [ac] acaddr=32'h12345678 acvmidext=4'h9",
        "@35ns sample@34ns [cr] crtrace=1'h1",
    ] {
        assert!(
            stdout.contains(expected),
            "missing `{expected}` in:\n{stdout}"
        );
    }
}

#[test]
fn extract_axi5_lite_cli_alias_automaps_five_single_transfer_channels() {
    let fixture = waveform_fixture("extract_axi5_lite.vcd");
    let output = wavepeek_cmd()
        .args([
            "extract",
            "axi",
            "--waves",
            fixture.as_str(),
            "--scope",
            "top",
            "--profile",
            "AXI5_LITE",
            "--map",
            "aclk=clk",
            "--include",
            "^axi5_lite_.*",
        ])
        .output()
        .expect("extract axi5-lite should execute");

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8(output.stderr).expect("diagnostics should be UTF-8");
    for decoy in [
        "axi5_lite_w_last_o",
        "axi5_lite_r_last_i",
        "axi5_lite_ac_valid_i",
        "axi5_lite_aw_pending_o",
        "axi5_lite_aw_valid_chk_o",
    ] {
        assert!(
            stderr.contains(&format!("ignored AXI include candidate '{decoy}'")),
            "missing decoy diagnostic for {decoy}:\n{stderr}"
        );
    }

    let stdout = String::from_utf8(output.stdout).expect("human output should be UTF-8");
    assert!(stdout.contains("profile: axi5-lite\nissue: L"));
    assert_eq!(
        human_transfer_channels(&stdout),
        ["aw", "w", "b", "ar", "r"]
    );
    for expected in [
        "@5ns sample@4ns [aw] awid=4'h1 awaddr=8'h24 awsize=3'h0 awidunq=1'h1",
        "@10ns sample@9ns [w] wdata=8'ha5 wpoison=1'h1",
        "@15ns sample@14ns [b] bresp=2'h2",
        "@20ns sample@19ns [ar] arid=4'h2 araddr=8'h48 arsize=3'h0 aridunq=1'h1",
        "@25ns sample@24ns [r] rdata=8'h5a rpoison=1'h1",
    ] {
        assert!(
            stdout.contains(expected),
            "missing `{expected}` in:\n{stdout}"
        );
    }
    assert!(!stdout.contains("wlast="));
    assert!(!stdout.contains("rlast="));
    assert!(!stdout.contains("[ac]"));
}

#[test]
fn extract_axi5_json_validates_issue_l_channels_and_payloads() {
    let fixture = waveform_fixture("extract_axi5.vcd");
    let output = wavepeek_cmd()
        .args([
            "extract",
            "axi",
            "--waves",
            fixture.as_str(),
            "--scope",
            "top",
            "--profile",
            "axi5",
            "--map",
            "aclk=clk",
            "--include",
            "^axi5_.*",
            "--json",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let value = parse_json(&output);
    assert_eq!(value["data"]["profile"], "axi5");
    assert_eq!(value["data"]["issue"], "L");
    let transfers = value["data"]["transfers"].as_array().unwrap();
    assert_eq!(
        transfers
            .iter()
            .map(|row| row["channel"].as_str().unwrap())
            .collect::<Vec<_>>(),
        ["aw", "w", "b", "ar", "r", "ac", "cr"]
    );
    assert!(transfers.iter().all(|row| row["profile"] == "axi5"));
    assert_eq!(transfers[0]["time"], "5ns");
    assert_eq!(transfers[0]["payload"]["awmecid"], "16'ha55a");
    assert_eq!(transfers[2]["payload"]["btagmatch"], "2'h2");
    assert_eq!(transfers[4]["payload"]["rchunknum"], "5'h07");
    assert_eq!(transfers[5]["payload"]["acaddr"], "32'h12345678");
    assert_eq!(transfers[6]["payload"]["crtrace"], "1'h1");
}

#[test]
fn extract_axi5_lite_jsonl_validates_issue_l_context_and_payloads() {
    let fixture = waveform_fixture("extract_axi5_lite.vcd");
    let output = wavepeek_cmd()
        .args([
            "extract",
            "axi",
            "--waves",
            fixture.as_str(),
            "--scope",
            "top",
            "--profile",
            "axi5-lite",
            "--map",
            "aclk=clk",
            "--include",
            "^axi5_lite_.*",
            "--jsonl",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let records = parse_stream(&output);
    assert_eq!(records.first().unwrap()["type"], "begin");
    assert_eq!(records.first().unwrap()["context"]["profile"], "axi5-lite");
    assert_eq!(records.first().unwrap()["context"]["issue"], "L");
    let items = records
        .iter()
        .filter(|record| record["type"] == "item")
        .collect::<Vec<_>>();
    assert_eq!(
        items
            .iter()
            .map(|record| record["item"]["channel"].as_str().unwrap())
            .collect::<Vec<_>>(),
        ["aw", "w", "b", "ar", "r"]
    );
    assert!(
        items
            .iter()
            .all(|record| record["item"]["profile"] == "axi5-lite")
    );
    assert_eq!(items[0]["item"]["payload"]["awidunq"], "1'h1");
    assert_eq!(items[1]["item"]["payload"]["wpoison"], "1'h1");
    assert_eq!(items[4]["item"]["payload"]["rpoison"], "1'h1");
    assert_eq!(records.last().unwrap()["type"], "end");
    assert_eq!(records.last().unwrap()["summary"]["items"], 5);
}

#[test]
fn extract_axi5_lite_source_accepts_hyphen_and_underscore_aliases() {
    let fixture = waveform_fixture("extract_axi5_lite.vcd");

    for profile in ["axi5-lite", "AXI5_LITE", "axi5_lite"] {
        let source = write_source(&format!(
            r#"{{
  "$schema": "{}",
  "kind": "extract.axi.source",
  "profile": "{profile}",
  "includes": ["^axi5_lite_(aw|w|b|ar|r)_"],
  "maps": {{"aclk": "clk"}}
}}"#,
            expected_input_schema_url()
        ));
        let source = source.path().to_string_lossy().into_owned();
        let output = wavepeek_cmd()
            .args([
                "extract",
                "axi",
                "--waves",
                fixture.as_str(),
                "--scope",
                "top",
                "--source",
                source.as_str(),
            ])
            .output()
            .expect("extract axi5-lite source should execute");

        assert!(
            output.status.success(),
            "profile {profile}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        let stdout = String::from_utf8(output.stdout).expect("human output should be UTF-8");
        assert!(stdout.contains("profile: axi5-lite\nissue: L"));
        assert_eq!(
            human_transfer_channels(&stdout),
            ["aw", "w", "b", "ar", "r"]
        );
    }
}

#[test]
fn extract_axi5_profiles_reject_out_of_profile_mappings() {
    for (profile, fixture_name, standard) in [
        ("axi5", "extract_axi5.vcd", "awbar"),
        ("axi5", "extract_axi5.vcd", "awunique"),
        ("axi5", "extract_axi5.vcd", "arbar"),
        ("axi5", "extract_axi5.vcd", "cdvalid"),
        ("axi5", "extract_axi5.vcd", "awpending"),
        ("axi5", "extract_axi5.vcd", "awakeup"),
        ("axi5", "extract_axi5.vcd", "varqosaccept"),
        ("axi5", "extract_axi5.vcd", "syscoreq"),
        ("axi5", "extract_axi5.vcd", "broadcastatomic"),
        ("axi5", "extract_axi5.vcd", "activatereq"),
        ("axi5-lite", "extract_axi5_lite.vcd", "awlen"),
        ("axi5-lite", "extract_axi5_lite.vcd", "awburst"),
        ("axi5-lite", "extract_axi5_lite.vcd", "awcache"),
        ("axi5-lite", "extract_axi5_lite.vcd", "wlast"),
        ("axi5-lite", "extract_axi5_lite.vcd", "rlast"),
        ("axi5-lite", "extract_axi5_lite.vcd", "arsnoop"),
        ("axi5-lite", "extract_axi5_lite.vcd", "acvalid"),
        ("axi5-lite", "extract_axi5_lite.vcd", "awpending"),
    ] {
        let fixture = waveform_fixture(fixture_name);
        wavepeek_cmd()
            .args([
                "extract",
                "axi",
                "--waves",
                fixture.as_str(),
                "--scope",
                "top",
                "--profile",
                profile,
                "--map",
                "aclk=clk",
                "--map",
                &format!("{standard}=clk"),
            ])
            .assert()
            .failure()
            .stderr(predicate::str::contains(format!(
                "AXI profile {profile} has no standard signal '{standard}'"
            )));
    }
}

#[test]
fn extract_axi5_profiles_match_between_vcd_and_fst() {
    for (profile, fixture_stem, include, expected_channels) in [
        (
            "axi5",
            "extract_axi5",
            "^axi5_.*",
            &["aw", "w", "b", "ar", "r", "ac", "cr"][..],
        ),
        (
            "axi5-lite",
            "extract_axi5_lite",
            "^axi5_lite_.*",
            &["aw", "w", "b", "ar", "r"][..],
        ),
    ] {
        let mut outputs = Vec::new();
        for extension in ["vcd", "fst"] {
            let fixture = waveform_fixture(&format!("{fixture_stem}.{extension}"));
            let output = wavepeek_cmd()
                .args([
                    "extract",
                    "axi",
                    "--waves",
                    fixture.as_str(),
                    "--scope",
                    "top",
                    "--profile",
                    profile,
                    "--map",
                    "aclk=clk",
                    "--include",
                    include,
                ])
                .output()
                .expect("cross-format AXI extraction should execute");
            assert!(
                output.status.success(),
                "{profile} {extension}: {}",
                String::from_utf8_lossy(&output.stderr)
            );
            let stdout = String::from_utf8(output.stdout.clone()).expect("stdout should be UTF-8");
            assert_eq!(human_transfer_channels(&stdout), expected_channels);
            outputs.push(output);
        }
        assert_eq!(outputs[0].stdout, outputs[1].stdout, "{profile} stdout");
        assert_eq!(outputs[0].stderr, outputs[1].stderr, "{profile} stderr");
    }
}

#[test]
fn extract_ace_human_automaps_base_and_coherency_channels() {
    let fixture = waveform_fixture("extract_ace.vcd");
    let output = wavepeek_cmd()
        .args([
            "extract",
            "axi",
            "--waves",
            fixture.as_str(),
            "--scope",
            "top",
            "--profile",
            "ace",
            "--map",
            "aclk=clk",
            "--include",
            "^ace_.*",
        ])
        .output()
        .expect("extract ace should execute");

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stderr.is_empty());
    let stdout = String::from_utf8(output.stdout).expect("human output should be UTF-8");
    assert!(stdout.contains("profile: ace\nissue: H.c"));
    assert_eq!(
        human_transfer_channels(&stdout),
        ["aw", "w", "b", "ar", "r", "ac", "cr", "cd"]
    );
    assert!(stdout.contains("[aw] awdomain=2'h2 awunique=1'h1"));
    assert!(stdout.contains("[ac] acaddr=16'h1234"));
    assert!(stdout.contains("[cr] crresp=5'h15"));
    assert!(stdout.contains("[cd] cddata=8'hc3 cdlast=1'h1"));
}

#[test]
fn extract_ace_lite_cli_alias_automaps_address_additions_only() {
    let fixture = waveform_fixture("extract_ace_lite.vcd");
    let output = wavepeek_cmd()
        .args([
            "extract",
            "axi",
            "--waves",
            fixture.as_str(),
            "--scope",
            "top",
            "--profile",
            "ACE_LITE",
            "--map",
            "aclk=clk",
            "--include",
            "^ace_lite_.*",
        ])
        .output()
        .expect("extract ace-lite should execute");

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stderr.is_empty());
    let stdout = String::from_utf8(output.stdout).expect("human output should be UTF-8");
    assert!(stdout.contains("profile: ace-lite\nissue: H.c"));
    assert_eq!(human_transfer_channels(&stdout), ["aw", "ar"]);
    assert!(stdout.contains("[aw] awdomain=2'h2 awsnoop=3'h0 awbar=2'h0 awunique=1'h1"));
    assert!(stdout.contains("[ar] ardomain=2'h1 arsnoop=4'h0 arbar=2'h1"));
    assert!(!stdout.contains("[ac]"));
    assert!(!stdout.contains("[cr]"));
    assert!(!stdout.contains("[cd]"));
}

#[test]
fn extract_ace_lite_source_accepts_hyphen_and_underscore_aliases() {
    let fixture = waveform_fixture("extract_ace_lite.vcd");

    for profile in ["ace-lite", "ACE_LITE", "ace_lite"] {
        let source = write_source(&format!(
            r#"{{
  "$schema": "{}",
  "kind": "extract.axi.source",
  "profile": "{profile}",
  "includes": ["^ace_lite_.*"],
  "maps": {{"aclk": "clk"}}
}}"#,
            expected_input_schema_url()
        ));
        let source = source.path().to_string_lossy().into_owned();
        let output = wavepeek_cmd()
            .args([
                "extract",
                "axi",
                "--waves",
                fixture.as_str(),
                "--scope",
                "top",
                "--source",
                source.as_str(),
            ])
            .output()
            .expect("extract ace-lite source should execute");

        assert!(
            output.status.success(),
            "profile {profile}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(output.stderr.is_empty());
        let stdout = String::from_utf8(output.stdout).expect("human output should be UTF-8");
        assert!(stdout.contains("profile: ace-lite\nissue: H.c"));
        assert_eq!(human_transfer_channels(&stdout), ["aw", "ar"]);
    }
}

#[test]
fn extract_ace5_human_automaps_representative_optional_payloads() {
    let fixture = waveform_fixture("extract_ace5.vcd");
    let output = wavepeek_cmd()
        .args([
            "extract",
            "axi",
            "--waves",
            fixture.as_str(),
            "--scope",
            "top",
            "--profile",
            "ace5",
            "--map",
            "aclk=clk",
            "--include",
            "^ace5_.*",
        ])
        .output()
        .expect("extract ace5 should execute");

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8(output.stderr).expect("diagnostics should be UTF-8");
    for check_signal in [
        "ace5_ac_valid_chk_i",
        "ace5_cr_valid_chk_o",
        "ace5_cd_valid_chk_o",
    ] {
        assert!(
            stderr.contains(&format!("ignored AXI include candidate '{check_signal}'")),
            "missing check-signal diagnostic for {check_signal}:\n{stderr}"
        );
    }
    let stdout = String::from_utf8(output.stdout).expect("human output should be UTF-8");
    assert!(stdout.contains("profile: ace5\nissue: H.c"));
    assert_eq!(
        human_transfer_channels(&stdout),
        ["aw", "w", "b", "ar", "r", "ac", "cr", "cd"]
    );
    for expected in [
        "[aw] awtrace=1'h1",
        "awidunq=1'h1",
        "[w] wpoison=1'h1",
        "[b] bidunq=1'h1",
        "[ar] arvmidext=4'hd",
        "aridunq=1'h1",
        "[r] rpoison=1'h1",
        "ridunq=1'h1",
        "[ac] acvmidext=4'ha",
        "[cr] crnsaid=4'h7",
        "[cd] cdpoison=1'h1",
    ] {
        assert!(
            stdout.contains(expected),
            "missing `{expected}` in:\n{stdout}"
        );
    }
    for truncated in [" awid=1'h1", " arid=1'h1", " bid=1'h1", " rid=1'h1"] {
        assert!(
            !stdout.contains(truncated),
            "split unique-ID signal was truncated to `{truncated}` in:\n{stdout}"
        );
    }
}

#[test]
fn extract_ace5_rejects_removed_barrier_mappings() {
    let fixture = waveform_fixture("extract_ace5.vcd");

    for standard in ["awbar", "arbar"] {
        wavepeek_cmd()
            .args([
                "extract",
                "axi",
                "--waves",
                fixture.as_str(),
                "--scope",
                "top",
                "--profile",
                "ace5",
                "--map",
                "aclk=clk",
                "--map",
                &format!("{standard}=clk"),
            ])
            .assert()
            .failure()
            .stderr(predicate::str::contains(format!(
                "AXI profile ace5 has no standard signal '{standard}'"
            )));
    }
}

#[test]
fn extract_ace_json_validates_profile_channels_and_payloads() {
    let fixture = waveform_fixture("extract_ace.vcd");
    let output = wavepeek_cmd()
        .args([
            "extract",
            "axi",
            "--waves",
            fixture.as_str(),
            "--scope",
            "top",
            "--profile",
            "ace",
            "--map",
            "aclk=clk",
            "--include",
            "^ace_.*",
            "--json",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let value = parse_json(&output);
    assert_eq!(value["data"]["profile"], "ace");
    assert_eq!(value["data"]["issue"], "H.c");
    let transfers = value["data"]["transfers"].as_array().unwrap();
    assert_eq!(
        transfers
            .iter()
            .map(|row| row["channel"].as_str().unwrap())
            .collect::<Vec<_>>(),
        ["aw", "w", "b", "ar", "r", "ac", "cr", "cd"]
    );
    assert!(transfers.iter().all(|row| row["profile"] == "ace"));
    assert_eq!(transfers[0]["payload"]["awunique"], "1'h1");
    assert_eq!(transfers[4]["payload"]["rresp"], "4'hd");
    assert_eq!(transfers[5]["payload"]["acaddr"], "16'h1234");
    assert_eq!(transfers[6]["payload"]["crresp"], "5'h15");
    assert_eq!(transfers[7]["payload"]["cdlast"], "1'h1");
}

#[test]
fn extract_ace_lite_json_validates_awunique_without_coherency_channels() {
    let fixture = waveform_fixture("extract_ace_lite.vcd");
    let output = wavepeek_cmd()
        .args([
            "extract",
            "axi",
            "--waves",
            fixture.as_str(),
            "--scope",
            "top",
            "--profile",
            "ace-lite",
            "--map",
            "aclk=clk",
            "--include",
            "^ace_lite_.*",
            "--json",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let value = parse_json(&output);
    assert_eq!(value["data"]["profile"], "ace-lite");
    let transfers = value["data"]["transfers"].as_array().unwrap();
    assert_eq!(
        transfers
            .iter()
            .map(|row| row["channel"].as_str().unwrap())
            .collect::<Vec<_>>(),
        ["aw", "ar"]
    );
    assert_eq!(transfers[0]["payload"]["awunique"], "1'h1");
    assert_eq!(transfers[1]["payload"]["arbar"], "2'h1");
}

#[test]
fn extract_ace5_jsonl_validates_context_and_optional_payloads() {
    let fixture = waveform_fixture("extract_ace5.vcd");
    let output = wavepeek_cmd()
        .args([
            "extract",
            "axi",
            "--waves",
            fixture.as_str(),
            "--scope",
            "top",
            "--profile",
            "ace5",
            "--map",
            "aclk=clk",
            "--include",
            "^ace5_.*",
            "--jsonl",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let records = parse_stream(&output);
    assert_eq!(records.first().unwrap()["type"], "begin");
    assert_eq!(records.first().unwrap()["context"]["profile"], "ace5");
    let items = records
        .iter()
        .filter(|record| record["type"] == "item")
        .collect::<Vec<_>>();
    assert_eq!(
        items
            .iter()
            .map(|record| record["item"]["channel"].as_str().unwrap())
            .collect::<Vec<_>>(),
        ["aw", "w", "b", "ar", "r", "ac", "cr", "cd"]
    );
    assert!(
        items
            .iter()
            .all(|record| record["item"]["profile"] == "ace5")
    );
    assert_eq!(items[0]["item"]["payload"]["awtrace"], "1'h1");
    assert_eq!(items[0]["item"]["payload"]["awidunq"], "1'h1");
    assert_eq!(items[2]["item"]["payload"]["bidunq"], "1'h1");
    assert_eq!(items[3]["item"]["payload"]["aridunq"], "1'h1");
    assert_eq!(items[4]["item"]["payload"]["ridunq"], "1'h1");
    assert_eq!(items[5]["item"]["payload"]["acvmidext"], "4'ha");
    assert_eq!(items[7]["item"]["payload"]["cdpoison"], "1'h1");
    assert_eq!(
        records
            .iter()
            .filter(|record| record["type"] == "diagnostic")
            .count(),
        3
    );
    assert_eq!(records.last().unwrap()["type"], "end");
    assert_eq!(records.last().unwrap()["summary"]["items"], 8);
}

#[test]
fn extract_ace_and_ace_lite_jsonl_validate_profile_branches() {
    for (profile, fixture_name, include, expected_channels) in [
        (
            "ace",
            "extract_ace.vcd",
            "^ace_.*",
            &["aw", "w", "b", "ar", "r", "ac", "cr", "cd"][..],
        ),
        (
            "ace-lite",
            "extract_ace_lite.vcd",
            "^ace_lite_.*",
            &["aw", "ar"][..],
        ),
    ] {
        let fixture = waveform_fixture(fixture_name);
        let output = wavepeek_cmd()
            .args([
                "extract",
                "axi",
                "--waves",
                fixture.as_str(),
                "--scope",
                "top",
                "--profile",
                profile,
                "--map",
                "aclk=clk",
                "--include",
                include,
                "--jsonl",
            ])
            .assert()
            .success()
            .get_output()
            .stdout
            .clone();

        let records = parse_stream(&output);
        assert_eq!(records.first().unwrap()["context"]["profile"], profile);
        let items = records
            .iter()
            .filter(|record| record["type"] == "item")
            .collect::<Vec<_>>();
        assert_eq!(
            items
                .iter()
                .map(|record| record["item"]["channel"].as_str().unwrap())
                .collect::<Vec<_>>(),
            expected_channels
        );
        assert!(
            items
                .iter()
                .all(|record| record["item"]["profile"] == profile)
        );
    }
}

#[test]
fn extract_axi_json_automaps_axi4_lite_and_gates_reset() {
    let fixture = waveform_fixture("extract_axi_lite.vcd");

    let output = wavepeek_cmd()
        .args([
            "extract",
            "axi",
            "--waves",
            fixture.as_str(),
            "--scope",
            "top",
            "--profile",
            "axi4-lite",
            "--map",
            "aclk=clk",
            "--map",
            "aresetn=aresetn",
            "--include",
            "^axi_(aw|w|b|ar|r)_",
            "--json",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let value = parse_json(&output);
    assert_eq!(value["$schema"], expected_schema_url());
    assert_eq!(value["command"], "extract axi");
    assert_eq!(value["diagnostics"], json!([]));
    assert_eq!(value["data"]["name"], "axi");
    assert_eq!(value["data"]["profile"], "axi4-lite");
    assert_eq!(value["data"]["issue"], "H.c");
    assert_eq!(
        value["data"]["mappings"]["awvalid"]["path"],
        "top.axi_aw_valid_o"
    );

    let transfers = value["data"]["transfers"].as_array().unwrap();
    assert_eq!(transfers.len(), 5);
    assert_eq!(
        transfers
            .iter()
            .map(|row| row["channel"].as_str().unwrap())
            .collect::<Vec<_>>(),
        vec!["aw", "w", "b", "ar", "r"]
    );
    assert!(transfers.iter().all(|row| row["profile"] == "axi4-lite"));
    assert_eq!(transfers[0]["time"], "5ns");
    assert_eq!(transfers[0]["sample_time"], "4ns");
    assert_eq!(transfers[0]["payload"]["awaddr"], "8'h12");
    assert_eq!(transfers[1]["payload"]["wdata"], "8'haa");
    assert_eq!(transfers[4]["payload"]["rresp"], "2'h2");
}

#[test]
fn extract_axi_human_defaults_to_axi4() {
    let fixture = waveform_fixture("extract_axi_lite.vcd");

    wavepeek_cmd()
        .args([
            "extract",
            "axi",
            "--waves",
            fixture.as_str(),
            "--scope",
            "top",
            "--map",
            "aclk=clk",
            "--map",
            "aresetn=aresetn",
            "--include",
            "^axi_(aw|w|b|ar|r)_",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("profile: axi4"))
        .stdout(predicate::str::contains("mappings:\n  aclk = clk"))
        .stdout(predicate::str::contains(
            "@5ns sample@4ns [aw] awaddr=8'h12 awprot=3'h2",
        ));
}

#[test]
fn extract_axi3_profile_extracts_wid() {
    let fixture = waveform_fixture("extract_axi3_w.vcd");

    let output = wavepeek_cmd()
        .args([
            "extract",
            "axi",
            "--waves",
            fixture.as_str(),
            "--scope",
            "top",
            "--profile",
            "axi3",
            "--map",
            "aclk=clk",
            "--include",
            "^axi_w",
            "--json",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let value = parse_json(&output);
    assert_eq!(value["data"]["profile"], "axi3");
    assert_eq!(value["data"]["transfers"].as_array().unwrap().len(), 1);
    assert_eq!(value["data"]["transfers"][0]["profile"], "axi3");
    assert_eq!(value["data"]["transfers"][0]["channel"], "w");
    assert_eq!(value["data"]["transfers"][0]["payload"]["wid"], "4'ha");
    assert_eq!(value["data"]["transfers"][0]["payload"]["wdata"], "8'hcc");
}

#[test]
fn extract_axi_source_jsonl_includes_begin_context() {
    let fixture_path = waveform_fixture("extract_axi_lite.vcd");
    let source = write_source(&format!(
        r#"{{
  "$schema": "{}",
  "kind": "extract.axi.source",
  "profile": "axi4-lite",
  "name": "cfg",
  "includes": ["^axi_(aw|w|b|ar|r)_"],
  "maps": {{"aclk": "clk", "aresetn": "aresetn"}}
}}"#,
        expected_input_schema_url()
    ));
    let source_path = source.path().to_string_lossy().into_owned();

    let output = wavepeek_cmd()
        .args([
            "extract",
            "axi",
            "--waves",
            fixture_path.as_str(),
            "--scope",
            "top",
            "--source",
            source_path.as_str(),
            "--jsonl",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let records = parse_stream(&output);
    assert_eq!(records.first().unwrap()["type"], "begin");
    assert_eq!(
        records.first().unwrap()["$schema"],
        expected_stream_schema_url()
    );
    assert_eq!(records.first().unwrap()["context"]["name"], "cfg");
    assert_eq!(records.first().unwrap()["context"]["profile"], "axi4-lite");
    assert_eq!(records[1]["type"], "item");
    assert_eq!(records[1]["item"]["profile"], "axi4-lite");
    assert_eq!(records[1]["item"]["channel"], "aw");
    assert_eq!(records.last().unwrap()["type"], "end");
    assert_eq!(records.last().unwrap()["summary"]["items"], 5);
}

#[test]
fn extract_axi_profile_flag_accepts_case_insensitive_alias() {
    let fixture = waveform_fixture("extract_axi3_w.vcd");

    let output = wavepeek_cmd()
        .args([
            "extract",
            "axi",
            "--waves",
            fixture.as_str(),
            "--scope",
            "top",
            "--profile",
            "AXI4_LITE",
            "--map",
            "aclk=clk",
            "--include",
            "^axi_w",
            "--json",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let value = parse_json(&output);
    assert_eq!(value["data"]["profile"], "axi4-lite");
}

#[test]
fn extract_axi_reuses_mapping_waveform_for_execution() {
    let fixture = waveform_fixture("extract_axi_lite.vcd");

    let output = wavepeek_cmd()
        .env("DEBUG", "1")
        .args([
            "extract",
            "axi",
            "--waves",
            fixture.as_str(),
            "--scope",
            "top",
            "--profile",
            "axi4-lite",
            "--map",
            "aclk=clk",
            "--map",
            "aresetn=aresetn",
            "--include",
            "^axi_(aw|w|b|ar|r)_",
            "--max",
            "1",
            "--json",
        ])
        .assert()
        .success()
        .get_output()
        .clone();

    let stderr = std::str::from_utf8(&output.stderr).expect("debug stderr should be UTF-8");
    assert_eq!(stderr.matches("backend.open.start").count(), 1);
    assert_eq!(stderr.matches("backend.open.done").count(), 1);
}

#[test]
fn extract_axi_source_rejects_explicit_null_strings() {
    let fixture = waveform_fixture("extract_axi_lite.vcd");

    for contents in [
        r#"{
  "$schema": "https://kleverhq.github.io/wavepeek/schema-input-v2.2.json",
  "kind": "extract.axi.source",
  "profile": null
}
"#,
        r#"{
  "$schema": "https://kleverhq.github.io/wavepeek/schema-input-v2.2.json",
  "kind": "extract.axi.source",
  "name": null
}
"#,
    ] {
        let source = write_source(contents);
        let source = source.path().to_string_lossy().into_owned();

        wavepeek_cmd()
            .args([
                "extract",
                "axi",
                "--waves",
                fixture.as_str(),
                "--scope",
                "top",
                "--source",
                source.as_str(),
            ])
            .assert()
            .failure()
            .stderr(predicate::str::contains("expected string, got null"));
    }
}

#[test]
fn extract_axi_source_rejects_legacy_generic_schema_url() {
    let fixture = waveform_fixture("extract_axi_lite.vcd");
    let source = write_source(
        r#"{
  "$schema": "https://kleverhq.github.io/wavepeek/schema-input-v2.1.json",
  "kind": "extract.axi.source",
  "profile": "axi4-lite",
  "maps": {"aclk": "clk"}
}
"#,
    );
    let source = source.path().to_string_lossy().into_owned();

    wavepeek_cmd()
        .args([
            "extract",
            "axi",
            "--waves",
            fixture.as_str(),
            "--scope",
            "top",
            "--source",
            source.as_str(),
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("uses unsupported $schema"));
}

#[test]
fn extract_axi_source_conflicts_with_explicit_profile() {
    let fixture_path = waveform_fixture("extract_axi_lite.vcd");
    let source = write_source(&format!(
        r#"{{"$schema":"{}","kind":"extract.axi.source","maps":{{"aclk":"clk"}}}}"#,
        expected_input_schema_url()
    ));
    let source_path = source.path().to_string_lossy().into_owned();

    wavepeek_cmd()
        .args([
            "extract",
            "axi",
            "--waves",
            fixture_path.as_str(),
            "--scope",
            "top",
            "--source",
            source_path.as_str(),
            "--profile",
            "axi3",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("cannot be used with"));
}

#[test]
fn extract_axi_warns_for_unmatched_include_candidates() {
    let fixture = waveform_fixture("extract_axi_lite.vcd");

    let output = wavepeek_cmd()
        .args([
            "extract",
            "axi",
            "--waves",
            fixture.as_str(),
            "--scope",
            "top",
            "--map",
            "aclk=clk",
            "--map",
            "aresetn=aresetn",
            "--map",
            "awvalid=axi_aw_valid_o",
            "--map",
            "awready=axi_aw_ready_i",
            "--include",
            "^axi_misc_o$",
            "--json",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let value = parse_json(&output);
    assert_eq!(value["data"]["transfers"].as_array().unwrap().len(), 1);
    assert_eq!(value["data"]["transfers"][0]["payload"], json!({}));
    assert_eq!(value["diagnostics"][0]["code"], "WPK-W0004");
}

#[test]
fn extract_axi_does_not_warn_for_explicitly_mapped_include_path() {
    let fixture = waveform_fixture("extract_axi3_w.vcd");

    let output = wavepeek_cmd()
        .args([
            "extract",
            "axi",
            "--waves",
            fixture.as_str(),
            "--scope",
            "top",
            "--profile",
            "axi3",
            "--map",
            "aclk=clk",
            "--include",
            "^(clk|axi_w.*)$",
            "--json",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let value = parse_json(&output);
    assert_eq!(value["diagnostics"], json!([]));
}

#[test]
fn extract_axi_rejects_single_candidate_matching_multiple_standards() {
    let fixture = waveform_fixture("extract_axi_multi_match.vcd");

    wavepeek_cmd()
        .args([
            "extract",
            "axi",
            "--waves",
            fixture.as_str(),
            "--scope",
            "top",
            "--map",
            "aclk=clk",
            "--include",
            ".*awvalid_awready.*",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "ambiguous AXI auto-mapping for 'axi_awvalid_awready'",
        ));
}

#[test]
fn extract_axi_rejects_ambiguous_auto_mapping() {
    let fixture = waveform_fixture("extract_axi_ambiguous.vcd");

    wavepeek_cmd()
        .args([
            "extract",
            "axi",
            "--waves",
            fixture.as_str(),
            "--scope",
            "top",
            "--map",
            "aclk=clk",
            "--include",
            ".*awvalid.*",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "ambiguous AXI auto-mapping for 'awvalid'",
        ));
}

#[test]
fn extract_axi_reports_ambiguous_auto_mapping_in_standard_order() {
    let fixture = waveform_fixture("extract_axi_multi_ambiguous.vcd");

    wavepeek_cmd()
        .args([
            "extract",
            "axi",
            "--waves",
            fixture.as_str(),
            "--scope",
            "top",
            "--map",
            "aclk=clk",
            "--include",
            ".*valid.*",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "ambiguous AXI auto-mapping for 'awvalid'",
        ));
}

#[test]
fn extract_axi_rejects_partial_ready_valid_pairs() {
    let fixture = waveform_fixture("extract_axi_lite.vcd");

    wavepeek_cmd()
        .args([
            "extract",
            "axi",
            "--waves",
            fixture.as_str(),
            "--scope",
            "top",
            "--map",
            "aclk=clk",
            "--map",
            "awvalid=axi_aw_valid_o",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "AXI channel 'aw' must map both awvalid and awready",
        ));
}
