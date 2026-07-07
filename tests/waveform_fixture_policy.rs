use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct WaveformPolicy {
    hand_dumps: Vec<HandDump>,
    source_backed: Vec<SourceBacked>,
    derived_outputs: Vec<DerivedOutput>,
}

#[derive(Debug, Deserialize)]
struct HandDump {
    path: String,
    reason: String,
}

#[derive(Debug, Deserialize)]
struct SourceBacked {
    name: String,
    source: String,
    outputs: Vec<String>,
    reason: String,
}

#[derive(Debug, Deserialize)]
struct DerivedOutput {
    source: String,
    output: String,
    reason: String,
}

#[test]
fn waveform_fixture_policy_manifest_matches_repository_layout() {
    let root = repository_root();
    let policy = load_policy(&root);

    assert_non_empty_reasons(&policy);
    assert_hand_dumps_are_manifested(&root, &policy);
    assert_source_fixtures_are_manifested(&root, &policy);
    assert_generated_outputs_exist(&root, &policy);
    assert_no_tracked_fsdb_fixtures(&root);
    assert_generated_directory_is_ignored(&root);
}

fn repository_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn load_policy(root: &Path) -> WaveformPolicy {
    let path = root.join("tests/fixtures/waveform_policy.json");
    let text = fs::read_to_string(&path).expect("waveform policy manifest should be readable");
    serde_json::from_str(&text).expect("waveform policy manifest should be valid JSON")
}

fn assert_non_empty_reasons(policy: &WaveformPolicy) {
    for hand in &policy.hand_dumps {
        assert!(
            !hand.reason.trim().is_empty(),
            "hand dump {} must have a non-empty reason",
            hand.path
        );
    }
    for source in &policy.source_backed {
        assert!(
            !source.name.trim().is_empty(),
            "source-backed fixture {} must have a non-empty name",
            source.source
        );
        assert!(
            !source.reason.trim().is_empty(),
            "source-backed fixture {} must have a non-empty reason",
            source.source
        );
    }
    for derived in &policy.derived_outputs {
        assert!(
            !derived.reason.trim().is_empty(),
            "derived output {} must have a non-empty reason",
            derived.output
        );
    }
}

fn assert_hand_dumps_are_manifested(root: &Path, policy: &WaveformPolicy) {
    let expected = policy
        .hand_dumps
        .iter()
        .map(|entry| entry.path.clone())
        .collect::<BTreeSet<_>>();
    let actual = tracked_files(root)
        .unwrap_or_else(|| fixture_filesystem_paths(root.join("tests/fixtures/hand")))
        .into_iter()
        .filter(|path| path.starts_with("tests/fixtures/hand/"))
        .filter(|path| has_waveform_dump_extension(path))
        .collect::<BTreeSet<_>>();

    assert_eq!(
        actual, expected,
        "tracked hand VCD/FST dumps must match tests/fixtures/waveform_policy.json"
    );
    for path in expected {
        assert!(
            root.join(&path).is_file(),
            "manifested hand dump is missing: {path}"
        );
    }
}

fn assert_source_fixtures_are_manifested(root: &Path, policy: &WaveformPolicy) {
    let expected = policy
        .source_backed
        .iter()
        .map(|entry| entry.source.clone())
        .collect::<BTreeSet<_>>();
    let actual = fixture_filesystem_paths(root.join("tests/fixtures/source"))
        .into_iter()
        .filter(|path| path.ends_with(".v"))
        .collect::<BTreeSet<_>>();

    assert_eq!(
        actual, expected,
        "Verilog fixture sources must match tests/fixtures/waveform_policy.json"
    );
}

fn assert_generated_outputs_exist(root: &Path, policy: &WaveformPolicy) {
    for entry in &policy.source_backed {
        assert!(
            root.join(&entry.source).is_file(),
            "source-backed fixture source is missing: {}",
            entry.source
        );
        for output in &entry.outputs {
            assert_generated_output(root, output);
        }
    }
    for entry in &policy.derived_outputs {
        assert!(
            root.join(&entry.source).is_file(),
            "derived fixture source is missing: {}",
            entry.source
        );
        assert_generated_output(root, &entry.output);
    }
}

fn assert_generated_output(root: &Path, output: &str) {
    assert!(
        output.starts_with("tests/fixtures/generated/"),
        "generated output must live under tests/fixtures/generated/: {output}"
    );
    assert!(
        root.join(output).is_file(),
        "generated fixture is missing: {output}; run `just prepare-waveform-fixtures`"
    );
}

fn assert_no_tracked_fsdb_fixtures(root: &Path) {
    let tracked = tracked_files(root).unwrap_or_default();
    let fsdb = tracked
        .iter()
        .filter(|path| path.ends_with(".fsdb") || path.contains(".fsdb."))
        .collect::<Vec<_>>();
    assert!(
        fsdb.is_empty(),
        "FSDB fixtures must not be tracked: {fsdb:?}"
    );
}

fn assert_generated_directory_is_ignored(root: &Path) {
    let gitignore = fs::read_to_string(root.join(".gitignore")).expect(".gitignore should exist");
    assert!(
        gitignore
            .lines()
            .any(|line| line.trim() == "/tests/fixtures/generated/"),
        ".gitignore must ignore /tests/fixtures/generated/"
    );
}

fn tracked_files(root: &Path) -> Option<Vec<String>> {
    let output = Command::new("git")
        .args(["ls-files", "--", "tests/fixtures"])
        .current_dir(root)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let stdout = String::from_utf8(output.stdout).ok()?;
    Some(stdout.lines().map(ToOwned::to_owned).collect())
}

fn fixture_filesystem_paths(root: PathBuf) -> Vec<String> {
    let repository_root = repository_root();
    let mut paths = Vec::new();
    collect_fixture_paths(&repository_root, &root, &mut paths);
    paths.sort();
    paths
}

fn collect_fixture_paths(repository_root: &Path, path: &Path, paths: &mut Vec<String>) {
    let Ok(entries) = fs::read_dir(path) else {
        return;
    };
    for entry in entries {
        let entry = entry.expect("fixture directory entry should be readable");
        let path = entry.path();
        if path.is_dir() {
            collect_fixture_paths(repository_root, &path, paths);
        } else {
            paths.push(
                path.strip_prefix(repository_root)
                    .expect("fixture path should be under repository root")
                    .to_string_lossy()
                    .replace('\\', "/"),
            );
        }
    }
}

fn has_waveform_dump_extension(path: &str) -> bool {
    path.ends_with(".vcd") || path.ends_with(".fst")
}
