//! Waveform adapter used by the engine layer.
//!
//! Canonical path policy:
//! - Paths are emitted as dot-separated full hierarchy paths.
//! - Scope and signal names are preserved exactly as provided by the parser.
//! - No additional escaping or normalization pass is applied.

#[allow(dead_code)]
pub(crate) mod expr_host;
#[cfg(feature = "fsdb")]
mod fsdb_backend;
#[cfg(not(feature = "fsdb"))]
mod fsdb_disabled;
#[cfg(any(test, feature = "fsdb"))]
mod fsdb_hierarchy;
#[cfg(feature = "fsdb")]
mod fsdb_native;
#[cfg(any(test, feature = "fsdb"))]
mod fsdb_time;
mod types;
mod wellen_backend;

use std::collections::HashMap;
use std::path::Path;

use crate::error::WavepeekError;
use crate::expr::SampledValue;

#[allow(unused_imports)]
pub(crate) use types::{
    ChangeCandidateCollectionMode, EXCLUDED_SCOPE_KIND_ALIASES, EXCLUDED_SIGNAL_KIND_ALIASES,
    ExprResolvedSignal, ResolvedSignal, STABLE_SCOPE_KIND_ALIASES, STABLE_SIGNAL_KIND_ALIASES,
    SampledSignal, SampledSignalState, ScopeEntry, SignalEntry, SignalId, SignalOffsetData,
    WaveformMetadata,
};

#[derive(Debug)]
pub struct Waveform {
    backend: Backend,
}

#[derive(Debug)]
enum Backend {
    Wellen(Box<wellen_backend::WellenBackend>),
    #[cfg(feature = "fsdb")]
    Fsdb(Box<fsdb_backend::FsdbBackend>),
}

impl Waveform {
    pub fn open(path: &Path) -> Result<Self, WavepeekError> {
        #[cfg(feature = "fsdb")]
        {
            Self::open_feature_enabled(path)
        }

        #[cfg(not(feature = "fsdb"))]
        match wellen_backend::WellenBackend::open(path) {
            Ok(backend) => Ok(Self {
                backend: Backend::Wellen(Box::new(backend)),
            }),
            Err(error) => {
                if fsdb_disabled::should_report_disabled_support(path, &error) {
                    return Err(fsdb_disabled::disabled_support_error());
                }
                Err(error)
            }
        }
    }

    #[cfg(feature = "fsdb")]
    fn open_feature_enabled(path: &Path) -> Result<Self, WavepeekError> {
        let fsdb_looking = is_fsdb_looking_path(path);
        let should_probe = path.is_file();
        let mut probe_result = None;

        if fsdb_looking && should_probe {
            let result = fsdb_backend::FsdbBackend::probe(path);
            if matches!(result, Ok(true)) {
                return fsdb_backend::FsdbBackend::open(path).map(|backend| Self {
                    backend: Backend::Fsdb(Box::new(backend)),
                });
            }
            probe_result = Some(result);
        }

        match wellen_backend::WellenBackend::open(path) {
            Ok(backend) => Ok(Self {
                backend: Backend::Wellen(Box::new(backend)),
            }),
            Err(wellen_error) => {
                if !should_probe {
                    return Err(wellen_error);
                }

                let result = match probe_result {
                    Some(result) => result,
                    None => fsdb_backend::FsdbBackend::probe(path),
                };
                match result {
                    Ok(true) => fsdb_backend::FsdbBackend::open(path).map(|backend| Self {
                        backend: Backend::Fsdb(Box::new(backend)),
                    }),
                    Ok(false) => Err(wellen_error),
                    Err(probe_error) if fsdb_looking => Err(probe_error),
                    Err(_) => Err(wellen_error),
                }
            }
        }
    }

    pub(crate) fn backend_name(&self) -> &'static str {
        match &self.backend {
            Backend::Wellen(backend) => backend.backend_name(),
            #[cfg(feature = "fsdb")]
            Backend::Fsdb(backend) => backend.backend_name(),
        }
    }

    pub(crate) fn format_name(&self) -> &'static str {
        match &self.backend {
            Backend::Wellen(backend) => backend.format_name(),
            #[cfg(feature = "fsdb")]
            Backend::Fsdb(backend) => backend.format_name(),
        }
    }

    pub fn metadata(&self) -> Result<WaveformMetadata, WavepeekError> {
        match &self.backend {
            Backend::Wellen(backend) => backend.metadata(),
            #[cfg(feature = "fsdb")]
            Backend::Fsdb(backend) => backend.metadata(),
        }
    }

    pub fn scopes_depth_first(
        &self,
        max_depth: Option<usize>,
    ) -> Result<Vec<ScopeEntry>, WavepeekError> {
        match &self.backend {
            Backend::Wellen(backend) => backend.scopes_depth_first(max_depth),
            #[cfg(feature = "fsdb")]
            Backend::Fsdb(backend) => backend.scopes_depth_first(max_depth),
        }
    }

    pub fn signals_in_scope(&self, scope_path: &str) -> Result<Vec<SignalEntry>, WavepeekError> {
        match &self.backend {
            Backend::Wellen(backend) => backend.signals_in_scope(scope_path),
            #[cfg(feature = "fsdb")]
            Backend::Fsdb(backend) => backend.signals_in_scope(scope_path),
        }
    }

    pub fn signals_in_scope_recursive(
        &self,
        scope_path: &str,
        max_depth: Option<usize>,
    ) -> Result<Vec<SignalEntry>, WavepeekError> {
        match &self.backend {
            Backend::Wellen(backend) => backend.signals_in_scope_recursive(scope_path, max_depth),
            #[cfg(feature = "fsdb")]
            Backend::Fsdb(backend) => backend.signals_in_scope_recursive(scope_path, max_depth),
        }
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

    pub fn previous_sample_time(&self, raw_time: u64) -> Option<u64> {
        match &self.backend {
            Backend::Wellen(backend) => backend.previous_sample_time(raw_time),
            #[cfg(feature = "fsdb")]
            Backend::Fsdb(backend) => backend.previous_sample_time(raw_time),
        }
    }

    pub fn resolve_signals(
        &self,
        canonical_paths: &[String],
    ) -> Result<Vec<ResolvedSignal>, WavepeekError> {
        match &self.backend {
            Backend::Wellen(backend) => backend.resolve_signals(canonical_paths),
            #[cfg(feature = "fsdb")]
            Backend::Fsdb(backend) => backend.resolve_signals(canonical_paths),
        }
    }

    #[allow(dead_code)]
    pub(crate) fn resolve_expr_signal(
        &self,
        canonical_path: &str,
    ) -> Result<ExprResolvedSignal, WavepeekError> {
        match &self.backend {
            Backend::Wellen(backend) => backend.resolve_expr_signal(canonical_path),
            #[cfg(feature = "fsdb")]
            Backend::Fsdb(backend) => backend.resolve_expr_signal(canonical_path),
        }
    }

    pub(crate) fn resolve_expr_signals(
        &self,
        canonical_paths: &[String],
    ) -> Result<Vec<ExprResolvedSignal>, WavepeekError> {
        match &self.backend {
            Backend::Wellen(backend) => backend.resolve_expr_signals(canonical_paths),
            #[cfg(feature = "fsdb")]
            Backend::Fsdb(backend) => backend.resolve_expr_signals(canonical_paths),
        }
    }

    pub fn sample_resolved_optional(
        &mut self,
        resolved: &[ResolvedSignal],
        query_time_raw: u64,
    ) -> Result<Vec<SampledSignalState>, WavepeekError> {
        match &mut self.backend {
            Backend::Wellen(backend) => backend.sample_resolved_optional(resolved, query_time_raw),
            #[cfg(feature = "fsdb")]
            Backend::Fsdb(backend) => backend.sample_resolved_optional(resolved, query_time_raw),
        }
    }

    #[allow(dead_code)]
    pub(crate) fn sample_expr_value(
        &mut self,
        resolved: &ExprResolvedSignal,
        query_time_raw: u64,
    ) -> Result<SampledValue, WavepeekError> {
        match &mut self.backend {
            Backend::Wellen(backend) => backend.sample_expr_value(resolved, query_time_raw),
            #[cfg(feature = "fsdb")]
            Backend::Fsdb(backend) => backend.sample_expr_value(resolved, query_time_raw),
        }
    }

    #[allow(dead_code)]
    pub(crate) fn expr_event_occurred(
        &mut self,
        resolved: &ExprResolvedSignal,
        query_time_raw: u64,
    ) -> Result<bool, WavepeekError> {
        match &mut self.backend {
            Backend::Wellen(backend) => backend.expr_event_occurred(resolved, query_time_raw),
            #[cfg(feature = "fsdb")]
            Backend::Fsdb(backend) => backend.expr_event_occurred(resolved, query_time_raw),
        }
    }

    pub(crate) fn debug_stats(&self) -> Option<serde_json::Value> {
        match &self.backend {
            Backend::Wellen(_) => None,
            #[cfg(feature = "fsdb")]
            Backend::Fsdb(backend) => backend
                .debug_stats_snapshot()
                .and_then(|snapshot| serde_json::to_value(snapshot).ok()),
        }
    }

    pub(crate) fn validate_expr_values_supported(
        &self,
        resolved: &[ExprResolvedSignal],
    ) -> Result<(), WavepeekError> {
        match &self.backend {
            Backend::Wellen(_) => {
                let _ = resolved;
                Ok(())
            }
            #[cfg(feature = "fsdb")]
            Backend::Fsdb(backend) => backend.validate_expr_values_supported(resolved),
        }
    }

    pub(crate) fn preload_expr_value_changes(
        &mut self,
        resolved: &[ExprResolvedSignal],
        from_raw: u64,
        to_raw: u64,
    ) -> Result<(), WavepeekError> {
        match &mut self.backend {
            Backend::Wellen(_) => {
                let _ = (resolved, from_raw, to_raw);
                Ok(())
            }
            #[cfg(feature = "fsdb")]
            Backend::Fsdb(backend) => {
                backend.preload_expr_value_changes(resolved, from_raw, to_raw)
            }
        }
    }

    pub(crate) fn preload_resolved_value_changes(
        &mut self,
        resolved: &[ResolvedSignal],
        from_raw: u64,
        to_raw: u64,
    ) -> Result<(), WavepeekError> {
        match &mut self.backend {
            Backend::Wellen(_) => {
                let _ = (resolved, from_raw, to_raw);
                Ok(())
            }
            #[cfg(feature = "fsdb")]
            Backend::Fsdb(backend) => {
                backend.preload_resolved_value_changes(resolved, from_raw, to_raw)
            }
        }
    }

    #[inline]
    pub(crate) fn indexed_timestamps(&self) -> Option<&[u64]> {
        match &self.backend {
            Backend::Wellen(backend) => Some(backend.indexed_timestamps()),
            #[cfg(feature = "fsdb")]
            Backend::Fsdb(backend) => backend.indexed_timestamps(),
        }
    }

    #[inline]
    pub(crate) fn indexed_signal_offset_at(
        &self,
        id: SignalId,
        time_table_idx: u32,
    ) -> Option<Option<SignalOffsetData>> {
        match &self.backend {
            Backend::Wellen(backend) => Some(backend.indexed_signal_offset_at(id, time_table_idx)),
            #[cfg(feature = "fsdb")]
            Backend::Fsdb(backend) => backend.indexed_signal_offset_at(id, time_table_idx),
        }
    }

    #[inline]
    pub(crate) fn decode_indexed_signal_at(
        &self,
        resolved: &ResolvedSignal,
        time_table_idx: u32,
    ) -> Result<Option<SampledSignalState>, WavepeekError> {
        match &self.backend {
            Backend::Wellen(backend) => Ok(Some(
                backend.decode_indexed_signal_at(resolved, time_table_idx)?,
            )),
            #[cfg(feature = "fsdb")]
            Backend::Fsdb(backend) => {
                Ok(backend.decode_indexed_signal_at(resolved, time_table_idx))
            }
        }
    }

    #[inline]
    pub(crate) fn ensure_indexed_signals_loaded(&mut self, ids: &[SignalId]) -> bool {
        match &mut self.backend {
            Backend::Wellen(backend) => backend.ensure_indexed_signals_loaded(ids),
            #[cfg(feature = "fsdb")]
            Backend::Fsdb(backend) => backend.ensure_indexed_signals_loaded(ids),
        }
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
        match &mut self.backend {
            Backend::Wellen(backend) => {
                backend.collect_change_times_with_mode(resolved, from_raw, to_raw, mode)
            }
            #[cfg(feature = "fsdb")]
            Backend::Fsdb(backend) => {
                backend.collect_change_times_with_mode(resolved, from_raw, to_raw, mode)
            }
        }
    }

    pub(crate) fn collect_expr_candidate_times_with_mode(
        &mut self,
        resolved: &[ExprResolvedSignal],
        from_raw: u64,
        to_raw: u64,
        mode: ChangeCandidateCollectionMode,
    ) -> Result<Vec<u64>, WavepeekError> {
        match &mut self.backend {
            Backend::Wellen(backend) => {
                backend.collect_expr_candidate_times_with_mode(resolved, from_raw, to_raw, mode)
            }
            #[cfg(feature = "fsdb")]
            Backend::Fsdb(backend) => {
                backend.collect_expr_candidate_times_with_mode(resolved, from_raw, to_raw, mode)
            }
        }
    }

    #[allow(dead_code)]
    pub fn should_use_streaming_candidate_collection(
        &self,
        signal_count: usize,
        from_raw: u64,
        to_raw: u64,
        mode: ChangeCandidateCollectionMode,
    ) -> bool {
        match &self.backend {
            Backend::Wellen(backend) => backend.should_use_streaming_candidate_collection(
                signal_count,
                from_raw,
                to_raw,
                mode,
            ),
            #[cfg(feature = "fsdb")]
            Backend::Fsdb(backend) => backend.should_use_streaming_candidate_collection(
                signal_count,
                from_raw,
                to_raw,
                mode,
            ),
        }
    }
}

#[cfg(feature = "fsdb")]
fn is_fsdb_looking_path(path: &Path) -> bool {
    let Some(file_name) = path.file_name() else {
        return false;
    };
    let file_name = file_name.to_string_lossy().to_lowercase();
    file_name.ends_with(".fsdb") || file_name.ends_with(".fsdb.gz")
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
