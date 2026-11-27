use std::path::PathBuf;

use crate::cli::path_style::PathOutputArgs;

use clap::Args;

#[derive(Debug, Args)]
pub struct ValueArgs {
    #[arg(long, value_name = "FILE", help_heading = "Input options", help = super::WAVES_HELP)]
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
        num_args = 1..,
        required = true,
        help_heading = "Selection options"
    )]
    pub signals: Vec<String>,
    /// Show canonical signal paths
    #[arg(long, help_heading = "Output options")]
    pub abs: bool,
    #[command(flatten)]
    pub(crate) path_output: PathOutputArgs,
    /// Machine-readable JSON output
    #[arg(long, help_heading = "Output options")]
    pub json: bool,
    /// Stream newline-delimited JSON output
    #[arg(long, conflicts_with = "json", help_heading = "Output options")]
    pub jsonl: bool,
}
