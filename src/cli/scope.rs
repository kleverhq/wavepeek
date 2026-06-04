use std::path::PathBuf;

use clap::Args;

use crate::cli::limits::LimitArg;

#[derive(Debug, Args)]
pub struct ScopeArgs {
    /// Path to VCD/FST/FSDB waveform file
    #[arg(long, value_name = "FILE", help_heading = "Input options")]
    pub waves: PathBuf,
    /// Maximum traversal depth (`unlimited` disables depth truncation)
    #[arg(long, default_value = "5", help_heading = "Selection options")]
    pub max_depth: LimitArg,
    /// Regex filter for full scope path
    #[arg(long, default_value = ".*", help_heading = "Selection options")]
    pub filter: String,
    /// Maximum number of entries (`unlimited` disables truncation, value must be > 0)
    #[arg(long, default_value = "50", help_heading = "Output options")]
    pub max: LimitArg,
    /// Render hierarchy as an indented tree
    #[arg(long, help_heading = "Output options")]
    pub tree: bool,
    /// Machine-readable JSON output
    #[arg(long, help_heading = "Output options")]
    pub json: bool,
}
