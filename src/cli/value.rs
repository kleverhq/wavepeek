use std::path::PathBuf;

use clap::Args;

#[derive(Debug, Args)]
pub struct ValueArgs {
    /// Path to VCD/FST waveform file (`--waves <FILE>` is required)
    #[arg(long, value_name = "FILE")]
    pub waves: PathBuf,
    /// Time point with explicit units (for example, 1337ns); bare numbers are rejected as argument errors
    #[arg(long)]
    pub at: String,
    /// Scope for short signal names (without --scope, names in --signals must be canonical paths)
    #[arg(long)]
    pub scope: Option<String>,
    /// Comma-separated signal names; output order matches this input order
    #[arg(long, value_delimiter = ',', num_args = 1.., required = true)]
    pub signals: Vec<String>,
    /// Show canonical signal paths in human output
    #[arg(long)]
    pub abs: bool,
    /// Machine-readable JSON output (contract: see `wavepeek schema`)
    #[arg(long)]
    pub json: bool,
}
