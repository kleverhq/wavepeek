#![allow(dead_code)]

use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;
use serde_json::Value;

use super::wavepeek_cmd;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CommandCaseCommand {
    Change,
    Property,
}

impl CommandCaseCommand {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Change => "change",
            Self::Property => "property",
        }
    }

    pub fn snapshot_suite_prefix(self) -> &'static str {
        match self {
            Self::Change => "change_cli",
            Self::Property => "property_cli",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OutputMode {
    Json,
    Human,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PositiveManifest {
    pub cases: Vec<PositiveCase>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PositiveCase {
    pub name: String,
    pub command: CommandCaseCommand,
    pub args: Vec<String>,
    pub output_mode: OutputMode,
    pub exit_code: i32,
    #[serde(default)]
    pub expected_data: Option<Value>,
    #[serde(default)]
    pub expected_warnings: Vec<String>,
    #[serde(default)]
    pub stdout_snapshot: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NegativeManifest {
    pub cases: Vec<NegativeCase>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NegativeCase {
    pub name: String,
    pub command: CommandCaseCommand,
    pub args: Vec<String>,
    pub exit_code: i32,
    pub stderr_prefix: String,
    #[serde(default)]
    pub stderr_snapshot: Option<String>,
}

pub fn fixture_cli_path(file_name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("cli")
        .join(file_name)
}

pub fn command_snapshot_path(file_name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("snapshots")
        .join(file_name)
}

pub fn command_manifest_file_names() -> Vec<String> {
    let mut entries = fs::read_dir(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("fixtures")
            .join("cli"),
    )
    .expect("command fixture directory should be readable")
    .map(|entry| entry.expect("fixture entry should be readable"))
    .filter_map(|entry| {
        entry
            .file_type()
            .expect("fixture entry type should be readable")
            .is_file()
            .then(|| entry.file_name().to_string_lossy().into_owned())
    })
    .collect::<Vec<_>>();
    entries.sort();
    entries
}

pub fn command_snapshot_file_names() -> Vec<String> {
    let mut entries = fs::read_dir(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("snapshots"),
    )
    .expect("snapshot directory should be readable")
    .map(|entry| entry.expect("snapshot entry should be readable"))
    .filter_map(|entry| {
        let file_name = entry.file_name().to_string_lossy().into_owned();
        (entry
            .file_type()
            .expect("snapshot entry type should be readable")
            .is_file()
            && ((file_name.starts_with("change_cli__expr_runtime_")
                || file_name.starts_with("property_cli__expr_runtime_"))
                && file_name.ends_with(".snap")))
        .then_some(file_name)
    })
    .collect::<Vec<_>>();
    entries.sort();
    entries
}

pub fn load_positive_manifest(file_name: &str) -> PositiveManifest {
    let payload = fs::read_to_string(fixture_cli_path(file_name))
        .unwrap_or_else(|error| panic!("manifest '{file_name}' should be readable: {error}"));
    let manifest = serde_json::from_str::<PositiveManifest>(&payload)
        .unwrap_or_else(|error| panic!("manifest '{file_name}' should be valid JSON: {error}"));
    validate_positive_manifest(manifest).unwrap_or_else(|error| {
        panic!("manifest '{file_name}' should match shared contract: {error}")
    })
}

pub fn load_negative_manifest(file_name: &str) -> NegativeManifest {
    let payload = fs::read_to_string(fixture_cli_path(file_name))
        .unwrap_or_else(|error| panic!("manifest '{file_name}' should be readable: {error}"));
    let manifest = serde_json::from_str::<NegativeManifest>(&payload)
        .unwrap_or_else(|error| panic!("manifest '{file_name}' should be valid JSON: {error}"));
    validate_negative_manifest(manifest).unwrap_or_else(|error| {
        panic!("manifest '{file_name}' should match shared contract: {error}")
    })
}

pub fn snapshot_file_name(command: CommandCaseCommand, snapshot_name: &str) -> String {
    format!(
        "{}__{}.snap",
        command.snapshot_suite_prefix(),
        snapshot_name
    )
}

pub fn referenced_snapshot_file_names(
    positive: &PositiveManifest,
    negative: &NegativeManifest,
) -> Vec<String> {
    let mut names = Vec::new();
    for case in &positive.cases {
        if let Some(snapshot) = case.stdout_snapshot.as_deref() {
            names.push(snapshot_file_name(case.command, snapshot));
        }
    }
    for case in &negative.cases {
        if let Some(snapshot) = case.stderr_snapshot.as_deref() {
            names.push(snapshot_file_name(case.command, snapshot));
        }
    }
    names.sort();
    names.dedup();
    names
}

pub fn assert_positive_case(case: &PositiveCase) {
    let output = run_case(case.command, case.args.as_slice());
    assert_eq!(
        output.status.code(),
        Some(case.exit_code),
        "case '{}' exit code",
        case.name
    );
    match case.output_mode {
        OutputMode::Json => {
            assert!(output.stderr.is_empty(), "case '{}' stderr", case.name);
            let value: Value = serde_json::from_slice(&output.stdout)
                .unwrap_or_else(|error| panic!("case '{}' stdout json: {error}", case.name));
            assert_eq!(
                value["command"],
                case.command.as_str(),
                "case '{}' command",
                case.name
            );
            assert_eq!(
                value["data"],
                case.expected_data
                    .clone()
                    .expect("json cases must declare expected_data"),
                "case '{}' data",
                case.name
            );
            assert_eq!(
                value["warnings"],
                serde_json::json!(case.expected_warnings),
                "case '{}' warnings",
                case.name
            );
        }
        OutputMode::Human => {
            let stdout = String::from_utf8(output.stdout).expect("stdout should be utf8");
            let stderr = String::from_utf8(output.stderr).expect("stderr should be utf8");
            if let Some(snapshot_name) = case.stdout_snapshot.as_deref() {
                let identifier = case_snapshot_identifier(case.command, snapshot_name);
                insta::with_settings!({
                    prepend_module_to_snapshot => false,
                    snapshot_path => "../snapshots",
                }, {
                    insta::assert_snapshot!(identifier.as_str(), stdout);
                });
            }
            let warning_lines = stderr
                .lines()
                .filter_map(|line| line.strip_prefix("warning: ").map(ToString::to_string))
                .collect::<Vec<_>>();
            assert_eq!(
                warning_lines, case.expected_warnings,
                "case '{}' warnings",
                case.name
            );
        }
    }
}

pub fn assert_negative_case(case: &NegativeCase) {
    let output = run_case(case.command, case.args.as_slice());
    assert_eq!(
        output.status.code(),
        Some(case.exit_code),
        "case '{}' exit code",
        case.name
    );
    assert!(output.stdout.is_empty(), "case '{}' stdout", case.name);
    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf8");
    assert!(
        stderr.starts_with(case.stderr_prefix.as_str()),
        "case '{}' stderr prefix mismatch: {stderr}",
        case.name
    );
    if let Some(snapshot_name) = case.stderr_snapshot.as_deref() {
        let identifier = case_snapshot_identifier(case.command, snapshot_name);
        insta::with_settings!({
            prepend_module_to_snapshot => false,
            snapshot_path => "../snapshots",
        }, {
            insta::assert_snapshot!(identifier.as_str(), stderr);
        });
    }
}

fn run_case(command: CommandCaseCommand, args: &[String]) -> std::process::Output {
    let mut full_args = vec![command.as_str().to_string()];
    full_args.extend(resolve_manifest_args(args));
    wavepeek_cmd()
        .args(full_args)
        .output()
        .expect("command fixture case should execute")
}

fn resolve_manifest_args(args: &[String]) -> Vec<String> {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let mut resolved = Vec::with_capacity(args.len());
    let mut previous_was_waves = false;
    for arg in args {
        let resolved_arg = if previous_was_waves && Path::new(arg).is_relative() {
            repo_root.join(arg).display().to_string()
        } else {
            arg.clone()
        };
        previous_was_waves = arg == "--waves";
        resolved.push(resolved_arg);
    }
    resolved
}

fn case_snapshot_identifier(command: CommandCaseCommand, snapshot_name: &str) -> String {
    format!("{}__{}", command.snapshot_suite_prefix(), snapshot_name)
}

fn validate_positive_manifest(manifest: PositiveManifest) -> Result<PositiveManifest, String> {
    for case in &manifest.cases {
        match case.output_mode {
            OutputMode::Json => {
                if case.expected_data.is_none() {
                    return Err(format!(
                        "positive case '{}' with json output must declare expected_data",
                        case.name
                    ));
                }
                if case.stdout_snapshot.is_some() {
                    return Err(format!(
                        "positive case '{}' with json output must not declare stdout_snapshot",
                        case.name
                    ));
                }
            }
            OutputMode::Human => {
                if case.stdout_snapshot.is_none() {
                    return Err(format!(
                        "positive case '{}' with human output must declare stdout_snapshot",
                        case.name
                    ));
                }
            }
        }
        if case.args.is_empty() {
            return Err(format!("positive case '{}' must declare args", case.name));
        }
    }
    Ok(manifest)
}

fn validate_negative_manifest(manifest: NegativeManifest) -> Result<NegativeManifest, String> {
    for case in &manifest.cases {
        if case.args.is_empty() {
            return Err(format!("negative case '{}' must declare args", case.name));
        }
        if !case.stderr_prefix.starts_with("error:") {
            return Err(format!(
                "negative case '{}' stderr_prefix must start with 'error:'",
                case.name
            ));
        }
    }
    Ok(manifest)
}
