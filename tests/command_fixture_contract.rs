mod common;

use common::command_cases::{
    command_manifest_file_names, command_snapshot_file_names, fixture_cli_path,
    load_negative_manifest, load_positive_manifest, referenced_snapshot_file_names,
};

#[test]
fn command_runtime_manifests_and_snapshots_follow_shared_contract() {
    let positive = load_positive_manifest("command_runtime_positive_manifest.json");
    let negative = load_negative_manifest("command_runtime_negative_manifest.json");

    assert_eq!(
        command_manifest_file_names(),
        vec![
            "AGENTS.md".to_string(),
            "command_runtime_negative_manifest.json".to_string(),
            "command_runtime_positive_manifest.json".to_string(),
        ]
    );

    let referenced_snapshots = referenced_snapshot_file_names(&positive, &negative);
    let existing_snapshots = command_snapshot_file_names();
    assert_eq!(referenced_snapshots, existing_snapshots);

    assert!(fixture_cli_path("command_runtime_positive_manifest.json").exists());
    assert!(fixture_cli_path("command_runtime_negative_manifest.json").exists());
}

#[test]
fn command_runtime_positive_manifest_cases_pass() {
    let manifest = load_positive_manifest("command_runtime_positive_manifest.json");
    for case in &manifest.cases {
        common::command_cases::assert_positive_case(case);
    }
}

#[test]
fn command_runtime_negative_manifest_cases_pass() {
    let manifest = load_negative_manifest("command_runtime_negative_manifest.json");
    for case in &manifest.cases {
        common::command_cases::assert_negative_case(case);
    }
}
