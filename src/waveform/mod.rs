//! Waveform adapter used by the engine layer.
//!
//! Canonical path policy in M2:
//! - Paths are emitted as dot-separated full hierarchy paths.
//! - Scope and signal names are preserved exactly as provided by the parser.
//! - No additional escaping or normalization pass is applied.

#[allow(dead_code)]
pub(crate) mod expr_host;

use std::cmp::Ordering;
use std::collections::BTreeSet;
use std::collections::HashMap;
use std::collections::HashSet;
use std::io::BufReader;
use std::path::Path;
use std::path::PathBuf;

use wellen::{ScopeRef, ScopeType, SignalRef, Timescale, TimescaleUnit, VarType, simple};

use crate::error::WavepeekError;
use crate::expr::{
    EnumLabelInfo, ExprStorage, ExprType, ExprTypeKind, IntegerLikeKind, SampledValue,
};

const STREAM_THRESHOLD_WORK: usize = 20_000;

#[derive(Debug)]
pub struct Waveform {
    inner: simple::Waveform,
    source_path: PathBuf,
    file_format: wellen::FileFormat,
    loaded_signals: HashSet<SignalRef>,
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
    pub signal_ref: SignalRef,
    pub width: u32,
}

#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub(crate) struct ExprResolvedSignal {
    pub path: String,
    pub signal_ref: SignalRef,
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
    fn new(start: usize, elements: u16) -> Self {
        Self { start, elements }
    }
}

impl PartialEq for SignalOffsetData {
    fn eq(&self, other: &Self) -> bool {
        self.start == other.start && self.elements == other.elements
    }
}

impl Eq for SignalOffsetData {}

impl Waveform {
    pub fn open(path: &Path) -> Result<Self, WavepeekError> {
        if !path.exists() {
            return Err(WavepeekError::File(format!(
                "cannot open '{}': No such file or directory",
                path.display()
            )));
        }

        let source_path = path.to_path_buf();
        let file = std::fs::File::open(path).map_err(|error| {
            WavepeekError::File(format!("cannot open '{}': {error}", path.display()))
        })?;
        let mut reader = BufReader::new(file);
        let file_format = wellen::viewers::detect_file_format(&mut reader);

        let inner = simple::read(path).map_err(|error| map_wellen_error(path, error))?;
        Ok(Self {
            inner,
            source_path,
            file_format,
            loaded_signals: HashSet::new(),
        })
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
        let (unique_paths, projection) = duplicate_preserving_projection(canonical_paths);
        let resolved = self.resolve_signals(&unique_paths)?;
        let sampled_unique = self.sample_resolved_optional(&resolved, query_time_raw)?;

        let sampled = projection
            .iter()
            .map(|unique_idx| sampled_unique[*unique_idx].clone())
            .collect::<Vec<_>>();

        sampled
            .into_iter()
            .map(|entry| {
                let bits = entry.bits.ok_or_else(|| {
                    WavepeekError::Signal(format!(
                        "signal '{}' has no value at or before requested time",
                        entry.path
                    ))
                })?;
                Ok(SampledSignal {
                    path: entry.path,
                    width: entry.width,
                    bits,
                })
            })
            .collect()
    }

    pub fn timestamps_raw_slice(&self) -> &[u64] {
        self.inner.time_table()
    }

    pub fn resolve_signals(
        &self,
        canonical_paths: &[String],
    ) -> Result<Vec<ResolvedSignal>, WavepeekError> {
        let hierarchy = self.inner.hierarchy();
        canonical_paths
            .iter()
            .map(|path| {
                let (signal_ref, width) = resolve_signal_ref_with_width(hierarchy, path.as_str())?;
                Ok(ResolvedSignal {
                    path: path.clone(),
                    signal_ref,
                    width,
                })
            })
            .collect()
    }

    #[allow(dead_code)]
    pub(crate) fn resolve_expr_signal(
        &self,
        canonical_path: &str,
    ) -> Result<ExprResolvedSignal, WavepeekError> {
        let hierarchy = self.inner.hierarchy();
        let var_ref = resolve_var_ref(hierarchy, canonical_path)?;
        let var = &hierarchy[var_ref];
        let expr_type = expr_type_from_var(hierarchy, var, canonical_path)?;
        Ok(ExprResolvedSignal {
            path: canonical_path.to_string(),
            signal_ref: var.signal_ref(),
            expr_type,
        })
    }

    pub(crate) fn resolve_expr_signals(
        &self,
        canonical_paths: &[String],
    ) -> Result<Vec<ExprResolvedSignal>, WavepeekError> {
        canonical_paths
            .iter()
            .map(|path| self.resolve_expr_signal(path.as_str()))
            .collect()
    }

    pub fn sample_resolved_optional(
        &mut self,
        resolved: &[ResolvedSignal],
        query_time_raw: u64,
    ) -> Result<Vec<SampledSignalState>, WavepeekError> {
        if resolved.is_empty() {
            return Ok(Vec::new());
        }

        let time_table = self.inner.time_table();
        let time_table_idx =
            floor_time_table_index(time_table, query_time_raw).ok_or_else(|| {
                WavepeekError::Internal("query time is before first dump timestamp".to_string())
            })?;

        let time_table_idx = u32::try_from(time_table_idx).map_err(|_| {
            WavepeekError::Internal("time table index exceeds u32 range".to_string())
        })?;

        let signal_refs = resolved
            .iter()
            .map(|signal| signal.signal_ref)
            .collect::<Vec<_>>();
        self.ensure_signals_loaded(&signal_refs);

        let mut sampled = Vec::with_capacity(resolved.len());
        for signal in resolved {
            sampled.push(self.decode_signal_at_index(signal, time_table_idx)?);
        }

        Ok(sampled)
    }

    #[allow(dead_code)]
    pub(crate) fn sample_expr_value(
        &mut self,
        resolved: &ExprResolvedSignal,
        query_time_raw: u64,
    ) -> Result<SampledValue, WavepeekError> {
        if matches!(&resolved.expr_type.kind, ExprTypeKind::Event) {
            return Err(WavepeekError::Internal(format!(
                "signal '{}' is a raw event and cannot be sampled as a value",
                resolved.path
            )));
        }

        let Some(time_table_idx) = floor_time_table_index(self.inner.time_table(), query_time_raw)
        else {
            return Ok(empty_sampled_value(&resolved.expr_type));
        };
        let time_table_idx = u32::try_from(time_table_idx).map_err(|_| {
            WavepeekError::Internal("time table index exceeds u32 range".to_string())
        })?;

        self.ensure_signals_loaded(&[resolved.signal_ref]);
        let loaded = self.inner.get_signal(resolved.signal_ref).ok_or_else(|| {
            WavepeekError::Internal(format!(
                "signal '{}' could not be loaded from waveform backend",
                resolved.path
            ))
        })?;

        let Some(offset) = loaded.get_offset(time_table_idx) else {
            return Ok(empty_sampled_value(&resolved.expr_type));
        };

        let value = loaded.get_value_at(&offset, offset.elements - 1);
        match value {
            wellen::SignalValue::Event => Err(WavepeekError::Internal(format!(
                "signal '{}' produced event data through value sampling",
                resolved.path
            ))),
            wellen::SignalValue::Binary(_, _)
            | wellen::SignalValue::FourValue(_, _)
            | wellen::SignalValue::NineValue(_, _) => {
                let bits = value.to_bit_string().ok_or_else(|| {
                    WavepeekError::Internal(format!(
                        "failed to convert value for signal '{}' to bit string",
                        resolved.path
                    ))
                })?;
                Ok(SampledValue::Integral {
                    label: enum_label_for_bits(&resolved.expr_type, bits.as_str()),
                    bits: Some(bits),
                })
            }
            wellen::SignalValue::String(raw) => Ok(SampledValue::String {
                value: Some(raw.to_string()),
            }),
            wellen::SignalValue::Real(value) => Ok(SampledValue::Real { value: Some(value) }),
        }
    }

    #[allow(dead_code)]
    pub(crate) fn expr_event_occurred(
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

        let Ok(time_table_idx) = self.inner.time_table().binary_search(&query_time_raw) else {
            return Ok(false);
        };
        let time_table_idx = u32::try_from(time_table_idx).map_err(|_| {
            WavepeekError::Internal("time table index exceeds u32 range".to_string())
        })?;

        self.ensure_signals_loaded(&[resolved.signal_ref]);
        let loaded = self.inner.get_signal(resolved.signal_ref).ok_or_else(|| {
            WavepeekError::Internal(format!(
                "signal '{}' could not be loaded from waveform backend",
                resolved.path
            ))
        })?;
        Ok(loaded.time_indices().binary_search(&time_table_idx).is_ok())
    }

    pub fn signal_offset_at_index(
        &self,
        signal_ref: SignalRef,
        time_table_idx: u32,
    ) -> Option<SignalOffsetData> {
        let loaded = self.inner.get_signal(signal_ref)?;
        loaded
            .get_offset(time_table_idx)
            .map(|offset| SignalOffsetData::new(offset.start, offset.elements))
    }

    pub fn decode_signal_at_index(
        &self,
        resolved: &ResolvedSignal,
        time_table_idx: u32,
    ) -> Result<SampledSignalState, WavepeekError> {
        let loaded = self.inner.get_signal(resolved.signal_ref).ok_or_else(|| {
            WavepeekError::Internal(format!(
                "signal '{}' could not be loaded from waveform backend",
                resolved.path
            ))
        })?;

        let Some(offset) = loaded.get_offset(time_table_idx) else {
            return Ok(SampledSignalState {
                path: resolved.path.clone(),
                width: resolved.width,
                bits: None,
            });
        };

        let value = loaded.get_value_at(&offset, offset.elements - 1);
        let bits = decode_signal_bits(value, resolved.path.as_str())?;
        Ok(SampledSignalState {
            path: resolved.path.clone(),
            width: resolved.width,
            bits,
        })
    }

    #[allow(dead_code)]
    pub fn collect_change_times(
        &mut self,
        resolved: &[ResolvedSignal],
        from_raw: u64,
        to_raw: u64,
    ) -> Result<Vec<u64>, WavepeekError> {
        self.collect_change_times_with_mode(
            resolved,
            from_raw,
            to_raw,
            ChangeCandidateCollectionMode::Auto,
        )
    }

    pub fn collect_change_times_with_mode(
        &mut self,
        resolved: &[ResolvedSignal],
        from_raw: u64,
        to_raw: u64,
        mode: ChangeCandidateCollectionMode,
    ) -> Result<Vec<u64>, WavepeekError> {
        if resolved.is_empty() {
            return Ok(Vec::new());
        }

        let (start_idx, end_idx_exclusive) = {
            let time_table = self.inner.time_table();
            let Some(window) = time_window_indices(time_table, from_raw, to_raw) else {
                return Ok(Vec::new());
            };
            window
        };

        if self.should_use_streaming_candidate_collection(resolved.len(), from_raw, to_raw, mode) {
            match self.collect_change_times_streaming(resolved, from_raw, to_raw) {
                Ok(times) => return Ok(times),
                Err(_) if mode == ChangeCandidateCollectionMode::Auto => {}
                Err(error) => return Err(error),
            }
        } else if mode == ChangeCandidateCollectionMode::Stream {
            return Err(WavepeekError::Internal(
                "forced stream candidate collection requires FST input and a non-empty time window"
                    .to_string(),
            ));
        }

        let signal_refs = resolved
            .iter()
            .map(|signal| signal.signal_ref)
            .collect::<Vec<_>>();
        self.ensure_signals_loaded(&signal_refs);
        let time_table = self.inner.time_table();

        let mut changed = BTreeSet::new();
        for signal in resolved {
            let loaded = self.inner.get_signal(signal.signal_ref).ok_or_else(|| {
                WavepeekError::Internal(format!(
                    "signal '{}' could not be loaded from waveform backend",
                    signal.path
                ))
            })?;

            let mut previous_offset = if start_idx == 0 {
                None
            } else {
                let prev_idx = u32::try_from(start_idx - 1).map_err(|_| {
                    WavepeekError::Internal("time table index exceeds u32 range".to_string())
                })?;
                loaded.get_offset(prev_idx)
            };

            for (idx, timestamp) in time_table
                .iter()
                .enumerate()
                .take(end_idx_exclusive)
                .skip(start_idx)
            {
                let current_idx = u32::try_from(idx).map_err(|_| {
                    WavepeekError::Internal("time table index exceeds u32 range".to_string())
                })?;
                let current_offset = loaded.get_offset(current_idx);
                if current_offset != previous_offset {
                    changed.insert(*timestamp);
                }
                previous_offset = current_offset;
            }
        }

        Ok(changed.into_iter().collect())
    }

    pub(crate) fn collect_expr_candidate_times_with_mode(
        &mut self,
        resolved: &[ExprResolvedSignal],
        from_raw: u64,
        to_raw: u64,
        mode: ChangeCandidateCollectionMode,
    ) -> Result<Vec<u64>, WavepeekError> {
        if resolved.is_empty() {
            return Ok(Vec::new());
        }

        let mut value_sources = Vec::new();
        let mut event_sources = Vec::new();
        for signal in resolved {
            if matches!(signal.expr_type.kind, ExprTypeKind::Event) {
                event_sources.push(signal.clone());
            } else {
                value_sources.push(ResolvedSignal {
                    path: signal.path.clone(),
                    signal_ref: signal.signal_ref,
                    width: signal.expr_type.width.max(1),
                });
            }
        }

        let mut changed = BTreeSet::new();
        if !value_sources.is_empty() {
            changed.extend(self.collect_change_times_with_mode(
                value_sources.as_slice(),
                from_raw,
                to_raw,
                mode,
            )?);
        }

        if !event_sources.is_empty() {
            let Some((start_idx, end_idx_exclusive)) =
                time_window_indices(self.inner.time_table(), from_raw, to_raw)
            else {
                return Ok(changed.into_iter().collect());
            };

            let signal_refs = event_sources
                .iter()
                .map(|signal| signal.signal_ref)
                .collect::<Vec<_>>();
            self.ensure_signals_loaded(signal_refs.as_slice());
            let time_table = self.inner.time_table();

            for signal in &event_sources {
                let loaded = self.inner.get_signal(signal.signal_ref).ok_or_else(|| {
                    WavepeekError::Internal(format!(
                        "signal '{}' could not be loaded from waveform backend",
                        signal.path
                    ))
                })?;

                for raw_index in loaded.time_indices() {
                    let idx = *raw_index as usize;
                    if idx < start_idx || idx >= end_idx_exclusive {
                        continue;
                    }
                    changed.insert(time_table[idx]);
                }
            }
        }

        Ok(changed.into_iter().collect())
    }

    pub fn should_use_streaming_candidate_collection(
        &self,
        signal_count: usize,
        from_raw: u64,
        to_raw: u64,
        mode: ChangeCandidateCollectionMode,
    ) -> bool {
        match mode {
            ChangeCandidateCollectionMode::Random => false,
            ChangeCandidateCollectionMode::Stream => {
                if self.file_format != wellen::FileFormat::Fst {
                    return false;
                }
                let time_table = self.inner.time_table();
                time_window_indices(time_table, from_raw, to_raw).is_some()
            }
            ChangeCandidateCollectionMode::Auto => {
                if self.file_format != wellen::FileFormat::Fst {
                    return false;
                }
                let time_table = self.inner.time_table();
                let Some((start_idx, end_idx_exclusive)) =
                    time_window_indices(time_table, from_raw, to_raw)
                else {
                    return false;
                };
                let window_len = end_idx_exclusive.saturating_sub(start_idx);
                let estimated_random_work = window_len.saturating_mul(signal_count);
                estimated_random_work > STREAM_THRESHOLD_WORK
            }
        }
    }

    fn collect_change_times_streaming(
        &self,
        resolved: &[ResolvedSignal],
        from_raw: u64,
        to_raw: u64,
    ) -> Result<Vec<u64>, WavepeekError> {
        let mut streaming = wellen::stream::read_from_file(
            self.source_path.as_path(),
            &wellen::LoadOptions::default(),
        )
        .map_err(|error| map_wellen_error(self.source_path.as_path(), error))?;

        let signal_refs = resolved
            .iter()
            .map(|signal| {
                resolve_signal_ref_with_width(streaming.hierarchy(), signal.path.as_str())
                    .map(|(signal_ref, _)| signal_ref)
            })
            .collect::<Result<Vec<_>, _>>()?;
        let filter = wellen::stream::Filter {
            start: from_raw,
            end: Some(to_raw),
            signals: Some(signal_refs.as_slice()),
        };

        let mut changed = BTreeSet::new();
        streaming
            .stream(&filter, |time, _signal_ref, _value| {
                changed.insert(time);
            })
            .map_err(|error| map_wellen_error(self.source_path.as_path(), error))?;

        Ok(changed.into_iter().collect())
    }

    pub fn ensure_signals_loaded(&mut self, signal_refs: &[SignalRef]) {
        if signal_refs.is_empty() {
            return;
        }

        let mut queued = HashSet::with_capacity(signal_refs.len());
        let to_load = signal_refs
            .iter()
            .copied()
            .filter(|signal_ref| !self.loaded_signals.contains(signal_ref))
            .filter(|signal_ref| queued.insert(*signal_ref))
            .collect::<Vec<_>>();
        if to_load.is_empty() {
            return;
        }

        if should_use_multi_thread_signal_load(self.file_format) {
            self.inner.load_signals_multi_threaded(&to_load);
        } else {
            self.inner.load_signals(&to_load);
        }
        self.loaded_signals.extend(to_load);
    }
}

fn duplicate_preserving_projection(canonical_paths: &[String]) -> (Vec<String>, Vec<usize>) {
    let mut unique_paths = Vec::with_capacity(canonical_paths.len());
    let mut projection = Vec::with_capacity(canonical_paths.len());
    let mut seen = HashMap::with_capacity(canonical_paths.len());

    for path in canonical_paths {
        if let Some(&idx) = seen.get(path.as_str()) {
            projection.push(idx);
            continue;
        }

        let idx = unique_paths.len();
        unique_paths.push(path.clone());
        seen.insert(path.as_str(), idx);
        projection.push(idx);
    }

    (unique_paths, projection)
}

fn should_use_multi_thread_signal_load(file_format: wellen::FileFormat) -> bool {
    file_format == wellen::FileFormat::Fst
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

fn time_window_indices(time_table: &[u64], from_raw: u64, to_raw: u64) -> Option<(usize, usize)> {
    if time_table.is_empty() || from_raw > to_raw {
        return None;
    }

    let start_idx = match time_table.binary_search(&from_raw) {
        Ok(index) | Err(index) => index,
    };
    let end_idx_exclusive = match time_table.binary_search(&to_raw) {
        Ok(index) => index.saturating_add(1),
        Err(index) => index,
    };

    if start_idx >= end_idx_exclusive {
        return None;
    }

    Some((start_idx, end_idx_exclusive))
}

fn decode_signal_bits(
    value: wellen::SignalValue,
    signal_path: &str,
) -> Result<Option<String>, WavepeekError> {
    match value {
        wellen::SignalValue::Event => Ok(Some(String::new())),
        wellen::SignalValue::Binary(_, _)
        | wellen::SignalValue::FourValue(_, _)
        | wellen::SignalValue::NineValue(_, _) => {
            let bits = value.to_bit_string().ok_or_else(|| {
                WavepeekError::Internal(format!(
                    "failed to convert value for signal '{}' to bit string",
                    signal_path
                ))
            })?;
            Ok(Some(bits))
        }
        wellen::SignalValue::String(_) | wellen::SignalValue::Real(_) => {
            Err(WavepeekError::Signal(format!(
                "signal '{}' has unsupported non-bit-vector encoding",
                signal_path
            )))
        }
    }
}

fn resolve_signal_ref_with_width(
    hierarchy: &wellen::Hierarchy,
    canonical_path: &str,
) -> Result<(SignalRef, u32), WavepeekError> {
    let var_ref = resolve_var_ref(hierarchy, canonical_path)?;
    let var = &hierarchy[var_ref];
    let width = var.length().ok_or_else(|| {
        WavepeekError::Signal(format!(
            "signal '{canonical_path}' has unsupported non-bit-vector encoding"
        ))
    })?;

    Ok((var.signal_ref(), width))
}

fn resolve_var_ref(
    hierarchy: &wellen::Hierarchy,
    canonical_path: &str,
) -> Result<wellen::VarRef, WavepeekError> {
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

    hierarchy
        .lookup_var(&scope_names, signal_name)
        .ok_or_else(|| {
            WavepeekError::Signal(format!("signal '{canonical_path}' not found in dump"))
        })
}

#[allow(dead_code)]
fn expr_type_from_var(
    hierarchy: &wellen::Hierarchy,
    var: &wellen::Var,
    canonical_path: &str,
) -> Result<ExprType, WavepeekError> {
    let (kind, width, is_four_state, is_signed, storage) = match var.var_type() {
        VarType::Byte => (
            ExprTypeKind::IntegerLike(IntegerLikeKind::Byte),
            8,
            false,
            true,
            ExprStorage::Scalar,
        ),
        VarType::ShortInt => (
            ExprTypeKind::IntegerLike(IntegerLikeKind::Shortint),
            16,
            false,
            true,
            ExprStorage::Scalar,
        ),
        VarType::Int => (
            ExprTypeKind::IntegerLike(IntegerLikeKind::Int),
            32,
            false,
            true,
            ExprStorage::Scalar,
        ),
        VarType::LongInt => (
            ExprTypeKind::IntegerLike(IntegerLikeKind::Longint),
            64,
            false,
            true,
            ExprStorage::Scalar,
        ),
        VarType::Integer => (
            ExprTypeKind::IntegerLike(IntegerLikeKind::Integer),
            32,
            true,
            true,
            ExprStorage::Scalar,
        ),
        VarType::Time => (
            ExprTypeKind::IntegerLike(IntegerLikeKind::Time),
            64,
            true,
            false,
            ExprStorage::Scalar,
        ),
        VarType::Real | VarType::RealTime | VarType::RealParameter | VarType::ShortReal => {
            (ExprTypeKind::Real, 64, false, false, ExprStorage::Scalar)
        }
        VarType::String => (ExprTypeKind::String, 0, false, false, ExprStorage::Scalar),
        VarType::Event => (ExprTypeKind::Event, 0, false, false, ExprStorage::Scalar),
        VarType::Enum => (
            ExprTypeKind::EnumCore,
            var.length().ok_or_else(|| {
                WavepeekError::Signal(format!(
                    "signal '{canonical_path}' is missing enum width metadata"
                ))
            })?,
            true,
            false,
            ExprStorage::Scalar,
        ),
        other => {
            let width = var.length().ok_or_else(|| {
                WavepeekError::Signal(format!(
                    "signal '{canonical_path}' has unsupported non-bit-vector encoding"
                ))
            })?;
            (
                ExprTypeKind::BitVector,
                width,
                var_type_is_four_state(other),
                var_type_is_signed(other),
                if width > 1 {
                    ExprStorage::PackedVector
                } else {
                    ExprStorage::Scalar
                },
            )
        }
    };

    let (enum_type_id, enum_labels) = match &kind {
        ExprTypeKind::EnumCore => match var.enum_type(hierarchy) {
            Some((name, labels)) => (
                Some(name.to_string()),
                Some(
                    labels
                        .into_iter()
                        .map(|(bits, label)| EnumLabelInfo {
                            name: label.to_string(),
                            bits: bits.to_string(),
                        })
                        .collect(),
                ),
            ),
            None => (None, None),
        },
        _ => (None, None),
    };

    Ok(ExprType {
        kind,
        storage,
        width,
        is_four_state,
        is_signed,
        enum_type_id,
        enum_labels,
    })
}

#[allow(dead_code)]
fn var_type_is_four_state(var_type: VarType) -> bool {
    !matches!(
        var_type,
        VarType::Bit
            | VarType::Byte
            | VarType::ShortInt
            | VarType::Int
            | VarType::LongInt
            | VarType::Boolean
            | VarType::BitVector
    )
}

#[allow(dead_code)]
fn var_type_is_signed(var_type: VarType) -> bool {
    matches!(
        var_type,
        VarType::Byte | VarType::ShortInt | VarType::Int | VarType::LongInt | VarType::Integer
    )
}

#[allow(dead_code)]
fn empty_sampled_value(ty: &ExprType) -> SampledValue {
    match &ty.kind {
        ExprTypeKind::Real => SampledValue::Real { value: None },
        ExprTypeKind::String => SampledValue::String { value: None },
        _ => SampledValue::Integral {
            bits: None,
            label: None,
        },
    }
}

#[allow(dead_code)]
fn enum_label_for_bits(ty: &ExprType, bits: &str) -> Option<String> {
    ty.enum_labels
        .as_ref()?
        .iter()
        .find(|entry| entry.bits == bits)
        .map(|entry| entry.name.clone())
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

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct EdgeClassification {
    pub posedge: bool,
    pub negedge: bool,
}

#[allow(dead_code)]
impl EdgeClassification {
    pub(crate) fn edge(self) -> bool {
        self.posedge || self.negedge
    }
}

#[allow(dead_code)]
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

#[allow(dead_code)]
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
        SampledSignal, ScopeEntry, Waveform, classify_edge, duplicate_preserving_projection,
        should_emit_delta_and_update_baseline,
    };

    const TEST_VCD: &str = "$date\n  today\n$end\n$version\n  wavepeek-test\n$end\n$timescale 1ns $end\n$scope module top $end\n$var wire 1 ! clk $end\n$var reg 8 \" data $end\n$var parameter 8 # cfg $end\n$scope module cpu $end\n$var wire 1 $ valid $end\n$upscope $end\n$scope function helper $end\n$var wire 1 & helper_flag $end\n$upscope $end\n$scope module mem $end\n$var wire 1 % ready $end\n$upscope $end\n$upscope $end\n$enddefinitions $end\n#0\n0!\nb00000000 \"\nb10101010 #\n0$\n0&\n0%\n#5\n1!\n1$\n1&\n#10\nb00001111 \"\n1%\n";

    const RICH_VALUE_VCD: &str = "$date\n  2026-03-12\n$end\n$version\n  wavepeek-rich-value\n$end\n$timescale 1ns $end\n$scope module top $end\n$var real 1 ! temp $end\n$var string 1 \" msg $end\n$upscope $end\n$enddefinitions $end\n#0\nr1.5 !\nsgo \"\n";

    const RECURSIVE_TEST_VCD: &str = "$date\n  2026-02-28\n$end\n$version\n  wavepeek-recursive-test\n$end\n$timescale 1ns $end\n$scope module top $end\n$var wire 1 ! clk $end\n$scope module cpu $end\n$var wire 1 \" valid $end\n$scope module core $end\n$var wire 1 # execute $end\n$upscope $end\n$upscope $end\n$scope module mem $end\n$var wire 1 $ ready $end\n$upscope $end\n$upscope $end\n$enddefinitions $end\n#0\n0!\n0\"\n0#\n0$\n#5\n1!\n1\"\n1#\n1$\n";

    const DELAYED_VALUE_VCD: &str = "$date\n  2026-03-03\n$end\n$version\n  wavepeek-delayed-value\n$end\n$timescale 1ns $end\n$scope module top $end\n$var wire 1 ! delayed $end\n$upscope $end\n$enddefinitions $end\n#0\n#5\n1!\n";

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
    fn sample_signals_at_time_stays_non_bit_vector_for_rich_values() {
        let fixture = write_fixture(RICH_VALUE_VCD, "rich-sample.vcd");

        let mut waveform = Waveform::open(fixture.path()).expect("fixture should open");
        let error = waveform
            .sample_signals_at_time(&["top.temp".to_string()], 0)
            .expect_err("rich real sampling should stay on the legacy CLI rejection path");

        assert_eq!(
            error.to_string(),
            "error: signal: signal 'top.temp' has unsupported non-bit-vector encoding"
        );
    }

    #[test]
    fn duplicate_projection_deduplicates_paths_and_tracks_requested_order() {
        let (unique_paths, projection) = duplicate_preserving_projection(&[
            "top.clk".to_string(),
            "top.data".to_string(),
            "top.clk".to_string(),
            "top.cpu.valid".to_string(),
            "top.data".to_string(),
        ]);

        assert_eq!(
            unique_paths,
            vec![
                "top.clk".to_string(),
                "top.data".to_string(),
                "top.cpu.valid".to_string()
            ]
        );
        assert_eq!(projection, vec![0, 1, 0, 2, 1]);
    }

    #[test]
    fn fst_uses_multi_thread_loader_in_shared_path() {
        assert!(super::should_use_multi_thread_signal_load(
            wellen::FileFormat::Fst
        ));
        assert!(!super::should_use_multi_thread_signal_load(
            wellen::FileFormat::Vcd
        ));
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
    fn signal_offset_at_index_compares_data_position_only() {
        let fixture = write_fixture(TEST_VCD, "sample.vcd");

        let mut waveform = Waveform::open(fixture.path()).expect("fixture should open");
        let resolved = waveform
            .resolve_signals(&["top.data".to_string()])
            .expect("signal should resolve");
        waveform.ensure_signals_loaded(&[resolved[0].signal_ref]);

        let offset_at_0 = waveform
            .signal_offset_at_index(resolved[0].signal_ref, 0)
            .expect("offset at #0 should exist");
        let offset_at_5 = waveform
            .signal_offset_at_index(resolved[0].signal_ref, 1)
            .expect("offset at #5 should exist");
        let offset_at_10 = waveform
            .signal_offset_at_index(resolved[0].signal_ref, 2)
            .expect("offset at #10 should exist");

        assert_eq!(offset_at_0, offset_at_5);
        assert_ne!(offset_at_5, offset_at_10);
    }

    #[test]
    fn signal_offset_at_index_returns_none_when_signal_is_not_loaded() {
        let fixture = write_fixture(TEST_VCD, "sample.vcd");

        let waveform = Waveform::open(fixture.path()).expect("fixture should open");
        let resolved = waveform
            .resolve_signals(&["top.data".to_string()])
            .expect("signal should resolve");

        assert_eq!(
            waveform.signal_offset_at_index(resolved[0].signal_ref, 0),
            None
        );
    }

    #[test]
    fn decode_signal_at_index_matches_sample_resolved_optional() {
        let fixture = write_fixture(TEST_VCD, "sample.vcd");

        let mut waveform = Waveform::open(fixture.path()).expect("fixture should open");
        let resolved = waveform
            .resolve_signals(&["top.clk".to_string(), "top.data".to_string()])
            .expect("signals should resolve");
        let signal_refs = resolved
            .iter()
            .map(|signal| signal.signal_ref)
            .collect::<Vec<_>>();
        waveform.ensure_signals_loaded(&signal_refs);

        let at_10 = waveform
            .sample_resolved_optional(&resolved, 10)
            .expect("batch sampling should succeed");
        let decoded = resolved
            .iter()
            .map(|signal| waveform.decode_signal_at_index(signal, 2))
            .collect::<Result<Vec<_>, _>>()
            .expect("point decode should succeed");

        assert_eq!(decoded, at_10);
    }

    #[test]
    fn decode_signal_at_index_returns_none_when_no_prior_value_exists() {
        let fixture = write_fixture(DELAYED_VALUE_VCD, "delayed.vcd");

        let mut waveform = Waveform::open(fixture.path()).expect("fixture should open");
        let resolved = waveform
            .resolve_signals(&["top.delayed".to_string()])
            .expect("signal should resolve");
        waveform.ensure_signals_loaded(&[resolved[0].signal_ref]);

        let sample_before_first_value = waveform
            .decode_signal_at_index(&resolved[0], 0)
            .expect("decode should succeed");
        let sample_after_first_value = waveform
            .decode_signal_at_index(&resolved[0], 1)
            .expect("decode should succeed");

        assert_eq!(sample_before_first_value.bits, None);
        assert_eq!(sample_after_first_value.bits.as_deref(), Some("1"));
    }

    #[test]
    fn decode_signal_at_index_requires_loaded_signal_data() {
        let fixture = write_fixture(TEST_VCD, "sample.vcd");

        let waveform = Waveform::open(fixture.path()).expect("fixture should open");
        let resolved = waveform
            .resolve_signals(&["top.data".to_string()])
            .expect("signal should resolve");

        let error = waveform
            .decode_signal_at_index(&resolved[0], 0)
            .expect_err("decode must fail before load");
        assert_eq!(
            error.to_string(),
            "error: internal: signal 'top.data' could not be loaded from waveform backend"
        );
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
