//! Ondas-backed hierarchy and value access.

#[cfg(all(test, feature = "fsdb"))]
#[path = "ondas_fsdb_tests.rs"]
mod fsdb_tests;
#[cfg(test)]
#[path = "ondas_tests.rs"]
mod tests;

use std::cell::{Cell, OnceCell, RefCell};
use std::collections::{BTreeSet, HashMap, HashSet};
use std::fmt;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use ondas::{Encoding, Format, Time, TimeRange, Trace, Value, ValueRef};

use crate::error::WavepeekError;
use crate::expr::{
    EnumLabelInfo, ExprStorage, ExprType, ExprTypeKind, IntegerLikeKind, SampledValue,
};

use super::ondas_error::{open_error, read_error};
use super::selection::format_path;
use super::types::{
    ChangeCandidateCollectionMode, ExprResolvedSignal, ResolvedSignal, SampledSignalState,
    ScopeEntry, SignalEntry, SignalId, SignalListing, SignalOffsetData, WaveformMetadata,
};

pub(super) struct OndasBackend {
    inner: RefCell<ondas::Waveform>,
    path: PathBuf,
    index: OnceCell<HierarchyIndex>,
    direct: RefCell<DirectSignals>,
    traces: HashMap<SignalId, CachedTrace>,
    sampling_window: Option<(u64, u64)>,
    indexed_times: Option<Vec<u64>>,
    compositions: RefCell<HashMap<SignalId, ondas::Signal>>,
}

impl fmt::Debug for OndasBackend {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("OndasBackend")
            .field("format", &self.inner.borrow().format())
            .finish_non_exhaustive()
    }
}

struct Declaration {
    entry: SignalEntry,
    path: ondas::HierarchyPath,
    id: Option<SignalId>,
    expr_type: Option<ExprType>,
    visible: bool,
}

struct DirectSignal {
    path: String,
    signal: ondas::Signal,
    width: u32,
    expr_type: Option<ExprType>,
}

#[derive(Default)]
struct DirectSignals {
    entries: Vec<DirectSignal>,
    by_path: HashMap<String, usize>,
    by_signal: HashMap<ondas::Signal, SignalId>,
}

impl DirectSignals {
    fn resolved(&self, path: &str) -> Option<ResolvedSignal> {
        let entry = self.entries.get(*self.by_path.get(path)?)?;
        Some(ResolvedSignal {
            path: entry.path.clone(),
            id: self.by_signal[&entry.signal],
            width: entry.width,
        })
    }

    fn insert(&mut self, entry: DirectSignal) -> ResolvedSignal {
        let offset = self.entries.len();
        let id = *self
            .by_signal
            .entry(entry.signal)
            .or_insert_with(|| SignalId::from_backend_index(u64::MAX - offset as u64));
        let resolved = ResolvedSignal {
            path: entry.path.clone(),
            id,
            width: entry.width,
        };
        self.by_path.insert(entry.path.clone(), offset);
        self.entries.push(entry);
        resolved
    }
}

fn declaration_order(left: &Declaration, right: &Declaration) -> std::cmp::Ordering {
    left.path
        .components()
        .take(left.path.len().saturating_sub(1))
        .cmp(
            right
                .path
                .components()
                .take(right.path.len().saturating_sub(1)),
        )
        .then_with(|| left.entry.name.cmp(&right.entry.name))
}

struct CachedTrace {
    trace: Trace,
    cursor: Cell<usize>,
}

enum SignalSource {
    Signal(ondas::Signal),
    Concat(Vec<ondas::Signal>),
}

struct HierarchyIndex {
    scopes: Vec<ScopeEntry>,
    scope_by_path: HashMap<ondas::HierarchyPath, Vec<usize>>,
    declarations: Vec<Declaration>,
    declarations_by_scope: Vec<Vec<usize>>,
    by_path: HashMap<ondas::HierarchyPath, Vec<usize>>,
    signals: Vec<SignalSource>,
}

impl OndasBackend {
    pub fn open(path: &Path) -> Result<Self, WavepeekError> {
        let inner = ondas::open(path).map_err(|error| open_error(path, error))?;
        // Available readers and declared product support are separate concerns.
        Ok(Self::new(inner, path))
    }

    pub fn open_bytes(path: &Path, bytes: Arc<[u8]>) -> Result<Self, WavepeekError> {
        let inner = ondas::open_bytes(path.to_string_lossy(), bytes)
            .map_err(|error| open_error(path, error))?;
        if !matches!(inner.format(), Format::Vcd | Format::Fst) {
            return Err(WavepeekError::File(
                "the browser supports only VCD and FST waveforms".into(),
            ));
        }
        Ok(Self::new(inner, path))
    }

    fn new(inner: ondas::Waveform, path: &Path) -> Self {
        Self {
            inner: RefCell::new(inner),
            path: path.to_owned(),
            index: OnceCell::new(),
            direct: RefCell::new(DirectSignals::default()),
            traces: HashMap::new(),
            sampling_window: None,
            indexed_times: None,
            compositions: RefCell::new(HashMap::new()),
        }
    }

    pub fn backend_name(&self) -> &'static str {
        "ondas"
    }

    pub fn format_name(&self) -> &'static str {
        match self.inner.borrow().format() {
            Format::Vcd => "vcd",
            Format::Fst => "fst",
            Format::Fsdb => "fsdb",
            Format::Shm => "shm",
            _ => "unknown",
        }
    }

    pub fn metadata(&self) -> Result<WaveformMetadata, WavepeekError> {
        Self::format_metadata(self.inner.borrow().metadata())
    }

    pub fn read_metadata(path: &Path) -> Result<WaveformMetadata, WavepeekError> {
        let metadata = ondas::read_metadata(path)
            .map_err(|error| read_error(path, "read metadata from", error))?;
        Self::format_metadata(&metadata)
    }

    fn format_metadata(metadata: &ondas::Metadata) -> Result<WaveformMetadata, WavepeekError> {
        let scale = metadata
            .timescale()
            .ok_or_else(|| WavepeekError::File("waveform is missing timescale metadata".into()))?;
        let unit = time_unit(scale.unit())?;
        let span = metadata.time_span();
        let start = span.map_or(0, |span| span.first().ticks());
        let end = span.map_or(start, |span| span.last().ticks());
        let normalize = |time: u64| -> Result<String, WavepeekError> {
            let value = time.checked_mul(u64::from(scale.factor())).ok_or_else(|| {
                WavepeekError::File("time value overflow while normalizing timestamps".into())
            })?;
            Ok(format!("{value}{unit}"))
        };
        Ok(WaveformMetadata {
            time_unit: format!("{}{unit}", scale.factor()),
            time_start: normalize(start)?,
            time_end: normalize(end)?,
        })
    }

    fn index(&self) -> &HierarchyIndex {
        self.index
            .get_or_init(|| HierarchyIndex::new(self.inner.borrow().hierarchy(), false))
    }

    pub fn scopes_depth_first(
        &self,
        max_depth: Option<usize>,
    ) -> Result<Vec<ScopeEntry>, WavepeekError> {
        let mut scopes = self.index.get().map_or_else(
            || HierarchyIndex::new(self.inner.borrow().hierarchy(), true).scopes,
            |index| index.scopes.clone(),
        );
        scopes.retain(|scope| max_depth.is_none_or(|max| scope.depth <= max));
        Ok(scopes)
    }

    pub fn signals_in_scope(&self, path: &str) -> Result<Vec<SignalEntry>, WavepeekError> {
        Ok(self.signals_in_scope_report(path)?.entries)
    }

    pub fn matching_signals(&self, matches: impl Fn(&str, &str) -> bool) -> Vec<SignalEntry> {
        let temporary;
        let index = if let Some(index) = self.index.get() {
            index
        } else {
            let inner = self.inner.borrow();
            let paths = inner
                .hierarchy()
                .variables()
                .filter(|variable| variable.parent().is_none_or(|scope| visible_scope(&scope)))
                .filter_map(|variable| {
                    let path = variable.path();
                    let name =
                        format_path(&ondas::HierarchyPath::from_components([variable.name()]));
                    matches(&format_path(&path), &name).then_some(path)
                })
                .collect::<HashSet<_>>();
            if paths.is_empty() {
                return Vec::new();
            }
            temporary = HierarchyIndex::build(inner.hierarchy(), false, Some(&paths));
            &temporary
        };
        let mut entries = index
            .declarations
            .iter()
            .filter(|declaration| declaration.visible)
            .filter(|declaration| {
                index.by_path[&declaration.path].len() == 1
                    && matches(&declaration.entry.path, &declaration.entry.name)
            })
            .collect::<Vec<_>>();
        entries.sort_by(|left, right| declaration_order(left, right));
        entries
            .into_iter()
            .map(|declaration| declaration.entry.clone())
            .collect()
    }

    pub fn signals_in_scope_report(&self, path: &str) -> Result<SignalListing, WavepeekError> {
        self.signals_in_scope_recursive_report(path, Some(0))
    }

    pub fn signals_in_scope_recursive_report(
        &self,
        path: &str,
        max_depth: Option<usize>,
    ) -> Result<SignalListing, WavepeekError> {
        let index = self.index();
        let parsed = ondas::HierarchyPath::parse(path)
            .map_err(|error| WavepeekError::Scope(error.to_string()))?;
        if parsed.is_empty() {
            let mut entries = Vec::new();
            let mut omitted = BTreeSet::new();
            for declaration in &index.declarations {
                if !declaration.visible
                    || max_depth.is_some_and(|max| declaration.path.len().saturating_sub(1) > max)
                {
                    continue;
                }
                if index.by_path[&declaration.path].len() > 1 {
                    omitted.insert(declaration.entry.path.clone());
                } else {
                    entries.push(declaration);
                }
            }
            entries.sort_by(|left, right| declaration_order(left, right));
            return Ok(SignalListing {
                entries: entries
                    .into_iter()
                    .map(|declaration| declaration.entry.clone())
                    .collect(),
                omitted_ambiguous_paths: omitted.into_iter().collect(),
            });
        }
        let Some(indices) = index.scope_by_path.get(&parsed) else {
            return Err(WavepeekError::Scope(format!(
                "scope '{path}' not found in dump"
            )));
        };
        if indices.len() != 1 {
            return Err(WavepeekError::Scope(format!(
                "scope '{path}' is ambiguous in dump"
            )));
        }
        let scope_index = indices[0];
        let scope_depth = index.scopes[scope_index].depth;
        let subtree_end = if max_depth == Some(0) {
            scope_index + 1
        } else {
            index.scopes[scope_index + 1..]
                .iter()
                .position(|scope| scope.depth <= scope_depth)
                .map_or(index.scopes.len(), |offset| scope_index + 1 + offset)
        };
        let mut entries = Vec::new();
        let mut omitted = BTreeSet::new();
        for parent in scope_index..subtree_end {
            let depth = index.scopes[parent].depth - scope_depth;
            if max_depth.is_some_and(|max| depth > max) {
                continue;
            }
            for &declaration_index in &index.declarations_by_scope[parent] {
                let declaration = &index.declarations[declaration_index];
                if !declaration.visible {
                    continue;
                }
                if index.by_path[&declaration.path].len() > 1 {
                    omitted.insert(declaration.entry.path.clone());
                } else {
                    entries.push((parent, &declaration.entry));
                }
            }
        }
        entries.sort_by(|(left_parent, left), (right_parent, right)| {
            left_parent
                .cmp(right_parent)
                .then_with(|| left.name.cmp(&right.name))
        });
        Ok(SignalListing {
            entries: entries
                .into_iter()
                .map(|(_, entry)| entry.clone())
                .collect(),
            omitted_ambiguous_paths: omitted.into_iter().collect(),
        })
    }

    fn declaration(&self, path: &str) -> Result<&Declaration, WavepeekError> {
        let parsed = ondas::HierarchyPath::parse(path)
            .map_err(|error| WavepeekError::Signal(error.to_string()))?;
        let index = self.index();
        let indices = index.by_path.get(&parsed).ok_or_else(|| {
            WavepeekError::SignalNotFound(format!("signal '{path}' not found in dump"))
        })?;
        if indices.len() != 1 {
            return Err(WavepeekError::Signal(format!(
                "signal '{path}' is ambiguous in dump: {} source declarations; no candidate was selected",
                indices.len()
            )));
        }
        Ok(&index.declarations[indices[0]])
    }

    pub fn resolve_signals(&self, paths: &[String]) -> Result<Vec<ResolvedSignal>, WavepeekError> {
        paths
            .iter()
            .map(|path| {
                if let Some(resolved) = self.direct_signal(path) {
                    return Ok(resolved);
                }
                let declaration = match self.value_declaration(path) {
                    Ok(declaration) => declaration,
                    Err(error) => {
                        if (matches!(error, WavepeekError::SignalNotFound(_))
                            || ondas::HierarchyPath::parse(path).is_err())
                            && let Some(projected) = self.resolve_projection(path)?
                        {
                            return Ok(projected);
                        }
                        return Err(error);
                    }
                };
                let width = declaration.entry.width.ok_or_else(|| unsupported(path))?;
                let id = declaration.id.ok_or_else(|| unsupported(path))?;
                Ok(ResolvedSignal {
                    path: declaration.entry.path.clone(),
                    id,
                    width,
                })
            })
            .collect()
    }

    pub fn resolve_expr_signal(&self, path: &str) -> Result<ExprResolvedSignal, WavepeekError> {
        if let Some(resolved) = self.direct_signal(path) {
            let direct = self.direct.borrow();
            let expr_type = direct
                .by_path
                .get(&resolved.path)
                .and_then(|&offset| direct.entries[offset].expr_type.clone());
            if let Some(expr_type) = expr_type {
                return Ok(ExprResolvedSignal {
                    path: resolved.path,
                    id: resolved.id,
                    expr_type,
                });
            }
        }
        let declaration = match self.declaration(path) {
            Ok(declaration) => declaration,
            Err(error) => {
                if (matches!(error, WavepeekError::SignalNotFound(_))
                    || ondas::HierarchyPath::parse(path).is_err())
                    && let Some(projected) = self.resolve_projection(path)?
                {
                    return self.resolve_expr_signal(&projected.path);
                }
                return Err(error);
            }
        };
        Ok(ExprResolvedSignal {
            path: declaration.entry.path.clone(),
            id: declaration.id.ok_or_else(|| unsupported(path))?,
            expr_type: declaration
                .expr_type
                .clone()
                .ok_or_else(|| unsupported(path))?,
        })
    }

    pub fn resolve_expr_signals(
        &self,
        paths: &[String],
    ) -> Result<Vec<ExprResolvedSignal>, WavepeekError> {
        paths
            .iter()
            .map(|path| self.resolve_expr_signal(path))
            .collect()
    }

    fn direct_signal(&self, path: &str) -> Option<ResolvedSignal> {
        let canonical = ondas::HierarchyPath::parse(path)
            .map(|path| format_path(&path))
            .unwrap_or_else(|_| path.to_owned());
        let mut direct = self.direct.borrow_mut();
        if let Some(resolved) = direct
            .resolved(path)
            .or_else(|| direct.resolved(&canonical))
        {
            return Some(resolved);
        }
        let inner = self.inner.borrow();
        let parsed = ondas::HierarchyPath::parse(path).ok()?;
        let variable = inner.hierarchy().variable_path(&parsed).ok()?;
        if variable
            .parent()
            .is_some_and(|scope| !visible_scope(&scope))
        {
            return None;
        }
        let signal = variable.signal()?;
        let width = match signal.encoding() {
            Encoding::Bits { width } => width,
            Encoding::Event => 0,
            _ => return None,
        };
        Some(direct.insert(DirectSignal {
            path: canonical,
            signal,
            width,
            expr_type: expression_type(&variable, signal),
        }))
    }

    fn resolve_projection(&self, path: &str) -> Result<Option<ResolvedSignal>, WavepeekError> {
        let Some((base, range)) = super::selection::parse_trailing_range(path)? else {
            return Ok(None);
        };
        let source = self.resolve_signals(&[base.to_owned()])?.remove(0);
        self.project_signal(&source, range.msb, range.lsb).map(Some)
    }

    pub fn project_signal(
        &self,
        source: &ResolvedSignal,
        msb: u32,
        lsb: u32,
    ) -> Result<ResolvedSignal, WavepeekError> {
        let mut path = format!("{}[{msb}:{lsb}]", source.path);
        // An existing literal declaration retains exact-name priority. Quote
        // the base component when necessary to make the slice reusable.
        if matches!(
            self.inner.borrow().hierarchy().variable(&path),
            Ok(_) | Err(ondas::LookupError::Ambiguous { .. })
        ) {
            let base = ondas::HierarchyPath::parse(&source.path)
                .map_err(|error| WavepeekError::Signal(error.to_string()))?;
            let name =
                serde_json::Value::String(base.name().unwrap_or_default().to_owned()).to_string();
            let parent = base
                .parent()
                .map(|path| format_path(&path))
                .unwrap_or_default();
            path = if parent.is_empty() {
                format!("{name}[{msb}:{lsb}]")
            } else {
                format!("{parent}.{name}[{msb}:{lsb}]")
            };
        }
        if let Some(resolved) = self.direct.borrow().resolved(&path) {
            return Ok(resolved);
        }
        let signal = self
            .signal(source.id)?
            .slice(msb, lsb)
            .map_err(|error| WavepeekError::Signal(error.to_string()))?;
        let mut expr_type = self.resolve_expr_signal(&source.path)?.expr_type;
        expr_type.width = msb - lsb + 1;
        expr_type.kind = ExprTypeKind::BitVector;
        expr_type.storage = ExprStorage::PackedVector;
        expr_type.is_signed = false;
        expr_type.enum_type_id = None;
        expr_type.enum_labels = None;
        Ok(self.direct.borrow_mut().insert(DirectSignal {
            path,
            signal,
            width: msb - lsb + 1,
            expr_type: Some(expr_type),
        }))
    }

    pub fn prepare_value_signals(&self, paths: &[String]) {
        for path in paths {
            self.direct_signal(path);
        }
    }

    pub fn previous_sample_time(&self, time: u64) -> Option<u64> {
        let first = self.inner.borrow().metadata().time_span()?.first().ticks();
        (time > first).then(|| time - 1)
    }

    fn signal(&self, id: SignalId) -> Result<ondas::Signal, WavepeekError> {
        if id.as_u64() & (1 << 63) != 0 {
            return self
                .direct
                .borrow()
                .entries
                .get((u64::MAX - id.as_u64()) as usize)
                .map(|direct| direct.signal)
                .ok_or_else(|| {
                    WavepeekError::Internal("invalid direct waveform signal handle".into())
                });
        }
        match self.index().signals.get(id.as_u64() as usize) {
            Some(SignalSource::Signal(signal)) => Ok(*signal),
            Some(SignalSource::Concat(parts)) => {
                if let Some(signal) = self.compositions.borrow().get(&id) {
                    return Ok(*signal);
                }
                let signal = self
                    .inner
                    .borrow_mut()
                    .concat(parts)
                    .map_err(|error| read_error(&self.path, "compose signals from", error))?;
                self.compositions.borrow_mut().insert(id, signal);
                Ok(signal)
            }
            _ => Err(WavepeekError::Internal(
                "invalid waveform signal handle".into(),
            )),
        }
    }

    fn cached_value(&self, id: SignalId, time: u64) -> Option<Option<ValueRef<'_>>> {
        let cached = self.traces.get(&id)?;
        let trace = &cached.trace;
        if !covers(trace, time, time) {
            return None;
        }
        let changes = trace.changes();
        let cursor = cached.cursor.get();
        let index = if changes
            .get(cursor)
            .is_some_and(|next| next.time().ticks() <= time)
        {
            if changes
                .get(cursor + 1)
                .is_some_and(|next| next.time().ticks() <= time)
            {
                cursor + changes[cursor..].partition_point(|change| change.time().ticks() <= time)
            } else {
                cursor + 1
            }
        } else if cursor > 0 && changes[cursor - 1].time().ticks() > time {
            if cursor > 1 && changes[cursor - 2].time().ticks() > time {
                changes[..cursor].partition_point(|change| change.time().ticks() <= time)
            } else {
                cursor - 1
            }
        } else {
            cursor
        };
        cached.cursor.set(index);
        if matches!(trace.signal().encoding(), Encoding::Event) {
            return Some(Some(
                index
                    .checked_sub(1)
                    .and_then(|index| {
                        let change = &changes[index];
                        (change.time().ticks() == time).then(|| change.value())
                    })
                    .unwrap_or(ValueRef::Event { occurrences: 0 }),
            ));
        }
        Some(
            index
                .checked_sub(1)
                .map(|index| changes[index].value())
                .or_else(|| trace.initial().map(|initial| initial.value())),
        )
    }

    fn values(&mut self, ids: &[SignalId], time: u64) -> Result<Vec<Option<Value>>, WavepeekError> {
        // Range commands revisit the same signals at many candidate ticks. Ondas
        // selections do not retain FST histories between queries; load each new
        // signal batch once for this command's window instead of replaying it.
        if let Some((from, to)) = self.sampling_window
            && self.inner.borrow().format() != Format::Fsdb
            && (from..=to).contains(&time)
        {
            self.preload(ids, from, to)?;
        }
        let mut result = Vec::with_capacity(ids.len());
        let mut missing = Vec::new();
        for (index, id) in ids.iter().enumerate() {
            if let Some(value) = self.cached_value(*id, time) {
                result.push(value.map(ValueRef::to_owned));
            } else {
                result.push(None);
                missing.push((index, self.signal(*id)?));
            }
        }
        if !missing.is_empty() {
            let signals = missing
                .iter()
                .map(|(_, signal)| *signal)
                .collect::<Vec<_>>();
            let samples = self
                .inner
                .borrow_mut()
                .samples(&signals, Time::from_ticks(time))
                .map_err(|error| read_error(&self.path, "read samples from", error))?;
            for ((index, _), sample) in missing.into_iter().zip(samples) {
                result[index] = match sample {
                    ondas::Sample::Value { value, .. } => Some(value),
                    ondas::Sample::Event { occurrences, .. } => Some(Value::Event { occurrences }),
                    _ => None,
                };
            }
        }
        Ok(result)
    }

    pub fn sample_resolved_optional(
        &mut self,
        resolved: &[ResolvedSignal],
        time: u64,
    ) -> Result<Vec<SampledSignalState>, WavepeekError> {
        if !resolved.is_empty()
            && self
                .inner
                .borrow()
                .metadata()
                .time_span()
                .is_some_and(|span| time < span.first().ticks())
        {
            return Err(WavepeekError::Internal(
                "query time is before first dump timestamp".into(),
            ));
        }
        let ids = resolved.iter().map(|signal| signal.id).collect::<Vec<_>>();
        let values = self.values(&ids, time)?;
        resolved
            .iter()
            .zip(values)
            .map(|(signal, value)| {
                let bits = match value.as_ref().map(Value::as_ref) {
                    Some(ValueRef::Bits(bits)) => Some(interpreted_bits(bits)),
                    Some(ValueRef::Event { .. }) => Some(String::new()),
                    None => None,
                    _ => return Err(unsupported(&signal.path)),
                };
                Ok(SampledSignalState {
                    path: signal.path.clone(),
                    width: signal.width,
                    bits,
                })
            })
            .collect()
    }

    pub fn sample_expr_value(
        &mut self,
        resolved: &ExprResolvedSignal,
        time: u64,
    ) -> Result<SampledValue, WavepeekError> {
        if matches!(resolved.expr_type.kind, ExprTypeKind::Event) {
            return Err(WavepeekError::Internal(format!(
                "signal '{}' is a raw event and cannot be sampled as a value",
                resolved.path
            )));
        }
        let values = self.values(&[resolved.id], time)?;
        match values[0].as_ref().map(Value::as_ref) {
            Some(ValueRef::Bits(bits)) => {
                let bits = bits.to_string();
                let label = resolved
                    .expr_type
                    .enum_labels
                    .as_ref()
                    .and_then(|labels| labels.iter().find(|label| label.bits == bits))
                    .map(|label| label.name.clone());
                Ok(SampledValue::Integral {
                    bits: Some(bits),
                    label,
                })
            }
            Some(ValueRef::Real(value)) => Ok(SampledValue::Real { value: Some(value) }),
            Some(ValueRef::String(value)) => Ok(SampledValue::String {
                value: Some(value.to_owned()),
            }),
            None => Ok(match resolved.expr_type.kind {
                ExprTypeKind::Real => SampledValue::Real { value: None },
                ExprTypeKind::String => SampledValue::String { value: None },
                _ => SampledValue::Integral {
                    bits: None,
                    label: None,
                },
            }),
            _ => Err(unsupported(&resolved.path)),
        }
    }

    pub fn expr_event_occurred(
        &mut self,
        resolved: &ExprResolvedSignal,
        time: u64,
    ) -> Result<bool, WavepeekError> {
        if !matches!(resolved.expr_type.kind, ExprTypeKind::Event) {
            return Err(WavepeekError::Internal(format!(
                "signal '{}' is not a raw event",
                resolved.path
            )));
        }
        let values = self.values(&[resolved.id], time)?;
        Ok(matches!(values[0], Some(Value::Event { occurrences }) if occurrences > 0))
    }

    pub fn validate_direct_value_supported(&self, path: &str) -> Result<(), WavepeekError> {
        self.value_declaration(path).map(|_| ())
    }

    fn value_declaration(&self, path: &str) -> Result<&Declaration, WavepeekError> {
        let declaration = self.declaration(path)?;
        if declaration.entry.width.is_none() {
            return Err(unsupported(path));
        }
        Ok(declaration)
    }

    fn preload(&mut self, ids: &[SignalId], from: u64, to: u64) -> Result<(), WavepeekError> {
        if from > to {
            return Ok(());
        }
        // Range sampling revisits cached handles at every candidate tick.
        if ids.iter().all(|id| {
            self.traces
                .get(id)
                .is_some_and(|cached| covers(&cached.trace, from, to))
        }) {
            return Ok(());
        }
        let mut missing = Vec::new();
        let mut seen = BTreeSet::new();
        for id in ids {
            if seen.insert(*id)
                && !self
                    .traces
                    .get(id)
                    .is_some_and(|cached| covers(&cached.trace, from, to))
            {
                missing.push((*id, self.signal(*id)?));
            }
        }
        if missing.is_empty() {
            return Ok(());
        }
        let signals = missing
            .iter()
            .map(|(_, signal)| *signal)
            .collect::<Vec<_>>();
        let traces = self
            .inner
            .borrow_mut()
            .traces(
                &signals,
                TimeRange::closed(Time::from_ticks(from), Time::from_ticks(to)),
            )
            .map_err(|error| read_error(&self.path, "read traces from", error))?;
        for ((id, _), trace) in missing.into_iter().zip(traces) {
            self.traces.insert(
                id,
                CachedTrace {
                    trace,
                    cursor: Cell::new(0),
                },
            );
        }
        Ok(())
    }

    pub fn preload_signal_ids(
        &mut self,
        ids: &[SignalId],
        from: u64,
        to: u64,
    ) -> Result<(), WavepeekError> {
        self.sampling_window = Some((from, to));
        self.preload(ids, from, to)
    }

    pub fn preload_expr_value_changes(
        &mut self,
        resolved: &[ExprResolvedSignal],
        from: u64,
        to: u64,
    ) -> Result<(), WavepeekError> {
        self.preload_signal_ids(
            &resolved.iter().map(|signal| signal.id).collect::<Vec<_>>(),
            from,
            to,
        )
    }

    pub fn preload_resolved_value_changes(
        &mut self,
        resolved: &[ResolvedSignal],
        from: u64,
        to: u64,
    ) -> Result<(), WavepeekError> {
        self.preload_signal_ids(
            &resolved.iter().map(|signal| signal.id).collect::<Vec<_>>(),
            from,
            to,
        )
    }

    // This grid indexes the loaded signals' net changes, not the file's global
    // timestamp table. Include the entering tick so indexed engines can seed state.
    pub fn ensure_indexed_signals_loaded(&mut self, ids: &[SignalId]) -> bool {
        if ids.iter().any(|id| {
            !matches!(
                self.signal(*id).map(|signal| signal.encoding()),
                Ok(Encoding::Bits { .. })
            )
        }) {
            return false;
        }
        let window = self.sampling_window.or_else(|| {
            self.inner
                .borrow()
                .metadata()
                .time_span()
                .map(|span| (span.first().ticks(), span.last().ticks()))
        });
        let Some((from, to)) = window else {
            return false;
        };
        if self.preload(ids, from, to).is_err() {
            return false;
        }
        let first = self
            .inner
            .borrow()
            .metadata()
            .time_span()
            .map_or(from, |span| span.first().ticks());
        let mut times = BTreeSet::from([from.max(first)]);
        for id in ids {
            if let Some(cached) = self.traces.get(id) {
                times.extend(
                    cached
                        .trace
                        .changes()
                        .iter()
                        .map(|change| change.time().ticks())
                        .filter(|time| (from..=to).contains(time)),
                );
            }
        }
        self.indexed_times = Some(times.into_iter().collect());
        true
    }

    pub fn indexed_timestamps(&self) -> Option<&[u64]> {
        self.indexed_times.as_deref()
    }

    pub fn indexed_signal_offset_at(&self, id: SignalId, index: u32) -> Option<SignalOffsetData> {
        let time = *self.indexed_times.as_ref()?.get(index as usize)?;
        let trace = &self.traces.get(&id)?.trace;
        let position = trace
            .changes()
            .partition_point(|change| change.time().ticks() <= time);
        if position == 0 && trace.initial().is_none() {
            return None;
        }
        Some(SignalOffsetData::new(position, 1))
    }

    pub fn decode_indexed_signal_at(
        &self,
        resolved: &ResolvedSignal,
        index: u32,
    ) -> Result<SampledSignalState, WavepeekError> {
        let missing = || {
            WavepeekError::Internal(format!(
                "signal '{}' could not be loaded from waveform backend",
                resolved.path
            ))
        };
        if !self.traces.contains_key(&resolved.id) {
            return Err(missing());
        }
        let time = self
            .indexed_times
            .as_ref()
            .and_then(|times| times.get(index as usize))
            .ok_or_else(missing)?;
        let value = self.cached_value(resolved.id, *time).ok_or_else(missing)?;
        let bits = match value {
            Some(ValueRef::Bits(bits)) => Some(interpreted_bits(bits)),
            None => None,
            _ => return Err(unsupported(&resolved.path)),
        };
        Ok(SampledSignalState {
            path: resolved.path.clone(),
            width: resolved.width,
            bits,
        })
    }

    fn candidate_times(
        &mut self,
        ids: &[SignalId],
        from: u64,
        to: u64,
        mode: ChangeCandidateCollectionMode,
    ) -> Result<Vec<u64>, WavepeekError> {
        if ids.is_empty() || from > to {
            return Ok(Vec::new());
        }
        if mode == ChangeCandidateCollectionMode::Stream
            && self.inner.borrow().format() == Format::Vcd
        {
            return Err(WavepeekError::Internal(
                "forced stream candidate collection requires FST input and a non-empty time window"
                    .into(),
            ));
        }
        let sample_from = from.saturating_sub(1);
        self.sampling_window = Some((sample_from, to));
        self.preload(ids, sample_from, to)?;
        let mut times = Vec::new();
        for id in ids {
            if let Some(cached) = self.traces.get(id) {
                times.extend(
                    cached
                        .trace
                        .changes()
                        .iter()
                        .map(|change| change.time().ticks())
                        .filter(|time| (from..=to).contains(time)),
                );
            }
        }
        times.sort_unstable();
        times.dedup();
        Ok(times)
    }

    pub fn collect_change_times_with_mode(
        &mut self,
        resolved: &[ResolvedSignal],
        from: u64,
        to: u64,
        mode: ChangeCandidateCollectionMode,
    ) -> Result<Vec<u64>, WavepeekError> {
        self.candidate_times(
            &resolved.iter().map(|signal| signal.id).collect::<Vec<_>>(),
            from,
            to,
            mode,
        )
    }

    pub fn collect_expr_candidate_times_with_mode(
        &mut self,
        resolved: &[ExprResolvedSignal],
        from: u64,
        to: u64,
        mode: ChangeCandidateCollectionMode,
    ) -> Result<Vec<u64>, WavepeekError> {
        self.candidate_times(
            &resolved.iter().map(|signal| signal.id).collect::<Vec<_>>(),
            from,
            to,
            mode,
        )
    }
}

fn visible_scope(scope: &ondas::Scope<'_>) -> bool {
    if scope.is_hidden() {
        return false;
    }
    let mut current = scope.parent();
    while let Some(scope) = current {
        if scope.is_hidden() {
            return false;
        }
        current = scope.parent();
    }
    true
}

impl HierarchyIndex {
    fn new(hierarchy: &ondas::Hierarchy, scopes_only: bool) -> Self {
        Self::build(hierarchy, scopes_only, None)
    }

    fn build(
        hierarchy: &ondas::Hierarchy,
        scopes_only: bool,
        selected: Option<&HashSet<ondas::HierarchyPath>>,
    ) -> Self {
        let mut scopes = if selected.is_none() {
            hierarchy
                .scopes()
                .filter(visible_scope)
                .map(|scope| (scope.path(), scope_type_alias(scope.kind()), scope))
                .collect::<Vec<_>>()
        } else {
            Vec::new()
        };
        scopes.sort_by(|left, right| left.0.components().cmp(right.0.components()));
        let mut scope_by_path: HashMap<ondas::HierarchyPath, Vec<usize>> = HashMap::new();
        let scope_entries = scopes
            .iter()
            .enumerate()
            .map(|(index, (path, kind, _))| {
                scope_by_path.entry(path.clone()).or_default().push(index);
                ScopeEntry {
                    depth: path.len().saturating_sub(1),
                    path: format_path(path),
                    kind: kind.clone(),
                }
            })
            .collect::<Vec<_>>();
        let mut index = Self {
            declarations_by_scope: vec![Vec::new(); scope_entries.len()],
            scopes: scope_entries,
            scope_by_path,
            declarations: Vec::new(),
            by_path: HashMap::new(),
            signals: Vec::new(),
        };
        if scopes_only {
            return index;
        }
        let mut ids = HashMap::new();
        let mut variables = Vec::new();
        for variable in hierarchy.variables() {
            if variable
                .parent()
                .is_some_and(|scope| !visible_scope(&scope))
            {
                continue;
            }
            let path = variable.path();
            if selected.is_some_and(|paths| !paths.contains(&path)) {
                continue;
            }
            let signal = variable.signal();
            let id = signal.map(|signal| {
                *ids.entry(signal).or_insert_with(|| {
                    let id = SignalId::from_backend_index(index.signals.len() as u64);
                    index.signals.push(SignalSource::Signal(signal));
                    id
                })
            });
            let parent = path
                .parent()
                .unwrap_or_else(|| ondas::HierarchyPath::from_components(Vec::<String>::new()));
            let declaration_index = index.declarations.len();
            if let Some(parents) = index.scope_by_path.get(&parent) {
                for &parent in parents {
                    if variable.parent().as_ref() == Some(&scopes[parent].2) {
                        index.declarations_by_scope[parent].push(declaration_index);
                    }
                }
            }
            index
                .by_path
                .entry(path.clone())
                .or_default()
                .push(declaration_index);
            index.declarations.push(Declaration {
                path: path.clone(),
                entry: SignalEntry {
                    name: format_path(&ondas::HierarchyPath::from_components([variable.name()])),
                    path: format_path(&path),
                    kind: if signal
                        .is_some_and(|signal| matches!(signal.encoding(), Encoding::Event))
                    {
                        "event".into()
                    } else {
                        var_type_alias(variable.kind())
                    },
                    width: signal.and_then(|signal| match signal.encoding() {
                        Encoding::Bits { width } => Some(width),
                        Encoding::Event => Some(0),
                        _ => None,
                    }),
                },
                id,
                expr_type: signal.and_then(|signal| expression_type(&variable, signal)),
                visible: true,
            });
            variables.push(variable);
        }
        for indices in index
            .by_path
            .values_mut()
            .filter(|indices| indices.len() > 1)
        {
            let Some(parts) = composition_parts(indices.iter().map(|&i| &variables[i])) else {
                continue;
            };
            let width = parts.iter().map(|part| part.width().unwrap_or(0)).sum();
            let id = SignalId::from_backend_index(index.signals.len() as u64);
            index.signals.push(SignalSource::Concat(parts));
            for &i in &indices[1..] {
                index.declarations[i].visible = false;
            }
            let first = &mut index.declarations[indices[0]];
            first.id = Some(id);
            first.entry.width = Some(width);
            if let Some(expr_type) = &mut first.expr_type {
                expr_type.width = width;
                expr_type.storage = ExprStorage::PackedVector;
                expr_type.enum_labels = None;
            }
            indices.truncate(1);
        }
        index
    }
}

// A shared name alone does not establish a vector. Require explicit ranges in
// one owning scope and a complete, consistently directed layout.
fn composition_parts<'a, 'h>(
    variables: impl Iterator<Item = &'a ondas::Variable<'h>>,
) -> Option<Vec<ondas::Signal>>
where
    'h: 'a,
{
    let mut parts = Vec::new();
    let mut parent = None;
    let mut direction = None;
    for variable in variables {
        if let Some(parent) = &parent {
            if *parent != variable.parent() {
                return None;
            }
        } else {
            parent = Some(variable.parent());
        }
        let range = variable.range()?;
        let signal = variable.signal()?;
        let width = signal.width()?;
        if range.msb().abs_diff(range.lsb()).checked_add(1)? != u64::from(width) {
            return None;
        }
        if range.msb() != range.lsb() {
            let descending = range.msb() > range.lsb();
            if direction.is_some_and(|current| current != descending) {
                return None;
            }
            direction = Some(descending);
        }
        parts.push((
            range.msb().min(range.lsb()),
            range.msb().max(range.lsb()),
            signal,
        ));
    }
    parts.sort_by_key(|part| part.0);
    if parts
        .windows(2)
        .any(|pair| pair[0].1.checked_add(1) != Some(pair[1].0))
    {
        return None;
    }
    let width = parts
        .iter()
        .try_fold(0u32, |width, part| width.checked_add(part.2.width()?))?;
    if width == 0 {
        return None;
    }
    if direction.unwrap_or(true) {
        parts.reverse();
    }
    Some(parts.into_iter().map(|part| part.2).collect())
}

fn interpreted_bits(bits: ondas::BitsRef<'_>) -> String {
    let text = bits.to_string();
    if text
        .bytes()
        .all(|byte| matches!(byte, b'0' | b'1' | b'x' | b'z'))
    {
        return text;
    }
    let mut bytes = text.into_bytes();
    for byte in &mut bytes {
        *byte = crate::logic::four_state(char::from(*byte)) as u8;
    }
    String::from_utf8(bytes).expect("four-state logic is ASCII")
}

fn expression_type(variable: &ondas::Variable<'_>, signal: ondas::Signal) -> Option<ExprType> {
    let enumeration = variable.enumeration();
    let mut ty = expression_value_type(
        signal.encoding(),
        variable.kind(),
        variable.signedness(),
        variable.logic_domain(),
        enumeration.is_some(),
    )?;
    ty.enum_type_id = enumeration
        .as_ref()
        .and_then(|value| value.name())
        .map(str::to_owned);
    ty.enum_labels = enumeration.map(|value| {
        value
            .variants()
            .map(|variant| EnumLabelInfo {
                name: variant.label.to_owned(),
                bits: variant.encoded.to_string(),
            })
            .collect()
    });
    Some(ty)
}

fn expression_value_type(
    encoding: Encoding,
    declaration_kind: &str,
    signedness: Option<ondas::Signedness>,
    logic_domain: Option<ondas::LogicDomain>,
    has_enumeration: bool,
) -> Option<ExprType> {
    let (kind, width, scalar, signed) = match encoding {
        Encoding::Real => (ExprTypeKind::Real, 64, true, false),
        Encoding::String => (ExprTypeKind::String, 0, true, false),
        Encoding::Event => (ExprTypeKind::Event, 0, true, false),
        Encoding::Bits { width } => {
            let integer = match declaration_kind {
                "byte" => Some(IntegerLikeKind::Byte),
                "shortint" | "short-int" | "short_int" => Some(IntegerLikeKind::Shortint),
                "int" => Some(IntegerLikeKind::Int),
                "longint" | "long-int" | "long_int" => Some(IntegerLikeKind::Longint),
                "integer" => Some(IntegerLikeKind::Integer),
                "time" => Some(IntegerLikeKind::Time),
                _ => None,
            };
            if has_enumeration || declaration_kind == "enum" {
                (ExprTypeKind::EnumCore, width, true, false)
            } else if let Some(integer) = integer {
                (
                    ExprTypeKind::IntegerLike(integer),
                    width,
                    true,
                    integer != IntegerLikeKind::Time,
                )
            } else {
                (ExprTypeKind::BitVector, width, width <= 1, false)
            }
        }
        _ => return None,
    };
    let integral = matches!(encoding, Encoding::Bits { .. });
    Some(ExprType {
        kind,
        width,
        storage: if scalar {
            ExprStorage::Scalar
        } else {
            ExprStorage::PackedVector
        },
        is_signed: integral
            && signedness.map_or(signed, |value| value == ondas::Signedness::Signed),
        // An unavailable domain is a four-state language default, not recovered metadata.
        is_four_state: integral && logic_domain != Some(ondas::LogicDomain::TwoState),
        enum_type_id: None,
        enum_labels: None,
    })
}

fn scope_type_alias(kind: &str) -> String {
    let kind = kind.replace('-', "_");
    if matches!(
        kind.as_str(),
        "modport" | "modport_ref" | "interface_port_ref"
    ) {
        "interface".into()
    } else if super::types::STABLE_SCOPE_KIND_ALIASES.contains(&kind.as_str()) {
        kind
    } else {
        "unknown".into()
    }
}

fn var_type_alias(kind: &str) -> String {
    let normalized = kind.replace('-', "_");
    let kind = match normalized.as_str() {
        "shortint" => "short_int".into(),
        "longint" => "long_int".into(),
        "realtime" => "real_time".into(),
        "realparameter" => "real_parameter".into(),
        "shortreal" => "short_real".into(),
        "sparsearray" => "sparse_array".into(),
        "bitvector" | "stdlogicvector" | "stdulogicvector" | "std_logic_vector"
        | "std_ulogic_vector" => "bit_vector".into(),
        "stdlogic" | "stdulogic" | "std_logic" | "std_ulogic" => "logic".into(),
        _ => normalized,
    };
    if super::types::STABLE_SIGNAL_KIND_ALIASES.contains(&kind.as_str()) {
        kind
    } else {
        "bit_vector".into()
    }
}

fn covers(trace: &Trace, from: u64, to: u64) -> bool {
    trace.range().start().ticks() <= from && trace.range().end().is_none_or(|end| to <= end.ticks())
}

fn time_unit(unit: ondas::TimeUnit) -> Result<&'static str, WavepeekError> {
    Ok(match unit {
        ondas::TimeUnit::Second => "s",
        ondas::TimeUnit::Millisecond => "ms",
        ondas::TimeUnit::Microsecond => "us",
        ondas::TimeUnit::Nanosecond => "ns",
        ondas::TimeUnit::Picosecond => "ps",
        ondas::TimeUnit::Femtosecond => "fs",
        ondas::TimeUnit::Attosecond => "as",
        ondas::TimeUnit::Zeptosecond => "zs",
        _ => {
            return Err(WavepeekError::File(
                "waveform has unknown timescale unit".into(),
            ));
        }
    })
}

fn unsupported(path: &str) -> WavepeekError {
    WavepeekError::Signal(format!(
        "signal '{path}' has unsupported non-bit-vector encoding"
    ))
}
