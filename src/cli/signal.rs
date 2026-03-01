use std::path::PathBuf;

use clap::Args;

use crate::cli::limits::LimitArg;

#[derive(Debug, Args)]
pub struct SignalArgs {
    /// Path to VCD/FST waveform file (`--waves <FILE>` is required)
    #[arg(long, value_name = "FILE")]
    pub waves: PathBuf,
    /// Exact scope path (`--scope` is required; for example, top.cpu)
    #[arg(long)]
    pub scope: String,
    /// Maximum number of entries (default: 50, `unlimited` disables truncation, value must be > 0)
    #[arg(long, default_value = "50")]
    pub max: LimitArg,
    /// Regex filter for signal name (default: `.*`; invalid regex is rejected as an argument error)
    #[arg(long, default_value = ".*")]
    pub filter: String,
    /// Recursively include nested child scopes
    #[arg(long)]
    pub recursive: bool,
    /// Maximum recursion depth below --scope (requires --recursive, `unlimited` disables this limit)
    #[arg(long)]
    pub max_depth: Option<LimitArg>,
    /// Show canonical signal paths in human output
    #[arg(long)]
    pub abs: bool,
    /// Machine-readable JSON output (contract: see `wavepeek schema`)
    #[arg(long)]
    pub json: bool,
}
