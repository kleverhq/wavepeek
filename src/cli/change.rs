use std::path::PathBuf;

use clap::{Args, ValueEnum};

use crate::cli::limits::LimitArg;
use crate::cli::sampling::SampleMode;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum, Default)]
#[value(rename_all = "kebab-case")]
pub enum RowMode {
    #[default]
    Dense,
    Sparse,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum, Default)]
#[value(rename_all = "kebab-case")]
pub enum RowValues {
    #[default]
    Full,
    Delta,
}

#[derive(Debug, Args)]
pub struct ChangeArgs {
    /// Path to a VCD, FST, or FSDB file, or an SHM database directory (for example, dump.fst)
    #[arg(long, value_name = "FILE", help_heading = "Input options")]
    pub waves: PathBuf,
    /// Start of the inclusive time range (for example, 1234ns; default: dump start)
    #[arg(long, help_heading = "Selection options")]
    pub from: Option<String>,
    /// End of the inclusive time range (for example, 2000ns; default: dump end)
    #[arg(long, help_heading = "Selection options")]
    pub to: Option<String>,
    /// Scope for relative signal and trigger names (for example, top.cpu)
    #[arg(long, help_heading = "Selection options")]
    pub scope: Option<String>,
    /// Signal paths or flat projections, comma-separated or repeated (for example, state,req or status[7:4])
    #[arg(
        long,
        value_delimiter = ',',
        num_args = 1..,
        required = true,
        help_heading = "Selection options"
    )]
    pub signals: Vec<String>,
    /// Event trigger expression (for example, 'posedge clk'; use `*` only with `--sample-mode native`)
    #[arg(long, required = true, help_heading = "Selection options")]
    pub on: String,
    /// Value sampling mode for event-selected rows
    #[arg(
        long,
        value_enum,
        default_value_t = SampleMode::PreEdge,
        value_name = "MODE",
        help_heading = "Selection options"
    )]
    pub sample_mode: SampleMode,
    /// Select whether every sampled event or only changed samples become rows
    #[arg(
        long,
        value_enum,
        default_value_t = RowMode::Dense,
        value_name = "MODE",
        help_heading = "Output options"
    )]
    pub row_mode: RowMode,
    /// Select whether rows contain all values or only changed values
    #[arg(
        long,
        value_enum,
        default_value_t = RowValues::Full,
        value_name = "VALUES",
        help_heading = "Output options"
    )]
    pub row_values: RowValues,
    /// Maximum number of snapshot rows (`unlimited` disables truncation, value must be > 0)
    #[arg(long, default_value = "50", help_heading = "Output options")]
    pub max: LimitArg,
    /// Print canonical paths
    #[arg(long, help_heading = "Output options")]
    pub abs: bool,
    /// Suppress result rows while retaining context and completeness metadata
    #[arg(long, help_heading = "Output options")]
    pub summary: bool,
    /// Machine-readable JSON output
    #[arg(long, help_heading = "Output options")]
    pub json: bool,
    /// Stream newline-delimited JSON output
    #[arg(long, conflicts_with = "json", help_heading = "Output options")]
    pub jsonl: bool,
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
