use std::path::PathBuf;

use clap::{Args, Subcommand, ValueEnum};

use crate::cli::limits::LimitArg;

use super::HelpArgs;

#[derive(Debug, Subcommand)]
pub enum ExtractCommand {
    #[command(
        about = "Extract manager-facing AHB address and data-phase events.",
        long_about = r#"Extract manager-facing AHB address and data-phase events.

Behavior:
- Supports AHB-Lite and AHB5 from Arm IHI 0033C, Issue C.
- Tracks one accepted address phase so data completions remain separate from idle clocks.
- Emits address, data-complete, reset, and desynchronized events by default. The include flags add stall, idle, or busy cycle events.
- Samples control and payload one dump tick before each rising HCLK edge.
- Uses manager-facing HREADY. HREADYOUT, HSELx, and parity or check signals are outside this interface.
- Explicit `STD_NAME=WAVES_NAME` mappings override signals found by include regexes.
- A source file can provide the profile, output name, include flags, regexes, and mappings.

Example:
  wavepeek extract ahb --waves dump.fst \
    --scope tb.dut.ahb_m --profile ahb-lite \
    --map hclk=clk --include '^m_ahb_'

Notes:
- `--source` conflicts with `--profile`, `--name`, `--map`, `--include`, `--include-stall`, `--include-idle`, and `--include-busy`.
- Pipeline warm-up starts before `--from`, so an in-range completion may belong to an earlier address phase. JSON field `initial_data_phase` records that state.
- JSON context includes the Issue C profile, resolved mappings, and ordered event rows.
- The extractor does not reconstruct bursts, combine transactions, or join address and data phases.
- See extract-ahb.md for mapping and stall examples. See machine-output.md for JSON and JSONL records."#
    )]
    Ahb(Box<AhbArgs>),
    #[command(
        about = "Extract APB Setup and Access events.",
        long_about = r#"Extract APB Setup and Access events.

Behavior:
- Supports APB3, APB4, and APB5 from Arm IHI 0024E.
- Emits setup and access-complete rows by default. `--include-wait` adds one row for each waited Access cycle.
- Mapped PREADY mode requires `pready`. Implicit-high mode forbids `pready` and wait rows.
- Maps one concrete Completer PSELx signal as `psel`.
- Samples reset, predicates, direction, and payload at the pre-edge sample point.
- Explicit `STD_NAME=WAVES_NAME` mappings override signals found by include regexes.
- A source file can provide the profile, PREADY mode, wait flag, output name, regexes, and mappings. Source files use canonical lowercase profile and mode values.

Example:
  wavepeek extract apb --waves dump.fst \
    --scope tb.dut.uart_apb --profile apb4 \
    --map pclk=clk --include '^uart_'

Notes:
- `--source` conflicts with `--profile`, `--pready-mode`, `--include-wait`, `--name`, `--map`, and `--include`.
- JSON context includes APB metadata, resolved mappings, and event rows.
- Rows are independent sampled events. The extractor does not correlate or validate transactions.
- See extract-apb.md for mapping and wait-state examples. See machine-output.md for JSON and JSONL records."#
    )]
    Apb(Box<ApbArgs>),
    #[command(
        about = "Extract ATB transfer, flush, and synchronization-request events.",
        long_about = r#"Extract ATB transfer, flush, and synchronization-request events.

Behavior:
- Supports ATB-A, ATB-B, and ATB-C from Arm IHI 0032C, Issue C.
- Builds separate event sources for complete ATVALID/ATREADY and AFVALID/AFREADY handshakes.
- A mapped SYNCREQ signal on ATB-B or ATB-C adds synchronization-request events.
- Samples reset, predicates, and transfer payload at the pre-edge sample point.
- Orders same-edge events as transfer, flush, then synchronization request.
- Preserves mapped ATBYTES, ATDATA, and ATID values without trace decoding.
- Explicit `STD_NAME=WAVES_NAME` mappings override signals found by include regexes.
- A source file can provide the profile, output name, regexes, and mappings. Source files use canonical hyphenated profile names.

Example:
  wavepeek extract atb --waves dump.fst \
    --scope tb.dut.etm --profile atb-c \
    --map atclk=trace_clk --include '^trace_(at|af|sync)'

Notes:
- `--source` conflicts with `--profile`, `--name`, `--map`, and `--include`.
- CLI profile aliases are `atb_a`, `atb_b`, `atb_c`, `atbv1.0`, and `atbv1.1`.
- JSON context includes ATB metadata, resolved mappings, and event rows.
- Rows are stateless sampled events. The extractor does not reconstruct packets, stalls, flush episodes, or synchronization episodes.
- See extract-atb.md for mapping examples. See machine-output.md for JSON and JSONL records."#
    )]
    Atb(Box<AtbArgs>),
    #[command(
        about = "Extract AXI-family ready/valid channel transfers.",
        long_about = r#"Extract AXI-family ready/valid channel transfers.

Behavior:
- Supports AXI3, AXI4, AXI4-Lite, AXI5, AXI5-Lite, ACE, ACE-Lite, ACE5, ACE5-Lite, ACE5-LiteDVM, and ACE5-LiteACP.
- AXI3, AXI4, AXI4-Lite, ACE, ACE-Lite, and ACE5 use Arm IHI 0022H.c. The remaining profiles use the Arm IHI 0022L ready/valid transport.
- Builds one event source for each complete ready/valid channel. AXI5 and ACE5-LiteDVM can add DVM `ac` and `cr` channels, but not `cd`.
- Samples reset, ready and valid predicates, and payload at the pre-edge sample point.
- Explicit `STD_NAME=WAVES_NAME` mappings override signals found by include regexes.
- A source file can provide the profile, output name, regexes, and mappings. Source files use canonical hyphenated profile names.

Example:
  wavepeek extract axi --waves dump.fst \
    --scope tb.dut.axi_m --profile axi4 \
    --map aclk=clk --include '^m_axi_(aw|w|b|ar|r)'

Notes:
- `--source` conflicts with `--profile`, `--name`, `--map`, and `--include`.
- CLI aliases include `ace5_lite`, `ace5-litedvm`, `ace5_litedvm`, `ace5_lite_dvm`, `ace5-liteacp`, `ace5_liteacp`, and `ace5_lite_acp`.
- JSON context includes AXI metadata, resolved mappings, and transfer rows.
- Rows are raw channel transfers, including eligible `ac` and `cr` transfers. The extractor does not decode DVM messages or coherency state. It does not reconstruct bursts, ordering, or outstanding request state.
- See extract-axi.md for mapping examples. See machine-output.md for JSON and JSONL records."#
    )]
    Axi(Box<AxiArgs>),
    #[command(
        name = "axistream",
        about = "Extract AXI-Stream transfers.",
        long_about = r#"Extract AXI-Stream transfers.

Behavior:
- Supports AXI4-Stream and AXI5-Stream from Arm IHI 0051B, Issue B.
- Mapped TREADY mode requires `tvalid` and `tready`. Implicit-high mode declares that physical TREADY is absent.
- Samples reset, handshake predicates, and payload at the pre-edge sample point for each rising ACLK edge.
- One invocation maps one stream interface and emits one row per completed transfer without adding a channel name.
- Explicit `STD_NAME=WAVES_NAME` mappings override signals found by include regexes.
- A source file can provide the profile, TREADY mode, output name, regexes, and mappings.

Example:
  wavepeek extract axistream --waves dump.fst \
    --scope tb.dut.video_out --profile axi4-stream \
    --map aclk=clk --include '^video_'

Notes:
- `--source` conflicts with `--profile`, `--tready-mode`, `--name`, `--map`, and `--include`.
- AXI5-Stream wake-up, parity, and check signals are outside this extractor.
- See extract-axis.md for mapping and implicit-high examples. See machine-output.md for JSON and JSONL records."#
    )]
    AxiStream(Box<AxiStreamArgs>),
    #[command(
        about = "Extract custom synchronous events and their payload values.",
        long_about = r#"Extract custom synchronous events and their payload values.

Behavior:
- `--on` selects edge-only events. `--when` and `--payload` are always sampled at the pre-edge sample point.
- `--payload` accepts comma-separated values, repeated options, or both. Entries may end in `[msb:lsb]`, and request order and duplicates are preserved.
- Use `[n:n]` for one bit. Exact waveform paths take precedence, and `[n]` remains path syntax.
- In CLI mode, `--on`, `--when`, and `--payload` define one source named by `--name`.
- A source file can define one or more sources. `--source` conflicts with `--name`, `--on`, `--when`, and `--payload`.
- JSON and JSONL rows include `time`, `sample_time`, `source`, and ordered payload values.

Examples:
  wavepeek extract generic --waves dump.fst \
    --scope tb.dut.queue --on 'posedge clk' \
    --when 'valid && ready' --payload data,last

  wavepeek extract generic --waves dump.fst \
    --scope tb.dut --source fifo-sources.json

Notes:
- Use a protocol extractor when it already supports the interface.
- See extract-transfers.md for generic extraction examples, boolean-expressions.md for `--when`, and event-expressions.md for `--on`.
- See machine-output.md for JSON and JSONL records."#
    )]
    Generic(Box<GenericArgs>),
    #[command(about = "Show detailed help for a command path.")]
    Help(HelpArgs),
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum AhbProfileArg {
    #[value(name = "ahb-lite", alias = "ahb_lite")]
    AhbLite,
    Ahb5,
}

impl AhbProfileArg {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::AhbLite => "ahb-lite",
            Self::Ahb5 => "ahb5",
        }
    }
}

impl std::fmt::Display for AhbProfileArg {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}

#[derive(Debug, Args)]
pub struct AhbArgs {
    /// Path to a VCD, FST, or FSDB file, or an SHM database directory (for example, dump.fst)
    #[arg(long, value_name = "FILE", help_heading = "Input options")]
    pub waves: PathBuf,
    /// AHB profile from Arm IHI 0033C
    #[arg(
        long,
        value_name = "PROFILE",
        value_enum,
        ignore_case = true,
        default_value_t = AhbProfileArg::AhbLite,
        conflicts_with = "source",
        help_heading = "Input options"
    )]
    pub profile: AhbProfileArg,
    /// JSON AHB source file with profile, include flags, name, regexes, and mappings (for example, ahb-source.json)
    #[arg(
        long,
        value_name = "FILE",
        conflicts_with_all = [
            "profile",
            "name",
            "maps",
            "includes",
            "include_stall",
            "include_idle",
            "include_busy"
        ],
        help_heading = "Input options"
    )]
    pub source: Option<PathBuf>,
    /// Interface name stored in output metadata (default: ahb)
    #[arg(long, help_heading = "Input options")]
    pub name: Option<String>,
    /// Start of the inclusive event range (for example, 1234ns; default: dump start)
    #[arg(long, help_heading = "Selection options")]
    pub from: Option<String>,
    /// End of the inclusive event range (for example, 2000ns; default: dump end)
    #[arg(long, help_heading = "Selection options")]
    pub to: Option<String>,
    /// Scope for relative AHB signal names and include regexes (for example, top.ahb_m)
    #[arg(long, help_heading = "Selection options")]
    pub scope: Option<String>,
    /// Explicit AHB signal mapping; may be repeated (for example, haddr=dmem_haddr)
    #[arg(
        long = "map",
        value_name = "STD=WAVES",
        help_heading = "Signal mapping options"
    )]
    pub maps: Vec<String>,
    /// Regex for AHB auto-mapping candidates; may be repeated (for example, '^m_ahb_')
    #[arg(
        long = "include",
        value_name = "REGEX",
        help_heading = "Signal mapping options"
    )]
    pub includes: Vec<String>,
    /// Emit one data-stall event for each active low-HREADY cycle
    #[arg(long, help_heading = "Event options")]
    pub include_stall: bool,
    /// Emit one idle event for each known-ready IDLE slot
    #[arg(long, help_heading = "Event options")]
    pub include_idle: bool,
    /// Emit one busy event for each known-ready BUSY slot
    #[arg(long, help_heading = "Event options")]
    pub include_busy: bool,
    /// Maximum number of public AHB event rows (`unlimited` disables truncation, value must be > 0)
    #[arg(long, default_value = "50", help_heading = "Output options")]
    pub max: LimitArg,
    /// Suppress result rows while retaining context and completeness metadata
    #[arg(long, help_heading = "Output options")]
    pub summary: bool,
    /// Print canonical mapping paths in human output
    #[arg(long, help_heading = "Output options")]
    pub abs: bool,
    /// Machine-readable JSON output
    #[arg(long, help_heading = "Output options")]
    pub json: bool,
    /// Stream newline-delimited JSON output
    #[arg(long, conflicts_with = "json", help_heading = "Output options")]
    pub jsonl: bool,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum ApbProfileArg {
    Apb3,
    Apb4,
    Apb5,
}

impl ApbProfileArg {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Apb3 => "apb3",
            Self::Apb4 => "apb4",
            Self::Apb5 => "apb5",
        }
    }
}

impl std::fmt::Display for ApbProfileArg {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum PreadyModeArg {
    Mapped,
    #[value(name = "implicit-high", alias = "implicit_high")]
    ImplicitHigh,
}

impl PreadyModeArg {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Mapped => "mapped",
            Self::ImplicitHigh => "implicit-high",
        }
    }
}

impl std::fmt::Display for PreadyModeArg {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}

#[derive(Debug, Args)]
pub struct ApbArgs {
    /// Path to a VCD, FST, or FSDB file, or an SHM database directory (for example, dump.fst)
    #[arg(long, value_name = "FILE", help_heading = "Input options")]
    pub waves: PathBuf,
    /// APB profile from Arm IHI 0024E
    #[arg(
        long,
        value_name = "PROFILE",
        value_enum,
        ignore_case = true,
        default_value_t = ApbProfileArg::Apb4,
        conflicts_with = "source",
        help_heading = "Input options"
    )]
    pub profile: ApbProfileArg,
    /// PREADY handling mode
    #[arg(
        long,
        value_name = "MODE",
        value_enum,
        ignore_case = true,
        default_value_t = PreadyModeArg::Mapped,
        conflicts_with = "source",
        help_heading = "Input options"
    )]
    pub pready_mode: PreadyModeArg,
    /// Emit one access-wait row per waited Access cycle
    #[arg(long, conflicts_with = "source", help_heading = "Input options")]
    pub include_wait: bool,
    /// JSON APB source file with profile, PREADY mode, wait flag, name, regexes, and mappings (for example, apb-source.json)
    #[arg(
        long,
        value_name = "FILE",
        conflicts_with_all = [
            "profile",
            "pready_mode",
            "include_wait",
            "name",
            "maps",
            "includes"
        ],
        help_heading = "Input options"
    )]
    pub source: Option<PathBuf>,
    /// Interface name stored in output metadata (default: apb)
    #[arg(long, help_heading = "Input options")]
    pub name: Option<String>,
    /// Start of the inclusive event range (for example, 1234ns; default: dump start)
    #[arg(long, help_heading = "Selection options")]
    pub from: Option<String>,
    /// End of the inclusive event range (for example, 2000ns; default: dump end)
    #[arg(long, help_heading = "Selection options")]
    pub to: Option<String>,
    /// Scope for relative APB signal names and include regexes (for example, top.uart_apb)
    #[arg(long, help_heading = "Selection options")]
    pub scope: Option<String>,
    /// Explicit APB signal mapping; may be repeated (for example, psel=uart_psel)
    #[arg(
        long = "map",
        value_name = "STD=WAVES",
        help_heading = "Signal mapping options"
    )]
    pub maps: Vec<String>,
    /// Regex for APB auto-mapping candidates; may be repeated (for example, '^uart_apb_')
    #[arg(
        long = "include",
        value_name = "REGEX",
        help_heading = "Signal mapping options"
    )]
    pub includes: Vec<String>,
    /// Maximum number of extracted event rows (`unlimited` disables truncation, value must be > 0)
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
    /// Suppress result rows while retaining context and completeness metadata
    #[arg(long, help_heading = "Output options")]
    pub summary: bool,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum AtbProfileArg {
    #[value(name = "atb-a", aliases = ["atb_a", "atbv1.0"])]
    AtbA,
    #[value(name = "atb-b", aliases = ["atb_b", "atbv1.1"])]
    AtbB,
    #[value(name = "atb-c", alias = "atb_c")]
    AtbC,
}

impl AtbProfileArg {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::AtbA => "atb-a",
            Self::AtbB => "atb-b",
            Self::AtbC => "atb-c",
        }
    }
}

impl std::fmt::Display for AtbProfileArg {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}

#[derive(Debug, Args)]
pub struct AtbArgs {
    /// Path to a VCD, FST, or FSDB file, or an SHM database directory (for example, dump.fst)
    #[arg(long, value_name = "FILE", help_heading = "Input options")]
    pub waves: PathBuf,
    /// ATB profile from Arm IHI 0032C Issue C
    #[arg(
        long,
        value_name = "PROFILE",
        value_enum,
        ignore_case = true,
        default_value_t = AtbProfileArg::AtbC,
        conflicts_with = "source",
        help_heading = "Input options"
    )]
    pub profile: AtbProfileArg,
    /// JSON ATB source file with profile, name, regexes, and mappings (for example, atb-source.json)
    #[arg(
        long,
        value_name = "FILE",
        conflicts_with_all = ["profile", "name", "maps", "includes"],
        help_heading = "Input options"
    )]
    pub source: Option<PathBuf>,
    /// Interface name stored in output metadata (default: atb)
    #[arg(long, help_heading = "Input options")]
    pub name: Option<String>,
    /// Start of the inclusive event range (for example, 1234ns; default: dump start)
    #[arg(long, help_heading = "Selection options")]
    pub from: Option<String>,
    /// End of the inclusive event range (for example, 2000ns; default: dump end)
    #[arg(long, help_heading = "Selection options")]
    pub to: Option<String>,
    /// Scope for relative ATB signal names and include regexes (for example, top.etm)
    #[arg(long, help_heading = "Selection options")]
    pub scope: Option<String>,
    /// Explicit ATB signal mapping; may be repeated (for example, atvalid=etm_atvalid)
    #[arg(
        long = "map",
        value_name = "STD=WAVES",
        help_heading = "Signal mapping options"
    )]
    pub maps: Vec<String>,
    /// Regex for ATB auto-mapping candidates; may be repeated (for example, '^etm_(at|af)')
    #[arg(
        long = "include",
        value_name = "REGEX",
        help_heading = "Signal mapping options"
    )]
    pub includes: Vec<String>,
    /// Maximum number of extracted event rows (`unlimited` disables truncation, value must be > 0)
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
    /// Suppress result rows while retaining context and completeness metadata
    #[arg(long, help_heading = "Output options")]
    pub summary: bool,
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

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum AxiStreamProfileArg {
    #[value(name = "axi4-stream", alias = "axi4_stream")]
    Axi4Stream,
    #[value(name = "axi5-stream", alias = "axi5_stream")]
    Axi5Stream,
}

impl AxiStreamProfileArg {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Axi4Stream => "axi4-stream",
            Self::Axi5Stream => "axi5-stream",
        }
    }
}

impl std::fmt::Display for AxiStreamProfileArg {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum TreadyModeArg {
    Mapped,
    #[value(name = "implicit-high", alias = "implicit_high")]
    ImplicitHigh,
}

impl TreadyModeArg {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Mapped => "mapped",
            Self::ImplicitHigh => "implicit-high",
        }
    }
}

impl std::fmt::Display for TreadyModeArg {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}

#[derive(Debug, Args)]
pub struct AxiArgs {
    /// Path to a VCD, FST, or FSDB file, or an SHM database directory (for example, dump.fst)
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
    /// JSON AXI source file with profile, name, regexes, and mappings (for example, axi-source.json)
    #[arg(
        long,
        value_name = "FILE",
        conflicts_with_all = ["profile", "name", "maps", "includes"],
        help_heading = "Input options"
    )]
    pub source: Option<PathBuf>,
    /// Interface name stored in output metadata (default: axi)
    #[arg(long, help_heading = "Input options")]
    pub name: Option<String>,
    /// Start of the inclusive event range (for example, 1234ns; default: dump start)
    #[arg(long, help_heading = "Selection options")]
    pub from: Option<String>,
    /// End of the inclusive event range (for example, 2000ns; default: dump end)
    #[arg(long, help_heading = "Selection options")]
    pub to: Option<String>,
    /// Scope for relative AXI signal names and include regexes (for example, top.axi_m)
    #[arg(long, help_heading = "Selection options")]
    pub scope: Option<String>,
    /// Explicit AXI signal mapping; may be repeated (for example, awvalid=cpu_dmem_awvalid)
    #[arg(
        long = "map",
        value_name = "STD=WAVES",
        help_heading = "Signal mapping options"
    )]
    pub maps: Vec<String>,
    /// Regex for AXI auto-mapping candidates; may be repeated (for example, '^axi_(aw|w|b|ar|r)_')
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
    /// Suppress result rows while retaining context and completeness metadata
    #[arg(long, help_heading = "Output options")]
    pub summary: bool,
}

#[derive(Debug, Args)]
pub struct AxiStreamArgs {
    /// Path to a VCD, FST, or FSDB file, or an SHM database directory (for example, dump.fst)
    #[arg(long, value_name = "FILE", help_heading = "Input options")]
    pub waves: PathBuf,
    /// AXI-Stream profile from Arm IHI 0051B Issue B
    #[arg(
        long,
        value_name = "PROFILE",
        value_enum,
        ignore_case = true,
        default_value_t = AxiStreamProfileArg::Axi4Stream,
        conflicts_with = "source",
        help_heading = "Input options"
    )]
    pub profile: AxiStreamProfileArg,
    /// Whether TREADY is mapped or physically omitted and implicitly HIGH
    #[arg(
        long,
        value_name = "MODE",
        value_enum,
        ignore_case = true,
        default_value_t = TreadyModeArg::Mapped,
        conflicts_with = "source",
        help_heading = "Input options"
    )]
    pub tready_mode: TreadyModeArg,
    /// JSON AXI-Stream source file with profile, TREADY mode, name, regexes, and mappings (for example, axistream-source.json)
    #[arg(
        long,
        value_name = "FILE",
        conflicts_with_all = ["profile", "tready_mode", "name", "maps", "includes"],
        help_heading = "Input options"
    )]
    pub source: Option<PathBuf>,
    /// Interface name stored in output metadata (default: axistream)
    #[arg(long, help_heading = "Input options")]
    pub name: Option<String>,
    /// Start of the inclusive event range (for example, 1234ns; default: dump start)
    #[arg(long, help_heading = "Selection options")]
    pub from: Option<String>,
    /// End of the inclusive event range (for example, 2000ns; default: dump end)
    #[arg(long, help_heading = "Selection options")]
    pub to: Option<String>,
    /// Scope for relative AXI-Stream signal names and include regexes (for example, top.video_out)
    #[arg(long, help_heading = "Selection options")]
    pub scope: Option<String>,
    /// Explicit AXI-Stream signal mapping; may be repeated (for example, tvalid=video_tvalid)
    #[arg(
        long = "map",
        value_name = "STD=WAVES",
        help_heading = "Signal mapping options"
    )]
    pub maps: Vec<String>,
    /// Regex for AXI-Stream auto-mapping candidates; may be repeated (for example, '^video_')
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
    /// Suppress result rows while retaining context and completeness metadata
    #[arg(long, help_heading = "Output options")]
    pub summary: bool,
}

#[derive(Debug, Args)]
pub struct GenericArgs {
    /// Path to a VCD, FST, or FSDB file, or an SHM database directory (for example, dump.fst)
    #[arg(long, value_name = "FILE", help_heading = "Input options")]
    pub waves: PathBuf,
    /// JSON file for multi-source extraction (for example, fifo-sources.json)
    #[arg(
        long,
        value_name = "FILE",
        conflicts_with_all = ["name", "on", "when", "payload"],
        help_heading = "Input options"
    )]
    pub source: Option<PathBuf>,
    /// Start of the inclusive event range (for example, 1234ns; default: dump start)
    #[arg(long, help_heading = "Selection options")]
    pub from: Option<String>,
    /// End of the inclusive event range (for example, 2000ns; default: dump end)
    #[arg(long, help_heading = "Selection options")]
    pub to: Option<String>,
    /// Scope for relative event, predicate, and payload names (for example, top.fifo)
    #[arg(long, help_heading = "Selection options")]
    pub scope: Option<String>,
    /// Source name for single-source CLI mode (default: transfer)
    #[arg(long, help_heading = "Selection options")]
    pub name: Option<String>,
    /// Edge-only event expression for single-source CLI mode (for example, 'posedge clk')
    #[arg(long, help_heading = "Selection options")]
    pub on: Option<String>,
    /// Pre-edge predicate for single-source CLI mode (for example, 'valid && ready')
    #[arg(long, help_heading = "Selection options")]
    pub when: Option<String>,
    /// Payload paths or flat projections, comma-separated or repeated (for example, data,last or status[7:4])
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
    /// Suppress result rows while retaining context and completeness metadata
    #[arg(long, help_heading = "Output options")]
    pub summary: bool,
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
