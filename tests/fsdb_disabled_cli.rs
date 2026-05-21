#![cfg(not(feature = "fsdb"))]

use std::io::Write;
use std::path::Path;

use assert_cmd::prelude::*;
use predicates::prelude::*;
use tempfile::NamedTempFile;

mod common;
use common::wavepeek_cmd;

const FSDB_DISABLED_STDERR: &str = "error: file: FSDB input requires a wavepeek binary built with FSDB support; reinstall with --features fsdb and provide a licensed VERDI_HOME\n";

fn write_temp_file(bytes: &[u8], suffix: &str) -> NamedTempFile {
    let mut file = tempfile::Builder::new()
        .suffix(suffix)
        .tempfile()
        .expect("temp file should be created");
    file.write_all(bytes).expect("temp file should be writable");
    file
}

fn assert_disabled_for_path(path: &Path) {
    let mut command = wavepeek_cmd();
    let path = path.to_string_lossy().into_owned();

    command
        .args(["info", "--waves", path.as_str()])
        .assert()
        .failure()
        .code(2)
        .stdout(predicate::str::is_empty())
        .stderr(predicate::eq(FSDB_DISABLED_STDERR));
}

#[test]
fn info_reports_feature_required_for_invalid_fsdb_suffix() {
    let file = write_temp_file(b"not-a-waveform", ".fsdb");

    assert_disabled_for_path(file.path());
}

#[test]
fn info_reports_feature_required_for_invalid_uppercase_fsdb_suffix() {
    let file = write_temp_file(b"not-a-waveform", ".FSDB");

    assert_disabled_for_path(file.path());
}

#[test]
fn info_reports_feature_required_for_invalid_fsdb_gz_suffix() {
    let file = write_temp_file(b"not-a-waveform", ".fsdb.gz");

    assert_disabled_for_path(file.path());
}

#[test]
fn info_missing_fsdb_suffix_keeps_cannot_open_error() {
    let dir = tempfile::tempdir().expect("tempdir should be created");
    let missing = dir.path().join("missing.fsdb");
    let missing = missing.to_string_lossy().into_owned();

    let mut command = wavepeek_cmd();
    command
        .args(["info", "--waves", missing.as_str()])
        .assert()
        .failure()
        .code(2)
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::starts_with("error: file: cannot open"))
        .stderr(predicate::str::contains("FSDB input requires").not());
}

#[test]
fn info_unrelated_invalid_suffix_keeps_parse_error() {
    let file = write_temp_file(b"not-a-waveform", ".notfsdb");
    let path = file.path().to_string_lossy().into_owned();

    let mut command = wavepeek_cmd();
    command
        .args(["info", "--waves", path.as_str()])
        .assert()
        .failure()
        .code(2)
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::starts_with("error: file: cannot parse"))
        .stderr(predicate::str::contains("FSDB input requires").not());
}

#[test]
fn info_valid_vcd_with_fsdb_suffix_still_succeeds() {
    let file = write_temp_file(
        b"$date\n  test\n$end\n$version wavepeek test $end\n$timescale 1ns $end\n$scope module top $end\n$var wire 1 ! clk $end\n$upscope $end\n$enddefinitions $end\n#0\n0!\n#10\n1!\n",
        ".fsdb",
    );
    let path = file.path().to_string_lossy().into_owned();

    let mut command = wavepeek_cmd();
    command
        .args(["info", "--waves", path.as_str()])
        .assert()
        .success()
        .stdout(predicate::str::contains("time_unit: 1ns"))
        .stdout(predicate::str::contains("time_start: 0ns"))
        .stdout(predicate::str::contains("time_end: 10ns"))
        .stderr(predicate::str::is_empty());
}
