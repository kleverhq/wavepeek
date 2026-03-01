use std::path::PathBuf;

use clap::Args;

use crate::cli::limits::LimitArg;

#[derive(Debug, Args)]
pub struct ScopeArgs {
    /// Path to VCD/FST waveform file (`--waves <FILE>` is required)
    #[arg(long, value_name = "FILE")]
    pub waves: PathBuf,
    /// Maximum number of entries (default: 50, `unlimited` disables truncation, value must be > 0)
    #[arg(long, default_value = "50")]
    pub max: LimitArg,
    /// Maximum traversal depth (default: 5, `unlimited` disables depth truncation)
    #[arg(long, default_value = "5")]
    pub max_depth: LimitArg,
    /// Regex filter for full scope path (default: `.*`; invalid regex is `error: args:`)
    #[arg(long, default_value = ".*")]
    pub filter: String,
    /// Render hierarchy as an indented tree in human output
    #[arg(long)]
    pub tree: bool,
    /// Machine-readable JSON output (contract: see `wavepeek schema`)
    #[arg(long)]
    pub json: bool,
}
