use assert_cmd::prelude::*;
use predicates::prelude::*;

mod common;
use common::wavepeek_cmd;

const VISIBLE_TOP_LEVEL_COMMANDS: [&str; 9] = [
    "info", "scope", "signal", "value", "change", "property", "schema", "docs", "help",
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

fn assert_same_stdout(left_args: &[&str], right_args: &[&str], label: &str) {
    let left = successful_stdout(left_args);
    let right = successful_stdout(right_args);

    assert_eq!(left, right, "{label}");
}

fn command_names_from_top_level_help(help: &str) -> Vec<String> {
    let mut names = Vec::new();
    let mut in_commands = false;

    for line in help.lines() {
        let trimmed_line = line.trim();
        if matches!(
            trimmed_line,
            "Commands:" | "Waveform commands:" | "Helper commands:"
        ) {
            in_commands = true;
            continue;
        }

        if !in_commands {
            continue;
        }

        if trimmed_line == "Options:" {
            break;
        }

        if trimmed_line.is_empty() {
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

fn assert_legacy_subcommand_rejected(legacy_name: &str) {
    let mut command = wavepeek_cmd();

    command
        .arg(legacy_name)
        .assert()
        .failure()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::starts_with("error: args:"))
        .stderr(predicate::str::contains(format!(
            "unrecognized subcommand '{legacy_name}'"
        )))
        .stderr(predicate::str::contains("See 'wavepeek --help'."));
}

fn assert_human_flag_rejected(args: &[&str], command_name: &str) {
    let mut command = wavepeek_cmd();

    command
        .args(args)
        .assert()
        .failure()
        .stderr(predicate::str::starts_with("error: args:"))
        .stderr(predicate::str::contains("unexpected argument '--human'"))
        .stderr(predicate::str::contains(format!(
            "See 'wavepeek {command_name} --help'."
        )));
}

#[test]
fn no_args_prints_top_level_help_and_exits_zero() {
    let mut command = wavepeek_cmd();

    command
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "wavepeek is a machine-friendly command-line tool for VCD/FST waveform inspection.",
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
            "wavepeek is a machine-friendly command-line tool for VCD/FST waveform inspection.",
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
        .stdout(predicate::str::contains("schema"))
        .stdout(predicate::str::contains("docs"))
        .stdout(predicate::str::contains("\n  help"));
}

#[test]
fn top_level_help_documents_general_conventions() {
    let mut command = wavepeek_cmd();

    command
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("General conventions:"))
        .stdout(predicate::str::contains(
            "Waveform-inspection commands require `--waves <FILE>`.",
        ))
        .stdout(
            predicate::str::contains(
                "Waveform-inspection commands keep their primary inputs as named flags",
            )
            .not(),
        )
        .stdout(
            predicate::str::contains("`schema`, `docs`, and `help` are the non-waveform surfaces")
                .not(),
        )
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
fn top_level_help_describes_shipped_subcommands_without_unimplemented_markers() {
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
            "List signal changes over a time range",
        ))
        .stdout(
            predicate::str::contains("List signal changes over a time range (not implemented yet)")
                .not(),
        )
        .stdout(predicate::str::contains(
            "Evaluate properties over a time range",
        ))
        .stdout(
            predicate::str::contains("Evaluate properties over a time range (not implemented yet)")
                .not(),
        );
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
fn top_level_short_help_is_compact_and_points_to_next_layers() {
    let short_help = successful_stdout_text(&["-h"]);
    let long_help = successful_stdout_text(&["--help"]);

    assert!(short_help.contains("Usage: wavepeek"));
    assert!(short_help.contains("wavepeek --help"));
    assert!(short_help.contains("wavepeek help <command-path...>"));
    assert!(short_help.contains("wavepeek docs"));
    assert!(
        !short_help.contains("General conventions:"),
        "top-level short help should stay compact and omit long-form conventions"
    );
    assert!(
        short_help.len() < long_help.len(),
        "top-level short help should be materially shorter than long help"
    );
}

#[test]
fn no_args_help_matches_short_help_output() {
    let no_args = successful_stdout(&[]);
    let short_help = successful_stdout(&["-h"]);

    assert_eq!(
        no_args, short_help,
        "wavepeek (no args) output must match wavepeek -h byte-for-byte"
    );
}

#[test]
fn top_level_long_help_describes_help_and_docs_entrypoints() {
    let long_help = successful_stdout_text(&["--help"]);

    assert!(long_help.contains("General conventions:"));
    assert!(long_help.contains("wavepeek help <command-path...>"));
    assert!(long_help.contains("wavepeek docs"));
    assert!(long_help.contains("Next steps:"));
    assert!(!long_help.contains("wavepeek docs topics"));
    assert!(!long_help.contains("wavepeek docs show <topic>"));
    assert!(!long_help.contains("wavepeek docs skill"));
}

#[test]
fn help_subcommand_matches_top_level_long_help() {
    assert_same_stdout(
        &["help"],
        &["--help"],
        "wavepeek help output must match wavepeek --help byte-for-byte",
    );
}

#[test]
fn help_subcommand_aliases_nested_long_help() {
    assert_same_stdout(
        &["help", "change"],
        &["change", "--help"],
        "wavepeek help change must match wavepeek change --help byte-for-byte",
    );
    assert_same_stdout(
        &["help", "docs", "show"],
        &["docs", "show", "--help"],
        "wavepeek help docs show must match wavepeek docs show --help byte-for-byte",
    );
}

#[test]
fn shipped_commands_list_matches_top_level_help_surface() {
    let expected: Vec<String> = VISIBLE_TOP_LEVEL_COMMANDS
        .iter()
        .map(|command_name| command_name.to_string())
        .collect();
    let actual = top_level_help_command_names();

    assert_eq!(
        actual, expected,
        "top-level help command list changed; update VISIBLE_TOP_LEVEL_COMMANDS and help contract tests"
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
fn change_help_is_layered_without_examples_or_see_also() {
    let short_help = successful_stdout_text(&["change", "-h"]);
    let long_help = successful_stdout_text(&["change", "--help"]);

    assert!(short_help.contains("Usage: wavepeek change"));
    assert!(!short_help.contains("Examples:"));
    assert!(!short_help.contains("See also:"));
    assert!(!long_help.contains("Examples:"));
    assert!(!long_help.contains("See also:"));
    assert!(
        short_help.len() < long_help.len(),
        "change -h should be materially shorter than change --help"
    );
}

#[test]
fn change_help_uses_aligned_summary_behavior_and_grouped_option_docs() {
    let short_help = successful_stdout_text(&["change", "-h"]);
    let long_help = successful_stdout_text(&["change", "--help"]);
    let alias_help = successful_stdout_text(&["help", "change"]);

    for help in [&short_help, &long_help, &alias_help] {
        assert_eq!(
            help.lines().next(),
            Some("Provides range-based delta snapshots for selected signals.")
        );
    }

    for fragment in [
        "Prints requested signal values for each `--on` trigger firing",
        "Similar to a modified SystemVerilog `$monitor`",
        "Range boundaries are inclusive",
        "Rows are emitted only when sampled signal values changed",
    ] {
        assert!(
            long_help.contains(fragment),
            "change long help should contain `{fragment}`"
        );
    }
    assert!(!long_help.contains("omitted `--on` behaves as wildcard"));
    assert!(!long_help.contains("Event triggers may use typed `iff` logical expressions."));

    for help in [&short_help, &long_help] {
        assert!(help.contains("Input options:"));
        assert!(help.contains("Selection options:"));
        assert!(help.contains("Output options:"));
        assert!(help.contains("Other options:"));
        assert!(help.contains("Path to VCD/FST waveform file"));
        assert!(
            help.contains("Start of inclusive time range (e.g. 1234ns; omitted means dump start)")
        );
        assert!(help.contains("End of inclusive time range (e.g. 1234ns; omitted means dump end)"));
        assert!(help.contains("Canonical scope path for scope-relative signal and trigger names"));
        assert!(help.contains(
            "Comma-separated top-related signal paths, or scope-relative names when --scope is set"
        ));
        assert!(help.contains(
            "Maximum number of snapshot rows (`unlimited` disables truncation, value must be > 0)"
        ));
        assert!(help.contains("Print canonical paths"));
        assert!(help.contains("Machine-readable JSON output"));
        assert!(!help.contains("(`--waves <FILE>` is required)"));
        assert!(!help.contains("(default: 50,"));
        assert!(!help.contains("human output"));
        assert!(!help.contains("(contract: see `wavepeek schema`)"));
    }
}

#[test]
fn docs_command_help_is_layered() {
    let short_help = successful_stdout_text(&["docs", "-h"]);
    let long_help = successful_stdout_text(&["docs", "--help"]);

    assert!(short_help.contains("Usage: wavepeek docs"));
    assert!(short_help.contains("wavepeek docs --help"));
    assert!(short_help.contains("wavepeek docs topics"));
    assert!(
        !short_help.contains("wavepeek docs export <OUT_DIR>"),
        "docs -h should stay compact and defer full command-family detail"
    );
    assert!(long_help.contains("wavepeek docs show <TOPIC>"));
    assert!(long_help.contains("wavepeek docs export <OUT_DIR>"));
    assert!(long_help.contains("wavepeek docs skill"));
    assert!(
        short_help.len() < long_help.len(),
        "docs -h should be materially shorter than docs --help"
    );
}

#[test]
fn info_help_uses_aligned_summary_and_simple_option_docs() {
    let short_help = successful_stdout_text(&["info", "-h"]);
    let long_help = successful_stdout_text(&["info", "--help"]);
    let alias_help = successful_stdout_text(&["help", "info"]);

    for help in [&short_help, &long_help, &alias_help] {
        assert_eq!(
            help.lines().next(),
            Some("Reports metadata for the selected waveform dump.")
        );
    }

    assert!(long_help.contains("Behavior:\n- Prints available metadata (e.g. time unit, start/end times, etc.) in free form\n- `--json` uses the machine contract defined by `wavepeek schema`."));
    for help in [&short_help, &long_help] {
        assert!(help.contains("Input options:"));
        assert!(help.contains("Output options:"));
        assert!(help.contains("Other options:"));
        assert!(help.contains("Path to VCD/FST waveform file"));
        assert!(help.contains("Machine-readable JSON output"));
        assert!(!help.contains("(`--waves <FILE>` is required)"));
        assert!(!help.contains("(contract: see `wavepeek schema`)"));
    }
}

#[test]
fn scope_help_uses_aligned_summary_behavior_and_simple_option_docs() {
    let short_help = successful_stdout_text(&["scope", "-h"]);
    let long_help = successful_stdout_text(&["scope", "--help"]);
    let alias_help = successful_stdout_text(&["help", "scope"]);

    for help in [&short_help, &long_help, &alias_help] {
        assert_eq!(
            help.lines().next(),
            Some("Provides deterministic hierarchy traversal over scope paths.")
        );
    }

    assert!(long_help.contains(
        "Behavior:\n- Finds all scopes matching `--filter` and displays scope name, depth, and kind."
    ));
    assert!(!long_help.contains("human output"));

    for help in [&short_help, &long_help] {
        assert!(help.contains("Input options:"));
        assert!(help.contains("Selection options:"));
        assert!(help.contains("Output options:"));
        assert!(help.contains("Other options:"));
        assert!(help.contains("Path to VCD/FST waveform file"));
        assert!(help.contains(
            "Maximum number of entries (`unlimited` disables truncation, value must be > 0)"
        ));
        assert!(help.contains("Maximum traversal depth (`unlimited` disables depth truncation)"));
        assert!(help.contains("Regex filter for full scope path"));
        assert!(help.contains("Render hierarchy as an indented tree"));
        assert!(help.contains("Machine-readable JSON output"));
        assert!(!help.contains("(`--waves <FILE>` is required)"));
        assert!(!help.contains("(default:"));
        assert!(!help.contains("invalid regex is rejected as an argument error"));
        assert!(!help.contains("(contract: see `wavepeek schema`)"));
    }
}

#[test]
fn signal_help_uses_aligned_summary_behavior_and_simple_option_docs() {
    let short_help = successful_stdout_text(&["signal", "-h"]);
    let long_help = successful_stdout_text(&["signal", "--help"]);
    let alias_help = successful_stdout_text(&["help", "signal"]);

    for help in [&short_help, &long_help, &alias_help] {
        assert_eq!(
            help.lines().next(),
            Some("Provides scope-local signal listings.")
        );
    }

    assert!(long_help.contains(
        "Behavior:\n- Finds all signals matching `--filter` within the selected scope and displays name, kind, and available metadata (for example width)."
    ));
    assert!(long_help.contains("Recursive mode walks child scopes depth-first in stable lexicographic order; `--max-depth` limits recursion when set."));
    assert!(long_help.contains("Includes parser-native signal kinds (not only wires)."));
    assert!(!long_help.contains("human output"));

    for help in [&short_help, &long_help] {
        assert!(help.contains("Input options:"));
        assert!(help.contains("Selection options:"));
        assert!(help.contains("Output options:"));
        assert!(help.contains("Other options:"));
        assert!(help.contains("Path to VCD/FST waveform file"));
        assert!(help.contains("Exact scope path (e.g. top.cpu)"));
        assert!(help.contains(
            "Maximum number of entries (`unlimited` disables truncation, value must be > 0)"
        ));
        assert!(help.contains("Regex filter for signal name"));
        assert!(
            help.contains(
                "Maximum recursion depth below --scope (`unlimited` disables this limit)"
            )
        );
        assert!(help.contains("[default: 5]"));
        assert!(help.contains("Show canonical signal paths"));
        assert!(help.contains("Machine-readable JSON output"));
        assert!(!help.contains("(`--waves <FILE>` is required)"));
        assert!(!help.contains("`--scope` is required"));
        assert!(!help.contains("(default:"));
        assert!(!help.contains("invalid regex is rejected as an argument error"));
        assert!(!help.contains("(contract: see `wavepeek schema`)"));
    }
}

#[test]
fn value_help_uses_aligned_summary_behavior_and_grouped_option_docs() {
    let short_help = successful_stdout_text(&["value", "-h"]);
    let long_help = successful_stdout_text(&["value", "--help"]);
    let alias_help = successful_stdout_text(&["help", "value"]);

    for help in [&short_help, &long_help, &alias_help] {
        assert_eq!(
            help.lines().next(),
            Some("Provides point-in-time sampling for selected signals.")
        );
    }

    for fragment in [
        "Prints values for the requested signals at the selected time point.",
        "By default, signal names are top-related canonical paths",
        "set `--scope` once with a canonical scope path",
        "Do not mix top-related canonical names and scope-relative names",
        "more precise than dump resolution",
    ] {
        assert!(
            long_help.contains(fragment),
            "value long help should contain `{fragment}`"
        );
    }

    for help in [&short_help, &long_help] {
        assert!(help.contains("Input options:"));
        assert!(help.contains("Selection options:"));
        assert!(help.contains("Output options:"));
        assert!(help.contains("Other options:"));
        assert!(help.contains("Path to VCD/FST waveform file"));
        assert!(help.contains("Time point with explicit units (e.g. 1337ns)"));
        assert!(help.contains("Canonical scope path for scope-relative signal names"));
        assert!(help.contains(
            "Comma-separated top-related signal paths, or scope-relative names when --scope is set"
        ));
        assert!(help.contains("Show canonical signal paths"));
        assert!(help.contains("Machine-readable JSON output"));
        assert!(!help.contains("(`--waves <FILE>` is required)"));
        assert!(!help.contains("for example"));
        assert!(!help.contains("bare numbers are rejected as argument errors"));
        assert!(!help.contains("human output"));
        assert!(!help.contains("(contract: see `wavepeek schema`)"));
    }
}

#[test]
fn docs_show_help_is_layered() {
    let short_help = successful_stdout_text(&["docs", "show", "-h"]);
    let long_help = successful_stdout_text(&["docs", "show", "--help"]);

    assert!(short_help.contains("Usage: wavepeek docs show <TOPIC>"));
    assert!(short_help.contains("wavepeek docs show --help"));
    assert!(
        !short_help.contains("excluding YAML front matter"),
        "docs show -h should stay compact"
    );
    assert!(long_help.contains("excluding YAML front matter"));
    assert!(long_help.contains("raw Markdown"));
    assert!(long_help.contains("--summary"));
    assert!(
        short_help.len() < long_help.len(),
        "docs show -h should be materially shorter than docs show --help"
    );
}

#[test]
fn nested_parse_errors_point_to_full_help_path() {
    wavepeek_cmd()
        .args(["docs", "show", "intro", "--wat"])
        .assert()
        .failure()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::starts_with("error: args:"))
        .stderr(predicate::str::contains("unexpected argument '--wat'"))
        .stderr(predicate::str::contains("See 'wavepeek docs show --help'."));
}

#[test]
fn shipped_commands_help_is_self_descriptive() {
    let command_contracts: [(&str, &[&str]); 7] = [
        (
            "info",
            &[
                "Reports metadata for the selected waveform dump.",
                "Prints available metadata",
                "time unit, start/end times",
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
                "Provides scope-local signal listings.",
                "Finds all signals matching `--filter`",
                "depth-first in stable lexicographic order",
                "Includes parser-native signal kinds",
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
                "Prints requested signal values for each `--on` trigger firing",
                "Similar to a modified SystemVerilog `$monitor`",
                "Empty-result and truncation conditions may emit warnings",
                "wavepeek schema",
            ],
        ),
        (
            "property",
            &[
                "Check property over event triggers",
                "Evaluate `--eval` on timestamps selected by `--on`",
                "when `--eval` references at least one signal or raw event",
                "--capture",
                "`switch` emits `assert` and `deassert` rows",
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
        .stdout(predicate::str::is_match(r"^\d+\.\d+\.\d+\n$").unwrap())
        .stderr(predicate::str::is_empty());

    let mut long_command = wavepeek_cmd();

    long_command
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::is_match(r"^wavepeek v\d+\.\d+\.\d+\n$").unwrap())
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
        .stdout(predicate::str::contains(
            "`switch` emits `assert` and `deassert` rows",
        ))
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
        .stdout(predicate::str::contains("limits recursion when set"));
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
fn property_accepts_capture_flag_in_cli_then_runs() {
    let fixture = common::fixture_path("m2_core.vcd");
    let fixture = fixture.to_string_lossy().into_owned();

    let output = wavepeek_cmd()
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
            "--json",
        ])
        .output()
        .expect("property should execute");

    assert!(output.status.success());
    assert!(output.stderr.is_empty());

    let value: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("stdout should be valid json");
    assert_eq!(value["command"], "property");
    assert!(
        value["data"].is_array(),
        "property json output should use an array payload"
    );
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
        .stdout(predicate::str::contains("Execution is not implemented yet.").not());
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
fn legacy_subcommands_are_rejected_without_alias() {
    for legacy_name in ["tree", "modules", "signals", "changes", "when", "at"] {
        assert_legacy_subcommand_rejected(legacy_name);
    }
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
    let cases: &[(&[&str], &str)] = &[
        (&["schema", "--human"], "schema"),
        (&["info", "--waves", "dump.vcd", "--human"], "info"),
        (&["scope", "--waves", "dump.vcd", "--human"], "scope"),
        (
            &["signal", "--waves", "dump.vcd", "--scope", "top", "--human"],
            "signal",
        ),
        (
            &[
                "value",
                "--waves",
                "dump.vcd",
                "--at",
                "1ns",
                "--signals",
                "sig",
                "--human",
            ],
            "value",
        ),
        (
            &[
                "change",
                "--waves",
                "dump.vcd",
                "--signals",
                "sig",
                "--human",
            ],
            "change",
        ),
        (
            &[
                "property", "--waves", "dump.vcd", "--on", "*", "--eval", "1", "--human",
            ],
            "property",
        ),
    ];

    for (args, command_name) in cases {
        assert_human_flag_rejected(args, command_name);
    }
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
