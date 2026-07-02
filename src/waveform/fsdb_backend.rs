//! FSDB-backed waveform adapter implementation.

use std::cell::{Ref, RefCell};
use std::collections::{BTreeSet, HashMap};
use std::path::Path;
use std::time::{Duration, Instant};

use serde::Serialize;

use crate::error::WavepeekError;
use crate::expr::{ExprType, ExprTypeKind, SampledValue};

use super::fsdb_hierarchy::{FsdbHierarchyIndex, FsdbValueEncoding};
use super::fsdb_native::{self, FsdbNativeMetadata, FsdbReader, FsdbSignalSession};
use super::fsdb_time::{normalize_raw_time, parse_scale_unit};
use super::types::{
    ChangeCandidateCollectionMode, ExprResolvedSignal, ResolvedSignal, SampledSignalState,
    ScopeEntry, SignalEntry, SignalId, SignalOffsetData, WaveformMetadata,
};

#[derive(Debug)]
pub(super) struct FsdbBackend {
    reader: FsdbReader,
    raw_metadata: RefCell<Option<FsdbNativeMetadata>>,
    hierarchy: RefCell<Option<FsdbHierarchyIndex>>,
    signal_session: Option<FsdbSignalSession>,
    loaded_session_idcodes: BTreeSet<u64>,
    expr_sample_cache: HashMap<(SignalId, u64), SampledValue>,
    event_occurrence_cache: HashMap<(SignalId, u64), bool>,
    debug_stats: Option<FsdbDebugStats>,
}

#[derive(Debug, Default)]
struct FsdbDebugStats {
    expr_sample_cache_hits: usize,
    expr_sample_cache_misses: usize,
    expr_sample_uncached_ns: u64,
    event_occurrence_cache_hits: usize,
    event_occurrence_cache_misses: usize,
    event_occurrence_uncached_ns: u64,
    sample_resolved_calls: usize,
    sample_resolved_idcodes: usize,
    sample_resolved_native_ns: u64,
    collect_change_times_calls: usize,
    collect_change_times_idcodes: usize,
    collect_change_times_ns: u64,
    signal_session_reuses: usize,
    signal_session_opens: usize,
    signal_session_open_ns: u64,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct FsdbDebugStatsSnapshot {
    pub expr_sample_cache_hits: usize,
    pub expr_sample_cache_misses: usize,
    pub expr_sample_uncached_ns: u64,
    pub expr_sample_cache_len: usize,
    pub event_occurrence_cache_hits: usize,
    pub event_occurrence_cache_misses: usize,
    pub event_occurrence_uncached_ns: u64,
    pub event_occurrence_cache_len: usize,
    pub sample_resolved_calls: usize,
    pub sample_resolved_idcodes: usize,
    pub sample_resolved_native_ns: u64,
    pub collect_change_times_calls: usize,
    pub collect_change_times_idcodes: usize,
    pub collect_change_times_ns: u64,
    pub signal_session_reuses: usize,
    pub signal_session_opens: usize,
    pub signal_session_open_ns: u64,
    pub loaded_session_idcodes: usize,
}

impl FsdbBackend {
    pub(super) fn open(path: &Path) -> Result<Self, WavepeekError> {
        Ok(Self {
            reader: FsdbReader::open(path)?,
            raw_metadata: RefCell::new(None),
            hierarchy: RefCell::new(None),
            signal_session: None,
            loaded_session_idcodes: BTreeSet::new(),
            expr_sample_cache: HashMap::new(),
            event_occurrence_cache: HashMap::new(),
            debug_stats: fsdb_debug_stats_enabled().then(FsdbDebugStats::default),
        })
    }

    pub(super) fn probe(path: &Path) -> Result<bool, WavepeekError> {
        fsdb_native::probe(path)
    }

    pub(super) fn backend_name(&self) -> &'static str {
        "fsdb"
    }

    pub(super) fn format_name(&self) -> &'static str {
        "fsdb"
    }

    pub(super) fn metadata(&self) -> Result<WaveformMetadata, WavepeekError> {
        let metadata = self.raw_metadata()?;
        let unit = parse_scale_unit(metadata.scale_unit.as_str())?;
        Ok(WaveformMetadata {
            time_unit: format!("{}{}", unit.factor, unit.suffix),
            time_start: normalize_raw_time(metadata.time_start_raw, unit)?,
            time_end: normalize_raw_time(metadata.time_end_raw, unit)?,
        })
    }

    pub(super) fn scopes_depth_first(
        &self,
        max_depth: Option<usize>,
    ) -> Result<Vec<ScopeEntry>, WavepeekError> {
        Ok(self.hierarchy()?.scopes_depth_first(max_depth))
    }

    pub(super) fn signals_in_scope(
        &self,
        scope_path: &str,
    ) -> Result<Vec<SignalEntry>, WavepeekError> {
        self.hierarchy()?.signals_in_scope(scope_path)
    }

    pub(super) fn signals_in_scope_recursive(
        &self,
        scope_path: &str,
        max_depth: Option<usize>,
    ) -> Result<Vec<SignalEntry>, WavepeekError> {
        self.hierarchy()?
            .signals_in_scope_recursive(scope_path, max_depth)
    }

    pub(super) fn resolve_signals(
        &self,
        canonical_paths: &[String],
    ) -> Result<Vec<ResolvedSignal>, WavepeekError> {
        let hierarchy = self.hierarchy()?;
        canonical_paths
            .iter()
            .map(|path| hierarchy.resolve_signal(path.as_str()))
            .collect()
    }

    pub(super) fn resolve_expr_signal(
        &self,
        canonical_path: &str,
    ) -> Result<ExprResolvedSignal, WavepeekError> {
        self.hierarchy()?.resolve_expr_signal(canonical_path)
    }

    pub(super) fn resolve_expr_signals(
        &self,
        canonical_paths: &[String],
    ) -> Result<Vec<ExprResolvedSignal>, WavepeekError> {
        let hierarchy = self.hierarchy()?;
        canonical_paths
            .iter()
            .map(|path| hierarchy.resolve_expr_signal(path.as_str()))
            .collect()
    }

    pub(super) fn previous_sample_time(&self, raw_time: u64) -> Option<u64> {
        let metadata = self.raw_metadata().ok()?;
        if raw_time <= metadata.time_start_raw {
            None
        } else {
            raw_time.checked_sub(1)
        }
    }

    pub(super) fn sample_resolved_optional(
        &mut self,
        resolved: &[ResolvedSignal],
        query_time_raw: u64,
    ) -> Result<Vec<SampledSignalState>, WavepeekError> {
        if resolved.is_empty() {
            return Ok(Vec::new());
        }

        self.raw_metadata()?;

        {
            let hierarchy = self.hierarchy()?;
            for signal in resolved {
                match hierarchy.signal_value_encoding(signal.path.as_str())? {
                    FsdbValueEncoding::BitVector => {}
                    FsdbValueEncoding::Unsupported | FsdbValueEncoding::DatatypeCandidate => {
                        return Err(unsupported_signal_value_encoding(signal.path.as_str()));
                    }
                }
            }
        }

        let idcodes = resolved
            .iter()
            .map(|signal| signal.id.as_u64())
            .collect::<Vec<_>>();
        let started_at = self.debug_stats.is_some().then(Instant::now);
        let samples = {
            let session = self.ensure_signal_session(&idcodes)?;
            session.sample_signal_values(&idcodes, query_time_raw)?
        };
        if let Some(stats) = self.debug_stats.as_mut() {
            stats.sample_resolved_calls += 1;
            stats.sample_resolved_idcodes += idcodes.len();
            if let Some(started_at) = started_at {
                stats.sample_resolved_native_ns += duration_ns(started_at.elapsed());
            }
        }
        if samples.len() != resolved.len() {
            return Err(WavepeekError::Internal(
                "FSDB Reader returned the wrong number of sampled values".to_string(),
            ));
        }

        resolved
            .iter()
            .zip(samples)
            .map(|(signal, sample)| {
                if sample.idcode != signal.id.as_u64() {
                    return Err(WavepeekError::Internal(
                        "FSDB Reader returned sampled values out of order".to_string(),
                    ));
                }
                if let Some(bits) = sample.bits.as_ref()
                    && bits.len() != sample.bit_width as usize
                {
                    return Err(WavepeekError::File(format!(
                        "FSDB Reader returned {} bits for {}-bit signal '{}'",
                        bits.len(),
                        sample.bit_width,
                        signal.path
                    )));
                }
                Ok(SampledSignalState {
                    path: signal.path.clone(),
                    width: sample.bit_width,
                    bits: sample.bits,
                })
            })
            .collect()
    }

    pub(super) fn sample_expr_value(
        &mut self,
        resolved: &ExprResolvedSignal,
        query_time_raw: u64,
    ) -> Result<SampledValue, WavepeekError> {
        let cache_key = (resolved.id, query_time_raw);
        if let Some(value) = self.expr_sample_cache.get(&cache_key) {
            if let Some(stats) = self.debug_stats.as_mut() {
                stats.expr_sample_cache_hits += 1;
            }
            return Ok(value.clone());
        }

        if let Some(stats) = self.debug_stats.as_mut() {
            stats.expr_sample_cache_misses += 1;
        }
        let started_at = self.debug_stats.is_some().then(Instant::now);
        let value = self.sample_expr_value_uncached(resolved, query_time_raw)?;
        if let Some(stats) = self.debug_stats.as_mut()
            && let Some(started_at) = started_at
        {
            stats.expr_sample_uncached_ns += duration_ns(started_at.elapsed());
        }
        self.expr_sample_cache.insert(cache_key, value.clone());
        Ok(value)
    }

    pub(super) fn expr_event_occurred(
        &mut self,
        resolved: &ExprResolvedSignal,
        query_time_raw: u64,
    ) -> Result<bool, WavepeekError> {
        if !matches!(&resolved.expr_type.kind, ExprTypeKind::Event) {
            return Err(WavepeekError::Internal(format!(
                "signal '{}' is not a raw event",
                resolved.path
            )));
        }

        let cache_key = (resolved.id, query_time_raw);
        if let Some(occurred) = self.event_occurrence_cache.get(&cache_key) {
            if let Some(stats) = self.debug_stats.as_mut() {
                stats.event_occurrence_cache_hits += 1;
            }
            return Ok(*occurred);
        }

        if let Some(stats) = self.debug_stats.as_mut() {
            stats.event_occurrence_cache_misses += 1;
        }
        let started_at = self.debug_stats.is_some().then(Instant::now);
        self.raw_metadata()?;
        let idcodes = [resolved.id.as_u64()];
        let occurred = {
            let session = self.ensure_signal_session(&idcodes)?;
            session.signal_event_occurred(resolved.id.as_u64(), query_time_raw)?
        };
        if let Some(stats) = self.debug_stats.as_mut()
            && let Some(started_at) = started_at
        {
            stats.event_occurrence_uncached_ns += duration_ns(started_at.elapsed());
        }
        self.event_occurrence_cache.insert(cache_key, occurred);
        Ok(occurred)
    }

    pub(super) fn indexed_timestamps(&self) -> Option<&[u64]> {
        None
    }

    pub(super) fn indexed_signal_offset_at(
        &self,
        _id: SignalId,
        _time_table_idx: u32,
    ) -> Option<Option<SignalOffsetData>> {
        None
    }

    pub(super) fn decode_indexed_signal_at(
        &self,
        _resolved: &ResolvedSignal,
        _time_table_idx: u32,
    ) -> Option<SampledSignalState> {
        None
    }

    pub(super) fn ensure_indexed_signals_loaded(&mut self, _ids: &[SignalId]) -> bool {
        false
    }

    pub(super) fn collect_change_times_with_mode(
        &mut self,
        resolved: &[ResolvedSignal],
        from_raw: u64,
        to_raw: u64,
        _mode: ChangeCandidateCollectionMode,
    ) -> Result<Vec<u64>, WavepeekError> {
        self.raw_metadata()?;
        let idcodes = resolved
            .iter()
            .map(|signal| signal.id.as_u64())
            .collect::<Vec<_>>();
        if idcodes.is_empty() || from_raw > to_raw {
            return Ok(Vec::new());
        }
        let started_at = self.debug_stats.is_some().then(Instant::now);
        let times = {
            let session = self.ensure_signal_session(&idcodes)?;
            session.collect_signal_change_times(idcodes.as_slice(), from_raw, to_raw)?
        };
        if let Some(stats) = self.debug_stats.as_mut() {
            stats.collect_change_times_calls += 1;
            stats.collect_change_times_idcodes += idcodes.len();
            if let Some(started_at) = started_at {
                stats.collect_change_times_ns += duration_ns(started_at.elapsed());
            }
        }
        Ok(times)
    }

    pub(super) fn collect_expr_candidate_times_with_mode(
        &mut self,
        resolved: &[ExprResolvedSignal],
        from_raw: u64,
        to_raw: u64,
        _mode: ChangeCandidateCollectionMode,
    ) -> Result<Vec<u64>, WavepeekError> {
        self.raw_metadata()?;
        self.validate_expr_values_supported(resolved)?;
        let idcodes = resolved
            .iter()
            .map(|signal| signal.id.as_u64())
            .collect::<Vec<_>>();
        if idcodes.is_empty() || from_raw > to_raw {
            return Ok(Vec::new());
        }
        let started_at = self.debug_stats.is_some().then(Instant::now);
        let times = {
            let session = self.ensure_signal_session(&idcodes)?;
            session.collect_signal_change_times(idcodes.as_slice(), from_raw, to_raw)?
        };
        if let Some(stats) = self.debug_stats.as_mut() {
            stats.collect_change_times_calls += 1;
            stats.collect_change_times_idcodes += idcodes.len();
            if let Some(started_at) = started_at {
                stats.collect_change_times_ns += duration_ns(started_at.elapsed());
            }
        }
        Ok(times)
    }

    pub(super) fn should_use_streaming_candidate_collection(
        &self,
        _signal_count: usize,
        _from_raw: u64,
        _to_raw: u64,
        _mode: ChangeCandidateCollectionMode,
    ) -> bool {
        false
    }

    fn ensure_signal_session(
        &mut self,
        idcodes: &[u64],
    ) -> Result<&FsdbSignalSession, WavepeekError> {
        if idcodes.is_empty() {
            return Err(WavepeekError::Internal(
                "FSDB signal session requested without idcodes".to_string(),
            ));
        }

        let requested = idcodes.iter().copied().collect::<BTreeSet<_>>();
        let can_reuse =
            self.signal_session.is_some() && requested.is_subset(&self.loaded_session_idcodes);
        if can_reuse {
            if let Some(stats) = self.debug_stats.as_mut() {
                stats.signal_session_reuses += 1;
            }
            return self.signal_session.as_ref().ok_or_else(|| {
                WavepeekError::Internal("FSDB signal session cache disappeared".to_string())
            });
        }

        let mut new_loaded = self.loaded_session_idcodes.clone();
        new_loaded.extend(requested.iter().copied());
        let new_idcodes = new_loaded.iter().copied().collect::<Vec<_>>();

        self.signal_session = None;
        self.loaded_session_idcodes.clear();
        let started_at = self.debug_stats.is_some().then(Instant::now);
        let session = self.reader.open_signal_session(&new_idcodes)?;
        if let Some(stats) = self.debug_stats.as_mut() {
            stats.signal_session_opens += 1;
            if let Some(started_at) = started_at {
                stats.signal_session_open_ns += duration_ns(started_at.elapsed());
            }
        }
        self.loaded_session_idcodes = new_loaded;
        self.signal_session = Some(session);
        Ok(self
            .signal_session
            .as_ref()
            .expect("FSDB signal session was just opened"))
    }

    pub(super) fn debug_stats_snapshot(&self) -> Option<FsdbDebugStatsSnapshot> {
        let stats = self.debug_stats.as_ref()?;
        Some(FsdbDebugStatsSnapshot {
            expr_sample_cache_hits: stats.expr_sample_cache_hits,
            expr_sample_cache_misses: stats.expr_sample_cache_misses,
            expr_sample_uncached_ns: stats.expr_sample_uncached_ns,
            expr_sample_cache_len: self.expr_sample_cache.len(),
            event_occurrence_cache_hits: stats.event_occurrence_cache_hits,
            event_occurrence_cache_misses: stats.event_occurrence_cache_misses,
            event_occurrence_uncached_ns: stats.event_occurrence_uncached_ns,
            event_occurrence_cache_len: self.event_occurrence_cache.len(),
            sample_resolved_calls: stats.sample_resolved_calls,
            sample_resolved_idcodes: stats.sample_resolved_idcodes,
            sample_resolved_native_ns: stats.sample_resolved_native_ns,
            collect_change_times_calls: stats.collect_change_times_calls,
            collect_change_times_idcodes: stats.collect_change_times_idcodes,
            collect_change_times_ns: stats.collect_change_times_ns,
            signal_session_reuses: stats.signal_session_reuses,
            signal_session_opens: stats.signal_session_opens,
            signal_session_open_ns: stats.signal_session_open_ns,
            loaded_session_idcodes: self.loaded_session_idcodes.len(),
        })
    }

    fn raw_metadata(&self) -> Result<FsdbNativeMetadata, WavepeekError> {
        if let Some(metadata) = self.raw_metadata.borrow().as_ref() {
            return Ok(metadata.clone());
        }
        let metadata = self.reader.metadata()?;
        *self.raw_metadata.borrow_mut() = Some(metadata.clone());
        Ok(metadata)
    }

    pub(super) fn validate_expr_values_supported(
        &self,
        resolved: &[ExprResolvedSignal],
    ) -> Result<(), WavepeekError> {
        let hierarchy = self.hierarchy()?;
        for signal in resolved {
            match signal.expr_type.kind {
                ExprTypeKind::Event => {}
                ExprTypeKind::Real | ExprTypeKind::String => {
                    return Err(unsupported_value_sampling(signal.path.as_str()));
                }
                ExprTypeKind::BitVector | ExprTypeKind::IntegerLike(_) | ExprTypeKind::EnumCore => {
                    match hierarchy.signal_value_encoding(signal.path.as_str())? {
                        FsdbValueEncoding::BitVector => {}
                        FsdbValueEncoding::Unsupported | FsdbValueEncoding::DatatypeCandidate => {
                            return Err(unsupported_value_sampling(signal.path.as_str()));
                        }
                    }
                }
            }
        }
        Ok(())
    }

    fn sample_expr_value_uncached(
        &mut self,
        resolved: &ExprResolvedSignal,
        query_time_raw: u64,
    ) -> Result<SampledValue, WavepeekError> {
        match resolved.expr_type.kind {
            ExprTypeKind::Real | ExprTypeKind::String => {
                return Err(unsupported_value_sampling(resolved.path.as_str()));
            }
            ExprTypeKind::Event => {
                return Err(WavepeekError::Signal(format!(
                    "raw event signal '{}' cannot be sampled as an expression value",
                    resolved.path
                )));
            }
            ExprTypeKind::BitVector | ExprTypeKind::IntegerLike(_) | ExprTypeKind::EnumCore => {}
        }

        let signal = ResolvedSignal {
            path: resolved.path.clone(),
            id: resolved.id,
            width: resolved.expr_type.width.max(1),
        };
        let mut samples =
            self.sample_resolved_optional(std::slice::from_ref(&signal), query_time_raw)?;
        let sample = samples.pop().ok_or_else(|| {
            WavepeekError::Internal("FSDB expression sampling returned no sample row".to_string())
        })?;
        let label = sample
            .bits
            .as_deref()
            .and_then(|bits| enum_label_for_bits(&resolved.expr_type, bits));
        Ok(SampledValue::Integral {
            bits: sample.bits,
            label,
        })
    }

    fn hierarchy(&self) -> Result<Ref<'_, FsdbHierarchyIndex>, WavepeekError> {
        if self.hierarchy.borrow().is_none() {
            let hierarchy = self.reader.read_hierarchy()?;
            *self.hierarchy.borrow_mut() = Some(hierarchy);
        }
        Ok(Ref::map(self.hierarchy.borrow(), |hierarchy| {
            hierarchy
                .as_ref()
                .expect("hierarchy was initialized before immutable borrow")
        }))
    }
}

fn fsdb_debug_stats_enabled() -> bool {
    std::env::var("DEBUG").as_deref() == Ok("1")
}

fn duration_ns(duration: Duration) -> u64 {
    u64::try_from(duration.as_nanos()).unwrap_or(u64::MAX)
}

pub(super) fn unsupported_value_sampling(path: &str) -> WavepeekError {
    WavepeekError::Signal(format!(
        "signal '{path}' has unsupported FSDB expression value encoding"
    ))
}

fn enum_label_for_bits(expr_type: &ExprType, bits: &str) -> Option<String> {
    expr_type
        .enum_labels
        .as_deref()?
        .iter()
        .find(|label| label.bits == bits)
        .map(|label| label.name.clone())
}

fn unsupported_signal_value_encoding(path: &str) -> WavepeekError {
    WavepeekError::Signal(format!(
        "signal '{path}' has unsupported non-bit-vector encoding"
    ))
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};
    use std::process::{Command, Stdio};

    use super::*;

    #[test]
    fn fsdb_expr_event_occurred_rejects_non_event_signal() {
        let fixture = GeneratedFsdbFixture::from_vcd("change_property_events.vcd");
        let mut backend = FsdbBackend::open(fixture.path()).expect("FSDB fixture should open");

        let tick = backend
            .resolve_expr_signal("top.tick")
            .expect("raw event signal should resolve");
        assert!(matches!(&tick.expr_type.kind, ExprTypeKind::Event));
        assert!(
            backend
                .expr_event_occurred(&tick, 10)
                .expect("raw event should be queryable")
        );

        let armed = backend
            .resolve_expr_signal("top.armed")
            .expect("ordinary signal should resolve");
        assert!(!matches!(&armed.expr_type.kind, ExprTypeKind::Event));
        let error = backend
            .expr_event_occurred(&armed, 10)
            .expect_err("ordinary signals must not be queryable as raw events");
        assert!(
            error
                .to_string()
                .contains("signal 'top.armed' is not a raw event"),
            "unexpected error: {error}"
        );
    }

    struct GeneratedFsdbFixture {
        _dir: tempfile::TempDir,
        path: PathBuf,
    }

    impl GeneratedFsdbFixture {
        fn from_vcd(name: &str) -> Self {
            let dir = tempfile::tempdir().expect("tempdir should be created");
            let source = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("tests")
                .join("fixtures")
                .join("hand")
                .join(name);
            let path = dir.path().join(name.replace(".vcd", ".fsdb"));
            let converter_output = Command::new("vcd2fsdb")
                .current_dir(dir.path())
                .arg(&source)
                .arg("-o")
                .arg(&path)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .output()
                .expect("vcd2fsdb should be available from the Verdi environment");
            assert!(
                converter_output.status.success(),
                "vcd2fsdb should convert {}; stdout:\n{}\nstderr:\n{}",
                source.display(),
                String::from_utf8_lossy(&converter_output.stdout),
                String::from_utf8_lossy(&converter_output.stderr)
            );
            Self { _dir: dir, path }
        }

        fn path(&self) -> &Path {
            &self.path
        }
    }
}
