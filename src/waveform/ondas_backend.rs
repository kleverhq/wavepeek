//! Ondas-backed hierarchy and value access.

#[cfg(all(test, feature = "fsdb"))]
#[path = "ondas_fsdb_tests.rs"]
mod fsdb_tests;
#[cfg(test)]
#[path = "ondas_tests.rs"]
mod tests;

use std::fmt;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use ondas::{Encoding, Format, SampleRef, Time, Value, ValueRef};

use crate::error::WavepeekError;
use crate::expr::{
    EnumLabelInfo, ExprStorage, ExprType, ExprTypeKind, IntegerLikeKind, SampledValue,
};

use super::expr_host::RuntimeExpressionHost;
use super::expr_query::{ExpressionQuery, ExpressionQueryTick};
use super::ondas_error::{open_error, read_error};
use super::selection::component_order;
use super::types::{
    ExprResolvedSignal, ResolvedSignal, SampledSignalState, ScopeEntry, SignalEntry, SignalListing,
    WaveformMetadata,
};

#[derive(Debug)]
pub(super) enum SignalResolutionError {
    NotFound(ondas::HierarchyPath),
    Other(WavepeekError),
}

impl SignalResolutionError {
    pub(super) fn into_error(self) -> WavepeekError {
        match self {
            Self::NotFound(path) => {
                WavepeekError::SignalNotFound(format!("signal '{path}' not found in dump"))
            }
            Self::Other(error) => error,
        }
    }
}

impl From<WavepeekError> for SignalResolutionError {
    fn from(error: WavepeekError) -> Self {
        Self::Other(error)
    }
}

pub(super) struct OndasBackend {
    inner: ondas::Waveform,
    path: PathBuf,
}

impl fmt::Debug for OndasBackend {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("OndasBackend")
            .field("format", &self.inner.format())
            .finish_non_exhaustive()
    }
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
            inner,
            path: path.to_owned(),
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
            Format::Shm => "shm",
            Format::Wlf => "wlf",
            Format::Wdb => "wdb",
            Format::Vwdb => "vwdb",
            _ => "unknown",
        }
    }

    pub fn metadata(&self) -> Result<WaveformMetadata, WavepeekError> {
        Self::format_metadata(self.inner.metadata())
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

    pub fn scopes_depth_first(
        &self,
        max_depth: Option<usize>,
    ) -> Result<Vec<ScopeEntry>, WavepeekError> {
        let mut scopes = self
            .inner
            .hierarchy()
            .scopes()
            .filter(visible_scope)
            .map(|scope| {
                let path = scope.path();
                ScopeEntry {
                    depth: path.len().saturating_sub(1),
                    path,
                    kind: scope_type_alias(scope.kind()),
                }
            })
            .filter(|scope| max_depth.is_none_or(|max| scope.depth <= max))
            .collect::<Vec<_>>();
        scopes.sort_by(|left, right| {
            left.path
                .components()
                .map(component_order)
                .cmp(right.path.components().map(component_order))
        });
        Ok(scopes)
    }

    pub fn signals_in_scope(&self, path: &str) -> Result<Vec<SignalEntry>, WavepeekError> {
        Ok(self.signals_in_scope_report(path)?.entries)
    }

    pub fn matching_signals(&self, matches: impl Fn(&str, &str) -> bool) -> Vec<SignalEntry> {
        let hierarchy = self.inner.hierarchy();
        let variables = hierarchy.variables().filter(|variable| {
            if variable
                .parent()
                .is_some_and(|scope| !visible_scope(&scope))
            {
                return false;
            }
            let path = variable.path();
            matches(&path.to_string(), &signal_name(&path))
        });
        self.signal_listing(variables).entries
    }

    pub fn signals_in_scope_report(&self, path: &str) -> Result<SignalListing, WavepeekError> {
        self.signals_in_scope_recursive_report(path, Some(0))
    }

    pub fn signals_in_scope_recursive_report(
        &self,
        path: &str,
        max_depth: Option<usize>,
    ) -> Result<SignalListing, WavepeekError> {
        let Some(scope) = self.scope_from_input(path)? else {
            return Ok(
                self.signal_listing(self.inner.hierarchy().variables().filter(|variable| {
                    variable.parent().is_none_or(|scope| visible_scope(&scope))
                        && max_depth
                            .is_none_or(|max| variable.path().len().saturating_sub(1) <= max)
                })),
            );
        };
        let mut variables = Vec::new();
        collect_variables(scope, max_depth, &mut variables);
        Ok(self.signal_listing(variables.into_iter()))
    }

    pub(super) fn validate_scope(&self, input: &str) -> Result<(), WavepeekError> {
        self.scope_from_input(input).map(|_| ())
    }

    fn scope_from_input(&self, input: &str) -> Result<Option<ondas::Scope<'_>>, WavepeekError> {
        let path = ondas::HierarchyPath::parse(input)
            .map_err(|error| WavepeekError::Scope(error.to_string()))?;
        if path.is_empty() {
            return Ok(None);
        }
        self.lookup_scope(&path, input).map(Some)
    }

    pub(super) fn canonical_scope(
        &self,
        path: &ondas::HierarchyPath,
    ) -> Result<ondas::HierarchyPath, WavepeekError> {
        if path.is_empty() {
            return Ok(path.clone());
        }
        Ok(self.lookup_scope(path, &path.to_string())?.path())
    }

    fn lookup_scope(
        &self,
        path: &ondas::HierarchyPath,
        input: &str,
    ) -> Result<ondas::Scope<'_>, WavepeekError> {
        let scope = self
            .inner
            .hierarchy()
            .scope_path(path)
            .map_err(|error| match error {
                ondas::LookupError::NotFound { .. } => {
                    WavepeekError::Scope(format!("scope '{input}' not found in dump"))
                }
                ondas::LookupError::Ambiguous { .. } => {
                    WavepeekError::Scope(format!("scope '{input}' is ambiguous in dump"))
                }
                error => WavepeekError::Scope(error.to_string()),
            })?;
        if !visible_scope(&scope) {
            return Err(WavepeekError::Scope(format!(
                "scope '{input}' not found in dump"
            )));
        }
        Ok(scope)
    }

    fn signal_listing<'h>(
        &self,
        variables: impl Iterator<Item = ondas::Variable<'h>>,
    ) -> SignalListing {
        let hierarchy = self.inner.hierarchy();
        let mut entries = Vec::new();
        let mut omitted = std::collections::BTreeSet::new();
        for variable in variables {
            let path = variable.path();
            if matches!(
                hierarchy.variable_path(&path),
                Err(ondas::LookupError::Ambiguous { .. })
            ) {
                omitted.insert(path.to_string());
            } else {
                entries.push(signal_entry(&variable, path));
            }
        }
        entries.sort_by(|left, right| {
            left.path
                .components()
                .take(left.path.len().saturating_sub(1))
                .map(component_order)
                .cmp(
                    right
                        .path
                        .components()
                        .take(right.path.len().saturating_sub(1))
                        .map(component_order),
                )
                .then_with(|| left.name.cmp(&right.name))
        });
        SignalListing {
            entries,
            omitted_ambiguous_paths: omitted.into_iter().collect(),
        }
    }

    fn resolve_selector(
        &self,
        selector: &ondas::SignalSelector,
        scope: Option<&str>,
    ) -> Result<ondas::ResolvedSelector<'_>, SignalResolutionError> {
        let scope = scope
            .map(ondas::HierarchyPath::parse)
            .transpose()
            .map_err(|error| WavepeekError::Scope(error.to_string()))?;
        let resolved = self
            .inner
            .hierarchy()
            .resolve_selector(selector, scope.as_ref())
            .map_err(lookup_error)?;
        if resolved
            .declaration()
            .parent()
            .is_some_and(|scope| !visible_scope(&scope))
        {
            return Err(SignalResolutionError::NotFound(
                resolved.declaration().path(),
            ));
        }
        Ok(resolved)
    }

    pub fn resolve_signal(
        &self,
        selector: &str,
        scope: Option<&str>,
    ) -> Result<ResolvedSignal, SignalResolutionError> {
        let selector =
            ondas::SignalSelector::parse(selector).map_err(|error| lookup_error(error.into()))?;
        self.resolve_typed_signal(&selector, scope)
    }

    pub(super) fn resolve_typed_signal(
        &self,
        selector: &ondas::SignalSelector,
        scope: Option<&str>,
    ) -> Result<ResolvedSignal, SignalResolutionError> {
        let selected = self.resolve_selector(selector, scope)?;
        let signal = selected.signal();
        let selector = selected.selector();
        let path = logical_selector(&selector);
        let width = match signal.encoding() {
            Encoding::Bits { width } => width,
            Encoding::Event => 0,
            _ => return Err(unsupported(&path).into()),
        };
        Ok(ResolvedSignal {
            path,
            signal,
            width,
            selector,
        })
    }

    pub fn resolve_signals(&self, paths: &[String]) -> Result<Vec<ResolvedSignal>, WavepeekError> {
        paths
            .iter()
            .map(|path| {
                self.resolve_signal(path, None)
                    .map_err(SignalResolutionError::into_error)
            })
            .collect()
    }

    pub fn resolve_expr_signal(
        &self,
        selector: &str,
        scope: Option<&str>,
    ) -> Result<ExprResolvedSignal, SignalResolutionError> {
        let selector =
            ondas::SignalSelector::parse(selector).map_err(|error| lookup_error(error.into()))?;
        let selected = self.resolve_selector(&selector, scope)?;
        let signal = selected.signal();
        let path = logical_selector(&selected.selector());
        let mut expr_type =
            expression_type(&selected.declaration(), signal).ok_or_else(|| unsupported(&path))?;
        if let Some((msb, lsb)) = selected.projection() {
            expr_type.width = msb - lsb + 1;
            expr_type.kind = ExprTypeKind::BitVector;
            expr_type.storage = ExprStorage::PackedVector;
            expr_type.is_signed = false;
            expr_type.enum_type_id = None;
            expr_type.enum_labels = None;
        }
        Ok(ExprResolvedSignal {
            path,
            signal,
            expr_type,
        })
    }

    fn values(
        &mut self,
        signals: &[ondas::Signal],
        time: u64,
    ) -> Result<Vec<Option<Value>>, WavepeekError> {
        if signals.is_empty() {
            return Ok(Vec::new());
        }
        let samples = self
            .inner
            .samples(signals, Time::from_ticks(time))
            .map_err(|error| read_error(&self.path, "read samples from", error))?;
        Ok(samples
            .into_iter()
            .map(|sample| match sample {
                ondas::Sample::Value { value, .. } => Some(value),
                ondas::Sample::Event { occurrences, .. } => Some(Value::Event { occurrences }),
                _ => None,
            })
            .collect())
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
        let signals = resolved
            .iter()
            .map(|entry| entry.signal)
            .collect::<Vec<_>>();
        let values = self.values(&signals, time)?;
        resolved
            .iter()
            .zip(&values)
            .map(|(signal, value)| sampled_signal(signal, value.as_ref().map(Value::as_ref)))
            .collect()
    }

    pub fn sample_resolved_optional_at_times(
        &mut self,
        resolved: &[ResolvedSignal],
        times: &[u64],
    ) -> Result<Vec<Vec<SampledSignalState>>, WavepeekError> {
        if !resolved.is_empty()
            && self
                .inner
                .metadata()
                .time_span()
                .is_some_and(|span| times.iter().any(|time| *time < span.first().ticks()))
        {
            return Err(WavepeekError::Internal(
                "query time is before first dump timestamp".into(),
            ));
        }
        let signals = resolved
            .iter()
            .map(|entry| entry.signal)
            .collect::<Vec<_>>();
        let times = times
            .iter()
            .copied()
            .map(Time::from_ticks)
            .collect::<Vec<_>>();
        let rows = self
            .inner
            .samples_at(&signals, &times)
            .map_err(|error| read_error(&self.path, "read samples from", error))?;
        rows.iter()
            .map(|(_, samples)| {
                resolved
                    .iter()
                    .zip(samples)
                    .map(|(signal, sample)| sampled_signal(signal, sample_value(sample.as_ref())))
                    .collect()
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
        let values = self.values(&[resolved.signal], time)?;
        expression_value(resolved, values[0].as_ref().map(Value::as_ref))
    }

    pub(super) fn query_expressions(
        &mut self,
        query: &ExpressionQuery,
        range: std::ops::RangeInclusive<u64>,
        initialize: bool,
        visitor: impl FnMut(
            &dyn RuntimeExpressionHost,
            ExpressionQueryTick,
        ) -> Result<std::ops::ControlFlow<()>, WavepeekError>,
    ) -> Result<(), WavepeekError> {
        let signals = query
            .signals
            .iter()
            .map(|entry| entry.signal)
            .collect::<Vec<_>>();
        let waveform = &mut self.inner;
        let mut selection = waveform
            .select(&signals)
            .map_err(|error| read_error(&self.path, "select expression signals from", error))?;
        query.run(&mut selection, range, initialize, &self.path, visitor)
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
        let values = self.values(&[resolved.signal], time)?;
        Ok(matches!(values[0], Some(Value::Event { occurrences }) if occurrences > 0))
    }
}

fn logical_selector(selector: &ondas::SignalSelector) -> String {
    selector
        .format(ondas::PathStyle::Logical)
        .expect("logical selectors are representable")
}

fn lookup_error(error: ondas::LookupError) -> SignalResolutionError {
    match error {
        ondas::LookupError::NotFound { path } => SignalResolutionError::NotFound(path),
        ondas::LookupError::Ambiguous { path, matches } => WavepeekError::Signal(format!(
            "signal '{path}' is ambiguous in dump: {matches} source declarations; no candidate was selected"
        )).into(),
        ondas::LookupError::NoSignal { path } => unsupported(&path.to_string()).into(),
        error => WavepeekError::Signal(error.to_string()).into(),
    }
}

fn collect_variables<'h>(
    scope: ondas::Scope<'h>,
    max_depth: Option<usize>,
    variables: &mut Vec<ondas::Variable<'h>>,
) {
    for item in scope.children() {
        match item {
            ondas::Item::Variable(variable) => variables.push(variable),
            ondas::Item::Scope(child) if max_depth != Some(0) && visible_scope(&child) => {
                collect_variables(child, max_depth.map(|max| max - 1), variables);
            }
            _ => {}
        }
    }
}

fn signal_entry(variable: &ondas::Variable<'_>, path: ondas::HierarchyPath) -> SignalEntry {
    let signal = variable.signal();
    SignalEntry {
        name: signal_name(&path),
        path,
        kind: if signal.is_some_and(|signal| matches!(signal.encoding(), Encoding::Event)) {
            "event".into()
        } else {
            var_type_alias(variable.kind())
        },
        width: signal.and_then(|signal| match signal.encoding() {
            Encoding::Bits { width } => Some(width),
            Encoding::Event => Some(0),
            _ => None,
        }),
    }
}

fn signal_name(path: &ondas::HierarchyPath) -> String {
    ondas::HierarchyPath::from_components(path.components().last()).to_string()
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

fn sample_value(sample: SampleRef<'_>) -> Option<ValueRef<'_>> {
    match sample {
        SampleRef::Value { value, .. } => Some(value),
        SampleRef::Event { occurrences, .. } => Some(ValueRef::Event { occurrences }),
        _ => None,
    }
}

fn sampled_signal(
    signal: &ResolvedSignal,
    value: Option<ValueRef<'_>>,
) -> Result<SampledSignalState, WavepeekError> {
    let bits = match value {
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

pub(super) fn expression_sample(
    resolved: &ExprResolvedSignal,
    sample: SampleRef<'_>,
) -> Result<SampledValue, WavepeekError> {
    expression_value(resolved, sample_value(sample))
}

fn expression_value(
    resolved: &ExprResolvedSignal,
    value: Option<ValueRef<'_>>,
) -> Result<SampledValue, WavepeekError> {
    if matches!(resolved.expr_type.kind, ExprTypeKind::Event) {
        return Err(WavepeekError::Internal(format!(
            "signal '{}' is a raw event and cannot be sampled as a value",
            resolved.path
        )));
    }
    match value {
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

fn unsupported(path: &str) -> WavepeekError {
    WavepeekError::Signal(format!(
        "signal '{path}' has unsupported non-bit-vector encoding"
    ))
}
