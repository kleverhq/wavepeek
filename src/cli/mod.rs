pub mod at;
pub mod changes;
pub mod info;
pub mod schema;
pub mod signals;
pub mod tree;
pub mod when;

use clap::{Parser, Subcommand};

use crate::engine::{self, Command as EngineCommand};
use crate::error::WavepeekError;

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
        long_about = "Outputs the JSON schema for wavepeek's default JSON output. By default this command returns structured JSON output; use --human for a human-friendly summary."
    )]
    Schema(schema::SchemaArgs),
    #[command(
        about = "Show dump metadata (time unit, precision, bounds)",
        long_about = "Outputs basic metadata about the waveform dump, including time unit, time precision, and dump boundaries. Requires --waves <file>."
    )]
    Info(info::InfoArgs),
    #[command(
        about = "List hierarchy instances (deterministic DFS)",
        long_about = "Outputs a flat list of module instances by recursively traversing the hierarchy. Traversal is deterministic and output is bounded by --max and --max-depth."
    )]
    Tree(tree::TreeArgs),
    #[command(
        about = "List signals in scope with metadata",
        long_about = "Lists signals within a specific scope with signal metadata. Listing is non-recursive, sorted by signal name, and bounded by --max."
    )]
    Signals(signals::SignalsArgs),
    #[command(
        about = "Get signal values at a specific time point",
        long_about = "Gets signal values at a specific time point. Supports full signal paths or names relative to --scope while preserving the order from --signals."
    )]
    At(at::AtArgs),
    #[command(
        about = "Get value snapshots over a time range",
        long_about = "Outputs snapshots of signal values over a time range. Supports unclocked mode (any tracked signal change) and clocked mode (posedge of --clk)."
    )]
    Changes(changes::ChangesArgs),
    #[command(
        about = "Find cycles where a condition is true",
        long_about = "Finds clock cycles where a boolean expression evaluates to true. The condition is evaluated on each posedge of --clk and can return all, first N, or last N matches."
    )]
    When(when::WhenArgs),
}

pub fn run() -> Result<(), WavepeekError> {
    let cli = Cli::parse();
    dispatch(cli.command)
}

fn dispatch(command: Command) -> Result<(), WavepeekError> {
    let engine_command = into_engine_command(command);

    engine::run(engine_command)
}

fn into_engine_command(command: Command) -> EngineCommand {
    match command {
        Command::Schema(args) => EngineCommand::Schema(args),
        Command::Info(args) => EngineCommand::Info(args),
        Command::Tree(args) => EngineCommand::Tree(args),
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

    use super::{Cli, EngineCommand, into_engine_command};

    #[test]
    fn info_dispatch_keeps_human_and_waves_args() {
        let cli = Cli::parse_from([
            "wavepeek",
            "info",
            "--waves",
            "fixtures/sample.vcd",
            "--human",
        ]);

        let command = into_engine_command(cli.command);
        match command {
            EngineCommand::Info(args) => {
                assert_eq!(args.waves, PathBuf::from("fixtures/sample.vcd"));
                assert!(args.human);
            }
            other => panic!("expected info command, got {other:?}"),
        }
    }

    #[test]
    fn tree_dispatch_keeps_bounded_query_args() {
        let cli = Cli::parse_from([
            "wavepeek",
            "tree",
            "--waves",
            "fixtures/sample.vcd",
            "--max",
            "12",
            "--max-depth",
            "3",
            "--filter",
            "^top\\..*",
            "--human",
        ]);

        let command = into_engine_command(cli.command);
        match command {
            EngineCommand::Tree(args) => {
                assert_eq!(args.waves, PathBuf::from("fixtures/sample.vcd"));
                assert_eq!(args.max, 12);
                assert_eq!(args.max_depth, 3);
                assert_eq!(args.filter, "^top\\..*");
                assert!(args.human);
            }
            other => panic!("expected tree command, got {other:?}"),
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
            "--human",
        ]);

        let command = into_engine_command(cli.command);
        match command {
            EngineCommand::Signals(args) => {
                assert_eq!(args.waves, PathBuf::from("fixtures/sample.vcd"));
                assert_eq!(args.scope, "top.cpu");
                assert_eq!(args.max, 7);
                assert_eq!(args.filter, ".*clk.*");
                assert!(args.human);
            }
            other => panic!("expected signals command, got {other:?}"),
        }
    }
}
