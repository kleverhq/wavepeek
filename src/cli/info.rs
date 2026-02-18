use std::path::PathBuf;

use clap::Args;

#[derive(Debug, Args)]
pub struct InfoArgs {
    /// Path to VCD/FST waveform file
    #[arg(long, value_name = "FILE")]
    pub waves: PathBuf,
    /// Strict JSON envelope output
    #[arg(long)]
    pub json: bool,
}
