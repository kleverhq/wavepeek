use std::path::PathBuf;

use clap::Args;

#[derive(Debug, Args)]
pub struct ScopeArgs {
    /// Path to VCD/FST waveform file
    #[arg(long, value_name = "FILE")]
    pub waves: PathBuf,
    /// Maximum number of entries in output
    #[arg(long, default_value_t = 50)]
    pub max: usize,
    /// Maximum traversal depth
    #[arg(long, default_value_t = 5)]
    pub max_depth: usize,
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
