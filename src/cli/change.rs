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
    /// Path to VCD/FST/FSDB waveform file
    #[arg(long, value_name = "FILE", help_heading = "Input options")]
    pub waves: PathBuf,
    /// Start of inclusive time range (e.g. 1234ns; omitted means dump start)
    #[arg(long, help_heading = "Selection options")]
    pub from: Option<String>,
    /// End of inclusive time range (e.g. 1234ns; omitted means dump end)
    #[arg(long, help_heading = "Selection options")]
    pub to: Option<String>,
    /// Canonical scope path for scope-relative signal and trigger names
    #[arg(long, help_heading = "Selection options")]
    pub scope: Option<String>,
    /// Comma-separated top-related signal paths, or scope-relative names when --scope is set
    #[arg(
        long,
        value_delimiter = ',',
        num_args = 1..,
        required = true,
        help_heading = "Selection options"
    )]
    pub signals: Vec<String>,
    /// Event trigger expression (default: `*` when omitted)
    #[arg(long, help_heading = "Selection options")]
    pub on: Option<String>,
    /// Maximum number of snapshot rows (`unlimited` disables truncation, value must be > 0)
    #[arg(long, default_value = "50", help_heading = "Output options")]
    pub max: LimitArg,
    /// Print canonical paths
    #[arg(long, help_heading = "Output options")]
    pub abs: bool,
    /// Machine-readable JSON output
    #[arg(long, help_heading = "Output options")]
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
