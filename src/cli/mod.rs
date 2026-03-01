pub mod at;
pub mod change;
pub mod info;
pub mod limits;
pub mod schema;
pub mod scope;
pub mod signal;
pub mod when;

use clap::error::ErrorKind;
use clap::{ArgAction, CommandFactory, FromArgMatches, Parser, Subcommand};

use crate::engine::{self, Command as EngineCommand};
use crate::error::WavepeekError;
use crate::output;

#[derive(Debug, Parser)]
#[command(
    name = "wavepeek",
    version,
    about = "wavepeek is a command-line tool for RTL waveform inspection.\nIt provides deterministic, machine-friendly output and a minimal set of primitives that compose into repeatable debug recipes.",
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
        long_about = r#"Reports dump metadata (`time_unit`, `time_start`, `time_end`).

Contract:
- --waves <FILE> is required.
- Default output is human-readable metadata; `--json` switches to strict envelope output.
- Errors use `error: <category>: <message>`.
- Argument mistakes use `error: args:` and append `See 'wavepeek info --help'.`.
- JSON mode returns one object envelope with metadata fields in `data`.

Use this command to confirm dump bounds before running scoped/time-based queries."#
    )]
    Info(info::InfoArgs),
    #[command(
        about = "List hierarchy scopes (deterministic DFS)",
        long_about = r#"Provides deterministic hierarchy traversal over scope paths.

Contract:
- `--waves <FILE>` is required.
- Defaults: --max=50, --max-depth=5, --filter=.*.
- Boundary rules: --max must be greater than 0; --max-depth bounds traversal depth.
- `--max unlimited` and `--max-depth unlimited` disable limits explicitly.
- Human output is list mode by default; `--tree` renders hierarchy; `--json` returns a JSON data array.
- Errors use `error: <category>: <message>`.
- Argument mistakes use `error: args:` and append `See 'wavepeek scope --help'.`.

Use this command to explore hierarchy shape before narrowing to signal-level queries."#
    )]
    Scope(scope::ScopeArgs),
    #[command(
        about = "List signals in scope with metadata",
        long_about = r#"Provides scope-local signal listing with deterministic ordering.

Contract:
- `--waves <FILE>` is required.
- --scope is required and identifies the anchor scope.
- Default mode is non-recursive with `--max=50` and `--filter=.*`.
- `--recursive` enables child traversal; `--max-depth requires --recursive`.
- `--max unlimited` and `--max-depth unlimited` disable limits explicitly.
- Human output uses short/relative names by default; `--abs` shows canonical paths.
- `--json` returns a JSON array of canonical signal objects (`name`, `path`, `kind`, `width`).
- Errors use `error: <category>: <message>`.
- Argument mistakes use `error: args:` and append `See 'wavepeek signal --help'.`.

Use this command after `scope` to inspect available signals in a target scope."#
    )]
    Signal(signal::SignalArgs),
    #[command(
        about = "Get signal values at a specific time point",
        long_about = r#"Provides point-in-time sampling for selected signals.

Contract:
- `--waves <FILE>`, `--time <TIME_WITH_UNIT>`, and `--signals` are required.
- --time requires explicit units aligned to dump precision.
- Name resolution: without `--scope`, `--signals` entries are canonical paths; with `--scope`, names are scope-relative.
- Output preserves --signals order.
- Human output prints `@<time>` and one `<display> <value>` line per signal; `--abs` switches to canonical display paths.
- `--json` returns a JSON object with `time` and ordered `signals` entries (`{path,value}`).
- Errors use `error: <category>: <message>`.
- Argument mistakes use `error: args:` and append `See 'wavepeek at --help'.`.

Use this command for deterministic spot checks at a specific timestamp."#
    )]
    At(at::AtArgs),
    #[command(
        about = "Get value snapshots over a time range",
        long_about = r#"Provides range-based delta snapshots for selected signals.

Contract:
- `--waves <FILE>` and `--signals` are required.
- Optional range is inclusive (`--from`/`--to`) with baseline initialization at range start.
- Default trigger is --when=* (--when defaults to *).
- Default row cap is `--max=50`; `--max unlimited` disables truncation.
- Rows are emitted only when sampled signal values changed at candidate timestamps.
- Human output prints `@<time> name=value` rows; `--abs` switches to canonical paths.
- `--json` returns JSON rows with `time` plus ordered `signals[{path,value}]`.
- Errors use `error: <category>: <message>`.
- Argument mistakes use `error: args:` and append `See 'wavepeek change --help'.`.

Use this command to inspect value transitions over bounded time windows."#
    )]
    Change(change::ChangeArgs),
    #[command(
        about = "Find cycles where a condition is true (not implemented yet)",
        long_about = r#"Find cycles where a condition is true.

Contract:
- Command intent: evaluate `--cond` on each posedge of `--clk` and report matching timestamps.
- `--waves <FILE>`, `--clk`, and `--cond` are required.
- Qualifiers: --first, --last, --max (mutually constrained as documented by flags).
- --max unlimited is accepted by parsing when no qualifier is used.
- Default output mode is human-readable; `--json` would use envelope mode.
- Execution is not implemented yet.
- Errors use `error: <category>: <message>`.
- Argument mistakes use `error: args:` and append `See 'wavepeek when --help'.`.

Use this help as the parse/contract reference until runtime execution is implemented."#
    )]
    When(when::WhenArgs),
    #[command(
        about = "Print canonical JSON schema contract",
        long_about = r#"Prints the canonical JSON schema document for wavepeek machine output contracts.

Contract:
- This command accepts no command-specific flags or positional arguments.
- It prints exactly one schema document to stdout.
- It does not use waveform --json envelope wrapping.
- Errors still use `error: <category>: <message>`.
- For usage constraints, See 'wavepeek schema --help'.

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

    let cli = match Cli::from_arg_matches(&matches) {
        Ok(cli) => cli,
        Err(error) => return handle_parse_error(error),
    };

    dispatch(cli.command)
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
