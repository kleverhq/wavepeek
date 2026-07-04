use std::path::PathBuf;

use clap::{Args, Subcommand};

use crate::cli::limits::LimitArg;

#[derive(Debug, Subcommand)]
pub enum ExtractCommand {
    #[command(
        about = "Extract AXI ready/valid transfer rows.",
        long_about = r#"Extract AXI ready/valid transfer rows.

Behavior:
- Supports initial profiles: axi3, axi4, and axi4-lite.
- Defaults to profile axi4 unless --profile or --source selects another profile.
- Builds one extraction source per complete ready/valid channel.
- Samples reset, ready/valid predicates, and payload values at the pre-edge sample point.
- In source-file mode, --source provides profile, name, includes, and maps and conflicts with --profile, --name, --map, and --include.
- Contract for source-file mode is defined by `wavepeek schema --input`.
- JSON output includes AXI metadata, mappings, and transfer rows.
- Reports channel transfers only; it does not reconstruct bursts, ordering, or outstanding request state.

Use this command to inspect AXI-family handshakes without writing one generic source per channel."#,
        after_long_help = "See also:\n  wavepeek docs show commands/extract"
    )]
    Axi(Box<AxiArgs>),
    #[command(
        about = "Extract protocol-neutral event rows from waveform signals.",
        long_about = r#"Extract protocol-neutral event rows from waveform signals.

Behavior:
- Selects edge-only event timestamps with --on.
- Always samples --when and --payload at the pre-edge sample point.
- In single-source mode, --on, --when, and --payload define one source named by --name or "transfer".
- In source-file mode, --source provides one or more sources and conflicts with --name, --on, --when, and --payload.
- Contract for source-file mode is defined by `wavepeek schema --input`.
- JSON and JSONL rows include time, sample_time, source, and ordered payload values.

Use this command to extract synchronous handshakes or transfer-like rows without joining property and value output outside wavepeek."#,
        after_long_help = "See also:\n  wavepeek docs show commands/extract"
    )]
    Generic(Box<GenericArgs>),
}

#[derive(Debug, Args)]
pub struct AxiArgs {
    /// Path to VCD/FST/FSDB waveform file
    #[arg(long, value_name = "FILE", help_heading = "Input options")]
    pub waves: PathBuf,
    /// AXI profile: axi3, axi4, or axi4-lite (defaults to axi4)
    #[arg(long, value_name = "PROFILE", help_heading = "Input options")]
    pub profile: Option<String>,
    /// JSON AXI source file with profile, name, includes, and maps
    #[arg(
        long,
        value_name = "FILE",
        conflicts_with_all = ["profile", "name", "maps", "includes"],
        help_heading = "Input options"
    )]
    pub source: Option<PathBuf>,
    /// AXI port name metadata for output (defaults to axi)
    #[arg(long, help_heading = "Input options")]
    pub name: Option<String>,
    /// Start of inclusive event time range (e.g. 1234ns; omitted means dump start)
    #[arg(long, help_heading = "Selection options")]
    pub from: Option<String>,
    /// End of inclusive event time range (e.g. 1234ns; omitted means dump end)
    #[arg(long, help_heading = "Selection options")]
    pub to: Option<String>,
    /// Canonical scope path for scope-relative AXI signal names and include regexes
    #[arg(long, help_heading = "Selection options")]
    pub scope: Option<String>,
    /// Explicit AXI mapping STD_NAME=WAVES_NAME; may be repeated
    #[arg(
        long = "map",
        value_name = "STD=WAVES",
        help_heading = "Signal mapping options"
    )]
    pub maps: Vec<String>,
    /// Regex selecting waveform signal candidates for AXI auto-mapping; may be repeated
    #[arg(
        long = "include",
        value_name = "REGEX",
        help_heading = "Signal mapping options"
    )]
    pub includes: Vec<String>,
    /// Maximum number of extracted transfer rows (`unlimited` disables truncation, value must be > 0)
    #[arg(long, default_value = "50", help_heading = "Output options")]
    pub max: LimitArg,
    /// Print canonical mapping and payload paths in human output
    #[arg(long, help_heading = "Output options")]
    pub abs: bool,
    /// Machine-readable JSON output
    #[arg(long, help_heading = "Output options")]
    pub json: bool,
    /// Stream newline-delimited JSON output
    #[arg(long, conflicts_with = "json", help_heading = "Output options")]
    pub jsonl: bool,
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
