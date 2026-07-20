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

const AUTO_INCLUDE: &str = "^trace_";

fn fixture(extension: &str) -> String {
    fixture_path(&format!("extract_atb.{extension}"))
        .to_string_lossy()
        .into_owned()
}

fn output_schema_validator() -> jsonschema::Validator {
    schema_validator("output.json")
}

fn stream_schema_validator() -> jsonschema::Validator {
    schema_validator("stream.json")
}

fn schema_validator(filename: &str) -> jsonschema::Validator {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("schema")
        .join(filename);
    let schema: Value = serde_json::from_str(
        &fs::read_to_string(path).unwrap_or_else(|_| panic!("{filename} should read")),
    )
    .unwrap_or_else(|_| panic!("{filename} should parse"));
    jsonschema::validator_for(&schema).unwrap_or_else(|_| panic!("{filename} should compile"))
}

fn parse_json(stdout: &[u8]) -> Value {
    let value: Value = serde_json::from_slice(stdout).expect("stdout should be JSON");
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

fn write_source(value: &Value) -> NamedTempFile {
    let file =
        NamedTempFile::with_suffix("extract-atb-source.json").expect("source file should create");
    fs::write(
        file.path(),
        serde_json::to_vec_pretty(value).expect("source should serialize"),
    )
    .expect("source should write");
    file
}

fn base_maps(include_reset: bool, include_payload: bool) -> Vec<&'static str> {
    let mut maps = vec![
        "atclk=trace_at_clk_i",
        "atvalid=trace_at_valid_o",
        "atready=trace_at_ready_i",
        "afvalid=trace_af_valid_i",
        "afready=trace_af_ready_o",
    ];
    if include_reset {
        maps.insert(1, "atresetn=trace_at_reset_n_i");
    }
    if include_payload {
        maps.extend([
            "atbytes=trace_at_bytes_o",
            "atdata=trace_at_data_o",
            "atid=trace_at_id_o",
        ]);
    }
    maps
}

fn command_with_maps(profile: &str, maps: &[&str]) -> std::process::Command {
    let waves = fixture("vcd");
    let mut command = wavepeek_cmd();
    command.args([
        "extract",
        "atb",
        "--waves",
        waves.as_str(),
        "--scope",
        "top",
        "--profile",
        profile,
    ]);
    for mapping in maps {
        command.args(["--map", mapping]);
    }
    command
}

fn event_kinds(data: &Value) -> Vec<&str> {
    data["events"]
        .as_array()
        .expect("events should be array")
        .iter()
        .map(|event| {
            event["event"]
                .as_str()
                .expect("event kind should be string")
        })
        .collect()
}

#[test]
fn extract_atb_default_human_automaps_and_emits_stateless_events() {
    let waves = fixture("vcd");
    let output = wavepeek_cmd()
        .args([
            "extract",
            "atb",
            "--waves",
            waves.as_str(),
            "--scope",
            "top",
            "--include",
            AUTO_INCLUDE,
        ])
        .output()
        .expect("ATB extraction should execute");

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).expect("human output should be UTF-8");
    assert!(
        stdout.starts_with(
            "name: atb\nprofile: atb-c\nissue: C\nmappings:\n  atclk = trace_at_clk_i"
        )
    );
    for mapping in [
        "atresetn = trace_at_reset_n_i",
        "atvalid = trace_at_valid_o",
        "atready = trace_at_ready_i",
        "atbytes = trace_at_bytes_o",
        "atdata = trace_at_data_o",
        "atid = trace_at_id_o",
        "afvalid = trace_af_valid_i",
        "afready = trace_af_ready_o",
        "syncreq = trace_sync_req_i",
    ] {
        assert!(
            stdout.contains(mapping),
            "missing mapping {mapping}:\n{stdout}"
        );
    }
    let rows = stdout
        .lines()
        .filter(|line| line.starts_with('@'))
        .collect::<Vec<_>>();
    assert_eq!(rows.len(), 10);
    assert_eq!(
        &rows[..6],
        [
            "@15ns sample@14ns [transfer] atbytes=2'h3 atdata=32'h44332211 atid=7'h10",
            "@15ns sample@14ns [flush]",
            "@15ns sample@14ns [sync-request]",
            "@20ns sample@19ns [transfer] atbytes=2'h3 atdata=32'h44332211 atid=7'h10",
            "@20ns sample@19ns [flush]",
            "@20ns sample@19ns [sync-request]",
        ]
    );
    assert!(
        rows.contains(&"@25ns sample@24ns [transfer] atbytes=2'h3 atdata=32'hxx332211 atid=7'h10")
    );
    assert!(
        rows.contains(&"@30ns sample@29ns [transfer] atbytes=2'h0 atdata=32'h00000010 atid=7'h7d")
    );
    assert_eq!(rows.last(), Some(&"@40ns sample@39ns [flush]"));
    assert!(rows.iter().all(|row| !row.starts_with("@5ns")));
    assert!(rows.iter().all(|row| !row.starts_with("@10ns")));
    assert!(rows.iter().all(|row| !row.starts_with("@35ns")));

    let stderr = String::from_utf8(output.stderr).expect("diagnostics should be UTF-8");
    for candidate in [
        "trace_at_clken_o",
        "trace_at_data8_o",
        "trace_at_ready_check_o",
        "trace_at_valid_chk_o",
        "trace_at_wakeup_o",
    ] {
        assert!(
            stderr.contains(&format!("ignored ATB include candidate '{candidate}'")),
            "missing warning for {candidate}:\n{stderr}"
        );
    }
}

#[test]
fn extract_atb_profiles_aliases_and_vcd_fst_are_equivalent() {
    for (requested, canonical, sync_rows) in [
        ("atb-a", "atb-a", 0),
        ("ATB_A", "atb-a", 0),
        ("atbv1.0", "atb-a", 0),
        ("atb-b", "atb-b", 3),
        ("ATB_B", "atb-b", 3),
        ("ATBV1.1", "atb-b", 3),
        ("ATB_C", "atb-c", 3),
    ] {
        let mut maps = base_maps(true, true);
        if canonical != "atb-a" {
            maps.push("syncreq=trace_sync_req_i");
        }
        let output = command_with_maps(requested, &maps)
            .arg("--json")
            .output()
            .expect("profile extraction should execute");
        assert!(
            output.status.success(),
            "{requested}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        let value = parse_json(&output.stdout);
        assert_eq!(value["data"]["profile"], canonical);
        assert_eq!(value["data"]["issue"], "C");
        assert_eq!(
            event_kinds(&value["data"])
                .into_iter()
                .filter(|kind| *kind == "sync-request")
                .count(),
            sync_rows,
            "{requested}"
        );
    }

    let mut by_format = Vec::new();
    for extension in ["vcd", "fst"] {
        let waves = fixture(extension);
        let output = wavepeek_cmd()
            .args([
                "extract",
                "atb",
                "--waves",
                waves.as_str(),
                "--scope",
                "top",
                "--include",
                AUTO_INCLUDE,
                "--json",
            ])
            .output()
            .expect("cross-format extraction should execute");
        assert!(output.status.success());
        by_format.push(parse_json(&output.stdout));
    }
    assert_eq!(by_format[0]["data"], by_format[1]["data"]);
    assert_eq!(by_format[0]["diagnostics"], by_format[1]["diagnostics"]);
}

#[test]
fn extract_atb_supports_independent_channels_optional_reset_and_payload() {
    let transfer = command_with_maps(
        "atb-c",
        &[
            "atclk=trace_at_clk_i",
            "atvalid=trace_at_valid_o",
            "atready=trace_at_ready_i",
        ],
    )
    .arg("--json")
    .output()
    .expect("handshake-only transfer should execute");
    assert!(transfer.status.success());
    let transfer = parse_json(&transfer.stdout);
    assert!(
        transfer["data"]["events"]
            .as_array()
            .expect("events")
            .iter()
            .all(|event| event["event"] == "transfer" && event["payload"] == json!({}))
    );
    assert_eq!(transfer["data"]["events"][0]["time"], "5ns");

    let flush = command_with_maps(
        "atb-a",
        &[
            "atclk=trace_at_clk_i",
            "atresetn=trace_at_reset_n_i",
            "afvalid=trace_af_valid_i",
            "afready=trace_af_ready_o",
        ],
    )
    .arg("--json")
    .output()
    .expect("flush-only extraction should execute");
    assert!(flush.status.success());
    let flush = parse_json(&flush.stdout);
    assert_eq!(event_kinds(&flush["data"]), ["flush", "flush", "flush"]);

    let data8 = command_with_maps(
        "atb-c",
        &[
            "atclk=trace_at_clk_i",
            "atresetn=trace_at_reset_n_i",
            "atvalid=trace_at_valid_o",
            "atready=trace_at_ready_i",
            "atdata=trace_at_data8_o",
        ],
    )
    .arg("--json")
    .output()
    .expect("8-bit transfer without ATBYTES should execute");
    assert!(data8.status.success());
    let data8 = parse_json(&data8.stdout);
    assert_eq!(
        data8["data"]["events"][0]["payload"],
        json!({"atdata": "8'ha5"})
    );
}

#[test]
fn extract_atb_json_abs_and_source_modes_preserve_contracts() {
    let waves = fixture("vcd");
    let source = write_source(&json!({
        "$schema": expected_input_schema_url(),
        "kind": "extract.atb.source",
        "profile": "atb_b",
        "name": "etm_trace",
        "includes": ["^trace_"],
        "maps": {"atclk": "trace_at_clk_i"},
        "x-extension": true
    }));
    let output = wavepeek_cmd()
        .args([
            "extract",
            "atb",
            "--waves",
            waves.as_str(),
            "--scope",
            "top",
            "--source",
            source.path().to_str().expect("source path should be UTF-8"),
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
    assert_eq!(value["$schema"], expected_schema_url());
    assert_eq!(value["command"], "extract atb");
    assert_eq!(value["data"]["name"], "etm_trace");
    assert_eq!(value["data"]["profile"], "atb-b");
    assert_eq!(
        value["data"]["mappings"]["atdata"]["path"],
        "top.trace_at_data_o"
    );
    assert_eq!(value["data"]["events"][0]["profile"], "atb-b");
    assert_eq!(
        value["data"]["events"][0]["payload"],
        json!({
            "atbytes": "2'h3",
            "atdata": "32'h44332211",
            "atid": "7'h10"
        })
    );

    let human = command_with_maps("atb-c", &base_maps(true, true))
        .arg("--abs")
        .output()
        .expect("absolute human extraction should execute");
    assert!(human.status.success());
    let human = String::from_utf8(human.stdout).expect("human output should be UTF-8");
    assert!(human.contains("atdata = top.trace_at_data_o"));
    assert!(human.contains("top.trace_at_data_o=32'h44332211"));
}

#[test]
fn extract_atb_jsonl_orders_rows_and_counts_max_split() {
    let waves = fixture("vcd");
    let output = wavepeek_cmd()
        .args([
            "extract",
            "atb",
            "--waves",
            waves.as_str(),
            "--scope",
            "top",
            "--include",
            AUTO_INCLUDE,
            "--max",
            "2",
            "--jsonl",
        ])
        .output()
        .expect("JSONL extraction should execute");
    assert!(output.status.success());
    let records = parse_stream(&output.stdout);
    assert_eq!(records[0]["type"], "begin");
    assert_eq!(records[0]["command"], "extract atb");
    assert_eq!(records[0]["$schema"], expected_stream_schema_url());
    assert_eq!(records[0]["context"]["profile"], "atb-c");
    assert_eq!(records[1]["item"]["event"], "transfer");
    assert_eq!(records[2]["item"]["event"], "flush");
    assert_eq!(records[1]["item"]["time"], records[2]["item"]["time"]);
    let diagnostics = records
        .iter()
        .filter(|record| record["type"] == "diagnostic")
        .collect::<Vec<_>>();
    assert_eq!(diagnostics.len(), 6);
    assert!(
        diagnostics
            .iter()
            .any(|record| record["diagnostic"]["code"] == "WPK-W0002")
    );
    let end = records.last().expect("stream should end");
    assert_eq!(end["type"], "end");
    assert_eq!(end["summary"]["items"], 2);
    assert_eq!(end["summary"]["diagnostics"], 6);
    assert_eq!(end["summary"]["truncated"], true);
}

#[test]
fn extract_atb_bounds_are_inclusive_and_use_pre_edge_samples() {
    let mut command = command_with_maps("atb-c", &base_maps(true, true));
    command.args(["--from", "15ns", "--to", "15ns", "--json"]);
    let output = command.output().expect("bounded extraction should execute");
    assert!(output.status.success());
    let value = parse_json(&output.stdout);
    assert_eq!(event_kinds(&value["data"]), ["transfer", "flush"]);
    assert!(
        value["data"]["events"]
            .as_array()
            .expect("events")
            .iter()
            .all(|event| event["time"] == "15ns" && event["sample_time"] == "14ns")
    );
}

#[test]
fn extract_atb_auto_mapping_is_exact_explicit_first_and_deterministic() {
    let waves = fixture("vcd");
    let ambiguous = wavepeek_cmd()
        .args([
            "extract",
            "atb",
            "--waves",
            waves.as_str(),
            "--scope",
            "top",
            "--map",
            "atclk=trace_at_clk_i",
            "--map",
            "atready=trace_at_ready_i",
            "--include",
            "^(trace|other)_at_valid_o$",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "ambiguous ATB auto-mapping for 'atvalid': other_at_valid_o, trace_at_valid_o",
        ));
    drop(ambiguous);

    let explicit = wavepeek_cmd()
        .args([
            "extract",
            "atb",
            "--waves",
            waves.as_str(),
            "--scope",
            "top",
            "--map",
            "atclk=trace_at_clk_i",
            "--map",
            "atvalid=trace_at_valid_o",
            "--map",
            "atready=trace_at_ready_i",
            "--include",
            "^(trace|other)_at_valid_o$",
            "--json",
        ])
        .output()
        .expect("explicit mapping should win");
    assert!(explicit.status.success());
    let explicit = parse_json(&explicit.stdout);
    assert_eq!(
        explicit["data"]["mappings"]["atvalid"]["path"],
        "top.trace_at_valid_o"
    );
}

#[test]
fn extract_atb_rejects_invalid_channel_and_profile_mappings() {
    let cases: &[(&[&str], &str)] = &[
        (
            &["atvalid=trace_at_valid_o", "atready=trace_at_ready_i"],
            "requires atclk",
        ),
        (
            &["atclk=trace_at_clk_i", "atvalid=trace_at_valid_o"],
            "must map both atvalid and atready",
        ),
        (
            &["atclk=trace_at_clk_i", "afvalid=trace_af_valid_i"],
            "must map both afvalid and afready",
        ),
        (
            &["atclk=trace_at_clk_i", "atdata=trace_at_data_o"],
            "transfer payload mappings require both atvalid and atready",
        ),
        (
            &["atclk=trace_at_clk_i"],
            "requires at least one complete transfer or flush handshake",
        ),
        (
            &["atclk=trace_at_clk_i", "syncreq=trace_sync_req_i"],
            "requires at least one complete transfer or flush handshake",
        ),
    ];
    for (maps, message) in cases {
        command_with_maps("atb-c", maps)
            .assert()
            .failure()
            .stderr(predicate::str::contains(*message));
    }

    for (profile, mapping) in [
        ("atb-a", "syncreq=trace_sync_req_i"),
        ("atb-a", "atclken=trace_at_clken_o"),
        ("atb-b", "atwakeup=trace_at_wakeup_o"),
        ("atb-c", "atwakeup=trace_at_wakeup_o"),
    ] {
        let mut maps = vec![
            "atclk=trace_at_clk_i",
            "atvalid=trace_at_valid_o",
            "atready=trace_at_ready_i",
        ];
        maps.push(mapping);
        command_with_maps(profile, &maps)
            .assert()
            .failure()
            .stderr(predicate::str::contains("has no standard signal"));
    }
}

#[test]
fn extract_atb_rejects_invalid_map_include_scope_and_source_inputs() {
    let waves = fixture("vcd");
    command_with_maps(
        "atb-c",
        &[
            "atclk=trace_at_clk_i",
            "ATCLK=trace_at_clk_i",
            "atvalid=trace_at_valid_o",
            "atready=trace_at_ready_i",
        ],
    )
    .assert()
    .failure()
    .stderr(predicate::str::contains("duplicate ATB mapping"));

    wavepeek_cmd()
        .args([
            "extract",
            "atb",
            "--waves",
            waves.as_str(),
            "--include",
            "[",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid ATB include regex"));

    command_with_maps(
        "atb-c",
        &[
            "atclk=top.trace_at_clk_i",
            "atvalid=trace_at_valid_o",
            "atready=trace_at_ready_i",
        ],
    )
    .assert()
    .failure()
    .stderr(predicate::str::contains("must use a scope-relative signal"));

    command_with_maps(
        "atb-c",
        &[
            "atclk=missing",
            "atvalid=trace_at_valid_o",
            "atready=trace_at_ready_i",
        ],
    )
    .assert()
    .failure()
    .stderr(predicate::str::contains("missing"));

    for source in [
        json!({
            "$schema": "https://example.invalid/input.json",
            "kind": "extract.atb.source",
            "maps": {}
        }),
        json!({
            "$schema": expected_input_schema_url(),
            "kind": "extract.axi.source",
            "maps": {}
        }),
        json!({
            "$schema": expected_input_schema_url(),
            "kind": "extract.atb.source",
            "profile": null
        }),
        json!({
            "$schema": expected_input_schema_url(),
            "kind": "extract.atb.source",
            "name": null
        }),
    ] {
        let source = write_source(&source);
        wavepeek_cmd()
            .args([
                "extract",
                "atb",
                "--waves",
                waves.as_str(),
                "--source",
                source.path().to_str().expect("source path should be UTF-8"),
            ])
            .assert()
            .failure();
    }

    let valid_source = write_source(&json!({
        "$schema": expected_input_schema_url(),
        "kind": "extract.atb.source",
        "maps": {
            "atclk": "top.trace_at_clk_i",
            "atvalid": "top.trace_at_valid_o",
            "atready": "top.trace_at_ready_i"
        }
    }));
    wavepeek_cmd()
        .args([
            "extract",
            "atb",
            "--waves",
            waves.as_str(),
            "--source",
            valid_source
                .path()
                .to_str()
                .expect("source path should be UTF-8"),
            "--name",
            "conflict",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "the argument '--source <FILE>' cannot be used with '--name <NAME>'",
        ));
}
