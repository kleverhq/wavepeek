use std::path::PathBuf;

use clap::Args;

#[derive(Debug, Args)]
pub struct WhenArgs {
    /// Path to VCD/FST waveform file
    #[arg(long, value_name = "FILE")]
    pub waves: PathBuf,
    /// Clock signal for posedge sampling
    #[arg(long)]
    pub clk: String,
    /// Start of time range (inclusive)
    #[arg(long)]
    pub from: Option<String>,
    /// End of time range (inclusive)
    #[arg(long)]
    pub to: Option<String>,
    /// Scope for short signal and clock names
    #[arg(long)]
    pub scope: Option<String>,
    /// Boolean expression in the expression language
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
    /// Maximum number of matches when no qualifier is used
    #[arg(long, value_name = "N", conflicts_with_all = ["first", "last"])]
    pub max: Option<usize>,
    /// Strict JSON envelope output
    #[arg(long)]
    pub json: bool,
}
