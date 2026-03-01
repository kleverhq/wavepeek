use std::path::PathBuf;

use clap::Args;

#[derive(Debug, Args)]
pub struct InfoArgs {
    /// Path to VCD/FST waveform file (`--waves <FILE>` is required)
    #[arg(long, value_name = "FILE")]
    pub waves: PathBuf,
    /// Strict JSON envelope output (`data` is an object with `time_unit`, `time_start`, `time_end`)
    #[arg(long)]
    pub json: bool,
}
