use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn noop_reports_success_without_loading_verdi() {
    let mut command = Command::cargo_bin("fsdb_demo").unwrap();
    command
        .arg("noop")
        .assert()
        .success()
        .stdout(predicate::str::contains("command: noop"))
        .stdout(predicate::str::contains("status: ok"));
}

#[test]
fn build_info_reports_mock_bridge_path() {
    let mut command = Command::cargo_bin("fsdb_demo").unwrap();
    command
        .arg("build-info")
        .assert()
        .success()
        .stdout(predicate::str::contains("mock-bridge-path: "))
        .stdout(predicate::str::contains("verdi-bridge-status: "))
        .stdout(predicate::str::contains("fsdb-writer-path: "));
}

#[test]
fn probe_works_with_explicit_mock_bridge() {
    let mut command = Command::cargo_bin("fsdb_demo").unwrap();
    command
        .args([
            "probe",
            "--waves",
            "fixture.fsdb",
            "--bridge",
            env!("FSDB_DEMO_MOCK_BRIDGE_PATH"),
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("bridge-kind: mock"))
        .stdout(predicate::str::contains("signal-count: 7"))
        .stdout(predicate::str::contains("end-time-raw: 4242"));
}

#[test]
fn probe_without_explicit_bridge_fails_when_verdi_bridge_was_not_built() {
    if env!("FSDB_DEMO_VERDI_BRIDGE_STATUS") == "built" {
        return;
    }

    let mut command = Command::cargo_bin("fsdb_demo").unwrap();
    command
        .args(["probe", "--waves", "fixture.fsdb"])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "error: demo: no Verdi bridge was built",
        ));
}
