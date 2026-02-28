use std::path::PathBuf;

use clap::Args;

use crate::cli::limits::LimitArg;

#[derive(Debug, Args)]
pub struct SignalArgs {
    /// Path to VCD/FST waveform file
    #[arg(long, value_name = "FILE")]
    pub waves: PathBuf,
    /// Exact scope path (for example, top.cpu)
    #[arg(long)]
    pub scope: String,
    /// Maximum number of entries in output (`unlimited` disables this limit)
    #[arg(long, default_value = "50")]
    pub max: LimitArg,
    /// Regex filter for signal name
    #[arg(long, default_value = ".*")]
    pub filter: String,
    /// Recursively include nested child scopes
    #[arg(long)]
    pub recursive: bool,
    /// Maximum recursion depth below --scope (requires --recursive, `unlimited` disables this limit)
    #[arg(long)]
    pub max_depth: Option<LimitArg>,
    /// Show absolute signal paths
    #[arg(long)]
    pub abs: bool,
    /// Strict JSON envelope output
    #[arg(long)]
    pub json: bool,
}
