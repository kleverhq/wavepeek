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

    pub fn scopes_depth_first(&self, max_depth: usize) -> Vec<ScopeEntry> {
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
            .map(|var_ref| {
                let var = &hierarchy[var_ref];
                SignalEntry {
                    name: var.name(hierarchy).to_string(),
                    path: var.full_name(hierarchy),
                    kind: var_type_alias(var.var_type()).to_string(),
                    width: var.length(),
                }
            })
            .collect::<Vec<_>>();

        signals.sort_by(|lhs, rhs| {
            lhs.name
                .cmp(&rhs.name)
                .then_with(|| lhs.path.cmp(&rhs.path))
        });
        Ok(signals)
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

    Ok(hierarchy[var_ref].signal_ref())
}

fn collect_scope_entries(
    hierarchy: &wellen::Hierarchy,
    scope_ref: ScopeRef,
    depth: usize,
    max_depth: usize,
    entries: &mut Vec<ScopeEntry>,
) {
    if depth > max_depth {
        return;
    }

    let scope = &hierarchy[scope_ref];
    entries.push(ScopeEntry {
        path: scope.full_name(hierarchy),
        depth,
        kind: scope_type_alias(scope.scope_type()).to_string(),
    });

    if depth == max_depth {
        return;
    }

    let mut children: Vec<ScopeRef> = scope.scopes(hierarchy).collect();
    sort_scope_refs(hierarchy, &mut children);
    for child in children {
        collect_scope_entries(hierarchy, child, depth + 1, max_depth, entries);
    }
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

#[cfg(test)]
mod tests {
    use std::io::Write;
    use std::path::Path;

    use tempfile::NamedTempFile;

    use super::{SampledSignal, ScopeEntry, Waveform};

    const TEST_VCD: &str = "$date\n  today\n$end\n$version\n  wavepeek-test\n$end\n$timescale 1ns $end\n$scope module top $end\n$var wire 1 ! clk $end\n$var reg 8 \" data $end\n$var parameter 8 # cfg $end\n$scope module cpu $end\n$var wire 1 $ valid $end\n$upscope $end\n$scope function helper $end\n$var wire 1 & helper_flag $end\n$upscope $end\n$scope module mem $end\n$var wire 1 % ready $end\n$upscope $end\n$upscope $end\n$enddefinitions $end\n#0\n0!\nb00000000 \"\nb10101010 #\n0$\n0&\n0%\n#5\n1!\n1$\n1&\n#10\nb00001111 \"\n1%\n";

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
        let scopes = waveform.scopes_depth_first(5);

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
