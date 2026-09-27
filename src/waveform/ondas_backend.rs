//! Ondas-backed hierarchy and value access.

#[cfg(all(test, feature = "fsdb"))]
#[path = "ondas_fsdb_tests.rs"]
mod fsdb_tests;
#[cfg(test)]
#[path = "ondas_tests.rs"]
mod tests;

use std::borrow::Cow;
use std::cell::{Cell, OnceCell, RefCell};
use std::collections::{BTreeSet, HashMap, HashSet};
use std::fmt;
use std::path::Path;
use std::sync::Arc;

use ondas::{Encoding, Format, Time, TimeRange, Trace, Value, ValueRef};

use crate::error::WavepeekError;
use crate::expr::{
    EnumLabelInfo, ExprStorage, ExprType, ExprTypeKind, IntegerLikeKind, SampledValue,
};

use super::types::{
    ChangeCandidateCollectionMode, ExprResolvedSignal, ResolvedSignal, SampledSignalState,
    ScopeEntry, SignalEntry, SignalId, SignalListing, SignalOffsetData, WaveformMetadata,
};

const MAX_DIRECT_SIGNALS: usize = 128;
const MAX_DIRECT_FST_BATCH: usize = 1024;

pub(super) struct OndasBackend {
    inner: ondas::Waveform,
    index: OnceCell<HierarchyIndex>,
    direct: RefCell<Vec<DirectSignal>>,
    traces: HashMap<SignalId, CachedTrace>,
    sampling_window: Option<(u64, u64)>,
    indexed_times: Option<Vec<u64>>,
}

impl fmt::Debug for OndasBackend {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("OndasBackend")
            .field("format", &self.inner.format())
            .finish_non_exhaustive()
    }
}

struct Declaration {
    entry: SignalEntry,
    parent: String,
    parent_order: usize,
    id: Option<SignalId>,
    expr_type: Option<ExprType>,
    range: Option<ondas::BitRange>,
    visible: bool,
}

struct BitPart {
    id: SignalId,
    lsb: usize,
    width: usize,
}

struct DirectSignal {
    path: String,
    signal: ondas::Signal,
    width: u32,
    expr_type: Option<ExprType>,
}

struct CachedTrace {
    trace: Trace,
    cursor: Cell<usize>,
}

enum SignalSource {
    Signal(ondas::Signal),
    Split(Vec<BitPart>),
}

struct HierarchyIndex {
    scopes: Vec<ScopeEntry>,
    declarations: Vec<Declaration>,
    by_path: HashMap<String, Vec<usize>>,
    signals: Vec<SignalSource>,
}

impl OndasBackend {
    pub fn open(path: &Path) -> Result<Self, WavepeekError> {
        #[cfg(feature = "fsdb")]
        let result = super::fsdb_output::quiet(|| ondas::open(path))?;
        #[cfg(not(feature = "fsdb"))]
        let result = ondas::open(path);
        let inner = result.map_err(|error| open_error(path, error))?;
        if inner.format() == Format::Fsdb {
            let mut paths = BTreeSet::new();
            for scope in inner
                .hierarchy()
                .scopes()
                .filter(|scope| visible_scope(scope) && !skipped_fsdb_scope(scope, Format::Fsdb))
            {
                let path = scope_path(&scope, Format::Fsdb);
                if !paths.insert(path.clone()) {
                    return Err(WavepeekError::File(format!(
                        "FSDB hierarchy contains ambiguous canonical scope path '{path}'"
                    )));
                }
            }
        }
        Ok(Self::new(inner))
    }

    pub fn open_bytes(path: &Path, bytes: Arc<[u8]>) -> Result<Self, WavepeekError> {
        let inner = ondas::open_bytes(path.to_string_lossy(), bytes)
            .map_err(|error| open_error(path, error))?;
        if !matches!(inner.format(), Format::Vcd | Format::Fst) {
            return Err(WavepeekError::File(
                "the browser supports only VCD and FST waveforms".into(),
            ));
        }
        Ok(Self::new(inner))
    }

    fn new(inner: ondas::Waveform) -> Self {
        Self {
            inner,
            index: OnceCell::new(),
            direct: RefCell::new(Vec::new()),
            traces: HashMap::new(),
            sampling_window: None,
            indexed_times: None,
        }
    }

    pub fn backend_name(&self) -> &'static str {
        "ondas"
    }

    pub fn format_name(&self) -> &'static str {
        match self.inner.format() {
            Format::Vcd => "vcd",
            Format::Fst => "fst",
            Format::Fsdb => "fsdb",
            _ => "unknown",
        }
    }

    pub fn metadata(&self) -> Result<WaveformMetadata, WavepeekError> {
        Self::format_metadata(self.inner.metadata())
    }

    pub fn read_metadata(path: &Path) -> Result<WaveformMetadata, WavepeekError> {
        #[cfg(feature = "fsdb")]
        let result = super::fsdb_output::quiet(|| ondas::read_metadata(path))?;
        #[cfg(not(feature = "fsdb"))]
        let result = ondas::read_metadata(path);
        let metadata = result.map_err(|error| open_error(path, error))?;
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
            .get_or_init(|| HierarchyIndex::new(self.inner.hierarchy(), self.inner.format(), false))
    }

    pub fn scopes_depth_first(
        &self,
        max_depth: Option<usize>,
    ) -> Result<Vec<ScopeEntry>, WavepeekError> {
        let mut scopes = self.index.get().map_or_else(
            || HierarchyIndex::new(self.inner.hierarchy(), self.inner.format(), true).scopes,
            |index| index.scopes.clone(),
        );
        scopes.retain(|scope| max_depth.is_none_or(|max| scope.depth <= max));
        Ok(scopes)
    }

    pub fn signals_in_scope(&self, path: &str) -> Result<Vec<SignalEntry>, WavepeekError> {
        Ok(self.signals_in_scope_report(path)?.entries)
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
        let Some(scope_index) = index.scopes.iter().position(|scope| scope.path == path) else {
            return Err(WavepeekError::Scope(format!(
                "scope '{path}' not found in dump"
            )));
        };
        let scope_depth = index.scopes[scope_index].depth;
        let subtree_end = index.scopes[scope_index + 1..]
            .iter()
            .position(|scope| scope.depth <= scope_depth)
            .map_or(index.scopes.len(), |offset| scope_index + 1 + offset);
        let subtree_orders = scope_index + 1..subtree_end + 1;
        let mut entries = Vec::new();
        let mut omitted = BTreeSet::new();
        for declaration in &index.declarations {
            if !declaration.visible {
                continue;
            }
            if !subtree_orders.contains(&declaration.parent_order) {
                continue;
            }
            let depth = index.scopes[declaration.parent_order - 1].depth - scope_depth;
            if max_depth.is_some_and(|max| depth > max) {
                continue;
            }
            if index.by_path[&declaration.entry.path].len() > 1 {
                omitted.insert(declaration.entry.path.clone());
            } else {
                entries.push((declaration.parent_order, &declaration.entry));
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
        let index = self.index();
        let indices = index.by_path.get(path).ok_or_else(|| {
            WavepeekError::SignalNotFound(format!("signal '{path}' not found in dump"))
        })?;
        if indices.len() != 1 {
            let message = if self.inner.format() == Format::Fsdb {
                format!("signal '{path}' is ambiguous in FSDB hierarchy; no candidate was selected")
            } else {
                format!("signal '{path}' is ambiguous in dump")
            };
            return Err(WavepeekError::Signal(message));
        }
        Ok(&index.declarations[indices[0]])
    }

    pub fn resolve_signals(&self, paths: &[String]) -> Result<Vec<ResolvedSignal>, WavepeekError> {
        paths
            .iter()
            .map(|path| {
                if let Some(resolved) = self
                    .direct_fst_signal(path)
                    .or_else(|| self.direct_fsdb_signal(path))
                {
                    return Ok(resolved);
                }
                self.validate_direct_value_supported(path)?;
                let declaration = self.declaration(path)?;
                let width = declaration.entry.width.ok_or_else(|| unsupported(path))?;
                let id = declaration.id.ok_or_else(|| unsupported(path))?;
                Ok(ResolvedSignal {
                    path: path.clone(),
                    id,
                    width,
                })
            })
            .collect()
    }

    pub fn resolve_expr_signal(&self, path: &str) -> Result<ExprResolvedSignal, WavepeekError> {
        if let Some(resolved) = self
            .direct_fst_signal(path)
            .or_else(|| self.direct_fsdb_signal(path))
        {
            let offset = (u64::MAX - resolved.id.as_u64()) as usize;
            if let Some(expr_type) = self.direct.borrow()[offset].expr_type.clone() {
                return Ok(ExprResolvedSignal {
                    path: path.to_owned(),
                    id: resolved.id,
                    expr_type,
                });
            }
        }
        let declaration = self.declaration(path)?;
        Ok(ExprResolvedSignal {
            path: path.to_owned(),
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

    fn direct_fst_signal(&self, path: &str) -> Option<ResolvedSignal> {
        if self.inner.format() != Format::Fst {
            return None;
        }
        if let Some((offset, width)) = self
            .direct
            .borrow()
            .iter()
            .enumerate()
            .find_map(|(offset, direct)| (direct.path == path).then_some((offset, direct.width)))
        {
            return Some(ResolvedSignal {
                path: path.to_owned(),
                id: SignalId::from_backend_index(u64::MAX - offset as u64),
                width,
            });
        }
        // ponytail: cap repeated scans at 128 paths; batch indexed lookup if larger scans matter.
        if self.index.get().is_some()
            || self.direct.borrow().len() >= MAX_DIRECT_SIGNALS
            || path.contains(['[', ']', '\\', '/'])
        {
            return None;
        }
        let hierarchy = self.inner.hierarchy();
        let selector = ondas::HierarchyPath::parse(path).ok()?;
        let leaf = selector.name()?;
        let mut matches = hierarchy
            .variables()
            .filter(|variable| variable.name() == leaf && variable.path() == selector);
        let variable = matches.next()?;
        if matches.next().is_some() {
            return None;
        }
        if variable
            .parent()
            .is_some_and(|parent| !visible_scope(&parent))
        {
            return None;
        }
        let signal = variable.signal()?;
        let width = match signal.encoding() {
            Encoding::Bits { width } => width,
            Encoding::Event => 0,
            _ => return None,
        };
        let parent = variable
            .parent()
            .map(|scope| scope_path(&scope, Format::Fst))
            .unwrap_or_default();
        let name = public_component(variable.name(), Format::Fst, variable.name_was_escaped());
        let canonical = if parent.is_empty() {
            name.into_owned()
        } else {
            format!("{parent}.{name}")
        };
        if canonical != path {
            return None;
        }
        // Packed FST fragments can have a different SDK name but the same public path.
        let prefix = format!("{}[", variable.name());
        let parent_path = variable.parent().map(|scope| scope.path());
        if hierarchy.variables().any(|other| {
            other.name().starts_with(&prefix)
                && other.parent().map(|scope| scope.path()) == parent_path
        }) {
            return None;
        }
        let mut direct = self.direct.borrow_mut();
        let id = SignalId::from_backend_index(u64::MAX - direct.len() as u64);
        direct.push(DirectSignal {
            path: path.to_owned(),
            signal,
            width,
            expr_type: expression_type(&variable, signal),
        });
        Some(ResolvedSignal {
            path: path.to_owned(),
            id,
            width,
        })
    }

    fn direct_fsdb_signal(&self, path: &str) -> Option<ResolvedSignal> {
        if self.inner.format() != Format::Fsdb {
            return None;
        }
        if let Some((offset, width)) = self
            .direct
            .borrow()
            .iter()
            .enumerate()
            .find_map(|(offset, direct)| (direct.path == path).then_some((offset, direct.width)))
        {
            return Some(ResolvedSignal {
                path: path.to_owned(),
                id: SignalId::from_backend_index(u64::MAX - offset as u64),
                width,
            });
        }
        if self.index.get().is_some() || self.direct.borrow().len() >= MAX_DIRECT_SIGNALS {
            return None;
        }
        let leaf = path.rsplit('.').next().filter(|leaf| !leaf.is_empty())?;
        let mut matched = None;
        for variable in self.inner.hierarchy().variables() {
            if variable
                .parent()
                .is_some_and(|parent| !visible_scope(&parent))
            {
                continue;
            }
            let raw_name = variable.reader_name().unwrap_or(variable.name()).trim();
            // The terminal public component survives FSDB spelling normalization.
            if !raw_name.contains(leaf) {
                continue;
            }
            let public_path = public_fsdb_variable_path(&variable);
            if public_path == path {
                if matched.is_some() {
                    return None;
                }
                matched = Some(variable);
            }
        }
        let variable = matched?;
        let signal = variable.signal()?;
        let Encoding::Bits { width } = signal.encoding() else {
            return None;
        };
        let mut direct = self.direct.borrow_mut();
        let id = SignalId::from_backend_index(u64::MAX - direct.len() as u64);
        direct.push(DirectSignal {
            path: path.to_owned(),
            signal,
            width,
            expr_type: expression_type(&variable, signal),
        });
        Some(ResolvedSignal {
            path: path.to_owned(),
            id,
            width,
        })
    }

    pub fn prepare_value_signals(&self, paths: &[String]) {
        if self.index.get().is_some() || !matches!(self.inner.format(), Format::Fst | Format::Fsdb)
        {
            return;
        }
        let batch_limit = if self.inner.format() == Format::Fst {
            MAX_DIRECT_FST_BATCH
        } else {
            MAX_DIRECT_SIGNALS
        };
        if paths.len() > batch_limit {
            self.index();
            return;
        }
        if paths.len() < 2 {
            return;
        }
        if self.inner.format() == Format::Fst {
            self.prepare_fst_signals(paths);
            return;
        }
        let targets = paths.iter().map(String::as_str).collect::<HashSet<_>>();
        let leaves = paths
            .iter()
            .filter_map(|path| path.rsplit('.').next().filter(|leaf| !leaf.is_empty()))
            .collect::<HashSet<_>>();
        let has_array_leaf = leaves.iter().any(|leaf| leaf.contains('['));
        let mut matches = HashMap::<String, Option<ondas::Variable<'_>>>::new();
        for variable in self.inner.hierarchy().variables() {
            if variable
                .parent()
                .is_some_and(|parent| !visible_scope(&parent))
            {
                continue;
            }
            let raw_name = variable.reader_name().unwrap_or(variable.name()).trim();
            // Only qualified or indexed names need a substring match beyond their leaf/base.
            let name = raw_name.strip_prefix('\\').unwrap_or(raw_name);
            let base = name.rsplit_once('[').map(|(base, _)| base);
            let needs_partial = name.contains(['.', '/']) || (has_array_leaf && name.contains('['));
            if !leaves.contains(name)
                && !base.is_some_and(|base| leaves.contains(base))
                && (!needs_partial || !leaves.iter().any(|leaf| raw_name.contains(leaf)))
            {
                continue;
            }
            let public_path = public_fsdb_variable_path(&variable);
            if targets.contains(public_path.as_str()) {
                matches
                    .entry(public_path)
                    .and_modify(|matched| *matched = None)
                    .or_insert(Some(variable));
            }
        }
        let mut direct = self.direct.borrow_mut();
        for path in paths {
            if direct.iter().any(|entry| entry.path == *path) {
                continue;
            }
            let Some(Some(variable)) = matches.remove(path) else {
                continue;
            };
            let Some(signal) = variable.signal() else {
                continue;
            };
            let Encoding::Bits { width } = signal.encoding() else {
                continue;
            };
            direct.push(DirectSignal {
                path: path.clone(),
                signal,
                width,
                expr_type: expression_type(&variable, signal),
            });
        }
    }

    fn prepare_fst_signals(&self, paths: &[String]) {
        let targets = paths
            .iter()
            .filter(|path| {
                !path.contains(['\\', '/']) && (!path.contains(['[', ']']) || path.contains(".["))
            })
            .filter_map(|path| {
                let sdk_path = path.replace(".[", "[");
                ondas::HierarchyPath::parse(&sdk_path)
                    .ok()
                    .map(|key| (key, path.as_str()))
            })
            .collect::<HashMap<_, _>>();
        let mut parent_names = HashMap::<&str, HashSet<Option<String>>>::new();
        for target in targets.keys() {
            if let Some(name) = target.name() {
                parent_names.entry(name).or_default().insert(
                    target
                        .parent()
                        .and_then(|parent| parent.name().map(str::to_owned)),
                );
            }
        }
        let leaves = parent_names.keys().copied().collect::<HashSet<_>>();
        let array_targets = paths
            .iter()
            .filter(|path| path.contains(".[") && !path.contains(['\\', '/']))
            .map(String::as_str)
            .collect::<HashSet<_>>();
        let array_leaves = array_targets
            .iter()
            .filter_map(|path| path.rsplit('.').next())
            .collect::<HashSet<_>>();
        let mut matches = HashMap::<&str, Option<ondas::Variable<'_>>>::new();
        let mut fragments = HashSet::new();
        for variable in self.inner.hierarchy().variables() {
            let name = variable.name();
            if let Some((base, _)) = name.split_once('[')
                && leaves.contains(base)
            {
                fragments.insert((variable.parent().map(|scope| scope.path()), base.to_owned()));
            }
            let array_path = if array_leaves.iter().any(|leaf| name.ends_with(leaf)) {
                public_fst_variable_path(&variable)
                    .and_then(|public| array_targets.get(public.as_str()).copied())
            } else {
                None
            };
            let path = array_path.or_else(|| {
                parent_names.get(name).and_then(|parents| {
                    let parent_name = variable.parent().map(|parent| parent.name());
                    parents
                        .iter()
                        .any(|candidate| candidate.as_deref() == parent_name)
                        .then(|| targets.get(&variable.path()).copied())
                        .flatten()
                })
            });
            if let Some(path) = path {
                matches
                    .entry(path)
                    .and_modify(|matched| *matched = None)
                    .or_insert(Some(variable));
            }
        }
        let mut direct = self.direct.borrow_mut();
        for path in paths {
            if direct.iter().any(|entry| entry.path == *path) {
                continue;
            }
            let Some(Some(variable)) = matches.remove(path.as_str()) else {
                continue;
            };
            if variable
                .parent()
                .is_some_and(|parent| !visible_scope(&parent))
                || fragments.contains(&(
                    variable.parent().map(|scope| scope.path()),
                    variable.name().to_owned(),
                ))
            {
                continue;
            }
            let Some(signal) = variable.signal() else {
                continue;
            };
            let width = match signal.encoding() {
                Encoding::Bits { width } => width,
                Encoding::Event => 0,
                _ => continue,
            };
            if public_fst_variable_path(&variable).as_deref() != Some(path) {
                continue;
            }
            direct.push(DirectSignal {
                path: path.clone(),
                signal,
                width,
                expr_type: expression_type(&variable, signal),
            });
        }
    }

    pub fn previous_sample_time(&self, time: u64) -> Option<u64> {
        let first = self.inner.metadata().time_span()?.first().ticks();
        (time > first).then(|| time - 1)
    }

    fn signal(&self, id: SignalId) -> Result<ondas::Signal, WavepeekError> {
        if id.as_u64() & (1 << 63) != 0 {
            return self
                .direct
                .borrow()
                .get((u64::MAX - id.as_u64()) as usize)
                .map(|direct| direct.signal)
                .ok_or_else(|| {
                    WavepeekError::Internal("invalid direct waveform signal handle".into())
                });
        }
        match self.index().signals.get(id.as_u64() as usize) {
            Some(SignalSource::Signal(signal)) => Ok(*signal),
            _ => Err(WavepeekError::Internal(
                "invalid or split waveform signal handle".into(),
            )),
        }
    }

    fn physical_ids(&self, ids: &[SignalId]) -> Vec<SignalId> {
        ids.iter()
            .flat_map(|id| {
                if id.as_u64() & (1 << 63) != 0 {
                    return vec![*id];
                }
                match self.index().signals.get(id.as_u64() as usize) {
                    Some(SignalSource::Split(parts)) => parts.iter().map(|part| part.id).collect(),
                    _ => vec![*id],
                }
            })
            .collect()
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
            return Some(index.checked_sub(1).and_then(|index| {
                let change = &changes[index];
                (change.time().ticks() == time).then(|| change.value())
            }));
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
                .samples(&signals, Time::from_ticks(time))
                .map_err(query_error)?;
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
                .metadata()
                .time_span()
                .is_some_and(|span| time < span.first().ticks())
        {
            return Err(WavepeekError::Internal(
                "query time is before first dump timestamp".into(),
            ));
        }
        let ids = resolved.iter().map(|signal| signal.id).collect::<Vec<_>>();
        if ids.iter().any(|id| {
            id.as_u64() & (1 << 63) == 0
                && matches!(
                    self.index().signals.get(id.as_u64() as usize),
                    Some(SignalSource::Split(_))
                )
        }) {
            let physical = self.physical_ids(&ids);
            let values = self.values(&physical, time)?;
            let by_id = physical.into_iter().zip(values).collect::<HashMap<_, _>>();
            return resolved
                .iter()
                .map(|signal| {
                    let bits = match self.index().signals.get(signal.id.as_u64() as usize) {
                        Some(SignalSource::Split(parts)) => {
                            let mut bits = vec![b'x'; signal.width as usize];
                            let mut present = false;
                            for part in parts {
                                if let Some(Some(Value::Bits(value))) = by_id.get(&part.id) {
                                    let value = value.as_ref().to_string();
                                    let end = bits.len() - part.lsb;
                                    bits[end - part.width..end].copy_from_slice(value.as_bytes());
                                    present = true;
                                }
                            }
                            present
                                .then(|| String::from_utf8(bits).expect("logic values are ASCII"))
                        }
                        _ => match by_id.get(&signal.id).and_then(Option::as_ref) {
                            Some(Value::Bits(value)) => Some(value.as_ref().to_string()),
                            None => None,
                            _ => return Err(unsupported(&signal.path)),
                        },
                    };
                    Ok(SampledSignalState {
                        path: signal.path.clone(),
                        width: signal.width,
                        bits,
                    })
                })
                .collect();
        }
        let values = self.values(&ids, time)?;
        resolved
            .iter()
            .zip(values)
            .map(|(signal, value)| {
                let bits = match value.as_ref().map(Value::as_ref) {
                    Some(ValueRef::Bits(bits)) => Some(bits.to_string()),
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
        self.validate_expr_values_supported(std::slice::from_ref(resolved))?;
        if resolved.id.as_u64() & (1 << 63) == 0
            && matches!(
                self.index().signals.get(resolved.id.as_u64() as usize),
                Some(SignalSource::Split(_))
            )
        {
            let signal = ResolvedSignal {
                path: resolved.path.clone(),
                id: resolved.id,
                width: resolved.expr_type.width,
            };
            let mut values = self.sample_resolved_optional(&[signal], time)?;
            return Ok(SampledValue::Integral {
                bits: values.remove(0).bits,
                label: None,
            });
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
        let declaration = self.declaration(path)?;
        if declaration.entry.width.is_none() {
            return Err(unsupported(path));
        }
        if self.inner.format() == Format::Fsdb {
            let id = declaration.id.ok_or_else(|| unsupported(path))?;
            if !matches!(self.signal(id)?.encoding(), Encoding::Bits { .. }) {
                return Err(unsupported(path));
            }
        }
        Ok(())
    }

    pub fn validate_expr_values_supported(
        &self,
        signals: &[ExprResolvedSignal],
    ) -> Result<(), WavepeekError> {
        if self.inner.format() == Format::Fsdb {
            for signal in signals {
                if matches!(
                    signal.expr_type.kind,
                    ExprTypeKind::Real | ExprTypeKind::String
                ) {
                    return Err(WavepeekError::Signal(format!(
                        "signal '{}' has unsupported FSDB expression value encoding",
                        signal.path
                    )));
                }
            }
        }
        Ok(())
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
        let ids = self.physical_ids(ids);
        let mut missing = Vec::new();
        let mut seen = BTreeSet::new();
        for id in &ids {
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
            .traces(
                &signals,
                TimeRange::closed(Time::from_ticks(from), Time::from_ticks(to)),
            )
            .map_err(query_error)?;
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
        self.validate_expr_values_supported(resolved)?;
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
            Some(ValueRef::Bits(bits)) => Some(bits.to_string()),
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
        if mode == ChangeCandidateCollectionMode::Stream && self.inner.format() == Format::Vcd {
            return Err(WavepeekError::Internal(
                "forced stream candidate collection requires FST input and a non-empty time window"
                    .into(),
            ));
        }
        let sample_from = from.saturating_sub(1);
        self.sampling_window = Some((sample_from, to));
        self.preload(ids, sample_from, to)?;
        let mut times = Vec::new();
        for id in &self.physical_ids(ids) {
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

    pub fn should_use_streaming_candidate_collection(
        &self,
        _count: usize,
        from: u64,
        to: u64,
        mode: ChangeCandidateCollectionMode,
    ) -> bool {
        self.inner.format() != Format::Vcd
            && from <= to
            && mode != ChangeCandidateCollectionMode::Random
    }
}

// Unpacked array indices are path components in Wavepeek's public hierarchy.
fn array_name_parts(mut name: &str) -> Vec<&str> {
    let mut indices = Vec::new();
    while name.ends_with(']') {
        let Some(start) = name.rfind('[') else { break };
        if start == 0 || name[start + 1..name.len() - 1].parse::<i64>().is_err() {
            break;
        }
        indices.push(&name[start..]);
        name = name[..start].trim_end();
    }
    indices.push(name);
    indices.reverse();
    indices
}

fn packed_name_range(name: &str, width: u32) -> Option<(&str, ondas::BitRange)> {
    let (base, suffix) = name.strip_suffix(']')?.rsplit_once('[')?;
    let base = base.trim_end();
    let (msb, lsb) = suffix.split_once(':').unwrap_or((suffix, suffix));
    let (msb, lsb) = (msb.parse::<i64>().ok()?, lsb.parse::<i64>().ok()?);
    if base.is_empty() || msb.abs_diff(lsb).checked_add(1) != Some(u64::from(width)) {
        return None;
    }
    Some((base, ondas::BitRange::new(msb, lsb)))
}

fn public_component(name: &str, format: Format, escaped: bool) -> Cow<'_, str> {
    match format {
        Format::Fsdb => Cow::Borrowed(name.strip_prefix('\\').unwrap_or(name)),
        Format::Vcd | Format::Fst if escaped || name.contains(['.', '/']) => {
            Cow::Owned(format!("\\{name}"))
        }
        _ => Cow::Borrowed(name),
    }
}

fn public_fst_variable_path(variable: &ondas::Variable<'_>) -> Option<String> {
    with_public_variable(
        variable,
        Format::Fst,
        variable.signal(),
        |mut parent, synthetic_scopes, name, _| {
            if synthetic_scopes.is_empty() && name != variable.name() {
                return None;
            }
            for component in synthetic_scopes {
                if !parent.is_empty() {
                    parent.push('.');
                }
                parent.push_str(&public_component(component, Format::Fst, false));
            }
            if !parent.is_empty() {
                parent.push('.');
            }
            parent.push_str(&public_component(
                name,
                Format::Fst,
                variable.name_was_escaped(),
            ));
            Some(parent)
        },
    )
}

fn public_fsdb_variable_path(variable: &ondas::Variable<'_>) -> String {
    with_public_variable(
        variable,
        Format::Fsdb,
        variable.signal(),
        |mut parent, synthetic_scopes, name, _| {
            for component in synthetic_scopes {
                if !parent.is_empty() {
                    parent.push('.');
                }
                parent.push_str(&public_component(component, Format::Fsdb, false));
            }
            if !parent.is_empty() {
                parent.push('.');
            }
            parent.push_str(&public_component(
                name,
                Format::Fsdb,
                variable.name_was_escaped(),
            ));
            parent
        },
    )
}

fn with_public_variable<R>(
    variable: &ondas::Variable<'_>,
    format: Format,
    signal: Option<ondas::Signal>,
    use_name: impl FnOnce(String, &[&str], &str, Option<ondas::BitRange>) -> R,
) -> R {
    let parent = variable
        .parent()
        .map(|scope| scope_path(&scope, format))
        .unwrap_or_default();
    let mut range = variable.range();
    let raw_name = variable.reader_name().unwrap_or(variable.name()).trim();
    let escaped_fsdb_name = format == Format::Fsdb && raw_name.starts_with('\\');
    let fsdb_name = if format == Format::Fsdb {
        let mut name = raw_name.strip_prefix('\\').unwrap_or(raw_name);
        if !escaped_fsdb_name
            && let Some(bit_range) = range
            && bit_range.msb() != bit_range.lsb()
        {
            let suffix = format!("[{}:{}]", bit_range.msb(), bit_range.lsb());
            name = name
                .strip_suffix(&suffix)
                .filter(|base| !base.is_empty())
                .unwrap_or(name);
        }
        Some(if !escaped_fsdb_name && name.contains('/') {
            Cow::Owned(name.replace('/', "."))
        } else {
            Cow::Borrowed(name)
        })
    } else {
        None
    };
    let mut name = fsdb_name.as_deref().unwrap_or_else(|| variable.name());
    if matches!(format, Format::Vcd | Format::Fst)
        && range.is_none()
        && let Some(Encoding::Bits { width }) = signal.map(|signal| signal.encoding())
        && let Some((base, packed_range)) = packed_name_range(name, width)
    {
        name = base;
        range = Some(packed_range);
    }
    let mut synthetic_scopes = Vec::new();
    if matches!(format, Format::Vcd | Format::Fst) && name.ends_with(']') {
        let parts = array_name_parts(name);
        synthetic_scopes.extend(parts[..parts.len() - 1].iter().copied());
        name = parts[parts.len() - 1];
    } else if format == Format::Fsdb {
        if range.is_some_and(|bit_range| bit_range.msb() != bit_range.lsb()) {
            let parts = array_name_parts(name);
            if parts.len() == 2
                && !parts[0].contains(['[', ']'])
                && (!escaped_fsdb_name || !parts[0].contains(['.', '/']))
            {
                synthetic_scopes.extend(parts[0].split('.'));
                name = parts[1];
            }
        }
        if !escaped_fsdb_name
            && synthetic_scopes.is_empty()
            && let Some((prefix, local)) = name.rsplit_once('.')
            && !prefix.is_empty()
            && !local.is_empty()
            && prefix.split('.').all(|part| !part.is_empty())
        {
            synthetic_scopes.extend(prefix.split('.'));
            name = local;
        }
    }
    use_name(parent, &synthetic_scopes, name, range)
}

// The previous FSDB reader ignored SDK struct/union begin/end callbacks.
fn skipped_fsdb_scope(scope: &ondas::Scope<'_>, format: Format) -> bool {
    format == Format::Fsdb
        && matches!(scope.kind(), "struct" | "union")
        && scope.packing().is_some()
}

fn public_scope_component(scope: &ondas::Scope<'_>, format: Format) -> String {
    let name = public_component(scope.name(), format, scope.name_was_escaped());
    if format == Format::Fsdb {
        name.replace('/', ".")
    } else {
        name.into_owned()
    }
}

fn scope_components(scope: &ondas::Scope<'_>, format: Format) -> Vec<String> {
    let mut components = Vec::new();
    if !skipped_fsdb_scope(scope, format) {
        components.push(public_scope_component(scope, format));
    }
    let mut current = scope.parent();
    while let Some(scope) = current {
        if !skipped_fsdb_scope(&scope, format) {
            components.push(public_scope_component(&scope, format));
        }
        current = scope.parent();
    }
    components.reverse();
    components
}

fn scope_path(scope: &ondas::Scope<'_>, format: Format) -> String {
    scope_components(scope, format).join(".")
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
    fn new(hierarchy: &ondas::Hierarchy, format: Format, scopes_only: bool) -> Self {
        let mut scope_keys = HashMap::new();
        let mut scopes = hierarchy
            .scopes()
            .filter(|scope| visible_scope(scope) && !skipped_fsdb_scope(scope, format))
            .map(|scope| {
                let components = scope_components(&scope, format);
                let path = components.join(".");
                let depth = components.len().saturating_sub(1);
                scope_keys.insert(path.clone(), components);
                ScopeEntry {
                    path,
                    depth,
                    kind: scope_type_alias(scope.kind()),
                }
            })
            .collect::<Vec<_>>();
        let mut scope_paths = scopes
            .iter()
            .map(|scope| scope.path.clone())
            .collect::<BTreeSet<_>>();
        let mut signals = Vec::new();
        let mut ids = HashMap::new();
        let mut declarations = Vec::new();
        let mut by_path: HashMap<String, Vec<usize>> = HashMap::new();
        for variable in hierarchy.variables() {
            if scopes_only && format == Format::Fsdb {
                let name = variable.reader_name().unwrap_or(variable.name()).trim();
                if !name.contains(['.', '/', '[']) {
                    continue;
                }
            }
            if variable
                .parent()
                .is_some_and(|scope| !visible_scope(&scope))
            {
                continue;
            }
            let signal = variable.signal();
            let id = if scopes_only {
                None
            } else {
                signal.map(|signal| {
                    *ids.entry(signal).or_insert_with(|| {
                        let id = SignalId::from_backend_index(signals.len() as u64);
                        signals.push(SignalSource::Signal(signal));
                        id
                    })
                })
            };
            with_public_variable(
                &variable,
                format,
                signal,
                |mut parent, synthetic_scopes, name, range| {
                    let mut components = scope_keys.get(&parent).cloned().unwrap_or_default();
                    let mut depth = components.len();
                    for component in synthetic_scopes {
                        let component = public_component(component, format, false);
                        components.push(component.to_string());
                        parent = if parent.is_empty() {
                            component.into_owned()
                        } else {
                            format!("{parent}.{component}")
                        };
                        if scope_paths.insert(parent.clone()) {
                            scope_keys.insert(parent.clone(), components.clone());
                            scopes.push(ScopeEntry {
                                path: parent.clone(),
                                depth,
                                kind: "unknown".into(),
                            });
                        }
                        depth += 1;
                    }
                    if scopes_only {
                        return;
                    }
                    let public_name = public_component(name, format, variable.name_was_escaped());
                    let name = public_name.as_ref();
                    let path = if parent.is_empty() {
                        name.to_owned()
                    } else {
                        format!("{parent}.{name}")
                    };
                    let width = signal.and_then(|signal| match signal.encoding() {
                        Encoding::Bits { width } => Some(width),
                        Encoding::Event => Some(if format == Format::Fsdb { 1 } else { 0 }),
                        _ => None,
                    });
                    let expr_type = signal.and_then(|signal| expression_type(&variable, signal));
                    let entry = SignalEntry {
                        name: name.to_owned(),
                        path: path.clone(),
                        kind: var_type_alias(variable.kind()),
                        width,
                    };
                    by_path.entry(path).or_default().push(declarations.len());
                    declarations.push(Declaration {
                        entry,
                        parent,
                        parent_order: 0,
                        id,
                        expr_type,
                        range,
                        visible: true,
                    });
                },
            );
        }
        scopes.sort_by(|left, right| scope_keys[&left.path].cmp(&scope_keys[&right.path]));
        if !scopes_only {
            let order = scopes
                .iter()
                .enumerate()
                .map(|(index, scope)| (scope.path.as_str(), index + 1))
                .collect::<HashMap<_, _>>();
            for declaration in &mut declarations {
                declaration.parent_order =
                    order.get(declaration.parent.as_str()).copied().unwrap_or(0);
            }
        }
        let mut index = Self {
            scopes,
            declarations,
            by_path,
            signals,
        };
        if !scopes_only && matches!(format, Format::Vcd | Format::Fst) {
            index.join_split_vectors();
        }
        index
    }
    fn join_split_vectors(&mut self) {
        for indices in self
            .by_path
            .values_mut()
            .filter(|indices| indices.len() > 1)
        {
            let mut ranges = Vec::new();
            for &index in indices.iter() {
                let declaration = &self.declarations[index];
                let (Some(range), Some(id), Some(width)) =
                    (declaration.range, declaration.id, declaration.entry.width)
                else {
                    break;
                };
                let low = range.msb().min(range.lsb());
                let high = range.msb().max(range.lsb());
                if high.checked_sub(low).and_then(|span| span.checked_add(1))
                    != Some(i64::from(width))
                {
                    break;
                }
                ranges.push((low, high, id, width));
            }
            if ranges.len() != indices.len() {
                continue;
            }
            ranges.sort_by_key(|part| part.0);
            if ranges.windows(2).any(|pair| pair[0].1 >= pair[1].0) {
                continue;
            }
            let low = ranges[0].0;
            let high = ranges[ranges.len() - 1].1;
            let Some(width) = high
                .checked_sub(low)
                .and_then(|span| span.checked_add(1))
                .and_then(|width| u32::try_from(width).ok())
            else {
                continue;
            };
            let parts = ranges
                .into_iter()
                .map(|(part_low, _, id, width)| BitPart {
                    id,
                    lsb: (part_low - low) as usize,
                    width: width as usize,
                })
                .collect();
            let id = SignalId::from_backend_index(self.signals.len() as u64);
            self.signals.push(SignalSource::Split(parts));
            for &index in &indices[1..] {
                self.declarations[index].visible = false;
            }
            let first = &mut self.declarations[indices[0]];
            first.id = Some(id);
            first.entry.width = Some(width);
            if let Some(expr_type) = &mut first.expr_type {
                expr_type.width = width;
                expr_type.storage = ExprStorage::PackedVector;
            }
            indices.truncate(1);
        }
    }
}

fn expression_type(variable: &ondas::Variable<'_>, signal: ondas::Signal) -> Option<ExprType> {
    let width = signal.width().unwrap_or(0);
    let (kind, width, scalar, four_state, signed) = match variable.kind() {
        "byte" => (
            ExprTypeKind::IntegerLike(IntegerLikeKind::Byte),
            8,
            true,
            false,
            true,
        ),
        "shortint" | "short-int" => (
            ExprTypeKind::IntegerLike(IntegerLikeKind::Shortint),
            16,
            true,
            false,
            true,
        ),
        "int" => (
            ExprTypeKind::IntegerLike(IntegerLikeKind::Int),
            32,
            true,
            false,
            true,
        ),
        "longint" | "long-int" => (
            ExprTypeKind::IntegerLike(IntegerLikeKind::Longint),
            64,
            true,
            false,
            true,
        ),
        "integer" => (
            ExprTypeKind::IntegerLike(IntegerLikeKind::Integer),
            32,
            true,
            true,
            true,
        ),
        "time" => (
            ExprTypeKind::IntegerLike(IntegerLikeKind::Time),
            64,
            true,
            true,
            false,
        ),
        "enum" => (ExprTypeKind::EnumCore, width, true, true, false),
        _ => match signal.encoding() {
            Encoding::Real => (ExprTypeKind::Real, 64, true, false, false),
            Encoding::String => (ExprTypeKind::String, 0, true, false, false),
            Encoding::Event => (ExprTypeKind::Event, 0, true, false, false),
            Encoding::Bits { .. } => (
                ExprTypeKind::BitVector,
                width,
                width <= 1,
                !matches!(variable.kind(), "bit" | "boolean" | "bit-vector"),
                false,
            ),
            _ => return None,
        },
    };
    let enumeration = variable.enumeration();
    Some(ExprType {
        kind,
        width,
        storage: if scalar {
            ExprStorage::Scalar
        } else {
            ExprStorage::PackedVector
        },
        is_signed: variable
            .signedness()
            .map_or(signed, |value| value == ondas::Signedness::Signed),
        is_four_state: variable
            .logic_domain()
            .map_or(four_state, |value| value != ondas::LogicDomain::TwoState),
        enum_type_id: enumeration
            .as_ref()
            .and_then(|value| value.name())
            .map(str::to_owned),
        enum_labels: enumeration.map(|value| {
            value
                .variants()
                .map(|variant| EnumLabelInfo {
                    name: variant.label.to_owned(),
                    bits: variant.encoded.to_owned(),
                })
                .collect()
        }),
    })
}

fn scope_type_alias(kind: &str) -> String {
    let kind = kind.replace('-', "_");
    if super::types::STABLE_SCOPE_KIND_ALIASES.contains(&kind.as_str()) {
        kind
    } else {
        "unknown".into()
    }
}

fn var_type_alias(kind: &str) -> String {
    let kind = match kind {
        "shortint" => "short_int".into(),
        "longint" => "long_int".into(),
        "realtime" => "real_time".into(),
        "realparameter" => "real_parameter".into(),
        "shortreal" => "short_real".into(),
        "sparsearray" => "sparse_array".into(),
        "bitvector" | "stdlogicvector" | "stdulogicvector" => "bit_vector".into(),
        "stdlogic" | "stdulogic" => "logic".into(),
        _ => kind.replace('-', "_"),
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

fn open_error(path: &Path, error: ondas::Error) -> WavepeekError {
    let operation = if matches!(error, ondas::Error::Io(_)) {
        "open"
    } else {
        "parse"
    };
    WavepeekError::File(format!("cannot {operation} '{}': {error}", path.display()))
}

fn query_error(error: ondas::Error) -> WavepeekError {
    match error {
        ondas::Error::UnsupportedSignal { .. } => WavepeekError::Signal(error.to_string()),
        _ => WavepeekError::File(error.to_string()),
    }
}
