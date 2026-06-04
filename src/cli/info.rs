use std::path::PathBuf;

use clap::Args;

#[derive(Debug, Args)]
pub struct InfoArgs {
    /// Path to VCD/FST/FSDB waveform file
    #[arg(long, value_name = "FILE", help_heading = "Input options")]
    pub waves: PathBuf,
    /// Machine-readable JSON output
    #[arg(long, help_heading = "Output options")]
    pub json: bool,
}
