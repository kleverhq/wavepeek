use std::path::PathBuf;

use clap::Args;

#[derive(Debug, Args)]
pub struct ValueArgs {
    /// Path to waveform file; default builds support VCD/FST and report a feature-required error for FSDB
    #[arg(long, value_name = "FILE", help_heading = "Input options")]
    pub waves: PathBuf,
    /// Time point with explicit units (e.g. 1337ns)
    #[arg(long, help_heading = "Selection options")]
    pub at: String,
    /// Canonical scope path for scope-relative signal names
    #[arg(long, help_heading = "Selection options")]
    pub scope: Option<String>,
    /// Comma-separated top-related signal paths, or scope-relative names when --scope is set
    #[arg(
        long,
        value_delimiter = ',',
        num_args = 1..,
        required = true,
        help_heading = "Selection options"
    )]
    pub signals: Vec<String>,
    /// Show canonical signal paths
    #[arg(long, help_heading = "Output options")]
    pub abs: bool,
    /// Machine-readable JSON output
    #[arg(long, help_heading = "Output options")]
    pub json: bool,
}
