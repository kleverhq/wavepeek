use std::path::PathBuf;

use clap::Args;

use crate::cli::limits::LimitArg;

#[derive(Debug, Args)]
pub struct ScopeArgs {
    /// Path to VCD/FST waveform file
    #[arg(long, value_name = "FILE")]
    pub waves: PathBuf,
    /// Maximum number of entries (`unlimited` disables truncation, value must be > 0)
    #[arg(long, default_value = "50")]
    pub max: LimitArg,
    /// Maximum traversal depth (`unlimited` disables depth truncation)
    #[arg(long, default_value = "5")]
    pub max_depth: LimitArg,
    /// Regex filter for full scope path
    #[arg(long, default_value = ".*")]
    pub filter: String,
    /// Render hierarchy as an indented tree
    #[arg(long)]
    pub tree: bool,
    /// Machine-readable JSON output
    #[arg(long)]
    pub json: bool,
}
