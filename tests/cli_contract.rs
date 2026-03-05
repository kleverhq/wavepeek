use assert_cmd::prelude::*;
use predicates::prelude::*;

mod common;
use common::wavepeek_cmd;

const SHIPPED_COMMANDS: [&str; 7] = [
    "info", "scope", "signal", "value", "change", "property", "schema",
];

fn successful_stdout(args: &[&str]) -> Vec<u8> {
    let mut command = wavepeek_cmd();
    let assert = command.args(args).assert().success();
    let output = assert.get_output();
    assert!(
        output.stderr.is_empty(),
        "expected empty stderr for args {:?}, got: {}",
        args,
        String::from_utf8_lossy(&output.stderr)
    );
    output.stdout.clone()
}

fn successful_stdout_text(args: &[&str]) -> String {
    String::from_utf8(successful_stdout(args)).expect("stdout should be UTF-8")
}

fn command_names_from_top_level_help(help: &str) -> Vec<String> {
    let mut names = Vec::new();
    let mut in_commands = false;

    for line in help.lines() {
        if line.trim() == "Commands:" {
            in_commands = true;
            continue;
        }

        if !in_commands {
            continue;
        }

        if line.trim() == "Options:" {
            break;
        }

        if line.trim().is_empty() {
            continue;
        }

        let trimmed = line.trim_start();
        if trimmed.is_empty() {
            continue;
        }

        let leading_spaces = line.len() - trimmed.len();
        if leading_spaces != 2 {
            continue;
        }

        if let Some(name) = trimmed.split_whitespace().next() {
            names.push(name.to_string());
        }
    }

    names
}

fn top_level_help_command_names() -> Vec<String> {
    let help = successful_stdout_text(&["--help"]);
    command_names_from_top_level_help(&help)
}

#[test]
fn no_args_prints_top_level_help_and_exits_zero() {
    let mut command = wavepeek_cmd();

    command
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "wavepeek is a command-line tool for RTL waveform inspection.",
        ))
        .stdout(predicate::str::contains("Usage: wavepeek"))
        .stderr(predicate::str::is_empty());
}

#[test]
fn help_lists_expected_subcommands() {
    let mut command = wavepeek_cmd();

    command
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "wavepeek is a command-line tool for RTL waveform inspection.",
        ))
        .stdout(predicate::str::contains("schema"))
        .stdout(predicate::str::contains("info"))
        .stdout(predicate::str::contains("scope"))
        .stdout(predicate::str::contains("\n  modules\n").not())
        .stdout(predicate::str::contains("\n  tree\n").not())
        .stdout(predicate::str::contains("signal"))
        .stdout(predicate::str::contains("\n  signals\n").not())
        .stdout(predicate::str::contains("value"))
        .stdout(predicate::str::contains("change"))
        .stdout(predicate::str::contains("\n  changes\n").not())
        .stdout(predicate::str::contains("property"))
        .stdout(predicate::str::contains("\n  help\n").not());
}

#[test]
fn top_level_help_documents_general_conventions() {
    let mut command = wavepeek_cmd();

    command
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("General conventions:"))
        .stdout(predicate::str::contains("No positional command arguments"))
        .stdout(predicate::str::contains("Output is bounded by default"))
        .stdout(predicate::str::contains("Default output is human-readable"))
        .stdout(predicate::str::contains(
            "Time values require explicit units",
        ))
        .stdout(predicate::str::contains(
            "time-window flags (`--from`, `--to`) use inclusive boundaries",
        ))
        .stdout(predicate::str::contains(
            "Errors follow `error: <category>: <message>`",
        ));
}

#[test]
fn top_level_help_marks_unimplemented_subcommands() {
    let mut command = wavepeek_cmd();

    command
        .arg("--help")
        .assert()
        .success()
        .stdout(
            predicate::str::contains("Get signal values at a specific time point").and(
                predicate::str::contains(
                    "Get signal values at a specific time point (not implemented yet)",
                )
                .not(),
            ),
        )
        .stdout(predicate::str::contains(
            "Get value snapshots over a time range",
        ))
        .stdout(
            predicate::str::contains("Get value snapshots over a time range (not implemented yet)")
                .not(),
        )
        .stdout(predicate::str::contains(
            "Check property over event triggers (not implemented yet)",
        ));
}

#[test]
fn help_lists_schema_after_waveform_commands() {
    let mut command = wavepeek_cmd();

    let assert = command.arg("--help").assert().success();
    let output = String::from_utf8_lossy(&assert.get_output().stdout);

    let schema_index = output
        .find("\n  schema")
        .expect("help output should list schema subcommand");
    let property_index = output
        .find("\n  property")
        .expect("help output should list property subcommand");

    assert!(
        schema_index > property_index,
        "schema should appear after waveform commands in top-level help"
    );
}

#[test]
fn short_help_flag_matches_long_help_behavior() {
    let mut command = wavepeek_cmd();

    command
        .arg("-h")
        .assert()
        .success()
        .stdout(predicate::str::contains("Usage: wavepeek"))
        .stdout(predicate::str::contains("Options:"))
        .stderr(predicate::str::is_empty());
}

#[test]
fn no_args_help_matches_long_help_output() {
    let no_args = successful_stdout(&[]);
    let long_help = successful_stdout(&["--help"]);

    assert_eq!(
        no_args, long_help,
        "wavepeek (no args) output must match wavepeek --help byte-for-byte"
    );
}

#[test]
fn top_level_short_and_long_help_are_identical() {
    let short_help = successful_stdout(&["-h"]);
    let long_help = successful_stdout(&["--help"]);

    assert_eq!(
        short_help, long_help,
        "wavepeek -h output must match wavepeek --help byte-for-byte"
    );
}

#[test]
fn short_and_long_help_are_identical_for_shipped_commands() {
    for command_name in SHIPPED_COMMANDS {
        let short_help = successful_stdout(&[command_name, "-h"]);
        let long_help = successful_stdout(&[command_name, "--help"]);
        assert_eq!(
            short_help, long_help,
            "wavepeek {command_name} -h output must match wavepeek {command_name} --help"
        );
    }
}

#[test]
fn shipped_commands_list_matches_top_level_help_surface() {
    let expected: Vec<String> = SHIPPED_COMMANDS
        .iter()
        .map(|command_name| command_name.to_string())
        .collect();
    let actual = top_level_help_command_names();

    assert_eq!(
        actual, expected,
        "top-level help command list changed; update SHIPPED_COMMANDS and help contract tests"
    );
}

#[test]
fn command_name_parser_ignores_wrapped_description_lines() {
    let help = "Usage: wavepeek <COMMAND>\n\nCommands:\n  info     Show dump metadata (time unit and bounds)\n           wrapped continuation text\n  scope    List hierarchy scopes (deterministic DFS)\n\nOptions:\n  -h, --help  Print help\n";

    assert_eq!(
        command_names_from_top_level_help(help),
        vec!["info", "scope"]
    );
}

#[test]
fn waveform_help_uses_schema_reference_without_inline_envelope_or_parse_hints() {
    for command_name in ["info", "scope", "signal", "value", "change", "property"] {
        let long_help = successful_stdout_text(&[command_name, "--help"]);

        assert!(
            long_help.contains("wavepeek schema"),
            "help for {command_name} should refer readers to wavepeek schema for JSON contract details"
        );
        assert!(
            !long_help.contains("$schema"),
            "help for {command_name} should not inline JSON envelope fields"
        );
        assert!(
            !long_help.contains("`data`"),
            "help for {command_name} should not inline JSON envelope field names"
        );
        assert!(
            !long_help.contains("`warnings`"),
            "help for {command_name} should not inline JSON envelope field names"
        );
        assert!(
            !long_help.contains("See 'wavepeek "),
            "help for {command_name} should not include repetitive parse-hint boilerplate"
        );
    }
}

#[test]
fn waveform_help_avoids_literal_error_or_warning_message_bodies() {
    for command_name in ["info", "scope", "signal", "value", "change", "property"] {
        let long_help = successful_stdout_text(&[command_name, "--help"]);

        assert!(
            !long_help.contains("error: "),
            "help for {command_name} should avoid inlining literal runtime/parser error prefixes"
        );
        assert!(
            !long_help.contains("warning: "),
            "help for {command_name} should avoid inlining literal warning-message prefixes"
        );
        assert!(
            !long_help.contains("no signal changes found in selected time range"),
            "help for {command_name} should avoid inlining concrete warning message bodies"
        );
        assert!(
            !long_help.contains("limit disabled:"),
            "help for {command_name} should avoid inlining concrete warning message bodies"
        );
    }
}

#[test]
fn subcommand_short_help_includes_long_help_contract_markers() {
    let command_markers: [(&str, &[&str]); 7] = [
        (
            "info",
            &["Reports dump metadata", "time_unit", "wavepeek schema"],
        ),
        (
            "scope",
            &[
                "deterministic hierarchy traversal",
                "pre-order depth-first",
                "Truncation and disabled-limit conditions emit warnings",
                "wavepeek schema",
            ],
        ),
        (
            "signal",
            &[
                "scope-local signal listing",
                "depth-first in stable lexicographic order",
                "applies only in recursive mode",
                "wavepeek schema",
            ],
        ),
        (
            "value",
            &[
                "point-in-time sampling",
                "input order from `--signals`",
                "Verilog literals",
                "wavepeek schema",
            ],
        ),
        (
            "change",
            &[
                "range-based delta snapshots",
                "Range boundaries are inclusive",
                "Empty-result and truncation conditions may emit warnings",
                "wavepeek schema",
            ],
        ),
        (
            "property",
            &[
                "Intended semantics",
                "Execution is not implemented yet",
                "--capture",
                "wavepeek schema",
            ],
        ),
        (
            "schema",
            &[
                "canonical JSON schema document",
                "schema/wavepeek.json",
                "source of truth for all `--json` command outputs",
            ],
        ),
    ];

    for (command_name, markers) in command_markers {
        let short_help = successful_stdout_text(&[command_name, "-h"]);
        for marker in markers {
            assert!(
                short_help.contains(marker),
                "short help for {command_name} must include marker `{marker}`"
            );
        }
    }
}

#[test]
fn shipped_commands_help_is_self_descriptive() {
    let command_contracts: [(&str, &[&str]); 7] = [
        (
            "info",
            &[
                "Reports dump metadata",
                "Human-readable output is the default terminal mode",
                "time_unit",
                "time_start",
                "wavepeek schema",
            ],
        ),
        (
            "scope",
            &[
                "deterministic hierarchy traversal",
                "pre-order depth-first",
                "lexicographic child ordering",
                "Truncation and disabled-limit conditions emit warnings",
                "wavepeek schema",
            ],
        ),
        (
            "signal",
            &[
                "scope-local signal listing",
                "Default mode lists only direct signals",
                "depth-first in stable lexicographic order",
                "applies only in recursive mode",
                "wavepeek schema",
            ],
        ),
        (
            "value",
            &[
                "point-in-time sampling",
                "input order from `--signals`",
                "align to dump precision",
                "Verilog literals",
                "wavepeek schema",
            ],
        ),
        (
            "change",
            &[
                "range-based delta snapshots",
                "Range boundaries are inclusive",
                "omitted `--on` behaves as wildcard",
                "Empty-result and truncation conditions may emit warnings",
                "logical-condition execution for `iff` is deferred",
                "wavepeek schema",
            ],
        ),
        (
            "property",
            &[
                "Check property over event triggers",
                "Intended semantics",
                "--capture",
                "Execution is not implemented yet",
                "wavepeek schema",
            ],
        ),
        (
            "schema",
            &[
                "canonical JSON schema document",
                "Accepts no command-specific flags or positional arguments",
                "Prints exactly one deterministic schema document",
                "schema/wavepeek.json",
                "source of truth for all `--json` command outputs",
            ],
        ),
    ];

    for (command_name, fragments) in command_contracts {
        let long_help = successful_stdout_text(&[command_name, "--help"]);
        for fragment in fragments {
            assert!(
                long_help.contains(fragment),
                "help for {command_name} must include self-descriptive fragment `{fragment}`"
            );
        }
    }
}

#[test]
fn version_flags_print_version_to_stdout() {
    let mut short_command = wavepeek_cmd();

    short_command
        .arg("-V")
        .assert()
        .success()
        .stdout(predicate::str::starts_with("wavepeek "))
        .stderr(predicate::str::is_empty());

    let mut long_command = wavepeek_cmd();

    long_command
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::starts_with("wavepeek "))
        .stderr(predicate::str::is_empty());
}

#[test]
fn subcommand_help_uses_extended_prd_descriptions() {
    let mut scope_command = wavepeek_cmd();

    scope_command
        .args(["scope", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "deterministic hierarchy traversal",
        ))
        .stdout(predicate::str::contains("pre-order depth-first"))
        .stdout(predicate::str::contains(
            "Truncation and disabled-limit conditions emit warnings",
        ));

    let mut property_command = wavepeek_cmd();

    property_command
        .args(["property", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Intended semantics"))
        .stdout(predicate::str::contains("--capture"));
}

#[test]
fn signal_help_documents_recursive_and_max_depth_flags() {
    let mut command = wavepeek_cmd();

    command
        .args(["signal", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--recursive"))
        .stdout(predicate::str::contains("--max-depth"))
        .stdout(predicate::str::contains("requires --recursive"));
}

#[test]
fn help_documents_unlimited_limit_literals_for_all_affected_commands() {
    wavepeek_cmd()
        .args(["scope", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--max"))
        .stdout(predicate::str::contains("--max-depth"))
        .stdout(predicate::str::contains("unlimited"));

    wavepeek_cmd()
        .args(["signal", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--max"))
        .stdout(predicate::str::contains("--max-depth"))
        .stdout(predicate::str::contains("unlimited"));

    wavepeek_cmd()
        .args(["change", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--max"))
        .stdout(predicate::str::contains("unlimited"));

    wavepeek_cmd()
        .args(["property", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--capture <MODE>"))
        .stdout(predicate::str::contains("switch"));
}

#[test]
fn property_accepts_capture_flag_in_cli_then_fails_as_unimplemented() {
    let fixture = common::fixture_path("m2_core.vcd");
    let fixture = fixture.to_string_lossy().into_owned();

    wavepeek_cmd()
        .args([
            "property",
            "--waves",
            fixture.as_str(),
            "--on",
            "posedge top.clk",
            "--eval",
            "1",
            "--capture",
            "switch",
        ])
        .assert()
        .failure()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::starts_with("error: unimplemented:"))
        .stderr(predicate::str::contains("error: args:").not())
        .stderr(predicate::str::contains(
            "`property` command execution is not implemented yet",
        ));
}

#[test]
fn unimplemented_subcommands_disclose_status_in_help() {
    let mut value_command = wavepeek_cmd();
    value_command
        .args(["value", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Execution is not implemented yet.").not());

    let mut change_command = wavepeek_cmd();
    change_command
        .args(["change", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Execution is not implemented yet.").not());

    let mut property_command = wavepeek_cmd();
    property_command
        .args(["property", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Execution is not implemented yet.",
        ));
}

#[test]
fn change_help_documents_on_trigger_and_does_not_expose_clk() {
    let mut command = wavepeek_cmd();

    command
        .args(["change", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--on"))
        .stdout(predicate::str::contains("--clk").not())
        .stdout(predicate::str::contains("--tune-engine").not())
        .stdout(predicate::str::contains("--tune-candidates").not())
        .stdout(predicate::str::contains("--tune-edge-fast-force").not())
        .stdout(predicate::str::contains("--perf-engine").not())
        .stdout(predicate::str::contains("--perf-candidates").not())
        .stdout(predicate::str::contains("--perf-edge-fast-force").not());
}

#[test]
fn change_rejects_legacy_when_flag_without_alias() {
    let fixture = common::fixture_path("m2_core.vcd");
    let fixture = fixture.to_string_lossy().into_owned();

    wavepeek_cmd()
        .args([
            "change",
            "--waves",
            fixture.as_str(),
            "--signals",
            "top.clk",
            "--when",
            "*",
        ])
        .assert()
        .failure()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::starts_with("error: args:"))
        .stderr(predicate::str::contains("unexpected argument '--when'"))
        .stderr(predicate::str::contains("See 'wavepeek change --help'."));
}

#[test]
fn waveform_commands_require_waves_flag() {
    let mut command = wavepeek_cmd();

    command
        .arg("info")
        .assert()
        .failure()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::starts_with("error: args:"))
        .stderr(predicate::str::contains(
            "required arguments were not provided",
        ))
        .stderr(predicate::str::contains("--waves <FILE>"))
        .stderr(predicate::str::contains("See 'wavepeek info --help'."));
}

#[test]
fn schema_does_not_accept_waves_flag() {
    let mut command = wavepeek_cmd();

    command
        .args(["schema", "--waves", "dump.vcd"])
        .assert()
        .failure()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::starts_with("error: args:"))
        .stderr(predicate::str::contains("unexpected argument '--waves'"))
        .stderr(predicate::str::contains("See 'wavepeek schema --help'."));
}

#[test]
fn schema_does_not_accept_json_flag() {
    let mut command = wavepeek_cmd();

    command
        .args(["schema", "--json"])
        .assert()
        .failure()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::starts_with("error: args:"))
        .stderr(predicate::str::contains("unexpected argument '--json'"))
        .stderr(predicate::str::contains("See 'wavepeek schema --help'."));
}

#[test]
fn schema_rejects_positional_arguments() {
    let mut command = wavepeek_cmd();

    command
        .args(["schema", "extra"])
        .assert()
        .failure()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::starts_with("error: args:"))
        .stderr(predicate::str::contains("unexpected argument 'extra'"))
        .stderr(predicate::str::contains("See 'wavepeek schema --help'."));
}

#[test]
fn legacy_tree_command_is_rejected_without_alias() {
    let mut command = wavepeek_cmd();

    command
        .arg("tree")
        .assert()
        .failure()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::starts_with("error: args:"))
        .stderr(predicate::str::contains("unrecognized subcommand 'tree'"))
        .stderr(predicate::str::contains("See 'wavepeek --help'."));
}

#[test]
fn legacy_modules_command_is_rejected_without_alias() {
    let mut command = wavepeek_cmd();

    command
        .arg("modules")
        .assert()
        .failure()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::starts_with("error: args:"))
        .stderr(predicate::str::contains(
            "unrecognized subcommand 'modules'",
        ))
        .stderr(predicate::str::contains("See 'wavepeek --help'."));
}

#[test]
fn legacy_signals_command_is_rejected_without_alias() {
    let mut command = wavepeek_cmd();

    command
        .arg("signals")
        .assert()
        .failure()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::starts_with("error: args:"))
        .stderr(predicate::str::contains(
            "unrecognized subcommand 'signals'",
        ))
        .stderr(predicate::str::contains("See 'wavepeek --help'."));
}

#[test]
fn legacy_changes_command_is_rejected_without_alias() {
    let mut command = wavepeek_cmd();

    command
        .arg("changes")
        .assert()
        .failure()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::starts_with("error: args:"))
        .stderr(predicate::str::contains(
            "unrecognized subcommand 'changes'",
        ))
        .stderr(predicate::str::contains("See 'wavepeek --help'."));
}

#[test]
fn legacy_when_command_is_rejected_without_alias() {
    let mut command = wavepeek_cmd();

    command
        .arg("when")
        .assert()
        .failure()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::starts_with("error: args:"))
        .stderr(predicate::str::contains("unrecognized subcommand 'when'"))
        .stderr(predicate::str::contains("See 'wavepeek --help'."));
}

#[test]
fn legacy_at_command_is_rejected_without_alias() {
    let mut command = wavepeek_cmd();

    command
        .arg("at")
        .assert()
        .failure()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::starts_with("error: args:"))
        .stderr(predicate::str::contains("unrecognized subcommand 'at'"))
        .stderr(predicate::str::contains("See 'wavepeek --help'."));
}

#[test]
fn value_rejects_legacy_time_flag_without_alias() {
    let mut command = wavepeek_cmd();

    command
        .args([
            "value",
            "--waves",
            "dump.vcd",
            "--time",
            "1ns",
            "--signals",
            "sig",
        ])
        .assert()
        .failure()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::starts_with("error: args:"))
        .stderr(predicate::str::contains("unexpected argument '--time'"))
        .stderr(predicate::str::contains("See 'wavepeek value --help'."));
}

#[test]
fn positional_arguments_are_rejected() {
    let mut command = wavepeek_cmd();

    command
        .args(["info", "--waves", "dump.vcd", "extra"])
        .assert()
        .failure()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::starts_with("error: args:"))
        .stderr(predicate::str::contains("unexpected argument 'extra'"))
        .stderr(predicate::str::contains("See 'wavepeek info --help'."));
}

#[test]
fn unknown_flags_are_normalized_to_args_category() {
    let mut command = wavepeek_cmd();

    command
        .args(["info", "--waves", "dump.vcd", "--wat"])
        .assert()
        .failure()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::starts_with("error: args:"))
        .stderr(predicate::str::contains("unexpected argument '--wat'"))
        .stderr(predicate::str::contains("See 'wavepeek info --help'."));
}

#[test]
fn all_commands_reject_human_flag() {
    let mut schema = wavepeek_cmd();
    schema
        .args(["schema", "--human"])
        .assert()
        .failure()
        .stderr(predicate::str::starts_with("error: args:"))
        .stderr(predicate::str::contains("unexpected argument '--human'"))
        .stderr(predicate::str::contains("See 'wavepeek schema --help'."));

    let mut info = wavepeek_cmd();
    info.args(["info", "--waves", "dump.vcd", "--human"])
        .assert()
        .failure()
        .stderr(predicate::str::starts_with("error: args:"))
        .stderr(predicate::str::contains("unexpected argument '--human'"))
        .stderr(predicate::str::contains("See 'wavepeek info --help'."));

    let mut scope = wavepeek_cmd();
    scope
        .args(["scope", "--waves", "dump.vcd", "--human"])
        .assert()
        .failure()
        .stderr(predicate::str::starts_with("error: args:"))
        .stderr(predicate::str::contains("unexpected argument '--human'"))
        .stderr(predicate::str::contains("See 'wavepeek scope --help'."));

    let mut signal = wavepeek_cmd();
    signal
        .args(["signal", "--waves", "dump.vcd", "--scope", "top", "--human"])
        .assert()
        .failure()
        .stderr(predicate::str::starts_with("error: args:"))
        .stderr(predicate::str::contains("unexpected argument '--human'"))
        .stderr(predicate::str::contains("See 'wavepeek signal --help'."));

    let mut value = wavepeek_cmd();
    value
        .args([
            "value",
            "--waves",
            "dump.vcd",
            "--at",
            "1ns",
            "--signals",
            "sig",
            "--human",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::starts_with("error: args:"))
        .stderr(predicate::str::contains("unexpected argument '--human'"))
        .stderr(predicate::str::contains("See 'wavepeek value --help'."));

    let mut change = wavepeek_cmd();
    change
        .args([
            "change",
            "--waves",
            "dump.vcd",
            "--signals",
            "sig",
            "--human",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::starts_with("error: args:"))
        .stderr(predicate::str::contains("unexpected argument '--human'"))
        .stderr(predicate::str::contains("See 'wavepeek change --help'."));

    let mut property = wavepeek_cmd();
    property
        .args([
            "property", "--waves", "dump.vcd", "--on", "*", "--eval", "1", "--human",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::starts_with("error: args:"))
        .stderr(predicate::str::contains("unexpected argument '--human'"))
        .stderr(predicate::str::contains("See 'wavepeek property --help'."));
}

#[test]
fn unknown_top_level_flag_uses_global_help_hint() {
    let mut command = wavepeek_cmd();

    command
        .args(["--wat"])
        .assert()
        .failure()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::starts_with("error: args:"))
        .stderr(predicate::str::contains("unexpected argument '--wat'"))
        .stderr(predicate::str::contains("See 'wavepeek --help'."));
}
