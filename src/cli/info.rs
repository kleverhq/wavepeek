use std::path::PathBuf;

use clap::Args;

#[derive(Debug, Args)]
pub struct InfoArgs {
    /// Path to VCD/FST waveform file (`--waves <FILE>` is required)
    #[arg(long, value_name = "FILE")]
    pub waves: PathBuf,
    /// Machine-readable JSON output (contract: see `wavepeek schema`)
    #[arg(long)]
    pub json: bool,
}
