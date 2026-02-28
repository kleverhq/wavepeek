use std::path::PathBuf;

use clap::Args;

use crate::cli::limits::LimitArg;

#[derive(Debug, Args)]
pub struct ScopeArgs {
    /// Path to VCD/FST waveform file
    #[arg(long, value_name = "FILE")]
    pub waves: PathBuf,
    /// Maximum number of entries in output (`unlimited` disables this limit)
    #[arg(long, default_value = "50")]
    pub max: LimitArg,
    /// Maximum traversal depth (`unlimited` disables this limit)
    #[arg(long, default_value = "5")]
    pub max_depth: LimitArg,
    /// Regex filter for scope path
    #[arg(long, default_value = ".*")]
    pub filter: String,
    /// Render hierarchy as a tree
    #[arg(long)]
    pub tree: bool,
    /// Strict JSON envelope output
    #[arg(long)]
    pub json: bool,
}
