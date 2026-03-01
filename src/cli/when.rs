use std::path::PathBuf;

use clap::Args;

use crate::cli::limits::LimitArg;

#[derive(Debug, Args)]
pub struct WhenArgs {
    /// Path to VCD/FST waveform file (`--waves <FILE>` is required)
    #[arg(long, value_name = "FILE")]
    pub waves: PathBuf,
    /// Clock signal for posedge sampling (`--clk` is required)
    #[arg(long)]
    pub clk: String,
    /// Start of inclusive time range (explicit units required)
    #[arg(long)]
    pub from: Option<String>,
    /// End of inclusive time range (explicit units required)
    #[arg(long)]
    pub to: Option<String>,
    /// Scope for short signal and clock names
    #[arg(long)]
    pub scope: Option<String>,
    /// Boolean expression in the expression language (`--cond` is required)
    #[arg(long)]
    pub cond: String,
    /// Return first N matches (or 1 when value omitted)
    #[arg(
        long,
        value_name = "N",
        num_args = 0..=1,
        default_missing_value = "1",
        conflicts_with_all = ["last", "max"]
    )]
    pub first: Option<usize>,
    /// Return last N matches (or 1 when value omitted)
    #[arg(
        long,
        value_name = "N",
        num_args = 0..=1,
        default_missing_value = "1",
        conflicts_with_all = ["first", "max"]
    )]
    pub last: Option<usize>,
    /// Maximum number of matches when no qualifier is used (`unlimited` is accepted by parsing)
    #[arg(long, value_name = "LIMIT", conflicts_with_all = ["first", "last"])]
    pub max: Option<LimitArg>,
    /// Machine-readable JSON output (contract: see `wavepeek schema`; runtime remains unimplemented)
    #[arg(long)]
    pub json: bool,
}
