//! Waveform adapter used by the engine layer.
//!
//! Canonical path policy:
//! - Paths are emitted as dot-separated full hierarchy paths.
//! - Scope traversal and depth follow hierarchy components, not path punctuation.
//! - Exact Ondas components use round-trippable canonical quoting.

pub(crate) mod expr_host;
pub(crate) mod expr_query;
mod ondas_backend;
mod ondas_error;
pub(crate) mod selection;
mod types;
pub(crate) use selection::format_path;

use std::cell::RefCell;
#[cfg(test)]
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::error::WavepeekError;
use crate::expr::SampledValue;

#[cfg(test)]
pub(crate) use types::{EXCLUDED_SCOPE_KIND_ALIASES, EXCLUDED_SIGNAL_KIND_ALIASES};

pub(crate) use types::{
    ExprResolvedSignal, ResolvedSignal, STABLE_SCOPE_KIND_ALIASES, STABLE_SIGNAL_KIND_ALIASES,
    SampledSignal, SampledSignalState, ScopeEntry, SignalEntry, SignalListing, WaveformMetadata,
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

fn require_values(sampled: Vec<SampledSignalState>) -> Result<Vec<SampledSignal>, WavepeekError> {
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

    pub(crate) fn read_metadata(path: &Path) -> Result<WaveformMetadata, WavepeekError> {
        if invocation_waveform(path).is_some() {
            return Self::open(path)?.metadata();
        }
        ondas_backend::OndasBackend::read_metadata(path)
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

    pub(crate) fn matching_signals(
        &self,
        matches: impl Fn(&str, &str) -> bool,
    ) -> Vec<SignalEntry> {
        self.backend.matching_signals(matches)
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

    #[cfg(test)]
    pub fn sample_signals_at_time(
        &mut self,
        canonical_paths: &[String],
        query_time_raw: u64,
    ) -> Result<Vec<SampledSignal>, WavepeekError> {
        let (unique_paths, projection) = duplicate_preserving_projection(canonical_paths);
        let resolved = self.resolve_signals(&unique_paths)?;
        let sampled_unique = self.sample_resolved_at_time(&resolved, query_time_raw)?;
        Ok(projection
            .iter()
            .map(|unique_idx| sampled_unique[*unique_idx].clone())
            .collect())
    }

    pub fn sample_resolved_at_time(
        &mut self,
        resolved: &[ResolvedSignal],
        query_time_raw: u64,
    ) -> Result<Vec<SampledSignal>, WavepeekError> {
        let sampled = self.sample_resolved_optional(resolved, query_time_raw)?;
        require_values(sampled)
    }

    pub fn sample_resolved_at_times(
        &mut self,
        resolved: &[ResolvedSignal],
        query_times_raw: &[u64],
    ) -> Result<Vec<Vec<SampledSignal>>, WavepeekError> {
        if let [time] = query_times_raw {
            return Ok(vec![self.sample_resolved_at_time(resolved, *time)?]);
        }
        self.backend
            .sample_resolved_optional_at_times(resolved, query_times_raw)?
            .into_iter()
            .map(require_values)
            .collect()
    }

    pub fn resolve_signals(
        &self,
        canonical_paths: &[String],
    ) -> Result<Vec<ResolvedSignal>, WavepeekError> {
        self.backend.resolve_signals(canonical_paths)
    }

    pub(crate) fn resolve_signal_with_diagnostic(
        &self,
        selector: &str,
        scope: Option<&str>,
        recursive: bool,
    ) -> Result<ResolvedSignal, WavepeekError> {
        self.backend
            .resolve_signal(selector, scope)
            .map_err(|error| {
                self.missing_signal_diagnostic(
                    selector,
                    scope,
                    SignalLookupKind::Direct,
                    recursive,
                    error,
                )
            })
    }

    pub(crate) fn resolve_expr_signal(
        &self,
        canonical_path: &str,
    ) -> Result<ExprResolvedSignal, WavepeekError> {
        self.backend
            .resolve_expr_signal(canonical_path, None)
            .map_err(ondas_backend::SignalResolutionError::into_error)
    }

    pub(crate) fn resolve_expr_signal_with_diagnostic(
        &self,
        selector: &str,
        scope: Option<&str>,
    ) -> Result<ExprResolvedSignal, WavepeekError> {
        self.backend
            .resolve_expr_signal(selector, scope)
            .map_err(|error| {
                self.missing_signal_diagnostic(
                    selector,
                    scope,
                    SignalLookupKind::Expression,
                    true,
                    error,
                )
            })
    }

    fn missing_signal_diagnostic(
        &self,
        query_name: &str,
        scope: Option<&str>,
        kind: SignalLookupKind,
        recursive: bool,
        failure: ondas_backend::SignalResolutionError,
    ) -> WavepeekError {
        let ondas_backend::SignalResolutionError::NotFound(path) = &failure else {
            return failure.into_error();
        };
        let Ok(listing) = self.signal_candidates(scope, recursive) else {
            return failure.into_error();
        };
        let basename = path.name().unwrap_or_default();
        let basename_chars = format_path(&ondas::HierarchyPath::from_components([basename]))
            .chars()
            .collect::<Vec<_>>();
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
        WavepeekError::SignalNotFound(format!("{location}\n{detail}"))
    }

    fn signal_candidate_is_resolvable(&self, path: &String, kind: SignalLookupKind) -> bool {
        let Ok(_) = self.resolve_expr_signal(path) else {
            return false;
        };
        match kind {
            SignalLookupKind::Direct => self.resolve_signals(std::slice::from_ref(path)).is_ok(),
            SignalLookupKind::Expression => true,
        }
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

    pub(crate) fn sample_expr_value(
        &mut self,
        resolved: &ExprResolvedSignal,
        query_time_raw: u64,
    ) -> Result<SampledValue, WavepeekError> {
        self.backend.sample_expr_value(resolved, query_time_raw)
    }

    pub(crate) fn query_expressions(
        &mut self,
        query: &expr_query::ExpressionQuery,
        range: std::ops::RangeInclusive<u64>,
        initialize: bool,
        visitor: impl FnMut(
            &dyn expr_host::RuntimeExpressionHost,
            expr_query::ExpressionQueryTick,
        ) -> Result<std::ops::ControlFlow<()>, WavepeekError>,
    ) -> Result<(), WavepeekError> {
        self.backend
            .query_expressions(query, range, initialize, visitor)
    }

    pub(crate) fn expr_event_occurred(
        &mut self,
        resolved: &ExprResolvedSignal,
        query_time_raw: u64,
    ) -> Result<bool, WavepeekError> {
        self.backend.expr_event_occurred(resolved, query_time_raw)
    }
}

pub(crate) fn display_signal_path<'a>(canonical_path: &'a str, scope: Option<&str>) -> &'a str {
    let scope = scope.map(|scope| {
        ondas::HierarchyPath::parse(scope)
            .map(|path| format_path(&path))
            .unwrap_or_else(|_| scope.to_string())
    });
    scope
        .as_deref()
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

#[cfg(test)]
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
