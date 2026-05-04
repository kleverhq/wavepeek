use std::path::PathBuf;

use clap::Args;

use crate::cli::limits::LimitArg;

#[derive(Debug, Args)]
pub struct SignalArgs {
    /// Path to VCD/FST waveform file
    #[arg(long, value_name = "FILE")]
    pub waves: PathBuf,
    /// Exact scope path (e.g. top.cpu)
    #[arg(long)]
    pub scope: String,
    /// Maximum number of entries (`unlimited` disables truncation, value must be > 0)
    #[arg(long, default_value = "50")]
    pub max: LimitArg,
    /// Regex filter for signal name
    #[arg(long, default_value = ".*")]
    pub filter: String,
    /// Recursively include nested child scopes
    #[arg(long)]
    pub recursive: bool,
    /// Maximum recursion depth below --scope (`unlimited` disables this limit)
    #[arg(long, default_value = "5", requires = "recursive")]
    pub max_depth: LimitArg,
    /// Show canonical signal paths
    #[arg(long)]
    pub abs: bool,
    /// Machine-readable JSON output
    #[arg(long)]
    pub json: bool,
}
