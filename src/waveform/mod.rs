//! Waveform adapter used by the engine layer.
//!
//! Canonical path policy:
//! - Paths are emitted as dot-separated full hierarchy paths.
//! - Scope traversal and depth follow hierarchy components, not path punctuation.
//! - Ondas components are adapted to the facade's public path spelling.

#[allow(dead_code)]
pub(crate) mod expr_host;
#[cfg(not(feature = "fsdb"))]
mod fsdb_disabled;
#[cfg(feature = "fsdb")]
mod fsdb_output;
mod ondas_backend;
mod types;

use std::cell::RefCell;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::error::WavepeekError;
use crate::expr::SampledValue;

#[allow(unused_imports)]
pub(crate) use types::{
    ChangeCandidateCollectionMode, EXCLUDED_SCOPE_KIND_ALIASES, EXCLUDED_SIGNAL_KIND_ALIASES,
    ExprResolvedSignal, ResolvedSignal, STABLE_SCOPE_KIND_ALIASES, STABLE_SIGNAL_KIND_ALIASES,
    SampledSignal, SampledSignalState, ScopeEntry, SignalEntry, SignalId, SignalListing,
    SignalOffsetData, WaveformMetadata,
};

#[derive(Debug)]
pub struct Waveform {
    backend: ondas_backend::OndasBackend,
}

#[derive(Clone, Copy)]
enum SignalLookupKind {
    Direct,
    Expression,
}

const MAX_SIGNAL_SUGGESTIONS: usize = 5;

#[derive(Clone)]
struct InvocationWaveform {
    path: PathBuf,
    bytes: Arc<[u8]>,
}

thread_local! {
    static INVOCATION_WAVEFORM: RefCell<Option<InvocationWaveform>> = const { RefCell::new(None) };
}

#[cfg(any(test, target_arch = "wasm32"))]
struct InvocationWaveformGuard(Option<InvocationWaveform>);

#[cfg(any(test, target_arch = "wasm32"))]
impl Drop for InvocationWaveformGuard {
    fn drop(&mut self) {
        INVOCATION_WAVEFORM.with(|slot| slot.replace(self.0.take()));
    }
}

#[cfg(any(test, target_arch = "wasm32"))]
pub(crate) fn with_waveform_bytes<T>(
    path: PathBuf,
    bytes: Arc<[u8]>,
    run: impl FnOnce() -> T,
) -> T {
    // ponytail: one slot matches the browser's one-worker execution model; thread explicit input
    // through engine commands only if concurrent invocations become a real requirement.
    let previous =
        INVOCATION_WAVEFORM.with(|slot| slot.replace(Some(InvocationWaveform { path, bytes })));
    let guard = InvocationWaveformGuard(previous);
    let result = run();
    drop(guard);
    result
}

fn invocation_waveform(path: &Path) -> Option<Arc<[u8]>> {
    INVOCATION_WAVEFORM.with(|slot| {
        slot.borrow()
            .as_ref()
            .filter(|source| source.path == path)
            .map(|source| Arc::clone(&source.bytes))
    })
}

impl Waveform {
    pub fn open(path: &Path) -> Result<Self, WavepeekError> {
        let result = if let Some(bytes) = invocation_waveform(path) {
            ondas_backend::OndasBackend::open_bytes(path, bytes)
        } else {
            ondas_backend::OndasBackend::open(path)
        };
        #[cfg(not(feature = "fsdb"))]
        let result = result.map_err(|error| {
            if fsdb_disabled::should_report_disabled_support(path, &error) {
                return fsdb_disabled::disabled_support_error();
            }
            error
        });
        result.map(|backend| Self { backend })
    }

    pub(crate) fn backend_name(&self) -> &'static str {
        self.backend.backend_name()
    }

    pub(crate) fn format_name(&self) -> &'static str {
        self.backend.format_name()
    }

    pub fn metadata(&self) -> Result<WaveformMetadata, WavepeekError> {
        self.backend.metadata()
    }

    pub fn scopes_depth_first(
        &self,
        max_depth: Option<usize>,
    ) -> Result<Vec<ScopeEntry>, WavepeekError> {
        self.backend.scopes_depth_first(max_depth)
    }

    pub fn signals_in_scope(&self, scope_path: &str) -> Result<Vec<SignalEntry>, WavepeekError> {
        self.backend.signals_in_scope(scope_path)
    }

    pub(crate) fn signals_in_scope_report(
        &self,
        scope_path: &str,
    ) -> Result<SignalListing, WavepeekError> {
        self.backend.signals_in_scope_report(scope_path)
    }

    #[cfg(test)]
    pub fn signals_in_scope_recursive(
        &self,
        scope_path: &str,
        max_depth: Option<usize>,
    ) -> Result<Vec<SignalEntry>, WavepeekError> {
        Ok(self
            .signals_in_scope_recursive_report(scope_path, max_depth)?
            .entries)
    }

    pub(crate) fn signals_in_scope_recursive_report(
        &self,
        scope_path: &str,
        max_depth: Option<usize>,
    ) -> Result<SignalListing, WavepeekError> {
        self.backend
            .signals_in_scope_recursive_report(scope_path, max_depth)
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
        self.backend.previous_sample_time(raw_time)
    }

    pub fn resolve_signals(
        &self,
        canonical_paths: &[String],
    ) -> Result<Vec<ResolvedSignal>, WavepeekError> {
        self.backend.resolve_signals(canonical_paths)
    }

    pub(crate) fn resolve_signals_with_diagnostics(
        &self,
        canonical_paths: &[String],
        query_names: &[String],
        scope: Option<&str>,
    ) -> Result<Vec<ResolvedSignal>, WavepeekError> {
        self.resolve_signals_with_diagnostic_depth(canonical_paths, query_names, scope, true)
    }

    pub(crate) fn resolve_local_signal_with_diagnostic(
        &self,
        canonical_path: &String,
        query_name: &String,
        scope: Option<&str>,
    ) -> Result<ResolvedSignal, WavepeekError> {
        self.resolve_signals_with_diagnostic_depth(
            std::slice::from_ref(canonical_path),
            std::slice::from_ref(query_name),
            scope,
            false,
        )?
        .into_iter()
        .next()
        .ok_or_else(|| WavepeekError::Internal("signal resolution returned no result".to_string()))
    }

    fn resolve_signals_with_diagnostic_depth(
        &self,
        canonical_paths: &[String],
        query_names: &[String],
        scope: Option<&str>,
        recursive: bool,
    ) -> Result<Vec<ResolvedSignal>, WavepeekError> {
        if canonical_paths.len() != query_names.len() {
            return Err(WavepeekError::Internal(
                "signal query names do not match canonical paths".to_string(),
            ));
        }

        match self.resolve_signals(canonical_paths) {
            Ok(resolved) => Ok(resolved),
            Err(_) => {
                for (canonical_path, query_name) in canonical_paths.iter().zip(query_names) {
                    if let Err(error) = self.resolve_signals(std::slice::from_ref(canonical_path)) {
                        return Err(self.missing_signal_diagnostic(
                            canonical_path,
                            query_name,
                            scope,
                            SignalLookupKind::Direct,
                            recursive,
                            error,
                        ));
                    }
                }
                Err(WavepeekError::Internal(
                    "bulk signal resolution failed after individual lookups succeeded".to_string(),
                ))
            }
        }
    }

    #[allow(dead_code)]
    pub(crate) fn resolve_expr_signal(
        &self,
        canonical_path: &str,
    ) -> Result<ExprResolvedSignal, WavepeekError> {
        self.backend.resolve_expr_signal(canonical_path)
    }

    pub(crate) fn resolve_expr_signals(
        &self,
        canonical_paths: &[String],
    ) -> Result<Vec<ExprResolvedSignal>, WavepeekError> {
        self.backend.resolve_expr_signals(canonical_paths)
    }

    pub(crate) fn resolve_expr_signal_with_diagnostic(
        &self,
        canonical_path: &str,
        query_name: &str,
        scope: Option<&str>,
    ) -> Result<ExprResolvedSignal, WavepeekError> {
        self.resolve_expr_signal(canonical_path).map_err(|error| {
            self.missing_signal_diagnostic(
                canonical_path,
                query_name,
                scope,
                SignalLookupKind::Expression,
                true,
                error,
            )
        })
    }

    pub(crate) fn resolve_expr_signals_with_diagnostics(
        &self,
        canonical_paths: &[String],
        query_names: &[String],
        scope: Option<&str>,
    ) -> Result<Vec<ExprResolvedSignal>, WavepeekError> {
        if canonical_paths.len() != query_names.len() {
            return Err(WavepeekError::Internal(
                "expression query names do not match canonical paths".to_string(),
            ));
        }

        match self.resolve_expr_signals(canonical_paths) {
            Ok(resolved) => Ok(resolved),
            Err(_) => canonical_paths
                .iter()
                .zip(query_names)
                .map(|(canonical_path, query_name)| {
                    self.resolve_expr_signal_with_diagnostic(canonical_path, query_name, scope)
                })
                .collect(),
        }
    }

    fn missing_signal_diagnostic(
        &self,
        canonical_path: &str,
        query_name: &str,
        scope: Option<&str>,
        kind: SignalLookupKind,
        recursive: bool,
        original: WavepeekError,
    ) -> WavepeekError {
        let Ok(listing) = self.signal_candidates(scope, recursive) else {
            return original;
        };
        if listing
            .entries
            .iter()
            .any(|entry| entry.path == canonical_path)
            || listing
                .omitted_ambiguous_paths
                .iter()
                .any(|path| path == canonical_path)
        {
            return original;
        }

        let basename = query_name.rsplit('.').next().unwrap_or(query_name);
        let basename_chars = basename.chars().collect::<Vec<_>>();
        let max_distance = if basename_chars.len() <= 3 { 1 } else { 2 };
        let mut candidates = Vec::with_capacity(MAX_SIGNAL_SUGGESTIONS);
        for entry in listing.entries {
            let Some(distance) = levenshtein_with_limit(
                basename_chars.as_slice(),
                entry.name.as_str(),
                max_distance,
            ) else {
                continue;
            };
            let display = display_signal_path(entry.path.as_str(), scope);
            if (!recursive && scope.is_some() && display.contains('.'))
                || !self.signal_candidate_is_resolvable(&entry.path, kind)
            {
                continue;
            }
            candidates.push((distance, entry));
            candidates.sort_by(|left, right| {
                left.0.cmp(&right.0).then_with(|| {
                    display_signal_path(left.1.path.as_str(), scope)
                        .cmp(display_signal_path(right.1.path.as_str(), scope))
                })
            });
            candidates.truncate(MAX_SIGNAL_SUGGESTIONS);
        }
        let suggestions = candidates
            .into_iter()
            .map(|(_, entry)| display_signal_path(entry.path.as_str(), scope).to_string())
            .collect::<Vec<_>>();

        let location = match scope {
            Some(scope) => format!("signal '{query_name}' not found under scope '{scope}'"),
            None => format!("signal '{query_name}' not found in dump"),
        };
        let detail = if suggestions.is_empty() {
            format!(
                "no dumped signal with basename '{basename}'; the RTL declaration may be optimized, aliased, or not dumped"
            )
        } else {
            format!("closest query names:\n  {}", suggestions.join("\n  "))
        };
        WavepeekError::Signal(format!("{location}\n{detail}"))
    }

    fn signal_candidate_is_resolvable(&self, path: &String, kind: SignalLookupKind) -> bool {
        let Ok(expr_signal) = self.resolve_expr_signal(path) else {
            return false;
        };
        if self
            .validate_expr_values_supported(std::slice::from_ref(&expr_signal))
            .is_err()
        {
            return false;
        }
        match kind {
            SignalLookupKind::Direct => {
                self.resolve_signals(std::slice::from_ref(path)).is_ok()
                    && self.validate_direct_value_supported(path).is_ok()
            }
            SignalLookupKind::Expression => true,
        }
    }

    fn validate_direct_value_supported(&self, _path: &str) -> Result<(), WavepeekError> {
        self.backend.validate_direct_value_supported(_path)
    }

    fn signal_candidates(
        &self,
        scope: Option<&str>,
        recursive: bool,
    ) -> Result<SignalListing, WavepeekError> {
        if let Some(scope) = scope {
            return if recursive {
                self.signals_in_scope_recursive_report(scope, None)
            } else {
                self.signals_in_scope_report(scope)
            };
        }

        let mut listing = SignalListing {
            entries: Vec::new(),
            omitted_ambiguous_paths: Vec::new(),
        };
        for root in self.scopes_depth_first(Some(0))? {
            let root_listing = self.signals_in_scope_recursive_report(root.path.as_str(), None)?;
            listing.entries.extend(root_listing.entries);
            listing
                .omitted_ambiguous_paths
                .extend(root_listing.omitted_ambiguous_paths);
        }
        Ok(listing)
    }

    pub fn sample_resolved_optional(
        &mut self,
        resolved: &[ResolvedSignal],
        query_time_raw: u64,
    ) -> Result<Vec<SampledSignalState>, WavepeekError> {
        self.backend
            .sample_resolved_optional(resolved, query_time_raw)
    }

    #[allow(dead_code)]
    pub(crate) fn sample_expr_value(
        &mut self,
        resolved: &ExprResolvedSignal,
        query_time_raw: u64,
    ) -> Result<SampledValue, WavepeekError> {
        self.backend.sample_expr_value(resolved, query_time_raw)
    }

    #[allow(dead_code)]
    pub(crate) fn expr_event_occurred(
        &mut self,
        resolved: &ExprResolvedSignal,
        query_time_raw: u64,
    ) -> Result<bool, WavepeekError> {
        self.backend.expr_event_occurred(resolved, query_time_raw)
    }

    pub(crate) fn debug_stats(&self) -> Option<serde_json::Value> {
        None
    }

    pub(crate) fn validate_expr_values_supported(
        &self,
        resolved: &[ExprResolvedSignal],
    ) -> Result<(), WavepeekError> {
        self.backend.validate_expr_values_supported(resolved)
    }

    pub(crate) fn preload_expr_value_changes(
        &mut self,
        resolved: &[ExprResolvedSignal],
        from_raw: u64,
        to_raw: u64,
    ) -> Result<(), WavepeekError> {
        self.backend
            .preload_expr_value_changes(resolved, from_raw, to_raw)
    }

    pub(crate) fn preload_resolved_value_changes(
        &mut self,
        resolved: &[ResolvedSignal],
        from_raw: u64,
        to_raw: u64,
    ) -> Result<(), WavepeekError> {
        self.backend
            .preload_resolved_value_changes(resolved, from_raw, to_raw)
    }

    #[inline]
    pub(crate) fn indexed_timestamps(&self) -> Option<&[u64]> {
        self.backend.indexed_timestamps()
    }

    #[inline]
    pub(crate) fn indexed_signal_offset_at(
        &self,
        id: SignalId,
        time_table_idx: u32,
    ) -> Option<Option<SignalOffsetData>> {
        Some(self.backend.indexed_signal_offset_at(id, time_table_idx))
    }

    #[inline]
    pub(crate) fn decode_indexed_signal_at(
        &self,
        resolved: &ResolvedSignal,
        time_table_idx: u32,
    ) -> Result<Option<SampledSignalState>, WavepeekError> {
        self.backend
            .decode_indexed_signal_at(resolved, time_table_idx)
            .map(Some)
    }

    #[inline]
    pub(crate) fn ensure_indexed_signals_loaded(&mut self, ids: &[SignalId]) -> bool {
        self.backend.ensure_indexed_signals_loaded(ids)
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
        self.backend
            .collect_change_times_with_mode(resolved, from_raw, to_raw, mode)
    }

    pub(crate) fn collect_expr_candidate_times_with_mode(
        &mut self,
        resolved: &[ExprResolvedSignal],
        from_raw: u64,
        to_raw: u64,
        mode: ChangeCandidateCollectionMode,
    ) -> Result<Vec<u64>, WavepeekError> {
        self.backend
            .collect_expr_candidate_times_with_mode(resolved, from_raw, to_raw, mode)
    }

    #[allow(dead_code)]
    pub fn should_use_streaming_candidate_collection(
        &self,
        signal_count: usize,
        from_raw: u64,
        to_raw: u64,
        mode: ChangeCandidateCollectionMode,
    ) -> bool {
        self.backend
            .should_use_streaming_candidate_collection(signal_count, from_raw, to_raw, mode)
    }
}

pub(crate) fn display_signal_path<'a>(canonical_path: &'a str, scope: Option<&str>) -> &'a str {
    scope
        .and_then(|scope| canonical_path.strip_prefix(scope))
        .and_then(|path| path.strip_prefix('.'))
        .unwrap_or(canonical_path)
}

fn levenshtein_with_limit(left: &[char], right: &str, limit: usize) -> Option<usize> {
    let right_len = right.chars().count();
    if left.len().abs_diff(right_len) > limit {
        return None;
    }
    let right = right.chars().collect::<Vec<_>>();

    let mut previous = (0..=right.len()).collect::<Vec<_>>();
    let mut current = vec![0; right.len() + 1];
    for (left_index, left_char) in left.iter().enumerate() {
        current[0] = left_index + 1;
        for (right_index, right_char) in right.iter().enumerate() {
            current[right_index + 1] = (previous[right_index + 1] + 1)
                .min(current[right_index] + 1)
                .min(previous[right_index] + usize::from(left_char != right_char));
        }
        if current.iter().copied().min().unwrap_or(usize::MAX) > limit {
            return None;
        }
        std::mem::swap(&mut previous, &mut current);
    }

    (previous[right.len()] <= limit).then_some(previous[right.len()])
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
