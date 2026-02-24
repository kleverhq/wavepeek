use std::path::PathBuf;

use clap::Args;

#[derive(Debug, Args)]
pub struct ChangeArgs {
    /// Path to VCD/FST waveform file
    #[arg(long, value_name = "FILE")]
    pub waves: PathBuf,
    /// Start of time range (inclusive)
    #[arg(long)]
    pub from: Option<String>,
    /// End of time range (inclusive)
    #[arg(long)]
    pub to: Option<String>,
    /// Scope for short signal and trigger names
    #[arg(long)]
    pub scope: Option<String>,
    /// Comma-separated signal names
    #[arg(long, value_delimiter = ',', num_args = 1.., required = true)]
    pub signals: Vec<String>,
    /// Event trigger expression (defaults to *)
    #[arg(long)]
    pub when: Option<String>,
    /// Maximum number of snapshot rows
    #[arg(long, default_value_t = 50)]
    pub max: usize,
    /// Print canonical paths in human output
    #[arg(long)]
    pub abs: bool,
    /// Strict JSON envelope output
    #[arg(long)]
    pub json: bool,
}
