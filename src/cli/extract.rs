use std::path::PathBuf;

use clap::{Args, Subcommand};

use crate::cli::limits::LimitArg;

#[derive(Debug, Subcommand)]
pub enum ExtractCommand {
    #[command(
        about = "Extract protocol-neutral event rows from waveform signals.",
        long_about = r#"Extract protocol-neutral event rows from waveform signals.

Behavior:
- Selects edge-only event timestamps with --on.
- Always samples --when and --payload at the pre-edge sample point.
- In single-source mode, --on, --when, and --payload define one source named by --name or "transfer".
- In source-file mode, --source provides one or more sources and conflicts with --name, --on, --when, and --payload.
- JSON and JSONL rows include time, sample_time, source, and ordered payload values.
- Empty-result, truncation, and explicitly disabled-limit conditions emit coded diagnostics.

Use this command to extract synchronous handshakes or transfer-like rows without joining property and value output outside wavepeek."#,
        after_long_help = "See also:\n  wavepeek docs show commands/extract-generic"
    )]
    Generic(GenericArgs),
}

#[derive(Debug, Args)]
pub struct GenericArgs {
    /// Path to VCD/FST/FSDB waveform file
    #[arg(long, value_name = "FILE", help_heading = "Input options")]
    pub waves: PathBuf,
    /// JSON source file for multi-source extraction
    #[arg(
        long,
        value_name = "FILE",
        conflicts_with_all = ["name", "on", "when", "payload"],
        help_heading = "Input options"
    )]
    pub source: Option<PathBuf>,
    /// Start of inclusive event time range (e.g. 1234ns; omitted means dump start)
    #[arg(long, help_heading = "Selection options")]
    pub from: Option<String>,
    /// End of inclusive event time range (e.g. 1234ns; omitted means dump end)
    #[arg(long, help_heading = "Selection options")]
    pub to: Option<String>,
    /// Canonical scope path for scope-relative event, predicate, and payload names
    #[arg(long, help_heading = "Selection options")]
    pub scope: Option<String>,
    /// Source name for single-source CLI mode (defaults to transfer)
    #[arg(long, help_heading = "Selection options")]
    pub name: Option<String>,
    /// Edge-only event trigger expression for single-source CLI mode
    #[arg(long, help_heading = "Selection options")]
    pub on: Option<String>,
    /// Logical predicate evaluated at the pre-edge sample point in single-source CLI mode
    #[arg(long, help_heading = "Selection options")]
    pub when: Option<String>,
    /// Comma-separated payload signal names for single-source CLI mode
    #[arg(
        long,
        value_delimiter = ',',
        num_args = 1..,
        value_name = "SIGNAL[,SIGNAL...]",
        help_heading = "Selection options"
    )]
    pub payload: Option<Vec<String>>,
    /// Maximum number of extracted rows across all sources (`unlimited` disables truncation, value must be > 0)
    #[arg(long, default_value = "50", help_heading = "Output options")]
    pub max: LimitArg,
    /// Print canonical payload paths in human output
    #[arg(long, help_heading = "Output options")]
    pub abs: bool,
    /// Machine-readable JSON output
    #[arg(long, help_heading = "Output options")]
    pub json: bool,
    /// Stream newline-delimited JSON output
    #[arg(long, conflicts_with = "json", help_heading = "Output options")]
    pub jsonl: bool,
}
