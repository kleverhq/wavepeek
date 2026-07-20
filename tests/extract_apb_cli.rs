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

const APB4_AUTO_INCLUDE: &str = "^uart_apb_(p_clk_i|presetn_i|psel_o|penable_o|pwrite_o|pready_i|p_addr_o|pprot_o|pwdata_o|pstrb_o|prdata_i|pslverr_i)$";
const APB4_INCLUDE_WITH_DECOYS: &str = "^uart_apb_(p_clk_i|presetn_i|psel_o|penable_o|pwrite_o|pready_i|p_addr_o|pprot_o|pwdata_o|pstrb_o|prdata_i|pslverr_i|misc_o|preadychk_i|psel0_o|pselx_o)$";
const APB5_AUTO_INCLUDE: &str = "^apb5_(pclk_i|presetn_i|psel_o|penable_o|pwrite_o|pready_i|paddr_o|pprot_o|pnse_o|pauser_o|pwdata_o|pstrb_o|pwuser_o|prdata_i|pslverr_i|pruser_i|pbuser_i)$";

fn output_schema_validator() -> jsonschema::Validator {
    schema_validator("output.json")
}

fn stream_schema_validator() -> jsonschema::Validator {
    schema_validator("stream.json")
}

fn input_schema_validator() -> jsonschema::Validator {
    schema_validator("input.json")
}

fn schema_validator(name: &str) -> jsonschema::Validator {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("schema")
        .join(name);
    let schema: Value = serde_json::from_str(
        &fs::read_to_string(path).unwrap_or_else(|error| panic!("{name} should read: {error}")),
    )
    .unwrap_or_else(|error| panic!("{name} should parse: {error}"));
    jsonschema::validator_for(&schema)
        .unwrap_or_else(|error| panic!("{name} should compile: {error}"))
}

fn fixture(filename: &str) -> String {
    fixture_path(filename).to_string_lossy().into_owned()
}

fn write_source(contents: &str) -> NamedTempFile {
    let source =
        NamedTempFile::with_suffix("extract-apb-source.json").expect("source file should create");
    fs::write(source.path(), contents).expect("source file should write");
    source
}

fn parse_json(stdout: &[u8]) -> Value {
    let value: Value = serde_json::from_slice(stdout).expect("stdout should be valid JSON");
    output_schema_validator()
        .validate(&value)
        .unwrap_or_else(|error| panic!("output should validate: {error}\n{value}"));
    value
}

fn parse_stream(stdout: &[u8]) -> Vec<Value> {
    let text = std::str::from_utf8(stdout).expect("stdout should be UTF-8 JSONL");
    assert!(text.ends_with('\n'));
    let validator = stream_schema_validator();
    text.lines()
        .map(|line| {
            let value: Value = serde_json::from_str(line).expect("JSONL line should parse");
            validator
                .validate(&value)
                .unwrap_or_else(|error| panic!("record should validate: {error}\n{value}"));
            value
        })
        .collect()
}

fn apb4_explicit_maps(include_reset: bool, include_pready: bool) -> Vec<String> {
    let mut maps = vec![
        "pclk=uart_apb_p_clk_i",
        "psel=uart_apb_psel_o",
        "penable=uart_apb_penable_o",
        "pwrite=uart_apb_pwrite_o",
        "paddr=uart_apb_p_addr_o",
        "pprot=uart_apb_pprot_o",
        "pwdata=uart_apb_pwdata_o",
        "pstrb=uart_apb_pstrb_o",
        "prdata=uart_apb_prdata_i",
        "pslverr=uart_apb_pslverr_i",
    ];
    if include_reset {
        maps.push("presetn=uart_apb_presetn_i");
    }
    if include_pready {
        maps.push("pready=uart_apb_pready_i");
    }
    maps.into_iter()
        .flat_map(|mapping| ["--map".to_string(), mapping.to_string()])
        .collect()
}

fn base_apb4_args() -> Vec<String> {
    vec![
        "extract".to_string(),
        "apb".to_string(),
        "--waves".to_string(),
        fixture("extract_apb4.vcd"),
        "--scope".to_string(),
        "top".to_string(),
    ]
}

fn json_events(value: &Value) -> &[Value] {
    value["data"]["events"]
        .as_array()
        .expect("events should be an array")
}

fn event_signature(value: &Value) -> Vec<(String, String, String, Value)> {
    json_events(value)
        .iter()
        .map(|event| {
            (
                event["time"].as_str().unwrap().to_string(),
                event["event"].as_str().unwrap().to_string(),
                event["direction"].as_str().unwrap().to_string(),
                event["payload"].clone(),
            )
        })
        .collect()
}

#[test]
fn extract_apb_human_defaults_to_apb4_and_suppresses_waits() {
    let mut args = base_apb4_args();
    args.extend([
        "--include".to_string(),
        APB4_INCLUDE_WITH_DECOYS.to_string(),
    ]);
    let output = wavepeek_cmd()
        .args(args)
        .output()
        .expect("APB extraction should execute");

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).expect("human output should be UTF-8");
    assert!(stdout.starts_with(
        "name: apb\nprofile: apb4\nissue: E\npready_mode: mapped\ninclude_wait: false\n"
    ));
    assert!(stdout.contains("  pclk = uart_apb_p_clk_i"));
    assert!(stdout.contains("  paddr = uart_apb_p_addr_o"));
    assert!(!stdout.contains("[access-wait"));
    assert!(stdout.contains("@5ns sample@4ns [setup write]"));
    assert!(stdout.contains("@20ns sample@19ns [access-complete write]"));
    assert!(stdout.contains("@25ns sample@24ns [setup read]"));
    assert!(stdout.contains("@30ns sample@29ns [access-complete read]"));
    assert!(stdout.contains("@35ns sample@34ns [access-complete write]"));
    assert!(stdout.contains("@40ns sample@39ns [setup unknown]"));
    assert!(!stdout.contains("@45ns"));
    assert!(!stdout.contains("@50ns"));
    assert!(!stdout.contains("@55ns"));
    assert!(!stdout.contains("@60ns"));

    let stderr = String::from_utf8(output.stderr).expect("diagnostics should be UTF-8");
    for decoy in [
        "uart_apb_misc_o",
        "uart_apb_preadychk_i",
        "uart_apb_psel0_o",
        "uart_apb_pselx_o",
    ] {
        assert!(
            stderr.contains(&format!("ignored APB include candidate '{decoy}'")),
            "missing warning for {decoy}:\n{stderr}"
        );
    }
}

#[test]
fn extract_apb_json_emits_waits_and_filters_payload_by_event_and_direction() {
    let mut args = base_apb4_args();
    args.extend(apb4_explicit_maps(true, true));
    args.extend(["--include-wait".to_string(), "--json".to_string()]);
    let output = wavepeek_cmd()
        .args(args)
        .output()
        .expect("APB JSON extraction should execute");

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stderr.is_empty());
    let value = parse_json(&output.stdout);
    assert_eq!(value["$schema"], expected_schema_url());
    assert_eq!(value["command"], "extract apb");
    assert_eq!(value["data"]["profile"], "apb4");
    assert_eq!(value["data"]["issue"], "E");
    assert_eq!(value["data"]["pready_mode"], "mapped");
    assert_eq!(value["data"]["include_wait"], true);
    assert_eq!(
        json_events(&value)
            .iter()
            .map(|event| event["event"].as_str().unwrap())
            .collect::<Vec<_>>(),
        [
            "setup",
            "access-wait",
            "access-wait",
            "access-complete",
            "setup",
            "access-complete",
            "access-complete",
            "setup",
        ]
    );

    let setup_write = &json_events(&value)[0];
    assert_eq!(setup_write["payload"]["pwrite"], "1'h1");
    assert_eq!(setup_write["payload"]["pwdata"], "8'hde");
    assert!(setup_write["payload"].get("prdata").is_none());
    assert!(setup_write["payload"].get("pslverr").is_none());

    let complete_write = &json_events(&value)[3];
    assert_eq!(complete_write["payload"]["pslverr"], "1'h0");
    assert!(complete_write["payload"].get("prdata").is_none());
    let complete_read = &json_events(&value)[5];
    assert_eq!(complete_read["payload"]["prdata"], "8'ha5");
    assert_eq!(complete_read["payload"]["pslverr"], "1'h1");
    assert_eq!(complete_read["payload"]["pstrb"], "1'h0");
    assert!(complete_read["payload"].get("pwdata").is_none());
    let setup_unknown = &json_events(&value)[7];
    assert_eq!(setup_unknown["direction"], "unknown");
    assert_eq!(setup_unknown["payload"]["pwrite"], "1'hx");
    assert_eq!(setup_unknown["payload"]["pwdata"], "8'hcc");
    assert!(setup_unknown["payload"].get("prdata").is_none());
}

#[test]
fn extract_apb_jsonl_streams_context_items_and_row_limit_summary() {
    let mut args = base_apb4_args();
    args.extend(apb4_explicit_maps(true, true));
    args.extend([
        "--include-wait".to_string(),
        "--max".to_string(),
        "2".to_string(),
        "--jsonl".to_string(),
    ]);
    let output = wavepeek_cmd()
        .args(args)
        .output()
        .expect("APB JSONL extraction should execute");

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stderr.is_empty());
    let records = parse_stream(&output.stdout);
    assert_eq!(records[0]["type"], "begin");
    assert_eq!(records[0]["$schema"], expected_stream_schema_url());
    assert_eq!(records[0]["command"], "extract apb");
    assert_eq!(records[0]["context"]["profile"], "apb4");
    assert_eq!(records[0]["context"]["pready_mode"], "mapped");
    assert_eq!(records[0]["context"]["include_wait"], true);
    assert_eq!(records[1]["item"]["event"], "setup");
    assert_eq!(records[2]["item"]["event"], "access-wait");
    assert_eq!(records[3]["diagnostic"]["code"], "WPK-W0002");
    assert_eq!(records[4]["summary"]["items"], 2);
    assert_eq!(records[4]["summary"]["diagnostics"], 1);
    assert_eq!(records[4]["summary"]["truncated"], true);
}

#[test]
fn extract_apb_implicit_high_forbids_pready_and_classifies_every_access() {
    let mut args = base_apb4_args();
    args.extend(apb4_explicit_maps(true, false));
    args.extend([
        "--pready-mode".to_string(),
        "IMPLICIT_HIGH".to_string(),
        "--json".to_string(),
    ]);
    let output = wavepeek_cmd()
        .args(args)
        .output()
        .expect("implicit-high extraction should execute");

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let value = parse_json(&output.stdout);
    assert_eq!(value["data"]["pready_mode"], "implicit-high");
    assert_eq!(value["data"]["include_wait"], false);
    assert!(value["data"]["mappings"].get("pready").is_none());
    assert_eq!(
        json_events(&value)
            .iter()
            .map(|event| (
                event["time"].as_str().unwrap(),
                event["event"].as_str().unwrap()
            ))
            .collect::<Vec<_>>(),
        [
            ("5ns", "setup"),
            ("10ns", "access-complete"),
            ("15ns", "access-complete"),
            ("20ns", "access-complete"),
            ("25ns", "setup"),
            ("30ns", "access-complete"),
            ("35ns", "access-complete"),
            ("40ns", "setup"),
            ("45ns", "access-complete"),
        ]
    );
    assert!(
        json_events(&value)
            .iter()
            .all(|event| event["event"] != "access-wait")
    );
    let unknown_complete = json_events(&value)
        .iter()
        .find(|event| event["time"] == "45ns")
        .expect("implicit-high mode should classify the unknown-PREADY Access");
    assert_eq!(unknown_complete["direction"], "unknown");
    assert_eq!(unknown_complete["payload"]["pwrite"], "1'hx");
    assert_eq!(unknown_complete["payload"]["pwdata"], "8'hcc");
    assert_eq!(unknown_complete["payload"]["prdata"], "8'ha5");
}

#[test]
fn extract_apb_validates_pready_mode_mappings_and_wait_capture() {
    let mut missing_pready = base_apb4_args();
    missing_pready.extend(apb4_explicit_maps(true, false));
    wavepeek_cmd()
        .args(missing_pready)
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "APB mapped PREADY mode requires pready",
        ));

    let mut forbidden_pready = base_apb4_args();
    forbidden_pready.extend(apb4_explicit_maps(true, true));
    forbidden_pready.extend(["--pready-mode".to_string(), "implicit-high".to_string()]);
    wavepeek_cmd()
        .args(forbidden_pready)
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "APB implicit-high PREADY mode forbids a pready mapping",
        ));

    let mut forbidden_wait = base_apb4_args();
    forbidden_wait.extend(apb4_explicit_maps(true, false));
    forbidden_wait.extend([
        "--pready-mode".to_string(),
        "implicit-high".to_string(),
        "--include-wait".to_string(),
    ]);
    wavepeek_cmd()
        .args(forbidden_wait)
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "--include-wait cannot be used with --pready-mode implicit-high",
        ));
}

#[test]
fn extract_apb_profiles_use_issue_e_and_profile_specific_mappings() {
    for (profile, fixture_name, include, required, forbidden) in [
        (
            "APB3",
            "extract_apb3.vcd",
            "^apb3_",
            vec!["paddr", "pwdata", "prdata", "pslverr"],
            vec!["pprot", "pstrb", "pnse"],
        ),
        (
            "apb4",
            "extract_apb4.vcd",
            APB4_AUTO_INCLUDE,
            vec!["pprot", "pstrb", "pslverr"],
            vec!["pnse", "pauser", "pbuser"],
        ),
        (
            "apb5",
            "extract_apb5.vcd",
            APB5_AUTO_INCLUDE,
            vec!["pnse", "pauser", "pwuser", "pruser", "pbuser"],
            vec!["pwakeup", "paddrchk"],
        ),
    ] {
        let output = wavepeek_cmd()
            .args([
                "extract",
                "apb",
                "--waves",
                fixture(fixture_name).as_str(),
                "--scope",
                "top",
                "--profile",
                profile,
                "--include",
                include,
                "--json",
            ])
            .output()
            .expect("profile extraction should execute");
        assert!(
            output.status.success(),
            "{profile}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        let value = parse_json(&output.stdout);
        assert_eq!(value["data"]["profile"], profile.to_ascii_lowercase());
        assert_eq!(value["data"]["issue"], "E");
        for signal in required {
            assert!(
                value["data"]["mappings"].get(signal).is_some(),
                "{profile} should map {signal}"
            );
        }
        for signal in forbidden {
            assert!(
                value["data"]["mappings"].get(signal).is_none(),
                "{profile} must not map {signal}"
            );
        }
    }
}

#[test]
fn extract_apb_vcd_and_fst_are_equivalent_for_every_profile() {
    for (profile, stem, include) in [
        ("apb3", "extract_apb3", "^apb3_"),
        ("apb4", "extract_apb4", APB4_AUTO_INCLUDE),
        ("apb5", "extract_apb5", APB5_AUTO_INCLUDE),
    ] {
        let mut values = Vec::new();
        for extension in ["vcd", "fst"] {
            let output = wavepeek_cmd()
                .args([
                    "extract",
                    "apb",
                    "--waves",
                    fixture(format!("{stem}.{extension}").as_str()).as_str(),
                    "--scope",
                    "top",
                    "--profile",
                    profile,
                    "--include",
                    include,
                    "--include-wait",
                    "--json",
                ])
                .output()
                .expect("cross-format extraction should execute");
            assert!(
                output.status.success(),
                "{profile}/{extension}: {}",
                String::from_utf8_lossy(&output.stderr)
            );
            values.push(parse_json(&output.stdout));
        }
        assert_eq!(
            event_signature(&values[0]),
            event_signature(&values[1]),
            "{profile} VCD/FST mismatch"
        );
    }
}

#[test]
fn extract_apb_source_mode_applies_defaults_wait_setting_and_strict_metadata() {
    let source = write_source(
        &json!({
            "$schema": expected_input_schema_url(),
            "kind": "extract.apb.source",
            "profile": "APB4",
            "pready_mode": "MAPPED",
            "include_wait": true,
            "name": "uart",
            "maps": {
                "pclk": "uart_apb_p_clk_i",
                "presetn": "uart_apb_presetn_i",
                "psel": "uart_apb_psel_o",
                "penable": "uart_apb_penable_o",
                "pwrite": "uart_apb_pwrite_o",
                "pready": "uart_apb_pready_i",
                "paddr": "uart_apb_p_addr_o"
            }
        })
        .to_string(),
    );
    let output = wavepeek_cmd()
        .args([
            "extract",
            "apb",
            "--waves",
            fixture("extract_apb4.vcd").as_str(),
            "--scope",
            "top",
            "--source",
            source.path().to_str().unwrap(),
            "--json",
        ])
        .output()
        .expect("source extraction should execute");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let value = parse_json(&output.stdout);
    assert_eq!(value["data"]["name"], "uart");
    assert_eq!(value["data"]["profile"], "apb4");
    assert_eq!(value["data"]["include_wait"], true);
    assert_eq!(
        json_events(&value)
            .iter()
            .filter(|event| event["event"] == "access-wait")
            .count(),
        2
    );

    let defaults_source = write_source(
        &json!({
            "$schema": expected_input_schema_url(),
            "kind": "extract.apb.source",
            "maps": {
                "pclk": "uart_apb_p_clk_i",
                "psel": "uart_apb_psel_o",
                "penable": "uart_apb_penable_o",
                "pwrite": "uart_apb_pwrite_o",
                "pready": "uart_apb_pready_i"
            }
        })
        .to_string(),
    );
    let defaults_output = wavepeek_cmd()
        .args([
            "extract",
            "apb",
            "--waves",
            fixture("extract_apb4.vcd").as_str(),
            "--scope",
            "top",
            "--source",
            defaults_source.path().to_str().unwrap(),
            "--json",
        ])
        .output()
        .expect("defaulted source extraction should execute");
    assert!(defaults_output.status.success());
    let defaults = parse_json(&defaults_output.stdout);
    assert_eq!(defaults["data"]["name"], "apb");
    assert_eq!(defaults["data"]["profile"], "apb4");
    assert_eq!(defaults["data"]["pready_mode"], "mapped");
    assert_eq!(defaults["data"]["include_wait"], false);

    for (field, field_value, error_fragment) in [
        (
            "$schema",
            json!("https://kleverhq.github.io/wavepeek/schema-input-v2.1.json"),
            "unsupported $schema",
        ),
        (
            "kind",
            json!("extract.axi.source"),
            "expected extract.apb.source",
        ),
        ("profile", Value::Null, "expected string, got null"),
        ("pready_mode", Value::Null, "expected string, got null"),
        ("include_wait", Value::Null, "expected boolean, got null"),
        ("name", Value::Null, "expected string, got null"),
    ] {
        let mut invalid = json!({
            "$schema": expected_input_schema_url(),
            "kind": "extract.apb.source",
            "maps": {}
        });
        invalid[field] = field_value;
        let source = write_source(&invalid.to_string());
        wavepeek_cmd()
            .args([
                "extract",
                "apb",
                "--waves",
                fixture("extract_apb4.vcd").as_str(),
                "--source",
                source.path().to_str().unwrap(),
            ])
            .assert()
            .failure()
            .stderr(predicate::str::contains(error_fragment));
    }

    let duplicate_source = write_source(&format!(
        r#"{{"$schema":"{}","kind":"extract.apb.source","maps":{{"pclk":"clk_a","pclk":"clk_b"}}}}"#,
        expected_input_schema_url()
    ));
    wavepeek_cmd()
        .args([
            "extract",
            "apb",
            "--waves",
            fixture("extract_apb4.vcd").as_str(),
            "--source",
            duplicate_source.path().to_str().unwrap(),
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("duplicate APB mapping key 'pclk'"));
}

#[test]
fn extract_apb_source_schema_accepts_canonical_values_only() {
    let validator = input_schema_validator();
    let canonical = json!({
        "$schema": expected_input_schema_url(),
        "kind": "extract.apb.source",
        "profile": "apb5",
        "pready_mode": "mapped",
        "include_wait": true,
        "maps": {"pclk": "clk", "pready": "ready"}
    });
    validator
        .validate(&canonical)
        .unwrap_or_else(|error| panic!("canonical source should validate: {error}"));

    for invalid in [
        json!({
            "$schema": expected_input_schema_url(), "kind": "extract.apb.source",
            "profile": "APB5"
        }),
        json!({
            "$schema": expected_input_schema_url(), "kind": "extract.apb.source",
            "pready_mode": "implicit_high"
        }),
        json!({
            "$schema": expected_input_schema_url(), "kind": "extract.apb.source",
            "pready_mode": "implicit-high", "include_wait": true
        }),
        json!({
            "$schema": expected_input_schema_url(), "kind": "extract.apb.source",
            "pready_mode": "implicit-high", "maps": {"pready": "ready"}
        }),
        json!({
            "$schema": expected_input_schema_url(), "kind": "extract.apb.source",
            "profile": "apb3", "maps": {"pprot": "prot"}
        }),
    ] {
        assert!(
            !validator.is_valid(&invalid),
            "source must be rejected: {invalid}"
        );
    }
}

#[test]
fn extract_apb_source_conflicts_with_profile_mapping_and_wait_flags() {
    let source = write_source(
        &json!({
            "$schema": expected_input_schema_url(),
            "kind": "extract.apb.source",
            "pready_mode": "implicit-high",
            "maps": {
                "pclk": "top.uart_apb_p_clk_i",
                "psel": "top.uart_apb_psel_o",
                "penable": "top.uart_apb_penable_o",
                "pwrite": "top.uart_apb_pwrite_o"
            }
        })
        .to_string(),
    );
    for conflicting in [
        vec!["--profile", "apb3"],
        vec!["--pready-mode", "implicit-high"],
        vec!["--include-wait"],
        vec!["--name", "uart"],
        vec!["--map", "paddr=top.uart_apb_p_addr_o"],
        vec!["--include", "^uart"],
    ] {
        let mut args = vec![
            "extract".to_string(),
            "apb".to_string(),
            "--waves".to_string(),
            fixture("extract_apb4.vcd"),
            "--source".to_string(),
            source.path().to_string_lossy().into_owned(),
        ];
        args.extend(conflicting.into_iter().map(str::to_string));
        wavepeek_cmd()
            .args(args)
            .assert()
            .failure()
            .stderr(predicate::str::contains("cannot be used with"));
    }
}

#[test]
fn extract_apb_bounds_limits_empty_diagnostics_and_reset_gate_are_row_based() {
    let mut bounded = base_apb4_args();
    bounded.extend(apb4_explicit_maps(true, true));
    bounded.extend([
        "--from".to_string(),
        "20ns".to_string(),
        "--to".to_string(),
        "30ns".to_string(),
        "--json".to_string(),
    ]);
    let output = wavepeek_cmd().args(bounded).output().unwrap();
    let value = parse_json(&output.stdout);
    assert_eq!(
        json_events(&value)
            .iter()
            .map(|event| event["time"].as_str().unwrap())
            .collect::<Vec<_>>(),
        ["20ns", "25ns", "30ns"]
    );

    let mut empty = base_apb4_args();
    empty.extend(apb4_explicit_maps(true, true));
    empty.extend([
        "--from".to_string(),
        "45ns".to_string(),
        "--to".to_string(),
        "60ns".to_string(),
        "--json".to_string(),
    ]);
    let output = wavepeek_cmd().args(empty).output().unwrap();
    let value = parse_json(&output.stdout);
    assert!(json_events(&value).is_empty());
    assert_eq!(value["diagnostics"][0]["code"], "WPK-W0003");

    let mut no_reset = base_apb4_args();
    no_reset.extend(apb4_explicit_maps(false, true));
    no_reset.extend([
        "--from".to_string(),
        "50ns".to_string(),
        "--to".to_string(),
        "60ns".to_string(),
        "--json".to_string(),
    ]);
    let output = wavepeek_cmd().args(no_reset).output().unwrap();
    let value = parse_json(&output.stdout);
    assert_eq!(json_events(&value).len(), 1);
    assert_eq!(json_events(&value)[0]["time"], "50ns");
    assert_eq!(json_events(&value)[0]["event"], "setup");
}

#[test]
fn extract_apb_mapping_validation_is_deterministic() {
    let mut indexed_select = base_apb4_args();
    indexed_select.extend([
        "--include".to_string(),
        "^uart_apb_(p_clk_i|psel0_o|penable_o|pwrite_o|pready_i)$".to_string(),
    ]);
    wavepeek_cmd()
        .args(indexed_select)
        .assert()
        .failure()
        .stderr(predicate::str::contains("APB extraction requires psel"));

    let mut ambiguous = base_apb4_args();
    ambiguous.extend(apb4_explicit_maps(true, true));
    let paddr_index = ambiguous
        .iter()
        .position(|arg| arg == "paddr=uart_apb_p_addr_o")
        .expect("paddr map should exist");
    ambiguous.drain((paddr_index - 1)..=paddr_index);
    ambiguous.extend([
        "--include".to_string(),
        "^uart_apb_(p_addr_o|shadow_paddr_o)$".to_string(),
    ]);
    wavepeek_cmd()
        .args(ambiguous)
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "ambiguous APB auto-mapping for 'paddr'",
        ));

    let mut multi_standard = base_apb4_args();
    multi_standard.extend(apb4_explicit_maps(true, true));
    multi_standard.extend([
        "--include".to_string(),
        "^uart_apb_paddr_pwrite_o$".to_string(),
    ]);
    wavepeek_cmd()
        .args(multi_standard)
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "ambiguous APB auto-mapping for 'uart_apb_paddr_pwrite_o'",
        ));

    let mut explicit_wins = base_apb4_args();
    explicit_wins.extend(apb4_explicit_maps(true, true));
    explicit_wins.extend([
        "--include".to_string(),
        "^uart_apb_shadow_paddr_o$".to_string(),
        "--max".to_string(),
        "1".to_string(),
        "--json".to_string(),
    ]);
    let output = wavepeek_cmd().args(explicit_wins).output().unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let value = parse_json(&output.stdout);
    assert_eq!(
        value["data"]["mappings"]["paddr"]["path"],
        "top.uart_apb_p_addr_o"
    );

    let mut unresolved = base_apb4_args();
    unresolved.extend(apb4_explicit_maps(true, true));
    let missing_index = unresolved
        .iter()
        .position(|arg| arg == "paddr=uart_apb_p_addr_o")
        .expect("paddr map should exist");
    unresolved[missing_index] = "paddr=missing_paddr".to_string();
    wavepeek_cmd()
        .args(unresolved)
        .assert()
        .failure()
        .stderr(predicate::str::contains("missing_paddr"));

    let mut duplicate = base_apb4_args();
    duplicate.extend(apb4_explicit_maps(true, true));
    duplicate.extend(["--map".to_string(), "PWRITE=uart_apb_pwrite_o".to_string()]);
    wavepeek_cmd()
        .args(duplicate)
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "duplicate APB mapping for standard signal 'pwrite'",
        ));

    let mut outside_profile = base_apb4_args();
    outside_profile.extend(apb4_explicit_maps(true, true));
    outside_profile.extend(["--map".to_string(), "pnse=uart_apb_psel_o".to_string()]);
    wavepeek_cmd()
        .args(outside_profile)
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "APB profile apb4 has no standard signal 'pnse'",
        ));
}

#[test]
fn extract_apb_abs_uses_canonical_mapping_and_payload_paths() {
    let mut args = base_apb4_args();
    args.extend(apb4_explicit_maps(true, true));
    args.extend(["--max".to_string(), "1".to_string(), "--abs".to_string()]);
    let output = wavepeek_cmd().args(args).output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("pclk = top.uart_apb_p_clk_i"));
    assert!(stdout.contains("top.uart_apb_pwrite_o=1'h1"));
    assert!(stdout.contains("top.uart_apb_p_addr_o=8'h40"));
}
