use std::fs;
use std::path::Path;

use regex::Regex;

// Capability-oriented naming targets enforced by this hygiene guard:
// - tests/expression_parse.rs
// - tests/expression_event_runtime.rs
// - tests/expression_integral_boolean.rs
// - tests/expression_rich_types.rs
// - bench/expr/expr_parser.rs
// - bench/expr/expr_event_runtime.rs
// - bench/expr/expr_integral_boolean.rs
// - bench/expr/expr_rich_types.rs

#[test]
fn expression_phase_tags_are_limited_to_roadmap_and_history() {
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let patterns = banned_patterns();
    let mut offenders = Vec::new();

    walk(repo_root, repo_root, &patterns, &mut offenders);
    offenders.sort();

    assert!(
        offenders.is_empty(),
        "forbidden expression rollout tags found outside allowlist:\n{}",
        offenders.join("\n")
    );
}

fn walk(root: &Path, path: &Path, patterns: &[(Regex, &'static str)], offenders: &mut Vec<String>) {
    let entries = fs::read_dir(path).expect("repository path should be readable");
    for entry in entries {
        let entry = entry.expect("directory entry should be readable");
        let entry_path = entry.path();
        let rel = entry_path
            .strip_prefix(root)
            .expect("entry should stay under repository root")
            .to_string_lossy()
            .replace('\\', "/");

        let file_type = entry
            .file_type()
            .expect("directory entry type should be readable");
        if file_type.is_dir() {
            if is_skipped_dir(rel.as_str()) {
                continue;
            }
            walk(root, entry_path.as_path(), patterns, offenders);
            continue;
        }

        if file_type.is_file() {
            scan_file(root, entry_path.as_path(), patterns, offenders);
        }
    }
}

fn scan_file(
    root: &Path,
    path: &Path,
    patterns: &[(Regex, &'static str)],
    offenders: &mut Vec<String>,
) {
    let rel = path
        .strip_prefix(root)
        .expect("file should stay under repository root")
        .to_string_lossy()
        .replace('\\', "/");
    if is_allowlisted(rel.as_str()) || rel.ends_with(".fst") {
        return;
    }

    if let Some(reason) = first_match(rel.as_str(), patterns) {
        offenders.push(format!("{rel}: path matches {reason}"));
        return;
    }

    if let Ok(contents) = fs::read_to_string(path)
        && let Some(reason) = first_match(contents.as_str(), patterns)
    {
        offenders.push(format!("{rel}: content matches {reason}"));
    }
}

fn first_match<'a>(text: &str, patterns: &'a [(Regex, &'static str)]) -> Option<&'a str> {
    patterns.iter().find_map(|(regex, name)| {
        if regex.is_match(text) {
            Some(*name)
        } else {
            None
        }
    })
}

fn banned_patterns() -> Vec<(Regex, &'static str)> {
    vec![
        (
            Regex::new(r"expression_c[1-4]").expect("pattern should compile"),
            "expression_c* test/file stem",
        ),
        (
            Regex::new(r"expr_c[1-4]").expect("pattern should compile"),
            "expr_c* benchmark target/file stem",
        ),
        (
            Regex::new(r"C[1-5]-(PARSE|SEMANTIC|RUNTIME)-").expect("pattern should compile"),
            "rollout-coded diagnostic family",
        ),
        (
            Regex::new(
                r"(tests/fixtures/expr|bench/expr/scenarios|bench/expr/runs|tests/snapshots)[^\n]*\bc[1-4](_|-)",
            )
            .expect("pattern should compile"),
            "rollout-coded manifest/scenario/run/snapshot prefix",
        ),
    ]
}

fn is_allowlisted(rel: &str) -> bool {
    rel == "docs/expression_roadmap.md"
        || rel == "docs/ROADMAP.md"
        || rel.starts_with("docs/exec-plans/completed/")
}

fn is_skipped_dir(rel: &str) -> bool {
    rel == ".git"
        || rel == "target"
        || rel == "__pycache__"
        || rel.ends_with("/__pycache__")
        || rel == ".venv"
        || rel.ends_with("/.venv")
        || rel == ".pytest_cache"
        || rel.ends_with("/.pytest_cache")
        || rel == ".mypy_cache"
        || rel.ends_with("/.mypy_cache")
        || rel == ".ruff_cache"
        || rel.ends_with("/.ruff_cache")
        || rel == "node_modules"
        || rel.ends_with("/node_modules")
}
