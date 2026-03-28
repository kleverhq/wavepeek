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
    /// Path to VCD/FST waveform file (`--waves <FILE>` is required)
    #[arg(long, value_name = "FILE")]
    pub waves: PathBuf,
    /// Start of inclusive time range (explicit units required)
    #[arg(long)]
    pub from: Option<String>,
    /// End of inclusive time range (explicit units required)
    #[arg(long)]
    pub to: Option<String>,
    /// Scope for short signal and event names
    #[arg(long)]
    pub scope: Option<String>,
    /// Event trigger expression (default: `*` when omitted)
    #[arg(long)]
    pub on: Option<String>,
    /// Logical expression evaluated at event timestamps (`--eval` is required)
    #[arg(long)]
    pub eval: String,
    /// Capture mode (`match`, `switch`, `assert`, `deassert`)
    #[arg(long, value_enum, default_value_t = CaptureMode::Switch, value_name = "MODE")]
    pub capture: CaptureMode,
    /// Machine-readable JSON output (contract: see `wavepeek schema`)
    #[arg(long)]
    pub json: bool,
}
