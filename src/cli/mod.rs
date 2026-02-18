pub mod at;
pub mod changes;
pub mod info;
pub mod modules;
pub mod schema;
pub mod signals;
pub mod when;

use clap::error::ErrorKind;
use clap::{CommandFactory, Parser, Subcommand};

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
        about = "Export JSON schema for default output",
        long_about = r#"Outputs the JSON schema for wavepeek's default JSON output.

By default this command returns structured JSON output; use --human for
a human-friendly summary."#
    )]
    Schema(schema::SchemaArgs),
    #[command(
        about = "Show dump metadata (time unit and bounds)",
        long_about = r#"Outputs basic metadata about the waveform dump, including
time unit and dump boundaries.

By default this command prints human-readable output.

Requires --waves <file>. Use --json for strict envelope mode."#
    )]
    Info(info::InfoArgs),
    #[command(
        about = "List hierarchy instances (deterministic DFS)",
        long_about = r#"Outputs a flat list of module instances by recursively
traversing the hierarchy.

Traversal is deterministic and output is bounded by --max and --max-depth.

Use --tree for visual hierarchy rendering in human mode. Use --json for
strict envelope mode."#
    )]
    Modules(modules::ModulesArgs),
    #[command(
        about = "List signals in scope with metadata",
        long_about = r#"Lists signals within a specific scope with signal metadata.

Listing is non-recursive, sorted by signal name, and bounded by --max.

Human mode prints short names by default; use --abs for full paths.
Use --json for strict envelope mode."#
    )]
    Signals(signals::SignalsArgs),
    #[command(
        about = "Get signal values at a specific time point",
        long_about = r#"Gets signal values at a specific time point.

Supports full signal paths or names relative to --scope while preserving
the order from --signals."#
    )]
    At(at::AtArgs),
    #[command(
        about = "Get value snapshots over a time range",
        long_about = r#"Outputs snapshots of signal values over a time range.

Supports unclocked mode (any tracked signal change) and clocked mode
(posedge of --clk)."#
    )]
    Changes(changes::ChangesArgs),
    #[command(
        about = "Find cycles where a condition is true",
        long_about = r#"Finds clock cycles where a boolean expression evaluates to true.

The condition is evaluated on each posedge of --clk and can return all,
first N, or last N matches."#
    )]
    When(when::WhenArgs),
}

pub fn run() -> Result<(), WavepeekError> {
    if std::env::args_os().len() == 1 {
        return print_top_level_help();
    }

    let cli = match Cli::try_parse() {
        Ok(cli) => cli,
        Err(error) => return handle_parse_error(error),
    };

    dispatch(cli.command)
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

fn print_top_level_help() -> Result<(), WavepeekError> {
    let mut command = Cli::command();
    command.print_help().map_err(|io_error| {
        WavepeekError::Internal(format!("failed to print top-level help: {io_error}"))
    })?;
    println!();
    Ok(())
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
        Command::Modules(args) => EngineCommand::Modules(args),
        Command::Signals(args) => EngineCommand::Signals(args),
        Command::At(args) => EngineCommand::At(args),
        Command::Changes(args) => EngineCommand::Changes(args),
        Command::When(args) => EngineCommand::When(args),
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use clap::Parser;

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
    fn modules_dispatch_keeps_bounded_query_args() {
        let cli = Cli::parse_from([
            "wavepeek",
            "modules",
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
            EngineCommand::Modules(args) => {
                assert_eq!(args.waves, PathBuf::from("fixtures/sample.vcd"));
                assert_eq!(args.max, 12);
                assert_eq!(args.max_depth, 3);
                assert_eq!(args.filter, "^top\\..*");
                assert!(args.tree);
                assert!(args.json);
            }
            other => panic!("expected modules command, got {other:?}"),
        }
    }

    #[test]
    fn signals_dispatch_keeps_scope_filter_and_max_args() {
        let cli = Cli::parse_from([
            "wavepeek",
            "signals",
            "--waves",
            "fixtures/sample.vcd",
            "--scope",
            "top.cpu",
            "--max",
            "7",
            "--filter",
            ".*clk.*",
            "--abs",
            "--json",
        ]);

        let command = into_engine_command(cli.command);
        match command {
            EngineCommand::Signals(args) => {
                assert_eq!(args.waves, PathBuf::from("fixtures/sample.vcd"));
                assert_eq!(args.scope, "top.cpu");
                assert_eq!(args.max, 7);
                assert_eq!(args.filter, ".*clk.*");
                assert!(args.abs);
                assert!(args.json);
            }
            other => panic!("expected signals command, got {other:?}"),
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
