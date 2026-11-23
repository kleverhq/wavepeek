use crate::expr::ExprType;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WaveformMetadata {
    pub time_unit: String,
    pub time_start: String,
    pub time_end: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScopeEntry {
    pub path: ondas::HierarchyPath,
    pub depth: usize,
    pub kind: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignalEntry {
    pub name: String,
    pub path: ondas::HierarchyPath,
    pub kind: String,
    pub width: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SignalListing {
    pub entries: Vec<SignalEntry>,
    pub omitted_ambiguous_paths: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SampledSignal {
    pub path: String,
    pub width: u32,
    /// Four-state MSB-first bits (0/1/x/z); raw events use an empty string.
    pub bits: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SampledSignalState {
    pub path: String,
    pub width: u32,
    /// Four-state MSB-first bits; None is Missing and events use an empty string.
    pub bits: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedSignal {
    pub path: String,
    pub signal: ondas::Signal,
    pub width: u32,
    pub(crate) selector: ondas::SignalSelector,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ExprResolvedSignal {
    pub path: String,
    pub signal: ondas::Signal,
    pub expr_type: ExprType,
}

// Stable machine-contract kind inventories intentionally exclude backend-specific
// GHW/VHDL spellings even when the waveform backend can expose them.
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) const STABLE_SCOPE_KIND_ALIASES: &[&str] = &[
    "module",
    "task",
    "function",
    "begin",
    "fork",
    "generate",
    "struct",
    "union",
    "class",
    "interface",
    "package",
    "program",
    "unknown",
];

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) const EXCLUDED_SCOPE_KIND_ALIASES: &[&str] = &[
    "vhdl_architecture",
    "vhdl_procedure",
    "vhdl_function",
    "vhdl_record",
    "vhdl_process",
    "vhdl_block",
    "vhdl_for_generate",
    "vhdl_if_generate",
    "vhdl_generate",
    "vhdl_package",
    "vhdl_array",
    "ghw_generic",
];

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) const STABLE_SIGNAL_KIND_ALIASES: &[&str] = &[
    "event",
    "integer",
    "parameter",
    "real",
    "reg",
    "supply0",
    "supply1",
    "time",
    "tri",
    "triand",
    "trior",
    "trireg",
    "tri0",
    "tri1",
    "wand",
    "wire",
    "wor",
    "string",
    "port",
    "sparse_array",
    "real_time",
    "real_parameter",
    "bit",
    "logic",
    "int",
    "short_int",
    "long_int",
    "byte",
    "enum",
    "short_real",
    "boolean",
    "bit_vector",
];

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) const EXCLUDED_SIGNAL_KIND_ALIASES: &[&str] = &[
    "std_logic",
    "std_ulogic",
    "std_logic_vector",
    "std_ulogic_vector",
];
