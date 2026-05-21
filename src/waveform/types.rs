use crate::expr::ExprType;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SignalId(u64);

impl SignalId {
    #[inline]
    pub(in crate::waveform) fn from_backend_index(index: u64) -> Self {
        Self(index)
    }

    #[cfg(test)]
    pub(crate) fn from_test_index(index: u64) -> Self {
        Self(index)
    }

    #[inline]
    pub(in crate::waveform) fn as_u64(self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WaveformMetadata {
    pub time_unit: String,
    pub time_start: String,
    pub time_end: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScopeEntry {
    pub path: String,
    pub depth: usize,
    pub kind: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignalEntry {
    pub name: String,
    pub path: String,
    pub kind: String,
    pub width: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SampledSignal {
    pub path: String,
    pub width: u32,
    pub bits: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SampledSignalState {
    pub path: String,
    pub width: u32,
    pub bits: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedSignal {
    pub path: String,
    pub id: SignalId,
    pub width: u32,
}

#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub(crate) struct ExprResolvedSignal {
    pub path: String,
    pub id: SignalId,
    pub expr_type: ExprType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ChangeCandidateCollectionMode {
    Auto,
    Random,
    Stream,
}

#[derive(Debug, Clone, Copy)]
pub struct SignalOffsetData {
    start: usize,
    elements: u16,
}

impl SignalOffsetData {
    pub(in crate::waveform) fn new(start: usize, elements: u16) -> Self {
        Self { start, elements }
    }
}

impl PartialEq for SignalOffsetData {
    fn eq(&self, other: &Self) -> bool {
        self.start == other.start && self.elements == other.elements
    }
}

impl Eq for SignalOffsetData {}

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
