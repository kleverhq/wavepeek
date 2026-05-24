#![cfg(feature = "fsdb")]

use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

use assert_cmd::prelude::*;
use predicates::prelude::*;
use serde_json::{Value, json};
use tempfile::{NamedTempFile, TempDir};

mod common;
use common::{expected_schema_url, fixture_path, wavepeek_cmd};

#[test]
fn fsdb_info_json_matches_vcd_derived_fixture() {
    let fixtures = GeneratedFsdbFixtures::new();
    let fixture = fixtures.signal_recursive_depth();
    let assert = wavepeek_cmd()
        .args(["info", "--waves", path_str(&fixture).as_str(), "--json"])
        .assert()
        .success()
        .stderr(predicate::str::is_empty());

    let value = parse_json(&assert.get_output().stdout);
    assert_eq!(value["$schema"], expected_schema_url());
    assert!(value.get("schema_version").is_none());
    assert_eq!(value["command"], "info");
    assert_eq!(value["warnings"], json!([]));
    assert_eq!(value["data"]["time_unit"], "1ns");
    assert_eq!(value["data"]["time_start"], "0ns");
    assert_eq!(value["data"]["time_end"], "10ns");
}

#[test]
fn fsdb_scope_json_is_sorted_and_depth_bounded() {
    let fixtures = GeneratedFsdbFixtures::new();
    let fixture = path_str(&fixtures.signal_recursive_depth());

    let all = wavepeek_cmd()
        .args([
            "scope",
            "--waves",
            fixture.as_str(),
            "--max",
            "unlimited",
            "--json",
        ])
        .assert()
        .success();
    let root_only = wavepeek_cmd()
        .args([
            "scope",
            "--waves",
            fixture.as_str(),
            "--max",
            "50",
            "--max-depth",
            "0",
            "--json",
        ])
        .assert()
        .success();

    let all = parse_json(&all.get_output().stdout);
    assert_eq!(
        all["data"],
        json!([
            {"path": "top", "depth": 0, "kind": "module"},
            {"path": "top.cpu", "depth": 1, "kind": "module"},
            {"path": "top.cpu.core", "depth": 2, "kind": "module"},
            {"path": "top.mem", "depth": 1, "kind": "module"},
        ])
    );
    assert_eq!(all["warnings"], json!(["limit disabled: --max=unlimited"]));

    let root_only = parse_json(&root_only.get_output().stdout);
    assert_eq!(
        root_only["data"],
        json!([{ "path": "top", "depth": 0, "kind": "module" }])
    );
    assert_eq!(root_only["warnings"], json!([]));
}

#[test]
fn fsdb_scope_preserves_task_and_function_kind_aliases() {
    let fixtures = GeneratedFsdbFixtures::new();
    let fixture = path_str(&fixtures.scope_mixed_kinds());
    let assert = wavepeek_cmd()
        .args([
            "scope",
            "--waves",
            fixture.as_str(),
            "--max",
            "unlimited",
            "--json",
        ])
        .assert()
        .success();

    let value = parse_json(&assert.get_output().stdout);
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

    let direct = wavepeek_cmd()
        .args([
            "signal",
            "--waves",
            fixture.as_str(),
            "--scope",
            "top",
            "--max",
            "unlimited",
            "--json",
        ])
        .assert()
        .success();
    let recursive = wavepeek_cmd()
        .args([
            "signal",
            "--waves",
            fixture.as_str(),
            "--scope",
            "top",
            "--recursive",
            "--max",
            "unlimited",
            "--json",
        ])
        .assert()
        .success();

    let direct = parse_json(&direct.get_output().stdout);
    assert_eq!(
        direct["data"],
        json!([
            {"name": "clk", "path": "top.clk", "kind": "wire", "width": 1},
            {"name": "reset_n", "path": "top.reset_n", "kind": "wire", "width": 1},
        ])
    );
    assert_eq!(
        direct["warnings"],
        json!(["limit disabled: --max=unlimited"])
    );

    let recursive = parse_json(&recursive.get_output().stdout);
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
        .stderr(predicate::str::starts_with("error: scope:"))
        .stderr(predicate::str::contains(
            "scope 'top.missing' not found in dump",
        ));
}

#[test]
fn fsdb_bundled_cpu_smoke_supports_info_scope_and_signal() {
    let fixture = bundled_cpu_fsdb_path();

    let info = wavepeek_cmd()
        .args(["info", "--waves", fixture.as_str(), "--json"])
        .assert()
        .success();
    let info = parse_json(&info.get_output().stdout);
    assert_eq!(info["data"]["time_unit"], "1ns");
    assert_eq!(info["data"]["time_start"], "0ns");
    assert_eq!(info["data"]["time_end"], "12500ns");

    let scope = wavepeek_cmd()
        .args(["scope", "--waves", fixture.as_str(), "--max", "5", "--json"])
        .assert()
        .success();
    let scope = parse_json(&scope.get_output().stdout);
    assert_eq!(
        scope["data"],
        json!([
            {"path": "system", "depth": 0, "kind": "module"},
            {"path": "system.CHILD1", "depth": 1, "kind": "module"},
            {"path": "system.CHILD1.FSM1_COMB", "depth": 2, "kind": "begin"},
            {"path": "system.CHILD1.FSM1_SEQ", "depth": 2, "kind": "begin"},
            {"path": "system.CHILD2", "depth": 1, "kind": "module"},
        ])
    );
    assert_eq!(
        scope["warnings"],
        json!(["truncated output to 5 entries (use --max to increase limit)"])
    );

    let signal = wavepeek_cmd()
        .args([
            "signal",
            "--waves",
            fixture.as_str(),
            "--scope",
            "system",
            "--recursive",
            "--max",
            "5",
            "--json",
        ])
        .assert()
        .success();
    let signal = parse_json(&signal.get_output().stdout);
    assert_eq!(
        signal["data"],
        json!([
            {"name": "En_A", "path": "system.En_A", "kind": "wire", "width": 1},
            {"name": "En_AB", "path": "system.En_AB", "kind": "wire", "width": 1},
            {"name": "En_AC", "path": "system.En_AC", "kind": "wire", "width": 1},
            {"name": "En_AD", "path": "system.En_AD", "kind": "wire", "width": 1},
            {"name": "En_B", "path": "system.En_B", "kind": "wire", "width": 1},
        ])
    );
}

#[test]
fn fsdb_unsupported_value_change_and_property_fail_clearly() {
    let fixture = bundled_cpu_fsdb_path();

    for (args, expected) in [
        (
            vec![
                "value",
                "--waves",
                fixture.as_str(),
                "--signals",
                "system.En_A",
                "--at",
                "0ns",
            ],
            "FSDB value sampling is not implemented yet; info, scope, and signal are supported by this build",
        ),
        (
            vec![
                "change",
                "--waves",
                fixture.as_str(),
                "--signals",
                "system.En_A",
                "--from",
                "0ns",
                "--to",
                "10ns",
            ],
            "FSDB change collection is not implemented yet; info, scope, and signal are supported by this build",
        ),
        (
            vec![
                "property",
                "--waves",
                fixture.as_str(),
                "--eval",
                "system.En_A",
                "--from",
                "0ns",
                "--to",
                "10ns",
            ],
            "FSDB property evaluation is not implemented yet; info, scope, and signal are supported by this build",
        ),
    ] {
        wavepeek_cmd()
            .args(args)
            .assert()
            .failure()
            .code(1)
            .stdout(predicate::str::is_empty())
            .stderr(predicate::str::contains(expected));
    }
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

fn parse_json(stdout: &[u8]) -> Value {
    serde_json::from_slice(stdout).expect("command output should be valid JSON")
}

struct GeneratedFsdbFixtures {
    _dir: TempDir,
    scope_mixed_kinds: std::path::PathBuf,
    signal_recursive_depth: std::path::PathBuf,
}

impl GeneratedFsdbFixtures {
    fn new() -> Self {
        let dir = tempfile::tempdir().expect("tempdir should be created");
        let scope_mixed_kinds = convert_vcd_fixture(dir.path(), "scope_mixed_kinds.vcd");
        let signal_recursive_depth = convert_vcd_fixture(dir.path(), "signal_recursive_depth.vcd");
        Self {
            _dir: dir,
            scope_mixed_kinds,
            signal_recursive_depth,
        }
    }

    fn scope_mixed_kinds(&self) -> std::path::PathBuf {
        self.scope_mixed_kinds.clone()
    }

    fn signal_recursive_depth(&self) -> std::path::PathBuf {
        self.signal_recursive_depth.clone()
    }
}

fn convert_vcd_fixture(dir: &Path, name: &str) -> std::path::PathBuf {
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
