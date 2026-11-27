use std::path::PathBuf;

use clap::Args;

#[derive(Debug, Args)]
pub struct InfoArgs {
    #[arg(long, value_name = "FILE", help_heading = "Input options", help = super::WAVES_HELP)]
    pub waves: PathBuf,
    /// Machine-readable JSON output
    #[arg(long, help_heading = "Output options")]
    pub json: bool,
    /// Stream newline-delimited JSON output
    #[arg(long, conflicts_with = "json", help_heading = "Output options")]
    pub jsonl: bool,
}
