pub mod at;
pub mod change;
pub mod info;
pub mod limits;
pub mod schema;
pub mod scope;
pub mod signal;
pub mod when;

use clap::error::ErrorKind;
use clap::parser::ValueSource;
use clap::{ArgAction, CommandFactory, FromArgMatches, Parser, Subcommand};

use crate::engine::{self, Command as EngineCommand};
use crate::error::WavepeekError;
use crate::output;

#[derive(Debug, Parser)]
#[command(
    name = "wavepeek",
    version,
    about = "wavepeek is a command-line tool for RTL waveform inspection.\nIt provides deterministic, machine-friendly output and a minimal set of primitives that compose into repeatable debug recipes.",
    long_about = r#"wavepeek is a command-line tool for RTL waveform inspection.
It provides deterministic, machine-friendly output and a minimal set of primitives that compose into repeatable debug recipes.

General conventions:
- No positional command arguments: after choosing a subcommand, inputs are named flags.
- Waveform commands require `--waves <FILE>`; `schema` is the exception and accepts no waveform input flags.
- Output is bounded by default (for example with `--max`, `--first`/`--last`, or finite command shape) and recursive traversals are depth-bounded.
- Default output is human-readable for waveform commands; `--json` enables machine-readable output and its contract is defined by `wavepeek schema`.
- Time values require explicit units (`zs`, `as`, `fs`, `ps`, `ns`, `us`, `ms`, `s`) and integer magnitudes.
- Parsed times are normalized to dump `time_unit`; time-window flags (`--from`, `--to`) use inclusive boundaries.
- Errors follow `error: <category>: <message>`.

Use `wavepeek <command> --help` (or `-h`) for detailed command behavior and examples."#,
    disable_help_subcommand = true
)]
pub struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    #[command(
        about = "Show dump metadata (time unit and bounds)",
        long_about = r#"Reports dump metadata for the selected waveform dump.

Behavior:
- Prints `time_unit`, `time_start`, and `time_end`.
- Human-readable output is the default terminal mode.
- `--json` uses the machine contract defined by `wavepeek schema`.

Use this command first to confirm dump bounds before time-window queries."#
    )]
    Info(info::InfoArgs),
    #[command(
        about = "List hierarchy scopes (deterministic DFS)",
        long_about = r#"Provides deterministic hierarchy traversal over scope paths.

Behavior:
- Traversal order is stable: pre-order depth-first, with lexicographic child ordering.
- Includes parser-native scope kinds from hierarchy data (not only modules).
- `--tree` switches human output from flat list to visual hierarchy rendering.
- Truncation and disabled-limit conditions emit warnings.
- `--json` uses the machine contract defined by `wavepeek schema`.

Use this command to explore hierarchy shape before narrowing to signal-level queries."#
    )]
    Scope(scope::ScopeArgs),
    #[command(
        about = "List signals in scope with metadata",
        long_about = r#"Provides scope-local signal listing with deterministic ordering.

Behavior:
- Default mode lists only direct signals in the selected scope.
- Recursive mode walks child scopes depth-first in stable lexicographic order.
- Human output uses short names in non-recursive mode and scope-relative display in recursive mode; `--abs` always shows canonical paths.
- `--max-depth` applies only in recursive mode.
- Truncation and disabled-limit conditions emit warnings.
- `--json` uses the machine contract defined by `wavepeek schema`.

Use this command after `scope` to inspect available signals in a target scope."#
    )]
    Signal(signal::SignalArgs),
    #[command(
        about = "Get signal values at a specific time point",
        long_about = r#"Provides point-in-time sampling for selected signals.

Behavior:
- Supports canonical names without `--scope` and scope-relative names with `--scope`.
- Output preserves the input order from `--signals`.
- Time tokens must include explicit units and align to dump precision.
- Values are emitted as Verilog literals (`<width>'h<digits>` with `x`/`z` support).
- Fails fast if any requested signal cannot be resolved.
- `--json` uses the machine contract defined by `wavepeek schema`.

Use this command for deterministic spot checks at a specific timestamp."#
    )]
    At(at::AtArgs),
    #[command(
        about = "Get value snapshots over a time range",
        long_about = r#"Provides range-based delta snapshots for selected signals.

Behavior:
- Range boundaries are inclusive; baseline state is initialized at range start.
- Candidate timestamps come from `--when` triggers; omitted `--when` behaves as wildcard (`*`).
- Rows are emitted only when sampled signal values changed from prior sampled state.
- Empty-result and truncation conditions may emit warnings.
- `iff` clauses are parsed, but logical-condition execution for `iff` is deferred in the current release.
- `--json` uses the machine contract defined by `wavepeek schema`.

Use this command to inspect value transitions over bounded time windows."#
    )]
    Change(change::ChangeArgs),
    #[command(
        about = "Find cycles where a condition is true (not implemented yet)",
        long_about = r#"Find cycles where a condition is true.

Behavior:
- Intended semantics: evaluate `--cond` on each posedge of `--clk` and report matching timestamps.
- Qualifiers select all matches, first N, or last N using `--max`, `--first`, and `--last`.
- `--max unlimited` is accepted by CLI parsing when no qualifier is used.
- Execution is not implemented yet.
- `--json` uses the machine contract defined by `wavepeek schema`.

Use this help as the parse/contract reference until runtime execution is implemented."#
    )]
    When(when::WhenArgs),
    #[command(
        about = "Print canonical JSON schema contract",
        long_about = r#"Prints the canonical JSON schema document for wavepeek machine output contracts.

Behavior:
- Accepts no command-specific flags or positional arguments.
- Prints exactly one deterministic schema document to stdout.
- Output bytes match the canonical artifact `schema/wavepeek.json`.
- This is the source of truth for all `--json` command outputs.

Use this command to fetch the machine-readable contract consumed by JSON-mode clients."#
    )]
    Schema(schema::SchemaArgs),
}

pub fn run() -> Result<(), WavepeekError> {
    let argv: Vec<_> = std::env::args_os().collect();
    let parse_argv = if argv.len() == 1 {
        vec![argv[0].clone(), "--help".into()]
    } else {
        argv
    };

    let matches = match build_cli_command().try_get_matches_from(parse_argv) {
        Ok(matches) => matches,
        Err(error) => return handle_parse_error(error),
    };

    if change_tune_overrides_requested(&matches) && !is_debug_mode_enabled() {
        return Err(WavepeekError::Args(
            "internal tuning overrides (--tune-*) require DEBUG=1. Set DEBUG=1 only for local diagnostics or CI debugging."
                .to_string(),
        ));
    }

    let cli = match Cli::from_arg_matches(&matches) {
        Ok(cli) => cli,
        Err(error) => return handle_parse_error(error),
    };

    dispatch(cli.command)
}

fn is_debug_mode_enabled() -> bool {
    std::env::var("DEBUG")
        .map(|value| value == "1")
        .unwrap_or(false)
}

fn change_tune_overrides_requested(matches: &clap::ArgMatches) -> bool {
    let Some(("change", change_matches)) = matches.subcommand() else {
        return false;
    };

    is_command_line_override(change_matches, "tune_engine")
        || is_command_line_override(change_matches, "tune_candidates")
        || is_command_line_override(change_matches, "tune_edge_fast_force")
}

fn is_command_line_override(matches: &clap::ArgMatches, arg: &str) -> bool {
    matches!(matches.value_source(arg), Some(ValueSource::CommandLine))
}

fn disable_default_help_flags_recursively(command: &mut clap::Command) {
    *command = std::mem::take(command)
        .disable_help_flag(true)
        .mut_subcommands(|subcommand| {
            let mut subcommand = subcommand;
            disable_default_help_flags_recursively(&mut subcommand);
            subcommand
        });
}

fn build_cli_command() -> clap::Command {
    let mut command = Cli::command();
    disable_default_help_flags_recursively(&mut command);

    command.arg(
        clap::Arg::new("help")
            .short('h')
            .long("help")
            .global(true)
            .help("Print help")
            .action(ArgAction::HelpLong),
    )
}

fn handle_parse_error(error: clap::Error) -> Result<(), WavepeekError> {
    match error.kind() {
        ErrorKind::DisplayHelp | ErrorKind::DisplayVersion => error.print().map_err(|io_error| {
            WavepeekError::Internal(format!("failed to print help: {io_error}"))
        }),
        _ => Err(WavepeekError::Args(normalize_clap_error(&error))),
    }
}

fn normalize_clap_error(error: &clap::Error) -> String {
    let rendered = error.render().to_string();
    let detail = clap_error_detail(rendered.as_str());
    let hint = help_hint_for_rendered_clap_error(rendered.as_str());

    format!("{detail} {hint}")
}

fn clap_error_detail(rendered: &str) -> String {
    let lines: Vec<&str> = rendered.lines().collect();
    if let Some(start_index) = lines
        .iter()
        .position(|line| line.trim_start().starts_with("error:"))
    {
        let mut chunks = Vec::new();

        for (index, line) in lines.iter().enumerate().skip(start_index) {
            let trimmed = line.trim();
            if index > start_index
                && (trimmed.starts_with("Usage:") || trimmed.starts_with("For more information"))
            {
                break;
            }

            if index == start_index {
                if let Some(rest) = trimmed.strip_prefix("error:") {
                    let rest = rest.trim();
                    if !rest.is_empty() {
                        chunks.push(rest.to_string());
                    }
                }
                continue;
            }

            if !trimmed.is_empty() {
                chunks.push(trimmed.to_string());
            }
        }

        if !chunks.is_empty() {
            return chunks.join(" ");
        }
    }

    for line in lines {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("error:") {
            return rest.trim().to_string();
        }
    }

    rendered
        .lines()
        .find_map(|line| {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_string())
            }
        })
        .unwrap_or_else(|| "invalid arguments".to_string())
}

fn help_hint_for_rendered_clap_error(rendered: &str) -> String {
    let usage_line = rendered
        .lines()
        .map(str::trim)
        .find(|line| line.starts_with("Usage:"));

    let Some(usage_line) = usage_line else {
        return "See 'wavepeek --help'.".to_string();
    };

    let usage = usage_line.trim_start_matches("Usage:").trim();
    let mut parts = usage.split_whitespace();
    let Some(command_name) = parts.next() else {
        return "See 'wavepeek --help'.".to_string();
    };
    if command_name != "wavepeek" {
        return "See 'wavepeek --help'.".to_string();
    }

    let Some(next_token) = parts.next() else {
        return "See 'wavepeek --help'.".to_string();
    };

    if next_token.starts_with('[') || next_token.starts_with('<') || next_token.starts_with('-') {
        return "See 'wavepeek --help'.".to_string();
    }

    format!("See 'wavepeek {next_token} --help'.")
}

fn dispatch(command: Command) -> Result<(), WavepeekError> {
    let engine_command = into_engine_command(command);
    let result = engine::run(engine_command)?;

    output::write(result)
}

fn into_engine_command(command: Command) -> EngineCommand {
    match command {
        Command::Schema(args) => EngineCommand::Schema(args),
        Command::Info(args) => EngineCommand::Info(args),
        Command::Scope(args) => EngineCommand::Scope(args),
        Command::Signal(args) => EngineCommand::Signal(args),
        Command::At(args) => EngineCommand::At(args),
        Command::Change(args) => EngineCommand::Change(args),
        Command::When(args) => EngineCommand::When(args),
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use clap::Parser;

    use crate::cli::limits::LimitArg;

    use super::{
        Cli, EngineCommand, clap_error_detail, help_hint_for_rendered_clap_error,
        into_engine_command, normalize_clap_error,
    };

    #[test]
    fn info_dispatch_keeps_json_and_waves_args() {
        let cli = Cli::parse_from([
            "wavepeek",
            "info",
            "--waves",
            "fixtures/sample.vcd",
            "--json",
        ]);

        let command = into_engine_command(cli.command);
        match command {
            EngineCommand::Info(args) => {
                assert_eq!(args.waves, PathBuf::from("fixtures/sample.vcd"));
                assert!(args.json);
            }
            other => panic!("expected info command, got {other:?}"),
        }
    }

    #[test]
    fn scope_dispatch_keeps_bounded_query_args() {
        let cli = Cli::parse_from([
            "wavepeek",
            "scope",
            "--waves",
            "fixtures/sample.vcd",
            "--max",
            "12",
            "--max-depth",
            "3",
            "--filter",
            "^top\\..*",
            "--tree",
            "--json",
        ]);

        let command = into_engine_command(cli.command);
        match command {
            EngineCommand::Scope(args) => {
                assert_eq!(args.waves, PathBuf::from("fixtures/sample.vcd"));
                assert_eq!(args.max, LimitArg::Numeric(12));
                assert_eq!(args.max_depth, LimitArg::Numeric(3));
                assert_eq!(args.filter, "^top\\..*");
                assert!(args.tree);
                assert!(args.json);
            }
            other => panic!("expected scope command, got {other:?}"),
        }
    }

    #[test]
    fn scope_dispatch_accepts_unlimited_limit_literals() {
        let cli = Cli::parse_from([
            "wavepeek",
            "scope",
            "--waves",
            "fixtures/sample.vcd",
            "--max",
            "unlimited",
            "--max-depth",
            "unlimited",
        ]);

        let command = into_engine_command(cli.command);
        match command {
            EngineCommand::Scope(args) => {
                assert_eq!(args.waves, PathBuf::from("fixtures/sample.vcd"));
                assert_eq!(args.max, LimitArg::Unlimited);
                assert_eq!(args.max_depth, LimitArg::Unlimited);
            }
            other => panic!("expected scope command, got {other:?}"),
        }
    }

    #[test]
    fn signal_dispatch_keeps_recursive_and_max_depth_args() {
        let cli = Cli::parse_from([
            "wavepeek",
            "signal",
            "--waves",
            "fixtures/sample.vcd",
            "--scope",
            "top.cpu",
            "--recursive",
            "--max-depth",
            "3",
            "--max",
            "7",
            "--filter",
            ".*clk.*",
            "--abs",
            "--json",
        ]);

        let command = into_engine_command(cli.command);
        match command {
            EngineCommand::Signal(args) => {
                assert_eq!(args.waves, PathBuf::from("fixtures/sample.vcd"));
                assert_eq!(args.scope, "top.cpu");
                assert!(args.recursive);
                assert_eq!(args.max_depth, Some(LimitArg::Numeric(3)));
                assert_eq!(args.max, LimitArg::Numeric(7));
                assert_eq!(args.filter, ".*clk.*");
                assert!(args.abs);
                assert!(args.json);
            }
            other => panic!("expected signal command, got {other:?}"),
        }
    }

    #[test]
    fn at_dispatch_keeps_scope_signals_abs_and_json_args() {
        let cli = Cli::parse_from([
            "wavepeek",
            "at",
            "--waves",
            "fixtures/sample.vcd",
            "--time",
            "10ns",
            "--scope",
            "top",
            "--signals",
            "clk,data",
            "--abs",
            "--json",
        ]);

        let command = into_engine_command(cli.command);
        match command {
            EngineCommand::At(args) => {
                assert_eq!(args.waves, PathBuf::from("fixtures/sample.vcd"));
                assert_eq!(args.time, "10ns");
                assert_eq!(args.scope.as_deref(), Some("top"));
                assert_eq!(args.signals, vec!["clk", "data"]);
                assert!(args.abs);
                assert!(args.json);
            }
            other => panic!("expected at command, got {other:?}"),
        }
    }

    #[test]
    fn change_dispatch_keeps_when_abs_and_limits() {
        let cli = Cli::parse_from([
            "wavepeek",
            "change",
            "--waves",
            "fixtures/sample.vcd",
            "--from",
            "1ns",
            "--to",
            "10ns",
            "--scope",
            "top",
            "--signals",
            "clk,data",
            "--when",
            "posedge clk",
            "--max",
            "12",
            "--abs",
            "--json",
        ]);

        let command = into_engine_command(cli.command);
        match command {
            EngineCommand::Change(args) => {
                assert_eq!(args.waves, PathBuf::from("fixtures/sample.vcd"));
                assert_eq!(args.from.as_deref(), Some("1ns"));
                assert_eq!(args.to.as_deref(), Some("10ns"));
                assert_eq!(args.scope.as_deref(), Some("top"));
                assert_eq!(args.signals, vec!["clk", "data"]);
                assert_eq!(args.when.as_deref(), Some("posedge clk"));
                assert_eq!(args.max, LimitArg::Numeric(12));
                assert!(args.abs);
                assert!(args.json);
            }
            other => panic!("expected change command, got {other:?}"),
        }
    }

    #[test]
    fn when_dispatch_parses_unlimited_max_literal() {
        let cli = Cli::parse_from([
            "wavepeek",
            "when",
            "--waves",
            "fixtures/sample.vcd",
            "--clk",
            "top.clk",
            "--cond",
            "1",
            "--max",
            "unlimited",
        ]);

        let command = into_engine_command(cli.command);
        match command {
            EngineCommand::When(args) => {
                assert_eq!(args.max, Some(LimitArg::Unlimited));
                assert_eq!(args.clk, "top.clk");
                assert_eq!(args.cond, "1");
            }
            other => panic!("expected when command, got {other:?}"),
        }
    }

    #[test]
    fn clap_errors_are_normalized_to_single_line_message() {
        let error = Cli::try_parse_from(["wavepeek", "schema", "--waves", "dump.vcd"])
            .expect_err("schema --waves should fail");

        let normalized = normalize_clap_error(&error);
        assert!(normalized.contains("unexpected argument '--waves'"));
        assert!(normalized.contains("See 'wavepeek schema --help'."));
        assert!(!normalized.contains("Usage:"));
    }

    #[test]
    fn clap_error_detail_preserves_missing_argument_names() {
        let rendered = "error: the following required arguments were not provided:\n  --waves <FILE>\n\nUsage: wavepeek info --waves <FILE>\n\nFor more information, try '--help'.\n";

        let normalized = clap_error_detail(rendered);
        assert!(normalized.contains("the following required arguments were not provided"));
        assert!(normalized.contains("--waves <FILE>"));
        assert!(!normalized.contains("Usage:"));
    }

    #[test]
    fn help_hint_uses_global_help_for_top_level_parse_failures() {
        let rendered = "error: unexpected argument '--wat' found\n\nUsage: wavepeek [OPTIONS] <COMMAND>\n\nFor more information, try '--help'.\n";
        let hint = help_hint_for_rendered_clap_error(rendered);
        assert_eq!(hint, "See 'wavepeek --help'.");
    }

    #[test]
    fn help_hint_uses_subcommand_help_for_subcommand_parse_failures() {
        let rendered = "error: unexpected argument '--wat' found\n\nUsage: wavepeek info --waves <FILE>\n\nFor more information, try '--help'.\n";
        let hint = help_hint_for_rendered_clap_error(rendered);
        assert_eq!(hint, "See 'wavepeek info --help'.");
    }
}
