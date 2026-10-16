use std::path::PathBuf;

use clap::Args;

#[derive(Debug, Args)]
pub struct ValueArgs {
    /// Path to a VCD, FST, or FSDB file, or an SHM database directory
    #[arg(long, value_name = "FILE", help_heading = "Input options")]
    pub waves: PathBuf,
    /// Time points with explicit units, comma-separated or repeated (for example, 1337ns or 10ns,20ns)
    #[arg(
        long,
        value_delimiter = ',',
        required = true,
        help_heading = "Selection options"
    )]
    pub at: Vec<String>,
    /// Scope for relative signal names (for example, top.cpu)
    #[arg(long, help_heading = "Selection options")]
    pub scope: Option<String>,
    /// Signal paths or flat projections, comma-separated or repeated (for example, state,pc or status[7:4])
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
    /// Stream newline-delimited JSON output
    #[arg(long, conflicts_with = "json", help_heading = "Output options")]
    pub jsonl: bool,
}
