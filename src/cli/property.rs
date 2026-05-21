use std::path::PathBuf;

use clap::{Args, ValueEnum};

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum, Default)]
#[value(rename_all = "kebab-case")]
pub enum CaptureMode {
    Match,
    #[default]
    Switch,
    Assert,
    Deassert,
}

#[derive(Debug, Args)]
pub struct PropertyArgs {
    /// Path to waveform file; default builds support VCD/FST and report a feature-required error for FSDB
    #[arg(long, value_name = "FILE", help_heading = "Input options")]
    pub waves: PathBuf,
    /// Start of inclusive time range (e.g. 1234ns; omitted means dump start)
    #[arg(long, help_heading = "Selection options")]
    pub from: Option<String>,
    /// End of inclusive time range (e.g. 1234ns; omitted means dump end)
    #[arg(long, help_heading = "Selection options")]
    pub to: Option<String>,
    /// Canonical scope path for scope-relative signal and event names
    #[arg(long, help_heading = "Selection options")]
    pub scope: Option<String>,
    /// Event trigger expression (default: `*` when omitted)
    #[arg(long, help_heading = "Selection options")]
    pub on: Option<String>,
    /// Logical expression evaluated at selected event timestamps
    #[arg(long, help_heading = "Selection options")]
    pub eval: String,
    /// Capture mode: level (`match`) or edge (`switch`, `assert`, `deassert`)
    #[arg(
        long,
        value_enum,
        default_value_t = CaptureMode::Switch,
        value_name = "MODE",
        help_heading = "Output options"
    )]
    pub capture: CaptureMode,
    /// Machine-readable JSON output
    #[arg(long, help_heading = "Output options")]
    pub json: bool,
}
