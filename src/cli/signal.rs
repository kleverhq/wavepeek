use std::path::PathBuf;

use clap::Args;

use crate::cli::limits::LimitArg;

#[derive(Debug, Args)]
pub struct SignalArgs {
    /// Path to VCD/FST/FSDB waveform file
    #[arg(long, value_name = "FILE", help_heading = "Input options")]
    pub waves: PathBuf,
    /// Exact scope path (e.g. top.cpu)
    #[arg(long, help_heading = "Input options")]
    pub scope: String,
    /// Maximum number of entries (`unlimited` disables truncation, value must be > 0)
    #[arg(long, default_value = "50", help_heading = "Output options")]
    pub max: LimitArg,
    /// Regex filter for signal name
    #[arg(long, default_value = ".*", help_heading = "Selection options")]
    pub filter: String,
    /// Recursively include nested child scopes
    #[arg(long, help_heading = "Selection options")]
    pub recursive: bool,
    /// Maximum recursion depth below --scope (`unlimited` disables this limit)
    #[arg(
        long,
        default_value = "5",
        requires = "recursive",
        help_heading = "Selection options"
    )]
    pub max_depth: LimitArg,
    /// Show canonical signal paths
    #[arg(long, help_heading = "Output options")]
    pub abs: bool,
    /// Machine-readable JSON output
    #[arg(long, help_heading = "Output options")]
    pub json: bool,
    /// Stream newline-delimited JSON output
    #[arg(long, conflicts_with = "json", help_heading = "Output options")]
    pub jsonl: bool,
}
