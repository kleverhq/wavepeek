//! Waveform adapter used by the engine layer.
//!
//! Canonical path policy in M2:
//! - Paths are emitted as dot-separated full hierarchy paths.
//! - Scope and signal names are preserved exactly as provided by the parser.
//! - No additional escaping or normalization pass is applied.

use std::cmp::Ordering;
use std::path::Path;

use wellen::{ScopeRef, ScopeType, SignalRef, Timescale, TimescaleUnit, VarType, simple};

use crate::error::WavepeekError;

#[derive(Debug)]
pub struct Waveform {
    inner: simple::Waveform,
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

impl Waveform {
    pub fn open(path: &Path) -> Result<Self, WavepeekError> {
        if !path.exists() {
            return Err(WavepeekError::File(format!(
                "cannot open '{}': No such file or directory",
                path.display()
            )));
        }

        let inner = simple::read(path).map_err(|error| map_wellen_error(path, error))?;
        Ok(Self { inner })
    }

    pub fn metadata(&self) -> Result<WaveformMetadata, WavepeekError> {
        let hierarchy = self.inner.hierarchy();
        let timescale = hierarchy.timescale().ok_or_else(|| {
            WavepeekError::File("waveform is missing timescale metadata".to_string())
        })?;
        let time_unit = format_timescale(timescale)?;

        let time_table = self.inner.time_table();
        let time_start = time_table.first().copied().unwrap_or(0);
        let time_end = time_table.last().copied().unwrap_or(time_start);

        Ok(WaveformMetadata {
            time_unit,
            time_start: normalize_time(time_start, timescale)?,
            time_end: normalize_time(time_end, timescale)?,
        })
    }

    pub fn scopes_depth_first(&self, max_depth: Option<usize>) -> Vec<ScopeEntry> {
        let hierarchy = self.inner.hierarchy();
        let mut roots: Vec<ScopeRef> = hierarchy.scopes().collect();
        sort_scope_refs(hierarchy, &mut roots);

        let mut entries = Vec::new();
        for scope_ref in roots {
            collect_scope_entries(hierarchy, scope_ref, 0, max_depth, &mut entries);
        }

        entries
    }

    pub fn signals_in_scope(&self, scope_path: &str) -> Result<Vec<SignalEntry>, WavepeekError> {
        let hierarchy = self.inner.hierarchy();
        let names: Vec<&str> = scope_path.split('.').collect();
        let scope_ref = hierarchy.lookup_scope(&names).ok_or_else(|| {
            WavepeekError::Scope(format!("scope '{scope_path}' not found in dump"))
        })?;

        let scope = &hierarchy[scope_ref];
        let mut signals = scope
            .vars(hierarchy)
            .map(|var_ref| signal_entry_from_var_ref(hierarchy, var_ref))
            .collect::<Vec<_>>();

        sort_signal_entries(&mut signals);
        Ok(signals)
    }

    pub fn signals_in_scope_recursive(
        &self,
        scope_path: &str,
        max_depth: Option<usize>,
    ) -> Result<Vec<SignalEntry>, WavepeekError> {
        let hierarchy = self.inner.hierarchy();
        let names: Vec<&str> = scope_path.split('.').collect();
        let scope_ref = hierarchy.lookup_scope(&names).ok_or_else(|| {
            WavepeekError::Scope(format!("scope '{scope_path}' not found in dump"))
        })?;

        let mut entries = Vec::new();
        collect_scope_signals(hierarchy, scope_ref, 0, max_depth, &mut entries);
        Ok(entries)
    }

    pub fn sample_signals_at_time(
        &mut self,
        canonical_paths: &[String],
        query_time_raw: u64,
    ) -> Result<Vec<SampledSignal>, WavepeekError> {
        let time_table = self.inner.time_table();
        let time_table_idx =
            floor_time_table_index(time_table, query_time_raw).ok_or_else(|| {
                WavepeekError::Internal("query time is before first dump timestamp".to_string())
            })?;
        let time_table_idx = u32::try_from(time_table_idx).map_err(|_| {
            WavepeekError::Internal("time table index exceeds u32 range".to_string())
        })?;

        let hierarchy = self.inner.hierarchy();
        let mut resolved = Vec::with_capacity(canonical_paths.len());
        for path in canonical_paths {
            let signal_ref = resolve_signal_ref(hierarchy, path.as_str())?;
            resolved.push((path.clone(), signal_ref));
        }

        let signal_refs = resolved
            .iter()
            .map(|(_, signal_ref)| *signal_ref)
            .collect::<Vec<_>>();
        self.inner.load_signals(&signal_refs);

        let mut sampled = Vec::with_capacity(resolved.len());
        for (path, signal_ref) in resolved {
            let signal = self.inner.get_signal(signal_ref).ok_or_else(|| {
                WavepeekError::Internal(format!(
                    "signal '{path}' could not be loaded from waveform backend"
                ))
            })?;

            let offset = signal.get_offset(time_table_idx).ok_or_else(|| {
                WavepeekError::Signal(format!(
                    "signal '{path}' has no value at or before requested time"
                ))
            })?;
            let value = signal.get_value_at(&offset, offset.elements - 1);

            let bits = match value {
                wellen::SignalValue::Event => String::new(),
                wellen::SignalValue::Binary(_, _)
                | wellen::SignalValue::FourValue(_, _)
                | wellen::SignalValue::NineValue(_, _) => {
                    value.to_bit_string().ok_or_else(|| {
                        WavepeekError::Internal(format!(
                            "failed to convert value for signal '{path}' to bit string"
                        ))
                    })?
                }
                wellen::SignalValue::String(_) | wellen::SignalValue::Real(_) => {
                    return Err(WavepeekError::Signal(format!(
                        "signal '{path}' has unsupported non-bit-vector encoding"
                    )));
                }
            };

            let width = value.bits().ok_or_else(|| {
                WavepeekError::Signal(format!(
                    "signal '{path}' has unsupported non-bit-vector encoding"
                ))
            })?;

            sampled.push(SampledSignal { path, width, bits });
        }

        Ok(sampled)
    }

    pub fn timestamps_raw(&self) -> Vec<u64> {
        self.inner.time_table().to_vec()
    }

    pub fn sample_signals_at_time_optional(
        &mut self,
        canonical_paths: &[String],
        query_time_raw: u64,
    ) -> Result<Vec<SampledSignalState>, WavepeekError> {
        let time_table = self.inner.time_table();
        let time_table_idx =
            floor_time_table_index(time_table, query_time_raw).ok_or_else(|| {
                WavepeekError::Internal("query time is before first dump timestamp".to_string())
            })?;
        let time_table_idx = u32::try_from(time_table_idx).map_err(|_| {
            WavepeekError::Internal("time table index exceeds u32 range".to_string())
        })?;

        let hierarchy = self.inner.hierarchy();
        let mut resolved = Vec::with_capacity(canonical_paths.len());
        for path in canonical_paths {
            let (signal_ref, width) = resolve_signal_ref_with_width(hierarchy, path.as_str())?;
            resolved.push((path.clone(), signal_ref, width));
        }

        let signal_refs = resolved
            .iter()
            .map(|(_, signal_ref, _)| *signal_ref)
            .collect::<Vec<_>>();
        self.inner.load_signals(&signal_refs);

        let mut sampled = Vec::with_capacity(resolved.len());
        for (path, signal_ref, width) in resolved {
            let signal = self.inner.get_signal(signal_ref).ok_or_else(|| {
                WavepeekError::Internal(format!(
                    "signal '{path}' could not be loaded from waveform backend"
                ))
            })?;

            let Some(offset) = signal.get_offset(time_table_idx) else {
                sampled.push(SampledSignalState {
                    path,
                    width,
                    bits: None,
                });
                continue;
            };

            let value = signal.get_value_at(&offset, offset.elements - 1);
            let bits = match value {
                wellen::SignalValue::Event => Some(String::new()),
                wellen::SignalValue::Binary(_, _)
                | wellen::SignalValue::FourValue(_, _)
                | wellen::SignalValue::NineValue(_, _) => {
                    Some(value.to_bit_string().ok_or_else(|| {
                        WavepeekError::Internal(format!(
                            "failed to convert value for signal '{path}' to bit string"
                        ))
                    })?)
                }
                wellen::SignalValue::String(_) | wellen::SignalValue::Real(_) => {
                    return Err(WavepeekError::Signal(format!(
                        "signal '{path}' has unsupported non-bit-vector encoding"
                    )));
                }
            };

            sampled.push(SampledSignalState { path, width, bits });
        }

        Ok(sampled)
    }
}

fn floor_time_table_index(time_table: &[u64], query_time_raw: u64) -> Option<usize> {
    if time_table.is_empty() {
        return None;
    }

    match time_table.binary_search(&query_time_raw) {
        Ok(index) => Some(index),
        Err(0) => None,
        Err(index) => Some(index - 1),
    }
}

fn resolve_signal_ref(
    hierarchy: &wellen::Hierarchy,
    canonical_path: &str,
) -> Result<SignalRef, WavepeekError> {
    let (signal_ref, _) = resolve_signal_ref_with_width(hierarchy, canonical_path)?;
    Ok(signal_ref)
}

fn resolve_signal_ref_with_width(
    hierarchy: &wellen::Hierarchy,
    canonical_path: &str,
) -> Result<(SignalRef, u32), WavepeekError> {
    if canonical_path.is_empty() {
        return Err(WavepeekError::Signal(format!(
            "signal '{canonical_path}' not found in dump"
        )));
    }

    let (scope_names, signal_name) = match canonical_path.rsplit_once('.') {
        Some((scope_path, signal_name)) if !scope_path.is_empty() && !signal_name.is_empty() => {
            (scope_path.split('.').collect::<Vec<_>>(), signal_name)
        }
        Some(_) => {
            return Err(WavepeekError::Signal(format!(
                "signal '{canonical_path}' not found in dump"
            )));
        }
        None => (Vec::new(), canonical_path),
    };

    let var_ref = hierarchy
        .lookup_var(&scope_names, signal_name)
        .ok_or_else(|| {
            WavepeekError::Signal(format!("signal '{canonical_path}' not found in dump"))
        })?;

    let var = &hierarchy[var_ref];
    let width = var.length().ok_or_else(|| {
        WavepeekError::Signal(format!(
            "signal '{canonical_path}' has unsupported non-bit-vector encoding"
        ))
    })?;

    Ok((var.signal_ref(), width))
}

fn collect_scope_entries(
    hierarchy: &wellen::Hierarchy,
    scope_ref: ScopeRef,
    depth: usize,
    max_depth: Option<usize>,
    entries: &mut Vec<ScopeEntry>,
) {
    if let Some(max_depth) = max_depth
        && depth > max_depth
    {
        return;
    }

    let scope = &hierarchy[scope_ref];
    entries.push(ScopeEntry {
        path: scope.full_name(hierarchy),
        depth,
        kind: scope_type_alias(scope.scope_type()).to_string(),
    });

    if max_depth == Some(depth) {
        return;
    }

    let mut children: Vec<ScopeRef> = scope.scopes(hierarchy).collect();
    sort_scope_refs(hierarchy, &mut children);
    for child in children {
        collect_scope_entries(hierarchy, child, depth + 1, max_depth, entries);
    }
}

fn collect_scope_signals(
    hierarchy: &wellen::Hierarchy,
    scope_ref: ScopeRef,
    depth: usize,
    max_depth: Option<usize>,
    entries: &mut Vec<SignalEntry>,
) {
    if let Some(max_depth) = max_depth
        && depth > max_depth
    {
        return;
    }

    let scope = &hierarchy[scope_ref];
    let mut signals = scope
        .vars(hierarchy)
        .map(|var_ref| signal_entry_from_var_ref(hierarchy, var_ref))
        .collect::<Vec<_>>();
    sort_signal_entries(&mut signals);
    entries.extend(signals);

    if max_depth == Some(depth) {
        return;
    }

    let mut children: Vec<ScopeRef> = scope.scopes(hierarchy).collect();
    sort_scope_refs(hierarchy, &mut children);
    for child in children {
        collect_scope_signals(hierarchy, child, depth + 1, max_depth, entries);
    }
}

fn signal_entry_from_var_ref(
    hierarchy: &wellen::Hierarchy,
    var_ref: wellen::VarRef,
) -> SignalEntry {
    let var = &hierarchy[var_ref];
    SignalEntry {
        name: var.name(hierarchy).to_string(),
        path: var.full_name(hierarchy),
        kind: var_type_alias(var.var_type()).to_string(),
        width: var.length(),
    }
}

fn sort_signal_entries(signals: &mut [SignalEntry]) {
    signals.sort_by(|lhs, rhs| {
        lhs.name
            .cmp(&rhs.name)
            .then_with(|| lhs.path.cmp(&rhs.path))
    });
}

fn sort_scope_refs(hierarchy: &wellen::Hierarchy, scopes: &mut [ScopeRef]) {
    scopes.sort_by(|lhs, rhs| {
        let lhs_scope = &hierarchy[*lhs];
        let rhs_scope = &hierarchy[*rhs];

        match lhs_scope.name(hierarchy).cmp(rhs_scope.name(hierarchy)) {
            Ordering::Equal => lhs_scope
                .full_name(hierarchy)
                .cmp(&rhs_scope.full_name(hierarchy)),
            order => order,
        }
    });
}

fn format_timescale(timescale: Timescale) -> Result<String, WavepeekError> {
    let unit = timescale_unit_suffix(timescale.unit)?;
    Ok(format!("{}{unit}", timescale.factor))
}

fn normalize_time(time: u64, timescale: Timescale) -> Result<String, WavepeekError> {
    let unit = timescale_unit_suffix(timescale.unit)?;
    let scaled = time
        .checked_mul(u64::from(timescale.factor))
        .ok_or_else(|| {
            WavepeekError::File("time value overflow while normalizing timestamps".to_string())
        })?;
    Ok(format!("{scaled}{unit}"))
}

fn timescale_unit_suffix(unit: TimescaleUnit) -> Result<&'static str, WavepeekError> {
    match unit {
        TimescaleUnit::ZeptoSeconds => Ok("zs"),
        TimescaleUnit::AttoSeconds => Ok("as"),
        TimescaleUnit::FemtoSeconds => Ok("fs"),
        TimescaleUnit::PicoSeconds => Ok("ps"),
        TimescaleUnit::NanoSeconds => Ok("ns"),
        TimescaleUnit::MicroSeconds => Ok("us"),
        TimescaleUnit::MilliSeconds => Ok("ms"),
        TimescaleUnit::Seconds => Ok("s"),
        TimescaleUnit::Unknown => Err(WavepeekError::File(
            "waveform timescale unit is unknown".to_string(),
        )),
    }
}

fn map_wellen_error(path: &Path, error: wellen::WellenError) -> WavepeekError {
    match error {
        wellen::WellenError::Io(io_error) => {
            WavepeekError::File(format!("cannot open '{}': {io_error}", path.display()))
        }
        other => WavepeekError::File(format!("cannot parse '{}': {other}", path.display())),
    }
}

fn var_type_alias(var_type: VarType) -> &'static str {
    match var_type {
        VarType::Event => "event",
        VarType::Integer => "integer",
        VarType::Parameter => "parameter",
        VarType::Real => "real",
        VarType::Reg => "reg",
        VarType::Supply0 => "supply0",
        VarType::Supply1 => "supply1",
        VarType::Time => "time",
        VarType::Tri => "tri",
        VarType::TriAnd => "triand",
        VarType::TriOr => "trior",
        VarType::TriReg => "trireg",
        VarType::Tri0 => "tri0",
        VarType::Tri1 => "tri1",
        VarType::WAnd => "wand",
        VarType::Wire => "wire",
        VarType::WOr => "wor",
        VarType::String => "string",
        VarType::Port => "port",
        VarType::SparseArray => "sparse_array",
        VarType::RealTime => "real_time",
        VarType::RealParameter => "real_parameter",
        VarType::Bit => "bit",
        VarType::Logic => "logic",
        VarType::Int => "int",
        VarType::ShortInt => "short_int",
        VarType::LongInt => "long_int",
        VarType::Byte => "byte",
        VarType::Enum => "enum",
        VarType::ShortReal => "short_real",
        VarType::Boolean => "boolean",
        VarType::BitVector => "bit_vector",
        VarType::StdLogic => "std_logic",
        VarType::StdLogicVector => "std_logic_vector",
        VarType::StdULogic => "std_ulogic",
        VarType::StdULogicVector => "std_ulogic_vector",
    }
}

fn scope_type_alias(scope_type: ScopeType) -> &'static str {
    match scope_type {
        ScopeType::Module => "module",
        ScopeType::Task => "task",
        ScopeType::Function => "function",
        ScopeType::Begin => "begin",
        ScopeType::Fork => "fork",
        ScopeType::Generate => "generate",
        ScopeType::Struct => "struct",
        ScopeType::Union => "union",
        ScopeType::Class => "class",
        ScopeType::Interface => "interface",
        ScopeType::Package => "package",
        ScopeType::Program => "program",
        ScopeType::VhdlArchitecture => "vhdl_architecture",
        ScopeType::VhdlProcedure => "vhdl_procedure",
        ScopeType::VhdlFunction => "vhdl_function",
        ScopeType::VhdlRecord => "vhdl_record",
        ScopeType::VhdlProcess => "vhdl_process",
        ScopeType::VhdlBlock => "vhdl_block",
        ScopeType::VhdlForGenerate => "vhdl_for_generate",
        ScopeType::VhdlIfGenerate => "vhdl_if_generate",
        ScopeType::VhdlGenerate => "vhdl_generate",
        ScopeType::VhdlPackage => "vhdl_package",
        ScopeType::GhwGeneric => "ghw_generic",
        ScopeType::VhdlArray => "vhdl_array",
        ScopeType::Unknown => "unknown",
        _ => "unknown",
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct EdgeClassification {
    pub posedge: bool,
    pub negedge: bool,
}

impl EdgeClassification {
    pub(crate) fn edge(self) -> bool {
        self.posedge || self.negedge
    }
}

pub(crate) fn classify_edge(previous_bits: &str, current_bits: &str) -> EdgeClassification {
    let Some(previous_lsb) = previous_bits.chars().last() else {
        return EdgeClassification {
            posedge: false,
            negedge: false,
        };
    };
    let Some(current_lsb) = current_bits.chars().last() else {
        return EdgeClassification {
            posedge: false,
            negedge: false,
        };
    };

    let previous = normalize_to_four_state(previous_lsb);
    let current = normalize_to_four_state(current_lsb);

    let posedge = matches!(
        (previous, current),
        ('0', '1' | 'x' | 'z') | ('x' | 'z', '1')
    );
    let negedge = matches!(
        (previous, current),
        ('1', '0' | 'x' | 'z') | ('x' | 'z', '0')
    );

    EdgeClassification { posedge, negedge }
}

pub(crate) fn should_emit_delta_and_update_baseline(
    previous_values: &mut [Option<String>],
    current_values: &[Option<String>],
) -> bool {
    let mut changed = false;

    for (previous, current) in previous_values.iter().zip(current_values) {
        if let (Some(previous), Some(current)) = (previous.as_ref(), current.as_ref())
            && previous != current
        {
            changed = true;
        }
    }

    for (previous, current) in previous_values.iter_mut().zip(current_values) {
        if let Some(current) = current {
            *previous = Some(current.clone());
        }
    }

    changed
}

fn normalize_to_four_state(bit: char) -> char {
    match bit.to_ascii_lowercase() {
        '0' => '0',
        '1' => '1',
        'z' => 'z',
        'x' | 'h' | 'u' | 'w' | 'l' | '-' => 'x',
        _ => 'x',
    }
}

#[cfg(test)]
mod tests {
    use std::io::Write;
    use std::path::Path;

    use tempfile::NamedTempFile;

    use super::{
        SampledSignal, ScopeEntry, Waveform, classify_edge, should_emit_delta_and_update_baseline,
    };

    const TEST_VCD: &str = "$date\n  today\n$end\n$version\n  wavepeek-test\n$end\n$timescale 1ns $end\n$scope module top $end\n$var wire 1 ! clk $end\n$var reg 8 \" data $end\n$var parameter 8 # cfg $end\n$scope module cpu $end\n$var wire 1 $ valid $end\n$upscope $end\n$scope function helper $end\n$var wire 1 & helper_flag $end\n$upscope $end\n$scope module mem $end\n$var wire 1 % ready $end\n$upscope $end\n$upscope $end\n$enddefinitions $end\n#0\n0!\nb00000000 \"\nb10101010 #\n0$\n0&\n0%\n#5\n1!\n1$\n1&\n#10\nb00001111 \"\n1%\n";

    const RECURSIVE_TEST_VCD: &str = "$date\n  2026-02-28\n$end\n$version\n  wavepeek-recursive-test\n$end\n$timescale 1ns $end\n$scope module top $end\n$var wire 1 ! clk $end\n$scope module cpu $end\n$var wire 1 \" valid $end\n$scope module core $end\n$var wire 1 # execute $end\n$upscope $end\n$upscope $end\n$scope module mem $end\n$var wire 1 $ ready $end\n$upscope $end\n$upscope $end\n$enddefinitions $end\n#0\n0!\n0\"\n0#\n0$\n#5\n1!\n1\"\n1#\n1$\n";

    #[test]
    fn open_and_read_metadata_from_vcd() {
        let fixture = write_fixture(TEST_VCD, "sample.vcd");

        let waveform = Waveform::open(fixture.path()).expect("fixture should open");
        let metadata = waveform.metadata().expect("metadata should be available");

        assert_eq!(metadata.time_unit, "1ns");
        assert_eq!(metadata.time_start, "0ns");
        assert_eq!(metadata.time_end, "10ns");
    }

    #[test]
    fn scopes_use_deterministic_depth_first_lexicographic_order_with_kind() {
        let fixture = write_fixture(TEST_VCD, "sample.vcd");

        let waveform = Waveform::open(fixture.path()).expect("fixture should open");
        let scopes = waveform.scopes_depth_first(Some(5));

        assert_eq!(
            scopes,
            vec![
                ScopeEntry {
                    path: "top".to_string(),
                    depth: 0,
                    kind: "module".to_string()
                },
                ScopeEntry {
                    path: "top.cpu".to_string(),
                    depth: 1,
                    kind: "module".to_string()
                },
                ScopeEntry {
                    path: "top.helper".to_string(),
                    depth: 1,
                    kind: "function".to_string()
                },
                ScopeEntry {
                    path: "top.mem".to_string(),
                    depth: 1,
                    kind: "module".to_string()
                },
            ]
        );
    }

    #[test]
    fn scopes_depth_first_none_includes_all_nested_depths() {
        let fixture = write_fixture(RECURSIVE_TEST_VCD, "recursive-sample.vcd");

        let waveform = Waveform::open(fixture.path()).expect("fixture should open");
        let bounded = waveform.scopes_depth_first(Some(1));
        let unbounded = waveform.scopes_depth_first(None);

        let bounded_paths = bounded
            .iter()
            .map(|entry| entry.path.clone())
            .collect::<Vec<_>>();
        let unbounded_paths = unbounded
            .iter()
            .map(|entry| entry.path.clone())
            .collect::<Vec<_>>();

        assert_eq!(bounded_paths, vec!["top", "top.cpu", "top.mem"]);
        assert_eq!(
            unbounded_paths,
            vec!["top", "top.cpu", "top.cpu.core", "top.mem"]
        );
    }

    #[test]
    fn signals_in_scope_are_sorted_and_preserve_parser_var_type_aliases() {
        let fixture = write_fixture(TEST_VCD, "sample.vcd");

        let waveform = Waveform::open(fixture.path()).expect("fixture should open");
        let signals = waveform
            .signals_in_scope("top")
            .expect("scope lookup should succeed");

        assert_eq!(signals.len(), 3);
        assert_eq!(signals[0].name, "cfg");
        assert_eq!(signals[0].path, "top.cfg");
        assert_eq!(signals[0].kind, "parameter");
        assert_eq!(signals[1].name, "clk");
        assert_eq!(signals[1].kind, "wire");
        assert_eq!(signals[2].name, "data");
        assert_eq!(signals[2].kind, "reg");
    }

    #[test]
    fn missing_scope_returns_scope_category_error() {
        let fixture = write_fixture(TEST_VCD, "sample.vcd");

        let waveform = Waveform::open(fixture.path()).expect("fixture should open");
        let error = waveform
            .signals_in_scope("top.nope")
            .expect_err("unknown scope should fail");

        assert_eq!(
            error.to_string(),
            "error: scope: scope 'top.nope' not found in dump"
        );
        assert_eq!(error.exit_code(), 1);
    }

    #[test]
    fn recursive_signals_in_scope_respect_depth_boundaries() {
        let fixture = write_fixture(RECURSIVE_TEST_VCD, "recursive-sample.vcd");

        let waveform = Waveform::open(fixture.path()).expect("fixture should open");
        let depth_0 = waveform
            .signals_in_scope_recursive("top", Some(0))
            .expect("depth-0 lookup should succeed");
        let depth_1 = waveform
            .signals_in_scope_recursive("top", Some(1))
            .expect("depth-1 lookup should succeed");
        let depth_2 = waveform
            .signals_in_scope_recursive("top", Some(2))
            .expect("depth-2 lookup should succeed");

        let depth_0_paths = depth_0
            .iter()
            .map(|entry| entry.path.clone())
            .collect::<Vec<_>>();
        let depth_1_paths = depth_1
            .iter()
            .map(|entry| entry.path.clone())
            .collect::<Vec<_>>();
        let depth_2_paths = depth_2
            .iter()
            .map(|entry| entry.path.clone())
            .collect::<Vec<_>>();

        assert_eq!(depth_0_paths, vec!["top.clk"]);
        assert_eq!(
            depth_1_paths,
            vec!["top.clk", "top.cpu.valid", "top.mem.ready"]
        );
        assert_eq!(
            depth_2_paths,
            vec![
                "top.clk",
                "top.cpu.valid",
                "top.cpu.core.execute",
                "top.mem.ready"
            ]
        );
    }

    #[test]
    fn recursive_signals_in_scope_none_depth_includes_all_nested_levels() {
        let fixture = write_fixture(RECURSIVE_TEST_VCD, "recursive-sample.vcd");

        let waveform = Waveform::open(fixture.path()).expect("fixture should open");
        let bounded = waveform
            .signals_in_scope_recursive("top", Some(1))
            .expect("bounded lookup should succeed");
        let unbounded = waveform
            .signals_in_scope_recursive("top", None)
            .expect("unbounded lookup should succeed");

        let bounded_paths = bounded
            .iter()
            .map(|entry| entry.path.clone())
            .collect::<Vec<_>>();
        let unbounded_paths = unbounded
            .iter()
            .map(|entry| entry.path.clone())
            .collect::<Vec<_>>();

        assert_eq!(
            bounded_paths,
            vec!["top.clk", "top.cpu.valid", "top.mem.ready"]
        );
        assert_eq!(
            unbounded_paths,
            vec![
                "top.clk",
                "top.cpu.valid",
                "top.cpu.core.execute",
                "top.mem.ready"
            ]
        );
    }

    #[test]
    fn recursive_signals_in_scope_are_deterministic_depth_first() {
        let fixture = write_fixture(RECURSIVE_TEST_VCD, "recursive-sample.vcd");

        let waveform = Waveform::open(fixture.path()).expect("fixture should open");
        let first = waveform
            .signals_in_scope_recursive("top", Some(2))
            .expect("first recursive lookup should succeed");
        let second = waveform
            .signals_in_scope_recursive("top", Some(2))
            .expect("second recursive lookup should succeed");

        assert_eq!(first, second);
        let ordered_paths = first
            .iter()
            .map(|entry| entry.path.clone())
            .collect::<Vec<_>>();
        assert_eq!(
            ordered_paths,
            vec![
                "top.clk",
                "top.cpu.valid",
                "top.cpu.core.execute",
                "top.mem.ready"
            ]
        );
    }

    #[test]
    fn open_missing_file_maps_to_file_error() {
        let error = Waveform::open(Path::new("/tmp/this-file-does-not-exist.vcd"))
            .expect_err("missing file should fail");

        assert!(error.to_string().starts_with("error: file: cannot open"));
        assert_eq!(error.exit_code(), 2);
    }

    #[test]
    fn parse_failures_map_to_file_error() {
        let fixture = write_fixture("not-a-waveform", "invalid.wave");

        let error = Waveform::open(fixture.path()).expect_err("invalid file should fail");

        assert!(error.to_string().starts_with("error: file: cannot parse"));
        assert_eq!(error.exit_code(), 2);
    }

    #[test]
    fn sample_signals_at_time_preserves_order_and_duplicates() {
        let fixture = write_fixture(TEST_VCD, "sample.vcd");

        let mut waveform = Waveform::open(fixture.path()).expect("fixture should open");
        let sampled = waveform
            .sample_signals_at_time(
                &[
                    "top.clk".to_string(),
                    "top.clk".to_string(),
                    "top.data".to_string(),
                ],
                10,
            )
            .expect("sampling should succeed");

        assert_eq!(
            sampled,
            vec![
                SampledSignal {
                    path: "top.clk".to_string(),
                    width: 1,
                    bits: "1".to_string()
                },
                SampledSignal {
                    path: "top.clk".to_string(),
                    width: 1,
                    bits: "1".to_string()
                },
                SampledSignal {
                    path: "top.data".to_string(),
                    width: 8,
                    bits: "00001111".to_string()
                },
            ]
        );
    }

    #[test]
    fn sample_signals_at_time_uses_latest_change_before_timestamp() {
        let fixture = write_fixture(TEST_VCD, "sample.vcd");

        let mut waveform = Waveform::open(fixture.path()).expect("fixture should open");
        let sampled = waveform
            .sample_signals_at_time(&["top.data".to_string()], 7)
            .expect("sampling should succeed");

        assert_eq!(sampled[0].width, 8);
        assert_eq!(sampled[0].bits, "00000000");
    }

    #[test]
    fn sample_signals_at_time_returns_signal_error_for_missing_path() {
        let fixture = write_fixture(TEST_VCD, "sample.vcd");

        let mut waveform = Waveform::open(fixture.path()).expect("fixture should open");
        let error = waveform
            .sample_signals_at_time(&["top.nope".to_string()], 10)
            .expect_err("missing signal should fail");

        assert_eq!(
            error.to_string(),
            "error: signal: signal 'top.nope' not found in dump"
        );
        assert_eq!(error.exit_code(), 1);
    }

    #[test]
    fn edge_classification_sv2023_matrix() {
        for (previous, current) in [("0", "1"), ("0", "x"), ("0", "z"), ("x", "1"), ("z", "1")] {
            let edge = classify_edge(previous, current);
            assert!(edge.posedge, "expected posedge for {previous}->{current}");
        }

        for (previous, current) in [("1", "0"), ("1", "x"), ("1", "z"), ("x", "0"), ("z", "0")] {
            let edge = classify_edge(previous, current);
            assert!(edge.negedge, "expected negedge for {previous}->{current}");
        }

        for previous in ["0", "1", "x", "z"] {
            for current in ["0", "1", "x", "z"] {
                let edge = classify_edge(previous, current);
                assert_eq!(edge.edge(), edge.posedge || edge.negedge);
            }
        }
    }

    #[test]
    fn edge_classification_ninestate_maps_to_x() {
        assert!(classify_edge("h", "1").posedge);
        assert!(classify_edge("1", "l").negedge);
        assert!(classify_edge("u", "0").negedge);
        assert!(classify_edge("0", "w").posedge);
        assert!(classify_edge("-", "1").posedge);
    }

    #[test]
    fn edge_detection_uses_lsb_only() {
        let msb_only = classify_edge("0001", "1001");
        assert!(!msb_only.edge());

        let lsb_flip = classify_edge("1000", "1001");
        assert!(lsb_flip.posedge);
    }

    #[test]
    fn delta_filter_initializes_without_prior_state() {
        let mut previous = vec![None];
        let current = vec![Some("1".to_string())];

        let emitted = should_emit_delta_and_update_baseline(&mut previous, &current);

        assert!(!emitted);
        assert_eq!(previous, vec![Some("1".to_string())]);
    }

    #[test]
    fn delta_filter_mixed_prior_state_emits_on_comparable_change() {
        let mut previous = vec![Some("0".to_string()), None];
        let current = vec![Some("1".to_string()), Some("1".to_string())];

        let emitted = should_emit_delta_and_update_baseline(&mut previous, &current);

        assert!(emitted);
        assert_eq!(previous, vec![Some("1".to_string()), Some("1".to_string())]);
    }

    fn write_fixture(contents: &str, filename: &str) -> NamedTempFile {
        let mut file = tempfile::Builder::new()
            .suffix(filename)
            .tempfile()
            .expect("tempfile should be created");
        file.write_all(contents.as_bytes())
            .expect("fixture should be written");
        file
    }
}
