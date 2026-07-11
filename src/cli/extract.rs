use std::path::PathBuf;

use clap::{Args, Subcommand, ValueEnum};

use crate::cli::limits::LimitArg;

#[derive(Debug, Subcommand)]
pub enum ExtractCommand {
    #[command(
        about = "Extract AXI ready/valid transfer rows.",
        long_about = r#"Extract AXI ready/valid transfer rows.

Behavior:
- AXI3, AXI4, AXI4-Lite, ACE, ACE-Lite, and ACE5 profiles use Arm IHI 0022H.c.
- AXI5, AXI5-Lite, ACE5-Lite, ACE5-LiteDVM, and ACE5-LiteACP profiles use Arm IHI 0022L ready/valid transport.
- Supports AXI3, AXI4, AXI4-Lite, AXI5, AXI5-Lite, ACE, ACE-Lite, ACE5, ACE5-Lite, ACE5-LiteDVM, and ACE5-LiteACP profiles.
- Signal mapping combines explicit STD_NAME=WAVES_NAME maps with include-regex auto-mapping; explicit maps win.
- Builds one extraction source per complete ready/valid channel.
- AXI5 and ACE5-LiteDVM can add DVM ac and cr channels but do not add cd.
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

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum AxiProfileArg {
    Axi3,
    Axi4,
    #[value(name = "axi4-lite", alias = "axi4_lite")]
    Axi4Lite,
    Axi5,
    #[value(name = "axi5-lite", alias = "axi5_lite")]
    Axi5Lite,
    Ace,
    #[value(name = "ace-lite", alias = "ace_lite")]
    AceLite,
    Ace5,
    #[value(name = "ace5-lite", alias = "ace5_lite")]
    Ace5Lite,
    #[value(
        name = "ace5-lite-dvm",
        aliases = ["ace5-litedvm", "ace5_litedvm", "ace5_lite_dvm"]
    )]
    Ace5LiteDvm,
    #[value(
        name = "ace5-lite-acp",
        aliases = ["ace5-liteacp", "ace5_liteacp", "ace5_lite_acp"]
    )]
    Ace5LiteAcp,
}

impl AxiProfileArg {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Axi3 => "axi3",
            Self::Axi4 => "axi4",
            Self::Axi4Lite => "axi4-lite",
            Self::Axi5 => "axi5",
            Self::Axi5Lite => "axi5-lite",
            Self::Ace => "ace",
            Self::AceLite => "ace-lite",
            Self::Ace5 => "ace5",
            Self::Ace5Lite => "ace5-lite",
            Self::Ace5LiteDvm => "ace5-lite-dvm",
            Self::Ace5LiteAcp => "ace5-lite-acp",
        }
    }
}

impl std::fmt::Display for AxiProfileArg {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}

#[derive(Debug, Args)]
pub struct AxiArgs {
    /// Path to VCD/FST/FSDB waveform file
    #[arg(long, value_name = "FILE", help_heading = "Input options")]
    pub waves: PathBuf,
    /// AXI profile from Arm IHI 0022H.c or IHI 0022L
    #[arg(
        long,
        value_name = "PROFILE",
        value_enum,
        ignore_case = true,
        default_value_t = AxiProfileArg::Axi4,
        conflicts_with = "source",
        help_heading = "Input options"
    )]
    pub profile: AxiProfileArg,
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
    /// Explicit AXI mapping STD_NAME=WAVES_NAME, e.g. awvalid=cpu_dmem_awvalid; may be repeated
    #[arg(
        long = "map",
        value_name = "STD=WAVES",
        help_heading = "Signal mapping options"
    )]
    pub maps: Vec<String>,
    /// Regex selecting waveform signal candidates for AXI auto-mapping, e.g. '^axi_(aw|w|b|ar|r)_'; may be repeated
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
