//! Waveform adapter used by the engine layer.
//!
//! Canonical path policy in M2:
//! - Paths are emitted as dot-separated full hierarchy paths.
//! - Scope and signal names are preserved exactly as provided by the parser.
//! - No additional escaping or normalization pass is applied.

#![allow(dead_code)]

use std::cmp::Ordering;
use std::path::Path;

use wellen::{ScopeRef, Timescale, TimescaleUnit, VarType, simple};

use crate::error::WavepeekError;

#[derive(Debug)]
pub struct Waveform {
    inner: simple::Waveform,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WaveformMetadata {
    pub time_unit: String,
    pub time_precision: String,
    pub time_start: String,
    pub time_end: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScopeEntry {
    pub path: String,
    pub depth: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignalEntry {
    pub name: String,
    pub path: String,
    pub kind: SignalKind,
    pub width: Option<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignalKind {
    Wire,
    Reg,
    Logic,
    Integer,
    Real,
    String,
    Unknown,
}

impl SignalKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Wire => "wire",
            Self::Reg => "reg",
            Self::Logic => "logic",
            Self::Integer => "integer",
            Self::Real => "real",
            Self::String => "string",
            Self::Unknown => "unknown",
        }
    }
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
        let time_precision = time_unit.clone();

        let time_table = self.inner.time_table();
        let time_start = time_table.first().copied().unwrap_or(0);
        let time_end = time_table.last().copied().unwrap_or(time_start);

        Ok(WaveformMetadata {
            time_unit,
            time_precision,
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
                    kind: map_signal_kind(var.var_type()),
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
}

fn collect_scope_entries(
    hierarchy: &wellen::Hierarchy,
    scope_ref: ScopeRef,
    depth: usize,
    max_depth: usize,
    entries: &mut Vec<ScopeEntry>,
) {
    let scope = &hierarchy[scope_ref];
    entries.push(ScopeEntry {
        path: scope.full_name(hierarchy),
        depth,
    });

    if depth >= max_depth {
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

fn map_signal_kind(var_type: VarType) -> SignalKind {
    match var_type {
        VarType::Wire
        | VarType::Tri
        | VarType::TriAnd
        | VarType::TriOr
        | VarType::TriReg
        | VarType::Tri0
        | VarType::Tri1
        | VarType::WAnd
        | VarType::WOr
        | VarType::Supply0
        | VarType::Supply1
        | VarType::Port => SignalKind::Wire,
        VarType::Reg => SignalKind::Reg,
        VarType::Logic => SignalKind::Logic,
        VarType::Integer
        | VarType::Int
        | VarType::ShortInt
        | VarType::LongInt
        | VarType::Byte
        | VarType::Time => SignalKind::Integer,
        VarType::Real | VarType::RealTime | VarType::RealParameter | VarType::ShortReal => {
            SignalKind::Real
        }
        VarType::String => SignalKind::String,
        _ => SignalKind::Unknown,
    }
}

#[cfg(test)]
mod tests {
    use std::io::Write;
    use std::path::Path;

    use tempfile::NamedTempFile;

    use super::{ScopeEntry, SignalKind, Waveform};

    const TEST_VCD: &str = "$date\n  today\n$end\n$version\n  wavepeek-test\n$end\n$timescale 1ns $end\n$scope module top $end\n$var wire 1 ! clk $end\n$var reg 8 \" data $end\n$var parameter 8 # cfg $end\n$scope module cpu $end\n$var wire 1 $ valid $end\n$upscope $end\n$scope module mem $end\n$var wire 1 % ready $end\n$upscope $end\n$upscope $end\n$enddefinitions $end\n#0\n0!\nb00000000 \"\nb10101010 #\n0$\n0%\n#5\n1!\n1$\n#10\nb00001111 \"\n1%\n";

    #[test]
    fn open_and_read_metadata_from_vcd() {
        let fixture = write_fixture(TEST_VCD, "sample.vcd");

        let waveform = Waveform::open(fixture.path()).expect("fixture should open");
        let metadata = waveform.metadata().expect("metadata should be available");

        assert_eq!(metadata.time_unit, "1ns");
        assert_eq!(metadata.time_precision, "1ns");
        assert_eq!(metadata.time_start, "0ns");
        assert_eq!(metadata.time_end, "10ns");
    }

    #[test]
    fn scopes_use_deterministic_depth_first_lexicographic_order() {
        let fixture = write_fixture(TEST_VCD, "sample.vcd");

        let waveform = Waveform::open(fixture.path()).expect("fixture should open");
        let scopes = waveform.scopes_depth_first(5);

        assert_eq!(
            scopes,
            vec![
                ScopeEntry {
                    path: "top".to_string(),
                    depth: 0
                },
                ScopeEntry {
                    path: "top.cpu".to_string(),
                    depth: 1
                },
                ScopeEntry {
                    path: "top.mem".to_string(),
                    depth: 1
                },
            ]
        );
    }

    #[test]
    fn signals_in_scope_are_sorted_and_use_unknown_fallback_kind() {
        let fixture = write_fixture(TEST_VCD, "sample.vcd");

        let waveform = Waveform::open(fixture.path()).expect("fixture should open");
        let signals = waveform
            .signals_in_scope("top")
            .expect("scope lookup should succeed");

        assert_eq!(signals.len(), 3);
        assert_eq!(signals[0].name, "cfg");
        assert_eq!(signals[0].path, "top.cfg");
        assert_eq!(signals[0].kind, SignalKind::Unknown);
        assert_eq!(signals[1].name, "clk");
        assert_eq!(signals[1].kind, SignalKind::Wire);
        assert_eq!(signals[2].name, "data");
        assert_eq!(signals[2].kind, SignalKind::Reg);
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
