#![cfg(feature = "fsdb")]

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use assert_cmd::prelude::*;
use predicates::prelude::*;
use serde_json::{Value, json};
use tempfile::{NamedTempFile, TempDir};

mod common;
use common::{expected_schema_url, fixture_path, wavepeek_cmd};

const SCOPE_KIND_ALIASES: &[&str] = &[
    "module",
    "task",
    "function",
    "begin",
    "fork",
    "generate",
    "struct",
    "union",
    "class",
    "interface",
    "package",
    "program",
    "unknown",
];

const SIGNAL_KIND_ALIASES: &[&str] = &[
    "event",
    "integer",
    "parameter",
    "real",
    "reg",
    "supply0",
    "supply1",
    "time",
    "tri",
    "triand",
    "trior",
    "trireg",
    "tri0",
    "tri1",
    "wand",
    "wire",
    "wor",
    "string",
    "port",
    "sparse_array",
    "real_time",
    "real_parameter",
    "bit",
    "logic",
    "int",
    "short_int",
    "long_int",
    "byte",
    "enum",
    "short_real",
    "boolean",
    "bit_vector",
];

#[test]
fn fsdb_info_json_matches_vcd_derived_fixture() {
    let fixtures = GeneratedFsdbFixtures::new();
    let fixture = path_str(&fixtures.signal_recursive_depth());
    let value = run_json_success(&["info", "--waves", fixture.as_str(), "--json"]);

    assert_eq!(value["$schema"], expected_schema_url());
    assert!(value.get("schema_version").is_none());
    assert_eq!(value["command"], "info");
    assert_eq!(value["diagnostics"], json!([]));
    assert_eq!(value["data"]["time_unit"], "1ns");
    assert_eq!(value["data"]["time_start"], "0ns");
    assert_eq!(value["data"]["time_end"], "10ns");
}

#[test]
fn fsdb_scope_json_is_sorted_and_depth_bounded() {
    let fixtures = GeneratedFsdbFixtures::new();
    let fixture = path_str(&fixtures.signal_recursive_depth());

    let all = run_json_success(&[
        "scope",
        "--waves",
        fixture.as_str(),
        "--max",
        "unlimited",
        "--json",
    ]);
    let root_only = run_json_success(&[
        "scope",
        "--waves",
        fixture.as_str(),
        "--max",
        "50",
        "--max-depth",
        "0",
        "--json",
    ]);

    assert_eq!(
        all["data"],
        json!([
            {"path": "top", "depth": 0, "kind": "module"},
            {"path": "top.cpu", "depth": 1, "kind": "module"},
            {"path": "top.cpu.core", "depth": 2, "kind": "module"},
            {"path": "top.mem", "depth": 1, "kind": "module"},
        ])
    );
    assert_eq!(
        all["diagnostics"],
        json!([{"kind": "warning", "code": "WPK-W0001", "message": "limit disabled: --max=unlimited"}])
    );

    assert_eq!(
        root_only["data"],
        json!([{ "path": "top", "depth": 0, "kind": "module" }])
    );
    assert_eq!(root_only["diagnostics"], json!([]));
}

#[test]
fn fsdb_scope_preserves_task_and_function_kind_aliases() {
    let fixtures = GeneratedFsdbFixtures::new();
    let fixture = path_str(&fixtures.scope_mixed_kinds());
    let value = run_json_success(&[
        "scope",
        "--waves",
        fixture.as_str(),
        "--max",
        "unlimited",
        "--json",
    ]);

    assert_eq!(
        value["data"],
        json!([
            {"path": "top", "depth": 0, "kind": "module"},
            {"path": "top.cpu", "depth": 1, "kind": "module"},
            {"path": "top.helper", "depth": 1, "kind": "function"},
            {"path": "top.worker", "depth": 1, "kind": "task"},
        ])
    );
}

#[test]
fn fsdb_signal_direct_and_recursive_queries_are_stable() {
    let fixtures = GeneratedFsdbFixtures::new();
    let fixture = path_str(&fixtures.signal_recursive_depth());

    let direct = run_json_success(&[
        "signal",
        "--waves",
        fixture.as_str(),
        "--scope",
        "top",
        "--max",
        "unlimited",
        "--json",
    ]);
    let recursive = run_json_success(&[
        "signal",
        "--waves",
        fixture.as_str(),
        "--scope",
        "top",
        "--recursive",
        "--max",
        "unlimited",
        "--json",
    ]);

    assert_eq!(
        direct["data"],
        json!([
            {"name": "clk", "path": "top.clk", "kind": "wire", "width": 1},
            {"name": "reset_n", "path": "top.reset_n", "kind": "wire", "width": 1},
        ])
    );
    assert_eq!(
        direct["diagnostics"],
        json!([{"kind": "warning", "code": "WPK-W0001", "message": "limit disabled: --max=unlimited"}])
    );

    assert_eq!(
        recursive["data"],
        json!([
            {"name": "clk", "path": "top.clk", "kind": "wire", "width": 1},
            {"name": "reset_n", "path": "top.reset_n", "kind": "wire", "width": 1},
            {"name": "valid", "path": "top.cpu.valid", "kind": "wire", "width": 1},
            {"name": "execute", "path": "top.cpu.core.execute", "kind": "wire", "width": 1},
            {"name": "ready", "path": "top.mem.ready", "kind": "wire", "width": 1},
        ])
    );
}

#[test]
fn fsdb_signal_reports_missing_scope_with_scope_category() {
    let fixtures = GeneratedFsdbFixtures::new();
    let fixture = path_str(&fixtures.signal_recursive_depth());

    wavepeek_cmd()
        .args([
            "signal",
            "--waves",
            fixture.as_str(),
            "--scope",
            "top.missing",
        ])
        .assert()
        .failure()
        .code(1)
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::starts_with("fatal: scope:"))
        .stderr(predicate::str::contains(
            "scope 'top.missing' not found in dump",
        ));
}

#[test]
fn fsdb_bundled_cpu_smoke_supports_info_scope_signal_and_value() {
    let fixture = bundled_cpu_fsdb_path();

    let info = run_json_success(&["info", "--waves", fixture.as_str(), "--json"]);
    let info_again = run_json_success(&["info", "--waves", fixture.as_str(), "--json"]);
    assert_eq!(info, info_again);
    assert_eq!(info["$schema"], expected_schema_url());
    assert_eq!(info["command"], "info");
    assert_eq!(info["diagnostics"], json!([]));
    for field in ["time_unit", "time_start", "time_end"] {
        assert!(
            info["data"][field]
                .as_str()
                .is_some_and(|value| !value.is_empty()),
            "{field} should be a non-empty string"
        );
    }
    assert!(info["data"].get("time_precision").is_none());

    let scopes = run_json_success(&[
        "scope",
        "--waves",
        fixture.as_str(),
        "--max",
        "unlimited",
        "--json",
    ]);
    let scopes_again = run_json_success(&[
        "scope",
        "--waves",
        fixture.as_str(),
        "--max",
        "unlimited",
        "--json",
    ]);
    assert_eq!(scopes["data"], scopes_again["data"]);
    assert_eq!(
        scopes["diagnostics"],
        json!([{"kind": "warning", "code": "WPK-W0001", "message": "limit disabled: --max=unlimited"}])
    );
    let scope_entries = scopes["data"]
        .as_array()
        .expect("scope data should be array");
    assert!(
        !scope_entries.is_empty(),
        "bundled FSDB should expose scopes"
    );
    for entry in scope_entries {
        assert_scope_entry(entry);
    }

    let truncated_scopes =
        run_json_success(&["scope", "--waves", fixture.as_str(), "--max", "1", "--json"]);
    assert_eq!(
        truncated_scopes["data"]
            .as_array()
            .expect("truncated scope data should be array")
            .len(),
        1
    );
    assert!(
        truncated_scopes["diagnostics"][0]["message"]
            .as_str()
            .expect("diagnostic message should be string")
            .contains("truncated output to 1 entries")
    );

    let root_scopes = run_json_success(&[
        "scope",
        "--waves",
        fixture.as_str(),
        "--max",
        "50",
        "--max-depth",
        "0",
        "--json",
    ]);
    let roots = root_scopes["data"]
        .as_array()
        .expect("root scope data should be array");
    assert!(!roots.is_empty(), "bundled FSDB should expose root scopes");
    assert!(roots.iter().all(|entry| entry["depth"] == 0));

    let root_scope = roots[0]["path"]
        .as_str()
        .expect("root scope path should be a string");
    let datatype_signals = run_json_success(&[
        "signal",
        "--waves",
        fixture.as_str(),
        "--scope",
        root_scope,
        "--recursive",
        "--max",
        "unlimited",
        "--json",
    ]);
    let datatype_signal_entries = datatype_signals["data"]
        .as_array()
        .expect("datatype signal data should be array");
    assert!(
        datatype_signal_entries
            .iter()
            .any(|entry| entry["kind"].as_str() == Some("enum")),
        "bundled FSDB should expose datatype-backed enum signals"
    );

    let (signal_scope, signals) = discover_bundled_signal_listing(&fixture);
    let signals_again = signal_listing(&fixture, signal_scope.as_str());
    assert_eq!(signals, signals_again);
    assert!(!signals.is_empty(), "bundled FSDB should expose signals");
    for signal in &signals {
        assert_signal_entry(signal);
    }

    let sample_path = discover_bundled_signal_path(&fixture);
    let sample_time = info["data"]["time_start"]
        .as_str()
        .expect("bundled info should expose time_start");
    let sampled = run_json_success(&[
        "value",
        "--waves",
        fixture.as_str(),
        "--signals",
        sample_path.as_str(),
        "--at",
        sample_time,
        "--json",
    ]);
    assert_eq!(sampled["command"], "value");
    assert_eq!(sampled["diagnostics"], json!([]));
    assert_eq!(sampled["data"][0]["time"], sample_time);
    assert_eq!(sampled["data"][0]["signals"][0]["path"], sample_path);
    assert!(
        sampled["data"][0]["signals"][0]["value"]
            .as_str()
            .is_some_and(|value| value.contains("'h")),
        "sampled FSDB value should render as a Verilog-style literal"
    );

    let change = run_json_success(&[
        "change",
        "--waves",
        fixture.as_str(),
        "--signals",
        sample_path.as_str(),
        "--from",
        sample_time,
        "--to",
        sample_time,
        "--max",
        "1",
        "--json",
    ]);
    assert_eq!(change["command"], "change");
    assert!(change["data"].is_array());

    let property = run_json_success(&[
        "property",
        "--waves",
        fixture.as_str(),
        "--eval",
        sample_path.as_str(),
        "--from",
        sample_time,
        "--to",
        sample_time,
        "--capture",
        "match",
        "--json",
    ]);
    assert_eq!(property["command"], "property");
    assert!(property["data"].is_array());
}

#[test]
fn fsdb_extract_generic_json_matches_vcd_sampling_contract() {
    let fixtures = GeneratedFsdbFixtures::new();
    let fsdb_fixture = path_str(&fixtures.value_vectors());
    let vcd_fixture = path_str(&fixture_path("value_vectors.vcd"));
    let args = [
        "extract",
        "generic",
        "--scope",
        "top",
        "--on",
        "posedge clk",
        "--when",
        "1",
        "--payload",
        "data",
        "--max",
        "3",
        "--json",
    ];

    let fsdb_value = run_json_success_with_waves(fsdb_fixture.as_str(), &args);
    let vcd_value = run_json_success_with_waves(vcd_fixture.as_str(), &args);

    assert_eq!(fsdb_value["command"], "extract generic");
    assert_eq!(fsdb_value["data"], vcd_value["data"]);
    assert_eq!(fsdb_value["diagnostics"], vcd_value["diagnostics"]);
}

#[test]
fn fsdb_value_json_matches_vcd_sampling_contract() {
    let fixtures = GeneratedFsdbFixtures::new();
    let fsdb_fixture = path_str(&fixtures.value_vectors());
    let vcd_fixture = path_str(&fixture_path("value_vectors.vcd"));
    let args = [
        "value",
        "--scope",
        "top",
        "--signals",
        "data,clk,data,nibble,status,asc",
        "--at",
        "7ns",
        "--json",
    ];
    let fsdb_value = run_json_success_with_waves(fsdb_fixture.as_str(), &args);
    let vcd_value = run_json_success_with_waves(vcd_fixture.as_str(), &args);

    assert_eq!(fsdb_value["$schema"], expected_schema_url());
    assert_eq!(fsdb_value["command"], "value");
    assert_eq!(fsdb_value["diagnostics"], json!([]));
    assert_eq!(fsdb_value["data"], vcd_value["data"]);
    assert_eq!(fsdb_value["data"][0]["time"], "7ns");
    assert_eq!(
        fsdb_value["data"][0]["signals"],
        json!([
            {"path": "top.data", "value": "8'h0f"},
            {"path": "top.clk", "value": "1'h1"},
            {"path": "top.data", "value": "8'h0f"},
            {"path": "top.nibble", "value": "4'hx"},
            {"path": "top.status", "value": "4'h3"},
            {"path": "top.asc", "value": "4'h3"},
        ])
    );
}

#[test]
fn fsdb_value_samples_exact_transitions_and_dump_end() {
    let fixtures = GeneratedFsdbFixtures::new();
    let fixture = path_str(&fixtures.value_vectors());

    for (time, expected) in [
        (
            "0ns",
            json!([
                {"path": "top.data", "value": "8'h00"},
                {"path": "top.nibble", "value": "4'ha"},
                {"path": "top.status", "value": "4'hz"},
                {"path": "top.asc", "value": "4'hc"},
            ]),
        ),
        (
            "5ns",
            json!([
                {"path": "top.data", "value": "8'h0f"},
                {"path": "top.nibble", "value": "4'hx"},
                {"path": "top.status", "value": "4'h3"},
                {"path": "top.asc", "value": "4'h3"},
            ]),
        ),
        (
            "10ns",
            json!([
                {"path": "top.data", "value": "8'hf0"},
                {"path": "top.nibble", "value": "4'h5"},
                {"path": "top.status", "value": "4'hx"},
                {"path": "top.asc", "value": "4'ha"},
            ]),
        ),
    ] {
        let value = run_json_success(&[
            "value",
            "--waves",
            fixture.as_str(),
            "--scope",
            "top",
            "--signals",
            "data,nibble,status,asc",
            "--at",
            time,
            "--json",
        ]);
        assert_eq!(value["data"][0]["time"], time);
        assert_eq!(value["data"][0]["signals"], expected);
    }
}

#[test]
fn fsdb_value_change_and_property_use_final_same_time_update() {
    let dir = tempfile::tempdir().expect("tempdir should be created");
    let fsdb_fixture = path_str(&convert_vcd_fixture(dir.path(), "same_time_updates.vcd"));
    let vcd_fixture = path_str(&fixture_path("same_time_updates.vcd"));

    for time in ["5ns", "7ns"] {
        let args = [
            "value",
            "--scope",
            "top",
            "--signals",
            "glitch,bus",
            "--at",
            time,
            "--json",
        ];
        let fsdb_value = run_json_success_with_waves(fsdb_fixture.as_str(), &args);
        let vcd_value = run_json_success_with_waves(vcd_fixture.as_str(), &args);
        assert_eq!(fsdb_value["data"], vcd_value["data"]);
        assert_eq!(
            fsdb_value["data"][0]["signals"],
            json!([
                {"path": "top.glitch", "value": "1'h1"},
                {"path": "top.bus", "value": "2'h2"}
            ])
        );
    }

    let change_args = [
        "change",
        "--scope",
        "top",
        "--signals",
        "glitch,bus",
        "--from",
        "0ns",
        "--to",
        "10ns",
        "--on",
        "*",
        "--max",
        "unlimited",
        "--json",
    ];
    let fsdb_change = run_json_success_with_waves(fsdb_fixture.as_str(), &change_args);
    let vcd_change = run_json_success_with_waves(vcd_fixture.as_str(), &change_args);
    assert_eq!(fsdb_change["data"], vcd_change["data"]);
    assert_eq!(
        fsdb_change["data"],
        json!([
            {"time": "5ns", "sample_time": "5ns", "signals": [
                {"path": "top.glitch", "value": "1'h1"},
                {"path": "top.bus", "value": "2'h2"}
            ]},
            {"time": "10ns", "sample_time": "10ns", "signals": [
                {"path": "top.glitch", "value": "1'h0"},
                {"path": "top.bus", "value": "2'h0"}
            ]}
        ])
    );

    let property_args = [
        "property",
        "--scope",
        "top",
        "--eval",
        "glitch && bus == 2'h2",
        "--from",
        "0ns",
        "--to",
        "10ns",
        "--capture",
        "match",
        "--json",
    ];
    let fsdb_property = run_json_success_with_waves(fsdb_fixture.as_str(), &property_args);
    let vcd_property = run_json_success_with_waves(vcd_fixture.as_str(), &property_args);
    assert_eq!(fsdb_property["data"], vcd_property["data"]);
    assert_eq!(
        fsdb_property["data"],
        json!([{ "time": "5ns", "sample_time": "5ns", "kind": "match" }])
    );
}

#[test]
fn fsdb_value_preserves_scope_relative_human_output_and_abs() {
    let fixtures = GeneratedFsdbFixtures::new();
    let fixture = path_str(&fixtures.value_vectors());

    let relative = run_stdout_success(&[
        "value",
        "--waves",
        fixture.as_str(),
        "--scope",
        "top",
        "--signals",
        "data,clk",
        "--at",
        "7ns",
    ]);
    assert_eq!(relative, "@7ns data=8'h0f clk=1'h1\n");
    assert!(!relative.contains("top.data"));

    let absolute = run_stdout_success(&[
        "value",
        "--waves",
        fixture.as_str(),
        "--scope",
        "top",
        "--signals",
        "data,clk",
        "--at",
        "7ns",
        "--abs",
    ]);
    assert_eq!(absolute, "@7ns top.data=8'h0f top.clk=1'h1\n");
}

#[test]
fn fsdb_value_uses_previous_sample_and_reports_missing_initial_value() {
    let fixtures = GeneratedFsdbFixtures::new();
    let fixture = path_str(&fixtures.value_delayed());
    let previous = run_json_success(&[
        "value",
        "--waves",
        fixture.as_str(),
        "--scope",
        "top",
        "--signals",
        "late",
        "--at",
        "7ns",
        "--json",
    ]);
    assert_eq!(
        previous["data"][0]["signals"],
        json!([{ "path": "top.late", "value": "1'h1" }])
    );

    wavepeek_cmd()
        .args([
            "value",
            "--waves",
            fixture.as_str(),
            "--scope",
            "top",
            "--signals",
            "late",
            "--at",
            "0ns",
        ])
        .assert()
        .failure()
        .code(1)
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::contains(
            "signal 'top.late' has no value at or before requested time",
        ));
}

#[test]
fn fsdb_value_rejects_non_bit_vector_signal() {
    let fixtures = GeneratedFsdbFixtures::new();
    let fixture = path_str(&fixtures.value_real());

    wavepeek_cmd()
        .args([
            "value",
            "--waves",
            fixture.as_str(),
            "--scope",
            "top",
            "--signals",
            "temp",
            "--at",
            "0ns",
        ])
        .assert()
        .failure()
        .code(1)
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::contains(
            "signal 'top.temp' has unsupported non-bit-vector encoding",
        ));
}

#[test]
fn fsdb_change_json_matches_vcd_contracts() {
    let fixtures = GeneratedChangePropertyFsdbFixtures::new();
    let fsdb_fixture = path_str(&fixtures.core());
    let vcd_fixture = path_str(&fixture_path("change_property_core.vcd"));

    let edge_args = [
        "change",
        "--scope",
        "top",
        "--signals",
        "valid,ready,data",
        "--from",
        "0ns",
        "--to",
        "35ns",
        "--on",
        "posedge clk iff valid",
        "--max",
        "10",
        "--json",
    ];
    let fsdb_edge = run_json_success_with_waves(fsdb_fixture.as_str(), &edge_args);
    let vcd_edge = run_json_success_with_waves(vcd_fixture.as_str(), &edge_args);
    assert_eq!(fsdb_edge["$schema"], expected_schema_url());
    assert_eq!(fsdb_edge["command"], "change");
    assert_eq!(fsdb_edge["diagnostics"], json!([]));
    assert_eq!(fsdb_edge["data"], vcd_edge["data"]);
    assert_eq!(
        fsdb_edge["data"],
        json!([
            {"time": "5ns", "sample_time": "5ns", "signals": [
                {"path": "top.valid", "value": "1'h1"},
                {"path": "top.ready", "value": "1'h0"},
                {"path": "top.data", "value": "8'h0f"}
            ]},
            {"time": "15ns", "sample_time": "15ns", "signals": [
                {"path": "top.valid", "value": "1'h1"},
                {"path": "top.ready", "value": "1'h1"},
                {"path": "top.data", "value": "8'h2a"}
            ]}
        ])
    );

    let wildcard_args = [
        "change",
        "--scope",
        "top",
        "--signals",
        "data",
        "--from",
        "0ns",
        "--to",
        "20ns",
        "--on",
        "*",
        "--max",
        "10",
        "--json",
    ];
    let fsdb_wildcard = run_json_success_with_waves(fsdb_fixture.as_str(), &wildcard_args);
    let vcd_wildcard = run_json_success_with_waves(vcd_fixture.as_str(), &wildcard_args);
    assert_eq!(fsdb_wildcard["$schema"], expected_schema_url());
    assert_eq!(fsdb_wildcard["command"], "change");
    assert_eq!(fsdb_wildcard["diagnostics"], vcd_wildcard["diagnostics"]);
    assert_eq!(fsdb_wildcard["data"], vcd_wildcard["data"]);
    assert_eq!(
        fsdb_wildcard["data"],
        json!([
            {"time": "5ns", "sample_time": "5ns", "signals": [{"path": "top.data", "value": "8'h0f"}]},
            {"time": "7ns", "sample_time": "7ns", "signals": [{"path": "top.data", "value": "8'h1f"}]},
            {"time": "15ns", "sample_time": "15ns", "signals": [{"path": "top.data", "value": "8'h2a"}]}
        ])
    );

    let truncated_args = [
        "change",
        "--scope",
        "top",
        "--signals",
        "data",
        "--from",
        "0ns",
        "--to",
        "20ns",
        "--on",
        "*",
        "--max",
        "1",
        "--json",
    ];
    let fsdb_truncated = run_json_success_with_waves(fsdb_fixture.as_str(), &truncated_args);
    let vcd_truncated = run_json_success_with_waves(vcd_fixture.as_str(), &truncated_args);
    assert_eq!(fsdb_truncated["diagnostics"], vcd_truncated["diagnostics"]);
    assert_eq!(
        fsdb_truncated["diagnostics"],
        json!([{"kind": "warning", "code": "WPK-W0002", "message": "truncated output to 1 entries (use --max to increase limit)"}])
    );
    assert_eq!(
        fsdb_truncated["data"],
        json!([{ "time": "5ns", "sample_time": "5ns", "signals": [{"path": "top.data", "value": "8'h0f"}] }])
    );

    let relative = run_stdout_success(&[
        "change",
        "--waves",
        fsdb_fixture.as_str(),
        "--scope",
        "top",
        "--signals",
        "data",
        "--from",
        "0ns",
        "--to",
        "7ns",
        "--on",
        "*",
        "--sample-mode",
        "native",
        "--max",
        "10",
    ]);
    assert!(relative.contains("@5ns data=8'h0f\n"));
    assert!(relative.contains("@7ns data=8'h1f\n"));
    assert!(!relative.contains("top.data"));

    let absolute = run_stdout_success(&[
        "change",
        "--waves",
        fsdb_fixture.as_str(),
        "--scope",
        "top",
        "--signals",
        "data",
        "--from",
        "0ns",
        "--to",
        "7ns",
        "--on",
        "*",
        "--sample-mode",
        "native",
        "--max",
        "10",
        "--abs",
    ]);
    assert!(absolute.contains("@5ns top.data=8'h0f\n"));
    assert!(absolute.contains("@7ns top.data=8'h1f\n"));
}

#[test]
fn fsdb_property_json_matches_vcd_contracts() {
    let fixtures = GeneratedChangePropertyFsdbFixtures::new();
    let fsdb_core = path_str(&fixtures.core());
    let vcd_core = path_str(&fixture_path("change_property_core.vcd"));

    let switch_args = [
        "property",
        "--scope",
        "top",
        "--on",
        "posedge clk",
        "--eval",
        "valid && ready",
        "--capture",
        "switch",
        "--json",
    ];
    let fsdb_switch = run_json_success_with_waves(fsdb_core.as_str(), &switch_args);
    let vcd_switch = run_json_success_with_waves(vcd_core.as_str(), &switch_args);
    assert_eq!(fsdb_switch["$schema"], expected_schema_url());
    assert_eq!(fsdb_switch["command"], "property");
    assert_eq!(fsdb_switch["diagnostics"], json!([]));
    assert_eq!(fsdb_switch["data"], vcd_switch["data"]);
    assert_eq!(
        fsdb_switch["data"],
        json!([
            {"time": "15ns", "sample_time": "15ns", "kind": "assert"},
            {"time": "25ns", "sample_time": "25ns", "kind": "deassert"},
            {"time": "35ns", "sample_time": "35ns", "kind": "assert"}
        ])
    );

    let match_args = [
        "property",
        "--scope",
        "top",
        "--eval",
        "data == 8'h2a",
        "--capture",
        "match",
        "--json",
    ];
    let fsdb_match = run_json_success_with_waves(fsdb_core.as_str(), &match_args);
    let vcd_match = run_json_success_with_waves(vcd_core.as_str(), &match_args);
    assert_eq!(fsdb_match["$schema"], expected_schema_url());
    assert_eq!(fsdb_match["command"], "property");
    assert_eq!(fsdb_match["diagnostics"], vcd_match["diagnostics"]);
    assert_eq!(fsdb_match["data"], vcd_match["data"]);
    assert_eq!(
        fsdb_match["data"],
        json!([{ "time": "15ns", "sample_time": "15ns", "kind": "match" }])
    );

    let assert_iff_args = [
        "property",
        "--scope",
        "top",
        "--from",
        "0ns",
        "--to",
        "35ns",
        "--on",
        "posedge clk iff valid",
        "--eval",
        "ready",
        "--capture",
        "assert",
        "--json",
    ];
    let fsdb_assert = run_json_success_with_waves(fsdb_core.as_str(), &assert_iff_args);
    let vcd_assert = run_json_success_with_waves(vcd_core.as_str(), &assert_iff_args);
    assert_eq!(fsdb_assert["diagnostics"], vcd_assert["diagnostics"]);
    assert_eq!(fsdb_assert["data"], vcd_assert["data"]);
    assert_eq!(
        fsdb_assert["data"],
        json!([{ "time": "15ns", "sample_time": "15ns", "kind": "assert" }])
    );

    let deassert_args = [
        "property",
        "--scope",
        "top",
        "--on",
        "posedge clk",
        "--eval",
        "valid && ready",
        "--capture",
        "deassert",
        "--json",
    ];
    let fsdb_deassert = run_json_success_with_waves(fsdb_core.as_str(), &deassert_args);
    let vcd_deassert = run_json_success_with_waves(vcd_core.as_str(), &deassert_args);
    assert_eq!(fsdb_deassert["diagnostics"], vcd_deassert["diagnostics"]);
    assert_eq!(fsdb_deassert["data"], vcd_deassert["data"]);
    assert_eq!(
        fsdb_deassert["data"],
        json!([{ "time": "25ns", "sample_time": "25ns", "kind": "deassert" }])
    );

    let fsdb_offset = path_str(&fixtures.offset_start());
    let vcd_offset = path_str(&fixture_path("change_property_offset_start.vcd"));
    let offset_args = [
        "property",
        "--from",
        "100ns",
        "--to",
        "110ns",
        "--on",
        "top.valid",
        "--eval",
        "top.ready",
        "--capture",
        "match",
        "--json",
    ];
    let fsdb_offset_value = run_json_success_with_waves(fsdb_offset.as_str(), &offset_args);
    let vcd_offset_value = run_json_success_with_waves(vcd_offset.as_str(), &offset_args);
    assert_eq!(fsdb_offset_value["data"], vcd_offset_value["data"]);
    assert_eq!(fsdb_offset_value["data"], json!([]));
}

#[test]
fn fsdb_raw_event_property_matches_vcd_when_converter_preserves_events() {
    let fixtures = GeneratedChangePropertyFsdbFixtures::new();
    let fsdb_fixture = path_str(&fixtures.events());
    let vcd_fixture = path_str(&fixture_path("change_property_events.vcd"));

    let signal_listing = run_json_success(&[
        "signal",
        "--waves",
        fsdb_fixture.as_str(),
        "--scope",
        "top",
        "--max",
        "unlimited",
        "--json",
    ]);
    assert_eq!(
        signal_listing["data"],
        json!([
            {"name": "armed", "path": "top.armed", "kind": "wire", "width": 1},
            {"name": "tick", "path": "top.tick", "kind": "event", "width": 1}
        ])
    );

    let args = [
        "property",
        "--scope",
        "top",
        "--on",
        "tick",
        "--eval",
        "armed",
        "--capture",
        "match",
        "--json",
    ];
    let fsdb_value = run_json_success_with_waves(fsdb_fixture.as_str(), &args);
    let vcd_value = run_json_success_with_waves(vcd_fixture.as_str(), &args);
    assert_eq!(fsdb_value["data"], vcd_value["data"]);
    assert_eq!(
        fsdb_value["data"],
        json!([
            {"time": "10ns", "sample_time": "10ns", "kind": "match"},
            {"time": "25ns", "sample_time": "25ns", "kind": "match"}
        ])
    );
}

#[test]
fn fsdb_change_property_reject_unsupported_real_operands_clearly() {
    let fixtures = GeneratedChangePropertyFsdbFixtures::new();
    let fixture = path_str(&fixtures.real_output());

    wavepeek_cmd()
        .args([
            "change",
            "--waves",
            fixture.as_str(),
            "--scope",
            "top",
            "--signals",
            "temp",
            "--from",
            "0ns",
            "--to",
            "10ns",
            "--on",
            "clk",
            "--sample-mode",
            "native",
        ])
        .assert()
        .failure()
        .code(1)
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::contains(
            "signal 'top.temp' has unsupported non-bit-vector encoding",
        ));

    for args in [
        vec![
            "property",
            "--waves",
            fixture.as_str(),
            "--scope",
            "top",
            "--on",
            "posedge clk",
            "--sample-mode",
            "native",
            "--eval",
            "temp > 1.0",
            "--capture",
            "match",
        ],
        vec![
            "property",
            "--waves",
            fixture.as_str(),
            "--scope",
            "top",
            "--from",
            "6ns",
            "--to",
            "9ns",
            "--on",
            "*",
            "--sample-mode",
            "native",
            "--eval",
            "temp > 1.0",
        ],
        vec![
            "property",
            "--waves",
            fixture.as_str(),
            "--scope",
            "top",
            "--from",
            "6ns",
            "--to",
            "9ns",
            "--on",
            "posedge clk",
            "--sample-mode",
            "native",
            "--eval",
            "temp > 1.0",
        ],
    ] {
        wavepeek_cmd()
            .args(args)
            .assert()
            .failure()
            .code(1)
            .stdout(predicate::str::is_empty())
            .stderr(predicate::str::contains(
                "signal 'top.temp' has unsupported FSDB expression value encoding",
            ));
    }
}

#[test]
fn fsdb_repeated_value_change_property_queries_are_stable() {
    let vector_fixtures = GeneratedFsdbFixtures::new();
    let property_fixtures = GeneratedChangePropertyFsdbFixtures::new();
    let value_fixture = path_str(&vector_fixtures.value_vectors());
    let property_fixture = path_str(&property_fixtures.core());

    let value_args = [
        "value",
        "--waves",
        value_fixture.as_str(),
        "--scope",
        "top",
        "--signals",
        "data,clk",
        "--at",
        "7ns",
        "--json",
    ];
    let first_value = run_json_success(&value_args);
    let second_value = run_json_success(&value_args);
    assert_eq!(first_value, second_value);

    let change_args = [
        "change",
        "--waves",
        property_fixture.as_str(),
        "--scope",
        "top",
        "--signals",
        "data,valid,ready",
        "--from",
        "0ns",
        "--to",
        "35ns",
        "--on",
        "posedge clk iff valid",
        "--max",
        "unlimited",
        "--json",
    ];
    let first_change = run_json_success(&change_args);
    let second_change = run_json_success(&change_args);
    assert_eq!(first_change, second_change);

    let property_args = [
        "property",
        "--waves",
        property_fixture.as_str(),
        "--scope",
        "top",
        "--on",
        "posedge clk",
        "--eval",
        "valid && ready",
        "--capture",
        "switch",
        "--json",
    ];
    let first_property = run_json_success(&property_args);
    let second_property = run_json_success(&property_args);
    assert_eq!(first_property, second_property);
}

#[test]
fn fsdb_file_failures_are_clean_file_errors() {
    let temp_dir = tempfile::tempdir().expect("tempdir should be created");
    let missing = temp_dir.path().join("missing.fsdb");
    assert_clean_fsdb_file_error(&["info", "--waves", path_str(&missing).as_str()]);

    let empty = temp_dir.path().join("empty.fsdb");
    fs::write(&empty, []).expect("empty temp FSDB should be writable");
    assert_clean_fsdb_file_error(&["info", "--waves", path_str(&empty).as_str()]);

    let garbage = temp_dir.path().join("garbage.fsdb");
    fs::write(&garbage, b"not an fsdb and not a waveform\n")
        .expect("garbage temp FSDB should be writable");
    assert_clean_fsdb_file_error(&["info", "--waves", path_str(&garbage).as_str()]);

    let fixtures = GeneratedChangePropertyFsdbFixtures::new();
    let truncated = temp_dir.path().join("truncated.fsdb");
    fs::copy(fixtures.core(), &truncated).expect("fixture copy should succeed");
    let len = fs::metadata(&truncated)
        .expect("truncated fixture metadata should be readable")
        .len();
    fs::OpenOptions::new()
        .write(true)
        .open(&truncated)
        .expect("truncated fixture should open for writing")
        .set_len((len / 3).max(1))
        .expect("truncated fixture length should be set");
    assert_clean_fsdb_file_error(&[
        "value",
        "--waves",
        path_str(&truncated).as_str(),
        "--scope",
        "top",
        "--signals",
        "data",
        "--at",
        "0ns",
    ]);
}

#[test]
fn fsdb_feature_keeps_valid_vcd_with_fsdb_suffix_on_wellen_path() {
    let mut file = NamedTempFile::with_suffix(".fsdb").expect("temp file should be created");
    file.write_all(
        b"$date\n  test\n$end\n$version wavepeek test $end\n$timescale 1ns $end\n$scope module top $end\n$var wire 1 ! clk $end\n$upscope $end\n$enddefinitions $end\n#0\n0!\n#10\n1!\n",
    )
    .expect("temp file should be writable");
    let path = path_str(file.path());

    wavepeek_cmd()
        .args(["info", "--waves", path.as_str()])
        .assert()
        .success()
        .stdout(predicate::str::contains("time_unit: 1ns"))
        .stdout(predicate::str::contains("time_end: 10ns"))
        .stderr(predicate::str::is_empty());
}

fn run_json_success(args: &[&str]) -> Value {
    let mut normalized_args = args.to_vec();
    if matches!(args.first(), Some(&"change" | &"property")) {
        if !args.contains(&"--on") {
            normalized_args.extend_from_slice(&["--on", "*"]);
        }
        if !args.contains(&"--sample-mode") {
            normalized_args.extend_from_slice(&["--sample-mode", "native"]);
        }
    }

    let assert = wavepeek_cmd()
        .args(normalized_args)
        .assert()
        .success()
        .stderr(predicate::str::is_empty());
    parse_json(&assert.get_output().stdout)
}

fn run_json_success_with_waves(waves: &str, args: &[&str]) -> Value {
    let mut full_args = if args.starts_with(&["extract", "generic"]) {
        vec!["extract", "generic", "--waves", waves]
    } else {
        vec![args[0], "--waves", waves]
    };
    let consumed = if args.starts_with(&["extract", "generic"]) {
        2
    } else {
        1
    };
    full_args.extend_from_slice(&args[consumed..]);
    run_json_success(&full_args)
}

fn run_stdout_success(args: &[&str]) -> String {
    let assert = wavepeek_cmd()
        .args(args)
        .assert()
        .success()
        .stderr(predicate::str::is_empty());
    String::from_utf8(assert.get_output().stdout.clone())
        .expect("command output should be valid UTF-8")
}

fn assert_clean_fsdb_file_error(args: &[&str]) {
    let assert = wavepeek_cmd()
        .args(args)
        .assert()
        .failure()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::starts_with("fatal: file:"));
    let stderr = String::from_utf8(assert.get_output().stderr.clone())
        .expect("stderr should be valid UTF-8");
    for forbidden in ["Novas", "SpringSoft", "Verdi"] {
        assert!(
            !stderr.contains(forbidden),
            "stderr should not leak native banner text {forbidden:?}: {stderr}"
        );
    }
}

fn parse_json(stdout: &[u8]) -> Value {
    serde_json::from_slice(stdout).expect("command output should be valid JSON")
}

fn assert_scope_entry(entry: &Value) {
    let path = entry["path"].as_str().expect("scope path should be string");
    assert!(!path.is_empty(), "scope path should not be empty");
    assert!(!path.contains('/'), "scope path should be dot-separated");
    assert!(
        entry["depth"].as_u64().is_some(),
        "scope depth should be a non-negative integer"
    );
    let kind = entry["kind"].as_str().expect("scope kind should be string");
    assert!(
        SCOPE_KIND_ALIASES.contains(&kind),
        "scope kind {kind:?} should be public-schema compatible"
    );
}

fn assert_signal_entry(entry: &Value) {
    let name = entry["name"]
        .as_str()
        .expect("signal name should be string");
    let path = entry["path"]
        .as_str()
        .expect("signal path should be string");
    assert!(!name.is_empty(), "signal name should not be empty");
    assert!(!path.is_empty(), "signal path should not be empty");
    assert!(!path.contains('/'), "signal path should be dot-separated");
    let kind = entry["kind"]
        .as_str()
        .expect("signal kind should be string");
    assert!(
        SIGNAL_KIND_ALIASES.contains(&kind),
        "signal kind {kind:?} should be public-schema compatible"
    );
    if let Some(width) = entry.get("width") {
        assert!(
            width.as_u64().is_some_and(|width| width > 0),
            "signal width should be a positive integer when present"
        );
    }
}

fn discover_bundled_signal_path(fixture: &str) -> String {
    let (_scope, signals) = discover_bundled_signal_listing(fixture);
    signals
        .iter()
        .filter_map(|signal| signal["path"].as_str())
        .find(|path| is_simple_expr_path(path))
        .expect("bundled FSDB should expose at least one signal usable as an expression path")
        .to_string()
}

fn discover_bundled_signal_listing(fixture: &str) -> (String, Vec<Value>) {
    let scopes = run_json_success(&["scope", "--waves", fixture, "--max", "unlimited", "--json"]);
    for scope in scopes["data"]
        .as_array()
        .expect("scope data should be array")
    {
        let Some(scope_path) = scope["path"].as_str() else {
            continue;
        };
        let signals = signal_listing(fixture, scope_path);
        if !signals.is_empty() {
            return (scope_path.to_string(), signals);
        }
    }
    panic!("bundled FSDB should have at least one scope with recursive signals");
}

fn signal_listing(fixture: &str, scope: &str) -> Vec<Value> {
    let value = run_json_success(&[
        "signal",
        "--waves",
        fixture,
        "--scope",
        scope,
        "--recursive",
        "--max",
        "5",
        "--json",
    ]);
    value["data"]
        .as_array()
        .expect("signal data should be array")
        .clone()
}

fn is_simple_expr_path(path: &str) -> bool {
    path.split('.').all(|segment| {
        let mut chars = segment.chars();
        chars
            .next()
            .is_some_and(|ch| ch == '_' || ch.is_ascii_alphabetic())
            && chars.all(|ch| ch == '_' || ch.is_ascii_alphanumeric())
    })
}

struct GeneratedFsdbFixtures {
    _dir: TempDir,
    scope_mixed_kinds: PathBuf,
    signal_recursive_depth: PathBuf,
    value_vectors: PathBuf,
    value_delayed: PathBuf,
    value_real: PathBuf,
}

struct GeneratedChangePropertyFsdbFixtures {
    _dir: TempDir,
    core: PathBuf,
    offset_start: PathBuf,
    events: PathBuf,
    real_output: PathBuf,
}

impl GeneratedChangePropertyFsdbFixtures {
    fn new() -> Self {
        require_vcd2fsdb();
        let dir = tempfile::tempdir().expect("tempdir should be created");
        let core = convert_vcd_fixture(dir.path(), "change_property_core.vcd");
        let offset_start = convert_vcd_fixture(dir.path(), "change_property_offset_start.vcd");
        let events = convert_vcd_fixture(dir.path(), "change_property_events.vcd");
        let real_output = convert_vcd_fixture(dir.path(), "change_property_real_output.vcd");
        Self {
            _dir: dir,
            core,
            offset_start,
            events,
            real_output,
        }
    }

    fn core(&self) -> PathBuf {
        self.core.clone()
    }

    fn offset_start(&self) -> PathBuf {
        self.offset_start.clone()
    }

    fn events(&self) -> PathBuf {
        self.events.clone()
    }

    fn real_output(&self) -> PathBuf {
        self.real_output.clone()
    }
}

impl GeneratedFsdbFixtures {
    fn new() -> Self {
        require_vcd2fsdb();
        let dir = tempfile::tempdir().expect("tempdir should be created");
        let scope_mixed_kinds = convert_vcd_fixture(dir.path(), "scope_mixed_kinds.vcd");
        let signal_recursive_depth = convert_vcd_fixture(dir.path(), "signal_recursive_depth.vcd");
        let value_vectors = convert_vcd_fixture(dir.path(), "value_vectors.vcd");
        let value_delayed = convert_vcd_fixture(dir.path(), "value_delayed.vcd");
        let value_real = convert_vcd_fixture(dir.path(), "value_real.vcd");
        Self {
            _dir: dir,
            scope_mixed_kinds,
            signal_recursive_depth,
            value_vectors,
            value_delayed,
            value_real,
        }
    }

    fn scope_mixed_kinds(&self) -> PathBuf {
        self.scope_mixed_kinds.clone()
    }

    fn signal_recursive_depth(&self) -> PathBuf {
        self.signal_recursive_depth.clone()
    }

    fn value_vectors(&self) -> PathBuf {
        self.value_vectors.clone()
    }

    fn value_delayed(&self) -> PathBuf {
        self.value_delayed.clone()
    }

    fn value_real(&self) -> PathBuf {
        self.value_real.clone()
    }
}

fn require_vcd2fsdb() {
    let status = Command::new("sh")
        .arg("-c")
        .arg("command -v vcd2fsdb >/dev/null 2>&1")
        .status()
        .expect("shell should be available to check vcd2fsdb");
    assert!(
        status.success(),
        "vcd2fsdb not found on PATH; load the Verdi environment or run WAVEPEEK_IN_CONTAINER=1 just test-fsdb"
    );
}

fn convert_vcd_fixture(dir: &Path, name: &str) -> PathBuf {
    let source = fixture_path(name);
    let output = dir.join(name.replace(".vcd", ".fsdb"));
    let converter_output = Command::new("vcd2fsdb")
        .current_dir(dir)
        .arg(source)
        .arg("-o")
        .arg(&output)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("vcd2fsdb should be available from the Verdi environment");
    assert!(
        converter_output.status.success(),
        "vcd2fsdb should convert {name}; stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&converter_output.stdout),
        String::from_utf8_lossy(&converter_output.stderr)
    );
    output
}

fn bundled_cpu_fsdb_path() -> String {
    std::path::Path::new(&std::env::var("VERDI_HOME").expect("VERDI_HOME must be set"))
        .join("share")
        .join("VIA")
        .join("demo")
        .join("waveform")
        .join("cpu.fsdb")
        .to_string_lossy()
        .into_owned()
}

fn path_str(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}
