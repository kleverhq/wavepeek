use std::path::PathBuf;

use clap::Args;

use crate::cli::limits::LimitArg;

#[derive(Debug, Args)]
pub struct ScopeArgs {
    /// Path to a VCD, FST, or FSDB file, or an SHM database directory (for example, dump.fst)
    #[arg(long, value_name = "FILE", help_heading = "Input options")]
    pub waves: PathBuf,
    /// Maximum traversal depth (`unlimited` disables depth truncation)
    #[arg(long, default_value = "5", help_heading = "Selection options")]
    pub max_depth: LimitArg,
    /// Regex filter for full scope path
    #[arg(long, default_value = ".*", help_heading = "Selection options")]
    pub filter: String,
    /// Maximum number of matching scopes (tree ancestors do not count; `unlimited` disables truncation)
    #[arg(long, default_value = "50", help_heading = "Output options")]
    pub max: LimitArg,
    /// Render hierarchy as an indented tree
    #[arg(long, help_heading = "Output options")]
    pub tree: bool,
    /// Suppress result rows while retaining context and completeness metadata
    #[arg(long, help_heading = "Output options")]
    pub summary: bool,
    /// Machine-readable JSON output
    #[arg(long, help_heading = "Output options")]
    pub json: bool,
    /// Stream newline-delimited JSON output
    #[arg(long, conflicts_with = "json", help_heading = "Output options")]
    pub jsonl: bool,
}
