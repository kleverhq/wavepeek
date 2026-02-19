use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::process::Command;

fn wavepeek_cmd() -> Command {
    Command::new(env!("CARGO_BIN_EXE_wavepeek"))
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
        .stdout(predicate::str::contains("hierarchy scopes by recursively"))
        .stdout(predicate::str::contains("traversing the hierarchy"))
        .stdout(predicate::str::contains("bounded by --max and --max-depth"));

    let mut when_command = wavepeek_cmd();

    when_command
        .args(["when", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "boolean expression evaluates to true",
        ))
        .stdout(predicate::str::contains("first N, or last N matches"));
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
fn change_unimplemented_error_uses_singular_command_name() {
    let mut command = wavepeek_cmd();

    command
        .args(["change", "--waves", "dump.vcd", "--signals", "clk"])
        .assert()
        .failure()
        .code(1)
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::contains(
            "error: unimplemented: `change` command execution is not implemented yet",
        ));
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
