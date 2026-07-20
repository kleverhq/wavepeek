use std::fs;

use assert_cmd::prelude::*;
use predicates::prelude::*;
use serde_json::{Value, json};
use tempfile::NamedTempFile;

mod common;
use common::{fixture_path, wavepeek_cmd};

fn waveform_fixture(filename: &str) -> String {
    fixture_path(filename).to_string_lossy().into_owned()
}

fn schema_validator(name: &str) -> jsonschema::Validator {
    let schema_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("schema")
        .join(name);
    let schema: Value =
        serde_json::from_str(&fs::read_to_string(schema_path).expect("schema should read"))
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
            let record: Value = serde_json::from_str(line).expect("JSONL line should parse");
            validator
                .validate(&record)
                .unwrap_or_else(|error| panic!("record should validate: {error}\n{record}"));
            record
        })
        .collect()
}

fn write_source(value: &Value) -> NamedTempFile {
    let source =
        NamedTempFile::with_suffix("axistream-source.json").expect("source file should create");
    fs::write(
        source.path(),
        serde_json::to_vec_pretty(value).expect("source should serialize"),
    )
    .expect("source should write");
    source
}

fn mapped_args(waves: &str) -> Vec<&str> {
    vec![
        "extract",
        "axistream",
        "--waves",
        waves,
        "--scope",
        "top",
        "--map",
        "aclk=clk",
        "--map",
        "aresetn=rst_n",
        "--include",
        "^s_axis_",
    ]
}

#[test]
fn extract_axistream_help_exposes_profiles_and_tready_modes() {
    wavepeek_cmd()
        .args(["extract", "axistream", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Extract AXI-Stream transfer rows"))
        .stdout(predicate::str::contains("axi4-stream"))
        .stdout(predicate::str::contains("axi5-stream"))
        .stdout(predicate::str::contains("implicit-high"))
        .stdout(predicate::str::contains("--source <FILE>"));

    wavepeek_cmd()
        .args(["extract", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("axistream"))
        .stdout(predicate::str::contains("axi-stream").not());
}

#[test]
fn extract_axistream_human_defaults_and_automaps_functional_signals() {
    let fixture = waveform_fixture("extract_axistream.vcd");
    let output = wavepeek_cmd()
        .args(mapped_args(fixture.as_str()))
        .output()
        .expect("AXI-Stream extraction should execute");

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).expect("stdout should be UTF-8");
    assert!(stdout.starts_with(
        "name: axistream\nprofile: axi4-stream\nissue: B\ntready_mode: mapped\nmappings:\n"
    ));
    for mapping in [
        "  aclk = clk",
        "  aresetn = rst_n",
        "  tvalid = s_axis_t_valid_o",
        "  tready = s_axis_tready_i",
        "  tdata = s_axis_t_data_o",
        "  tstrb = s_axis_tstrb_o",
        "  tkeep = s_axis_t_keep_o",
        "  tlast = s_axis_tlast_o",
        "  tid = s_axis_tid_o",
        "  tdest = s_axis_t_dest_o",
        "  tuser = s_axis_tuser_o",
    ] {
        assert!(stdout.contains(mapping), "missing {mapping}:\n{stdout}");
    }
    assert!(!stdout.contains("[channel]"));
    assert!(!stdout.contains("twakeup="));
    assert_eq!(
        stdout.lines().filter(|line| line.starts_with('@')).count(),
        3
    );
    assert!(stdout.contains("@10ns sample@9ns tdata=32'h33333333"));
    assert!(stdout.contains("@20ns sample@19ns tdata=32'hdeadbeef"));
    assert!(stdout.contains("@25ns sample@24ns tdata=32'hdeadbeef"));

    let stderr = String::from_utf8(output.stderr).expect("stderr should be UTF-8");
    for decoy in [
        "s_axis_tvalidchk_o",
        "s_axis_t_ready_chk_i",
        "s_axis_tdatachk_o",
        "s_axis_twakeup_o",
    ] {
        assert!(
            stderr.contains(&format!("ignored AXI-Stream include candidate '{decoy}'")),
            "missing diagnostic for {decoy}:\n{stderr}"
        );
    }
}

#[test]
fn extract_axistream_json_profiles_and_jsonl_context_validate() {
    let fixture = waveform_fixture("extract_axistream.vcd");
    let output = wavepeek_cmd()
        .args([
            "extract",
            "axistream",
            "--waves",
            fixture.as_str(),
            "--scope",
            "top",
            "--profile",
            "AXI5_STREAM",
            "--tready-mode",
            "IMPLICIT_HIGH",
            "--name",
            "video_out",
            "--map",
            "aclk=clk",
            "--map",
            "aresetn=rst_n",
            "--include",
            "^m_axis_",
            "--json",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let value = parse_json(&output);
    assert_eq!(value["command"], "extract axistream");
    assert_eq!(value["data"]["name"], "video_out");
    assert_eq!(value["data"]["profile"], "axi5-stream");
    assert_eq!(value["data"]["issue"], "B");
    assert_eq!(value["data"]["tready_mode"], "implicit-high");
    assert!(value["data"]["mappings"].get("tready").is_none());
    assert_eq!(value["data"]["transfers"].as_array().unwrap().len(), 4);
    assert_eq!(
        value["data"]["transfers"][0],
        json!({
            "time": "10ns",
            "sample_time": "9ns",
            "profile": "axi5-stream",
            "payload": {"tdata": "16'h2002", "tlast": "1'h0"}
        })
    );

    let stream_output = wavepeek_cmd()
        .args([
            "extract",
            "axistream",
            "--waves",
            fixture.as_str(),
            "--scope",
            "top",
            "--profile",
            "axi4-stream",
            "--map",
            "aclk=clk",
            "--map",
            "aresetn=rst_n",
            "--map",
            "tvalid=s_axis_t_valid_o",
            "--map",
            "tready=s_axis_tready_i",
            "--map",
            "tdata=s_axis_t_data_o",
            "--max",
            "2",
            "--jsonl",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let records = parse_stream(&stream_output);
    assert_eq!(records[0]["type"], "begin");
    assert_eq!(records[0]["command"], "extract axistream");
    assert_eq!(records[0]["context"]["profile"], "axi4-stream");
    assert_eq!(records[0]["context"]["tready_mode"], "mapped");
    assert_eq!(records[1]["item"]["profile"], "axi4-stream");
    assert!(records[1]["item"].get("channel").is_none());
    assert_eq!(records.last().unwrap()["summary"]["items"], 2);
    assert_eq!(records.last().unwrap()["summary"]["truncated"], true);
}

#[test]
fn extract_axistream_vcd_and_fst_outputs_match_for_both_profiles() {
    let vcd = waveform_fixture("extract_axistream.vcd");
    let fst = waveform_fixture("extract_axistream.fst");
    let run = |waves: &str, profile: &str| {
        let mut args = mapped_args(waves);
        args.extend(["--profile", profile, "--json"]);
        let output = wavepeek_cmd()
            .args(args)
            .output()
            .expect("AXI-Stream extraction should execute");
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(output.stderr.is_empty());
        parse_json(&output.stdout)
    };

    for profile in ["axi4-stream", "axi5-stream"] {
        let vcd = run(vcd.as_str(), profile);
        let fst = run(fst.as_str(), profile);
        assert_eq!(vcd["data"], fst["data"]);
        assert_eq!(vcd["diagnostics"], fst["diagnostics"]);
    }
}

#[test]
fn extract_axistream_handshake_only_keeps_reset_gating_and_repeated_rows() {
    let fixture = waveform_fixture("extract_axistream.vcd");
    let value = wavepeek_cmd()
        .args([
            "extract",
            "axistream",
            "--waves",
            fixture.as_str(),
            "--scope",
            "top",
            "--map",
            "aclk=clk",
            "--map",
            "aresetn=rst_n",
            "--map",
            "tvalid=s_axis_t_valid_o",
            "--map",
            "tready=s_axis_tready_i",
            "--json",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let value = parse_json(&value);
    let rows = value["data"]["transfers"].as_array().unwrap();
    assert_eq!(rows.len(), 3);
    assert_eq!(rows[0]["time"], "10ns");
    assert_eq!(rows[0]["sample_time"], "9ns");
    assert_eq!(rows[0]["payload"], json!({}));
    assert_eq!(rows[1]["time"], "20ns");
    assert_eq!(rows[2]["time"], "25ns");
}

#[test]
fn extract_axistream_validates_tready_modes_and_required_mappings() {
    let fixture = waveform_fixture("extract_axistream.vcd");

    for (extra, expected) in [
        (
            vec!["--map", "aclk=clk", "--map", "tvalid=s_axis_t_valid_o"],
            "tready-mode mapped requires a tready mapping",
        ),
        (
            vec![
                "--tready-mode",
                "implicit-high",
                "--map",
                "aclk=clk",
                "--map",
                "tvalid=s_axis_t_valid_o",
                "--map",
                "tready=s_axis_tready_i",
            ],
            "tready-mode implicit-high forbids a tready mapping",
        ),
        (
            vec![
                "--map",
                "tvalid=s_axis_t_valid_o",
                "--map",
                "tready=s_axis_tready_i",
            ],
            "mapping requires aclk",
        ),
        (
            vec!["--map", "aclk=clk", "--map", "tdata=s_axis_t_data_o"],
            "mapping requires tvalid",
        ),
    ] {
        let mut args = vec![
            "extract",
            "axistream",
            "--waves",
            fixture.as_str(),
            "--scope",
            "top",
        ];
        args.extend(extra);
        wavepeek_cmd()
            .args(args)
            .assert()
            .failure()
            .stderr(predicate::str::contains(expected));
    }
}

#[test]
fn extract_axistream_rejects_excluded_and_invalid_mappings() {
    let fixture = waveform_fixture("extract_axistream.vcd");
    for standard in [
        "twakeup",
        "tvalidchk",
        "treadychk",
        "tdatachk",
        "tstrbchk",
        "tkeepchk",
        "tlastchk",
        "tidchk",
        "tdestchk",
        "tuserchk",
        "twakeupchk",
    ] {
        wavepeek_cmd()
            .args([
                "extract",
                "axistream",
                "--waves",
                fixture.as_str(),
                "--scope",
                "top",
                "--map",
                "aclk=clk",
                "--map",
                "tvalid=s_axis_t_valid_o",
                "--map",
                "tready=s_axis_tready_i",
                "--map",
                &format!("{standard}=s_axis_tvalidchk_o"),
            ])
            .assert()
            .failure()
            .stderr(predicate::str::contains(format!(
                "has no standard signal '{standard}'"
            )));
    }

    wavepeek_cmd()
        .args([
            "extract",
            "axistream",
            "--waves",
            fixture.as_str(),
            "--scope",
            "top",
            "--map",
            "aclk=clk",
            "--map",
            "tvalid=s_axis_t_valid_o",
            "--map",
            "TVALID=m_axis_tvalid_o",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("duplicate AXI-Stream mapping"));
}

#[test]
fn extract_axistream_auto_mapping_is_ambiguous_and_explicit_maps_win() {
    let fixture = waveform_fixture("extract_axistream.vcd");
    wavepeek_cmd()
        .args([
            "extract",
            "axistream",
            "--waves",
            fixture.as_str(),
            "--scope",
            "top",
            "--tready-mode",
            "implicit-high",
            "--map",
            "aclk=clk",
            "--include",
            "^(s|m)_axis_",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "ambiguous AXI-Stream auto-mapping for 'tvalid'",
        ));

    let value = wavepeek_cmd()
        .args([
            "extract",
            "axistream",
            "--waves",
            fixture.as_str(),
            "--scope",
            "top",
            "--tready-mode",
            "implicit-high",
            "--map",
            "aclk=clk",
            "--map",
            "tvalid=m_axis_tvalid_o",
            "--map",
            "tdata=m_axis_tdata_o",
            "--include",
            "^s_axis_t_data_o$",
            "--json",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let value = parse_json(&value);
    assert_eq!(
        value["data"]["mappings"]["tdata"]["path"],
        "top.m_axis_tdata_o"
    );
    assert_eq!(
        value["data"]["transfers"][0]["payload"]["tdata"],
        "16'h1001"
    );
}

#[test]
fn extract_axistream_source_mode_defaults_aliases_and_conflicts() {
    let fixture = waveform_fixture("extract_axistream.vcd");
    let default_source = write_source(&json!({
        "$schema": "https://kleverhq.github.io/wavepeek/schema-input-v2.2.json",
        "kind": "extract.axistream.source",
        "maps": {
            "aclk": "clk",
            "aresetn": "rst_n",
            "tvalid": "s_axis_t_valid_o",
            "tready": "s_axis_tready_i"
        }
    }));
    let default_value = wavepeek_cmd()
        .args([
            "extract",
            "axistream",
            "--waves",
            fixture.as_str(),
            "--scope",
            "top",
            "--source",
            default_source.path().to_str().unwrap(),
            "--json",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let default_value = parse_json(&default_value);
    assert_eq!(default_value["data"]["name"], "axistream");
    assert_eq!(default_value["data"]["profile"], "axi4-stream");
    assert_eq!(default_value["data"]["tready_mode"], "mapped");

    let source = write_source(&json!({
        "$schema": "https://kleverhq.github.io/wavepeek/schema-input-v2.2.json",
        "kind": "extract.axistream.source",
        "profile": "AXI5_STREAM",
        "tready_mode": "IMPLICIT_HIGH",
        "name": "source_stream",
        "includes": ["^m_axis_"],
        "maps": {"aclk": "clk", "aresetn": "rst_n"}
    }));
    let value = wavepeek_cmd()
        .args([
            "extract",
            "axistream",
            "--waves",
            fixture.as_str(),
            "--scope",
            "top",
            "--source",
            source.path().to_str().unwrap(),
            "--json",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let value = parse_json(&value);
    assert_eq!(value["data"]["name"], "source_stream");
    assert_eq!(value["data"]["profile"], "axi5-stream");
    assert_eq!(value["data"]["tready_mode"], "implicit-high");

    wavepeek_cmd()
        .args([
            "extract",
            "axistream",
            "--waves",
            fixture.as_str(),
            "--source",
            source.path().to_str().unwrap(),
            "--name",
            "conflict",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("cannot be used with"));
}

#[test]
fn extract_axistream_source_mode_rejects_wrong_contract_and_null_defaults() {
    let fixture = waveform_fixture("extract_axistream.vcd");
    for (field, value, expected) in [
        (
            "$schema",
            json!("https://example.invalid/input.json"),
            "uses unsupported $schema",
        ),
        ("kind", json!("extract.axi.source"), "has kind"),
        ("profile", Value::Null, "expected string, got null"),
        ("tready_mode", Value::Null, "expected string, got null"),
        ("name", Value::Null, "expected string, got null"),
    ] {
        let mut source_value = json!({
            "$schema": "https://kleverhq.github.io/wavepeek/schema-input-v2.2.json",
            "kind": "extract.axistream.source",
            "tready_mode": "implicit-high",
            "maps": {"aclk": "clk", "tvalid": "m_axis_tvalid_o"}
        });
        source_value[field] = value;
        let source = write_source(&source_value);
        wavepeek_cmd()
            .args([
                "extract",
                "axistream",
                "--waves",
                fixture.as_str(),
                "--scope",
                "top",
                "--source",
                source.path().to_str().unwrap(),
            ])
            .assert()
            .failure()
            .stderr(predicate::str::contains(expected));
    }
}
