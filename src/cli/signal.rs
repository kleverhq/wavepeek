use std::path::PathBuf;

use clap::Args;

#[derive(Debug, Args)]
pub struct SignalArgs {
    /// Path to VCD/FST waveform file
    #[arg(long, value_name = "FILE")]
    pub waves: PathBuf,
    /// Exact scope path (for example, top.cpu)
    #[arg(long)]
    pub scope: String,
    /// Maximum number of entries in output
    #[arg(long, default_value_t = 50)]
    pub max: usize,
    /// Regex filter for signal name
    #[arg(long, default_value = ".*")]
    pub filter: String,
    /// Show absolute signal paths
    #[arg(long)]
    pub abs: bool,
    /// Strict JSON envelope output
    #[arg(long)]
    pub json: bool,
}
