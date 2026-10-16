use std::io;

use assert_cmd::prelude::*;
use predicates::prelude::*;

mod common;
use common::wavepeek_cmd;

const VISIBLE_TOP_LEVEL_COMMANDS: [&str; 9] = [
    "info", "scope", "signal", "value", "change", "property", "extract", "skill", "help",
];

#[cfg(feature = "fsdb")]
const EXPECTED_FSDB_FEATURE_STATUS: &str = "FSDB - enabled";
#[cfg(not(feature = "fsdb"))]
const EXPECTED_FSDB_FEATURE_STATUS: &str = "FSDB - disabled (FSDB support is currently Linux x86_64 only; reinstall with Cargo flag `--features fsdb` and provide the Synopsys Verdi FSDB Reader SDK)";

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
        .stderr(predicate::str::starts_with("fatal: args:"))
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
        .stderr(predicate::str::starts_with("fatal: args:"))
        .stderr(predicate::str::contains("unexpected argument '--human'"))
        .stderr(predicate::str::contains(format!(
            "See 'wavepeek {command_name} --help'."
        )));
}

#[test]
fn closed_stdout_is_silent_success() {
    for args in [
        &["skill", "--help"][..],
        &["--help"][..],
        &["-V"][..],
        &["--version"][..],
    ] {
        let (reader, writer) = io::pipe().expect("pipe should open");
        drop(reader);

        let output = wavepeek_cmd()
            .args(args)
            .stdout(writer)
            .output()
            .expect("wavepeek should run");

        assert!(
            output.status.success(),
            "expected success for args {args:?}, got {:?}: {}",
            output.status.code(),
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(
            output.stderr.is_empty(),
            "expected empty stderr for args {args:?}, got: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

#[test]
fn no_args_prints_top_level_help_and_exits_zero() {
    let mut command = wavepeek_cmd();

    command
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "wavepeek queries saved RTL waveform dumps.",
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
            "wavepeek queries saved RTL waveform dumps.",
        ))
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
        .stdout(predicate::str::contains("extract"))
        .stdout(predicate::str::contains("skill"))
        .stdout(predicate::str::contains("\n  help"));
}

#[test]
fn top_level_help_documents_general_conventions() {
    let mut command = wavepeek_cmd();

    command
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Behavior:"))
        .stdout(predicate::str::contains(
            "Each waveform command opens one waveform dump, runs one query, writes its output, and exits.",
        ))
        .stdout(predicate::str::contains(
            "Every build supports VCD and FST. FSDB requires Linux x86_64",
        ))
        .stdout(predicate::str::contains("Optional features:"))
        .stdout(predicate::str::contains(EXPECTED_FSDB_FEATURE_STATUS))
        .stdout(predicate::str::contains(if cfg!(feature = "shm") { "SHM - enabled" } else { "SHM - disabled" }))
        .stdout(predicate::str::contains(
            "Use `--json` for one JSON value or `--jsonl` for a stream of JSON records.",
        ))
        .stdout(predicate::str::contains(
            "for example `250ps`, `10ns`, or `2us`",
        ))
        .stdout(predicate::str::contains(
            "The `--from` and `--to` boundaries are inclusive.",
        ))
        .stdout(predicate::str::contains(
            "Names ending in `.md` refer to files in the packaged skill.",
        ))
        .stdout(predicate::str::contains("./wavepeek-skill/references/"));
}

#[test]
fn top_level_help_describes_shipped_subcommands_without_unimplemented_markers() {
    let mut command = wavepeek_cmd();

    command
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Read selected signal values at one or more times.",
        ))
        .stdout(predicate::str::contains(
            "Read signal values at selected events over a time range.",
        ))
        .stdout(predicate::str::contains(
            "Evaluate a Boolean expression at selected events.",
        ))
        .stdout(predicate::str::contains("not implemented yet").not());
}

#[test]
fn top_level_short_help_is_compact_and_points_to_next_layers() {
    let short_help = successful_stdout_text(&["-h"]);
    let long_help = successful_stdout_text(&["--help"]);

    assert!(short_help.contains("Usage: wavepeek"));
    assert!(short_help.contains("wavepeek queries saved RTL waveform dumps."));
    assert!(short_help.contains("skill"));
    assert!(short_help.contains("help"));
    assert!(!short_help.contains("Behavior:"));
    assert!(!short_help.contains("Examples:"));
    assert!(!short_help.contains("Notes:"));
    assert!(!short_help.contains("Optional features:"));
    assert!(!short_help.contains("Next steps:"));
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
fn top_level_long_help_describes_help_and_skill_entrypoints() {
    let long_help = successful_stdout_text(&["--help"]);

    assert!(long_help.contains("Behavior:"));
    assert!(long_help.contains("wavepeek help extract axi"));
    assert!(long_help.contains("wavepeek skill ./wavepeek-skill"));
    assert!(long_help.contains("./wavepeek-skill/references/"));
    assert!(!long_help.contains("Next steps:"));
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
        &["help", "skill"],
        &["skill", "--help"],
        "wavepeek help skill must match wavepeek skill --help byte-for-byte",
    );
    assert_same_stdout(
        &["help", "extract", "axi"],
        &["extract", "axi", "--help"],
        "wavepeek help extract axi must match wavepeek extract axi --help byte-for-byte",
    );
    assert_same_stdout(
        &["extract", "help", "axi"],
        &["extract", "axi", "--help"],
        "wavepeek extract help axi must match wavepeek extract axi --help byte-for-byte",
    );
}

#[test]
fn help_command_documents_command_paths() {
    let help = successful_stdout_text(&["help", "--help"]);

    assert_eq!(
        help.lines().next(),
        Some("Show detailed help for a command path.")
    );
    assert!(help.contains("wavepeek help extract axi"));
    assert!(help.contains("Command path to describe (for example, extract axi)"));
    assert!(help.contains("Notes:"));
    assert!(help.contains("commands.md"));
}

#[test]
fn help_commands_reject_path_delimiters_without_panicking() {
    for args in [
        vec!["help", "--", "help", "anything", "--"],
        vec!["extract", "help", "--", "axi", "anything", "--"],
    ] {
        wavepeek_cmd()
            .args(args)
            .assert()
            .failure()
            .stdout(predicate::str::is_empty())
            .stderr(predicate::str::starts_with("fatal: args:"))
            .stderr(predicate::str::contains(
                "help command paths cannot contain '--'",
            ));
    }
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
fn waveform_help_avoids_inline_envelope_or_parse_hints() {
    for command_name in ["info", "scope", "signal", "value", "change", "property"] {
        let long_help = successful_stdout_text(&[command_name, "--help"]);

        assert!(
            !long_help.contains("`data`"),
            "help for {command_name} should not inline JSON envelope field names"
        );
        assert!(
            !long_help.contains("`diagnostics`"),
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
fn change_help_is_layered_with_examples_and_notes() {
    let short_help = successful_stdout_text(&["change", "-h"]);
    let long_help = successful_stdout_text(&["change", "--help"]);

    assert!(short_help.contains("Usage: wavepeek change"));
    assert!(!short_help.contains("Behavior:"));
    assert!(!short_help.contains("Examples:"));
    assert!(!short_help.contains("Notes:"));
    assert!(long_help.contains("Behavior:"));
    assert!(long_help.contains("Examples:"));
    assert!(long_help.contains("Notes:"));
    assert!(long_help.contains("inspect-values.md"));
    assert!(long_help.contains("sampling.md"));
    assert!(long_help.contains("event-expressions.md"));
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
            Some("Read signal values at selected events over a time range.")
        );
    }

    for fragment in [
        "`--on` selects events, and `--signals` selects the values printed for each event.",
        "`--signals` accepts comma-separated values, repeated options, or both.",
        "`--row-mode dense` prints every sampled event.",
        "`--row-values full` prints every requested value.",
        "Range boundaries are inclusive.",
        "Signal names may end in `[msb:lsb]`.",
        "Exact waveform paths take precedence",
        "wildcard comparisons use the projected values",
    ] {
        assert!(
            long_help.contains(fragment),
            "change long help should contain `{fragment}`"
        );
    }

    for help in [&short_help, &long_help] {
        assert!(help.contains("Input options:"));
        assert!(help.contains("Selection options:"));
        assert!(help.contains("Output options:"));
        assert!(help.contains("Other options:"));
        assert!(help.contains("Path to a VCD, FST, or FSDB file, or an SHM database directory"));
        assert!(help.contains(
            "Start of the inclusive time range (for example, 1234ns; default: dump start)"
        ));
        assert!(help.contains("Signal paths or flat projections, comma-separated or repeated"));
        assert!(
            help.contains(
                "End of the inclusive time range (for example, 2000ns; default: dump end)"
            )
        );
        assert!(
            help.contains("Scope for relative signal and trigger names (for example, top.cpu)")
        );
        assert!(help.contains(
            "flat projections, comma-separated or repeated (for example, state,req or status[7:4])"
        ));
        assert!(help.contains("[default: dense]"));
        assert!(help.contains("[possible values: dense, sparse]"));
        assert!(help.contains("[default: full]"));
        assert!(help.contains("[possible values: full, delta]"));
        assert!(help.contains("Machine-readable JSON output"));
    }
}

#[test]
fn property_help_uses_aligned_summary_behavior_and_grouped_option_docs() {
    let short_help = successful_stdout_text(&["property", "-h"]);
    let long_help = successful_stdout_text(&["property", "--help"]);
    let alias_help = successful_stdout_text(&["help", "property"]);

    for help in [&short_help, &long_help, &alias_help] {
        assert_eq!(
            help.lines().next(),
            Some("Evaluate a Boolean expression at selected events.")
        );
    }

    for fragment in [
        "`--on` selects events, and `--eval` defines the expression checked at each event.",
        "`--capture match` prints every selected event where the expression is true.",
        "`--capture switch` prints both result transitions.",
        "`assert` prints false-to-true transitions",
        "This is a sampled Boolean check, not a SystemVerilog temporal assertion.",
    ] {
        assert!(
            long_help.contains(fragment),
            "property long help should contain `{fragment}`"
        );
    }

    for help in [&short_help, &long_help] {
        assert!(help.contains("Input options:"));
        assert!(help.contains("Selection options:"));
        assert!(help.contains("Output options:"));
        assert!(help.contains("Other options:"));
        assert!(help.contains("Path to a VCD, FST, or FSDB file, or an SHM database directory"));
        assert!(help.contains(
            "Start of the inclusive time range (for example, 1234ns; default: dump start)"
        ));
        assert!(
            help.contains(
                "End of the inclusive time range (for example, 2000ns; default: dump end)"
            )
        );
        assert!(
            help.contains("Scope for relative event and expression names (for example, top.cpu)")
        );
        assert!(help.contains(
            "Logical expression evaluated at selected events (for example, 'ready && !stall')"
        ));
        assert!(help.contains("[possible values: match, switch, assert, deassert]"));
        assert!(help.contains("Machine-readable JSON output"));
    }
}

#[test]
fn extract_command_without_subcommand_prints_help() {
    let no_args = successful_stdout_text(&["extract"]);
    let short_help = successful_stdout_text(&["extract", "-h"]);
    let long_help = successful_stdout_text(&["extract", "--help"]);
    let alias_help = successful_stdout_text(&["help", "extract"]);

    for help in [&no_args, &short_help, &long_help, &alias_help] {
        assert_eq!(
            help.lines().next(),
            Some("Extract event rows from waveform signals.")
        );
        assert!(help.contains("Usage: wavepeek extract"));
        assert!(help.contains("Commands:"));
        assert!(help.contains("axi"));
        assert!(help.contains("axistream"));
        assert!(help.contains("generic"));
        assert!(!help.contains("fatal: args:"));
    }

    assert_eq!(
        no_args, short_help,
        "wavepeek extract should show short help"
    );
    assert!(
        short_help.len() < long_help.len(),
        "extract -h should be materially shorter than extract --help"
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
            Some("Show metadata for one waveform dump.")
        );
        assert!(help.contains("Path to a VCD, FST, or FSDB file, or an SHM database directory"));
    }

    assert!(!short_help.contains("Behavior:"));
    assert!(
        long_help.contains("Behavior:\n- Reports the dump time unit, start time, and end time.")
    );
    assert!(long_help.contains("Example:\n  wavepeek info --waves dump.fst"));
    assert!(long_help.contains("explore-dump.md"));
    assert_eq!(long_help, alias_help);
}

#[test]
fn scope_help_uses_aligned_summary_behavior_and_simple_option_docs() {
    let short_help = successful_stdout_text(&["scope", "-h"]);
    let long_help = successful_stdout_text(&["scope", "--help"]);
    let alias_help = successful_stdout_text(&["help", "scope"]);

    for help in [&short_help, &long_help, &alias_help] {
        assert_eq!(
            help.lines().next(),
            Some("List scopes in a waveform hierarchy.")
        );
        assert!(help.contains("Path to a VCD, FST, or FSDB file, or an SHM database directory"));
        assert!(help.contains("[default: 5]"));
        assert!(help.contains("[default: .*]"));
        assert!(help.contains("[default: 50]"));
    }

    assert!(!short_help.contains("Behavior:"));
    assert!(long_help.contains("Matches `--filter` against full scope paths"));
    assert!(long_help.contains("stable pre-order depth-first traversal"));
    assert!(long_help.contains("Examples:"));
    assert!(long_help.contains("explore-dump.md"));
    assert_eq!(long_help, alias_help);
}

#[test]
fn signal_help_uses_aligned_summary_behavior_and_simple_option_docs() {
    let short_help = successful_stdout_text(&["signal", "-h"]);
    let long_help = successful_stdout_text(&["signal", "--help"]);
    let alias_help = successful_stdout_text(&["help", "signal"]);

    for help in [&short_help, &long_help, &alias_help] {
        assert_eq!(
            help.lines().next(),
            Some("List signals in one waveform scope.")
        );
        assert!(help.contains("Path to a VCD, FST, or FSDB file, or an SHM database directory"));
        assert!(help.contains("Exact scope path (for example, top.cpu)"));
        assert!(help.contains("[default: 5]"));
        assert!(help.contains("[default: .*]"));
        assert!(help.contains("[default: 50]"));
    }

    assert!(!short_help.contains("Behavior:"));
    assert!(long_help.contains("Matches `--filter` against signal names"));
    assert!(long_help.contains("`--recursive` also visits child scopes"));
    assert!(long_help.contains("requires --recursive"));
    assert!(long_help.contains("Examples:"));
    assert!(long_help.contains("paths.md"));
    assert_eq!(long_help, alias_help);
}

#[test]
fn value_help_uses_aligned_summary_behavior_and_grouped_option_docs() {
    let short_help = successful_stdout_text(&["value", "-h"]);
    let long_help = successful_stdout_text(&["value", "--help"]);
    let alias_help = successful_stdout_text(&["help", "value"]);

    for help in [&short_help, &long_help, &alias_help] {
        assert_eq!(
            help.lines().next(),
            Some("Read selected signal values at one or more times.")
        );
        assert!(help.contains("Path to a VCD, FST, or FSDB file, or an SHM database directory"));
        assert!(help.contains("for example, 1337ns or 10ns,20ns"));
        assert!(help.contains("Scope for relative signal names (for example, top.cpu)"));
        assert!(help.contains(
            "flat projections, comma-separated or repeated (for example, state,pc or status[7:4])"
        ));
    }

    for fragment in [
        "`--at` and `--signals` accept comma-separated values, repeated options, or both.",
        "preserves request order and duplicates",
        "Uses canonical signal paths by default",
        "A trailing `[msb:lsb]` selects bits from a flat integral signal",
        "`[n]` remains part of an ordinary waveform path",
        "finer than the dump resolution",
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
        assert!(help.contains("Time points with explicit units, comma-separated or repeated"));
        assert!(help.contains("Signal paths or flat projections, comma-separated or repeated"));
        assert!(help.contains("Machine-readable JSON output"));
    }
    assert!(long_help.contains("Examples:"));
    assert!(long_help.contains("inspect-values.md"));
    assert_eq!(long_help, alias_help);
}

#[test]
fn skill_help_surfaces_are_aligned_and_trimmed() {
    let short_help = successful_stdout_text(&["skill", "-h"]);
    let long_help = successful_stdout_text(&["skill", "--help"]);
    let alias_help = successful_stdout_text(&["help", "skill"]);

    for help in [&short_help, &long_help, &alias_help] {
        assert_eq!(
            help.lines().next(),
            Some("Extract the packaged agent skill into a directory.")
        );
        assert!(help.contains("for example, ./wavepeek-skill"));
        assert!(!help.contains("--json"));
    }
    assert!(!short_help.contains("Behavior:"));
    assert!(long_help.contains("Behavior:"));
    assert!(long_help.contains("wavepeek skill ./wavepeek-skill"));
    assert!(long_help.contains("Notes:"));
    assert!(long_help.contains("quickstart.md"));
    assert_eq!(long_help, alias_help);
}

#[test]
fn shipped_commands_help_is_self_descriptive() {
    let command_contracts: [(&str, &[&str]); 7] = [
        (
            "info",
            &["Show metadata for one waveform dump.", "dump time unit"],
        ),
        (
            "scope",
            &[
                "List scopes in a waveform hierarchy.",
                "pre-order depth-first traversal",
                "Truncation produces a coded diagnostic",
            ],
        ),
        (
            "signal",
            &[
                "List signals in one waveform scope.",
                "Matches `--filter` against signal names",
                "normalized signal kinds",
            ],
        ),
        (
            "value",
            &[
                "Read selected signal values at one or more times.",
                "preserves request order and duplicates",
                "Verilog literals",
            ],
        ),
        (
            "change",
            &[
                "Read signal values at selected events over a time range.",
                "`--row-mode dense`",
                "`--row-values full`",
            ],
        ),
        (
            "property",
            &[
                "Evaluate a Boolean expression at selected events.",
                "`--capture match`",
                "false-to-true transitions",
            ],
        ),
        (
            "skill",
            &["Extract the packaged agent skill into a directory."],
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
fn extract_ahb_help_is_self_descriptive() {
    let long_help = successful_stdout_text(&["extract", "ahb", "--help"]);
    for fragment in [
        "Extract manager-facing AHB address and data-phase events.",
        "Supports AHB-Lite and AHB5 from Arm IHI 0033C, Issue C.",
        "data completions remain separate from idle clocks",
        "The include flags add stall, idle, or busy cycle events.",
        "Uses manager-facing HREADY.",
        "Explicit `STD_NAME=WAVES_NAME` mappings override signals found by include regexes.",
        "[default: ahb-lite]",
        "[possible values: ahb-lite, ahb5]",
        "`--source` conflicts with `--profile`",
        "Pipeline warm-up starts before `--from`",
        "`initial_data_phase`",
        "does not reconstruct bursts",
        "extract-ahb.md",
    ] {
        assert!(
            long_help.contains(fragment),
            "extract ahb long help should contain `{fragment}`"
        );
    }
}

#[test]
fn extract_apb_help_is_self_descriptive() {
    let long_help = successful_stdout_text(&["extract", "apb", "--help"]);
    for fragment in [
        "Extract APB Setup and Access events.",
        "Supports APB3, APB4, and APB5 from Arm IHI 0024E.",
        "`--include-wait` adds one row for each waited Access cycle.",
        "Mapped PREADY mode requires `pready`.",
        "Maps one concrete Completer PSELx signal as `psel`.",
        "canonical lowercase profile and mode values",
        "[default: apb4]",
        "[possible values: apb3, apb4, apb5]",
        "[default: mapped]",
        "[possible values: mapped, implicit-high]",
        "`--source` conflicts with `--profile`",
        "does not correlate or validate transactions",
        "extract-apb.md",
    ] {
        assert!(
            long_help.contains(fragment),
            "extract apb long help should contain `{fragment}`"
        );
    }
}

#[test]
fn extract_atb_help_is_self_descriptive() {
    let long_help = successful_stdout_text(&["extract", "atb", "--help"]);
    for fragment in [
        "Extract ATB transfer, flush, and synchronization-request events.",
        "Supports ATB-A, ATB-B, and ATB-C from Arm IHI 0032C, Issue C.",
        "complete ATVALID/ATREADY and AFVALID/AFREADY handshakes",
        "A mapped SYNCREQ signal",
        "Orders same-edge events as transfer, flush, then synchronization request.",
        "without trace decoding",
        "[default: atb-c]",
        "[possible values: atb-a, atb-b, atb-c]",
        "CLI profile aliases are `atb_a`",
        "does not reconstruct packets",
        "extract-atb.md",
    ] {
        assert!(
            long_help.contains(fragment),
            "extract atb long help should contain `{fragment}`"
        );
    }
}

#[test]
fn extract_axi_help_is_self_descriptive() {
    let long_help = successful_stdout_text(&["extract", "axi", "--help"]);
    for fragment in [
        "Extract AXI-family ready/valid channel transfers.",
        "Supports AXI3, AXI4, AXI4-Lite, AXI5, AXI5-Lite, ACE, ACE-Lite, ACE5, ACE5-Lite, ACE5-LiteDVM, and ACE5-LiteACP.",
        "use Arm IHI 0022H.c",
        "Arm IHI 0022L ready/valid transport",
        "one event source for each complete ready/valid channel",
        "DVM `ac` and `cr` channels, but not `cd`",
        "canonical hyphenated profile names",
        "[default: axi4]",
        "[possible values: axi3, axi4, axi4-lite, axi5, axi5-lite, ace, ace-lite, ace5, ace5-lite, ace5-lite-dvm, ace5-lite-acp]",
        "CLI aliases include `ace5_lite`",
        "does not decode DVM messages or coherency state",
        "does not reconstruct bursts",
        "extract-axi.md",
    ] {
        assert!(
            long_help.contains(fragment),
            "extract axi long help should contain `{fragment}`"
        );
    }
}

#[test]
fn extract_axistream_help_is_self_descriptive() {
    let long_help = successful_stdout_text(&["extract", "axistream", "--help"]);
    for fragment in [
        "Extract AXI-Stream transfers.",
        "Supports AXI4-Stream and AXI5-Stream from Arm IHI 0051B, Issue B.",
        "Mapped TREADY mode requires `tvalid` and `tready`.",
        "physical TREADY is absent",
        "pre-edge sample point for each rising ACLK edge",
        "without adding a channel name",
        "[default: axi4-stream]",
        "[possible values: axi4-stream, axi5-stream]",
        "[default: mapped]",
        "[possible values: mapped, implicit-high]",
        "wake-up, parity, and check signals are outside this extractor",
        "extract-axis.md",
    ] {
        assert!(
            long_help.contains(fragment),
            "extract axistream long help should contain `{fragment}`"
        );
    }
}

#[test]
fn extract_generic_help_is_self_descriptive() {
    let long_help = successful_stdout_text(&["extract", "generic", "--help"]);
    for fragment in [
        "Extract custom synchronous events and their payload values.",
        "`--on` selects edge-only events.",
        "always sampled at the pre-edge sample point",
        "`--payload` accepts comma-separated values, repeated options, or both.",
        "Entries may end in `[msb:lsb]`",
        "Exact waveform paths take precedence",
        "Payload paths or flat projections, comma-separated or repeated",
        "Scope for relative event, predicate, and payload names (for example, top.fifo)",
        "`--source` conflicts with `--name`, `--on`, `--when`, and `--payload`",
        "JSON and JSONL rows include `time`, `sample_time`, `source`, and ordered payload values.",
        "extract-transfers.md",
    ] {
        assert!(
            long_help.contains(fragment),
            "extract generic long help should contain `{fragment}`"
        );
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
fn subcommand_help_uses_extended_descriptions() {
    let mut scope_command = wavepeek_cmd();

    scope_command
        .args(["scope", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("pre-order depth-first traversal"))
        .stdout(predicate::str::contains(
            "Truncation produces a coded diagnostic.",
        ))
        .stdout(predicate::str::contains("Examples:"))
        .stdout(predicate::str::contains("Notes:"));

    let mut property_command = wavepeek_cmd();

    property_command
        .args(["property", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "`--capture switch` prints both result transitions.",
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
        .stdout(predicate::str::contains("--max <MAX>"))
        .stdout(predicate::str::contains("unlimited"))
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
        .stderr(predicate::str::starts_with("fatal: args:"))
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
        .stderr(predicate::str::starts_with("fatal: args:"))
        .stderr(predicate::str::contains(
            "required arguments were not provided",
        ))
        .stderr(predicate::str::contains("--waves <FILE>"))
        .stderr(predicate::str::contains("See 'wavepeek info --help'."));
}

#[test]
fn legacy_subcommands_are_rejected_without_alias() {
    for legacy_name in [
        "tree", "modules", "signals", "changes", "when", "at", "docs",
    ] {
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
        .stderr(predicate::str::starts_with("fatal: args:"))
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
        .stderr(predicate::str::starts_with("fatal: args:"))
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
        .stderr(predicate::str::starts_with("fatal: args:"))
        .stderr(predicate::str::contains("unexpected argument '--wat'"))
        .stderr(predicate::str::contains("See 'wavepeek info --help'."));
}

#[test]
fn all_commands_reject_human_flag() {
    let cases: &[(&[&str], &str)] = &[
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
        .stderr(predicate::str::starts_with("fatal: args:"))
        .stderr(predicate::str::contains("unexpected argument '--wat'"))
        .stderr(predicate::str::contains("See 'wavepeek --help'."));
}
