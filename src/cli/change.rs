use std::path::PathBuf;

use clap::{Args, ValueEnum};

use crate::cli::limits::LimitArg;

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum, Default)]
#[value(rename_all = "kebab-case")]
pub enum TuneChangeEngineMode {
    #[default]
    Auto,
    Baseline,
    Fused,
    EdgeFast,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum, Default)]
#[value(rename_all = "kebab-case")]
pub enum TuneChangeCandidateMode {
    #[default]
    Auto,
    Random,
    Stream,
}

#[derive(Debug, Args)]
pub struct ChangeArgs {
    /// Path to VCD/FST waveform file (`--waves <FILE>` is required)
    #[arg(long, value_name = "FILE")]
    pub waves: PathBuf,
    /// Start of inclusive time range (explicit units required)
    #[arg(long)]
    pub from: Option<String>,
    /// End of inclusive time range (explicit units required)
    #[arg(long)]
    pub to: Option<String>,
    /// Scope for short signal and trigger names
    #[arg(long)]
    pub scope: Option<String>,
    /// Comma-separated signal names (`--signals` is required)
    #[arg(long, value_delimiter = ',', num_args = 1.., required = true)]
    pub signals: Vec<String>,
    /// Event trigger expression (default: `*` when omitted)
    #[arg(long)]
    pub on: Option<String>,
    /// Maximum number of snapshot rows (default: 50, `unlimited` disables truncation, value must be > 0)
    #[arg(long, default_value = "50")]
    pub max: LimitArg,
    /// Print canonical paths in human output
    #[arg(long)]
    pub abs: bool,
    /// Machine-readable JSON output (contract: see `wavepeek schema`)
    #[arg(long)]
    pub json: bool,
    /// Unstable internal performance control (requires DEBUG=1).
    #[arg(
        long = "tune-engine",
        value_enum,
        default_value_t = TuneChangeEngineMode::Auto,
        hide = true
    )]
    pub tune_engine: TuneChangeEngineMode,
    /// Unstable internal performance control (requires DEBUG=1).
    #[arg(
        long = "tune-candidates",
        value_enum,
        default_value_t = TuneChangeCandidateMode::Auto,
        hide = true
    )]
    pub tune_candidates: TuneChangeCandidateMode,
    /// Unstable internal performance control (requires DEBUG=1).
    #[arg(long = "tune-edge-fast-force", hide = true)]
    pub tune_edge_fast_force: bool,
}
