use assert_cmd::prelude::*;
use predicates::prelude::*;

mod common;
use common::wavepeek_cmd;

const SHIPPED_COMMANDS: [&str; 7] = ["info", "scope", "signal", "at", "change", "when", "schema"];

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
        .stdout(predicate::str::contains("at"))
        .stdout(predicate::str::contains("change"))
        .stdout(predicate::str::contains("\n  changes\n").not())
        .stdout(predicate::str::contains("when"))
        .stdout(predicate::str::contains("\n  help\n").not());
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
            "Find cycles where a condition is true (not implemented yet)",
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
    let when_index = output
        .find("\n  when")
        .expect("help output should list when subcommand");

    assert!(
        schema_index > when_index,
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
fn subcommand_short_help_includes_long_help_contract_markers() {
    let command_markers = [
        (
            "info",
            [
                "Reports dump metadata",
                "time_unit",
                "error: args:",
                "See 'wavepeek info --help'.",
            ],
        ),
        (
            "scope",
            [
                "deterministic hierarchy traversal",
                "--max-depth",
                "error: args:",
                "See 'wavepeek scope --help'.",
            ],
        ),
        (
            "signal",
            [
                "scope-local signal listing",
                "--max-depth requires --recursive",
                "error: args:",
                "See 'wavepeek signal --help'.",
            ],
        ),
        (
            "at",
            [
                "point-in-time sampling",
                "--time",
                "error: args:",
                "See 'wavepeek at --help'.",
            ],
        ),
        (
            "change",
            [
                "range-based delta snapshots",
                "--when defaults to *",
                "error: args:",
                "See 'wavepeek change --help'.",
            ],
        ),
        (
            "when",
            [
                "Command intent",
                "Execution is not implemented yet",
                "error: args:",
                "See 'wavepeek when --help'.",
            ],
        ),
        (
            "schema",
            [
                "canonical JSON schema document",
                "accepts no command-specific flags",
                "error: <category>: <message>",
                "See 'wavepeek schema --help'.",
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
    let command_contracts = [
        (
            "info",
            [
                "Reports dump metadata",
                "--waves <FILE> is required",
                "Default output is human-readable",
                "error: args:",
                "time_unit",
            ],
        ),
        (
            "scope",
            [
                "deterministic hierarchy traversal",
                "Defaults: --max=50, --max-depth=5, --filter=.*",
                "--max must be greater than 0",
                "error: args:",
                "JSON data array",
            ],
        ),
        (
            "signal",
            [
                "scope-local signal listing",
                "--scope is required",
                "--max-depth requires --recursive",
                "error: args:",
                "JSON array of canonical signal objects",
            ],
        ),
        (
            "at",
            [
                "point-in-time sampling",
                "--time requires explicit units",
                "Output preserves --signals order",
                "error: args:",
                "JSON object with `time` and ordered `signals`",
            ],
        ),
        (
            "change",
            [
                "range-based delta snapshots",
                "Default trigger is --when=*",
                "inclusive",
                "error: args:",
                "JSON rows",
            ],
        ),
        (
            "when",
            [
                "Find cycles where a condition is true",
                "Qualifiers: --first, --last, --max",
                "--max unlimited",
                "Execution is not implemented yet",
                "error: args:",
            ],
        ),
        (
            "schema",
            [
                "canonical JSON schema document",
                "accepts no command-specific flags or positional arguments",
                "prints exactly one schema document",
                "error: <category>: <message>",
                "does not use waveform --json envelope wrapping",
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
        .stdout(predicate::str::contains(
            "Defaults: --max=50, --max-depth=5, --filter=.*",
        ))
        .stdout(predicate::str::contains("JSON data array"));

    let mut when_command = wavepeek_cmd();

    when_command
        .args(["when", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Command intent"))
        .stdout(predicate::str::contains(
            "Qualifiers: --first, --last, --max",
        ));
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
        .stdout(predicate::str::contains("--max-depth requires --recursive"));
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
        .args(["when", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--max <LIMIT>"))
        .stdout(predicate::str::contains("--max"))
        .stdout(predicate::str::contains("unlimited"));
}

#[test]
fn when_accepts_unlimited_max_in_cli_then_fails_as_unimplemented() {
    let fixture = common::fixture_path("m2_core.vcd");
    let fixture = fixture.to_string_lossy().into_owned();

    wavepeek_cmd()
        .args([
            "when",
            "--waves",
            fixture.as_str(),
            "--clk",
            "top.clk",
            "--cond",
            "1",
            "--max",
            "unlimited",
        ])
        .assert()
        .failure()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::starts_with("error: unimplemented:"))
        .stderr(predicate::str::contains("error: args:").not())
        .stderr(predicate::str::contains(
            "`when` command execution is not implemented yet",
        ));
}

#[test]
fn unimplemented_subcommands_disclose_status_in_help() {
    let mut at_command = wavepeek_cmd();
    at_command
        .args(["at", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Execution is not implemented yet.").not());

    let mut change_command = wavepeek_cmd();
    change_command
        .args(["change", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Execution is not implemented yet.").not());

    let mut when_command = wavepeek_cmd();
    when_command
        .args(["when", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Execution is not implemented yet.",
        ));
}

#[test]
fn change_help_documents_when_trigger_and_does_not_expose_clk() {
    let mut command = wavepeek_cmd();

    command
        .args(["change", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--when"))
        .stdout(predicate::str::contains("--clk").not());
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

    let mut at = wavepeek_cmd();
    at.args([
        "at",
        "--waves",
        "dump.vcd",
        "--time",
        "1ns",
        "--signals",
        "sig",
        "--human",
    ])
    .assert()
    .failure()
    .stderr(predicate::str::starts_with("error: args:"))
    .stderr(predicate::str::contains("unexpected argument '--human'"))
    .stderr(predicate::str::contains("See 'wavepeek at --help'."));

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

    let mut when = wavepeek_cmd();
    when.args([
        "when", "--waves", "dump.vcd", "--clk", "clk", "--cond", "1", "--human",
    ])
    .assert()
    .failure()
    .stderr(predicate::str::starts_with("error: args:"))
    .stderr(predicate::str::contains("unexpected argument '--human'"))
    .stderr(predicate::str::contains("See 'wavepeek when --help'."));
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
