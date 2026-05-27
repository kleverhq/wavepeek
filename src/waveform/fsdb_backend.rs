//! FSDB-backed waveform adapter implementation.

use std::cell::{Ref, RefCell};
use std::collections::HashMap;
use std::path::Path;

use crate::error::WavepeekError;
use crate::expr::{ExprTypeKind, SampledValue};

use super::fsdb_hierarchy::{FsdbHierarchyIndex, FsdbValueEncoding};
use super::fsdb_native::{self, FsdbNativeMetadata, FsdbReader};
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
    expr_sample_cache: HashMap<(SignalId, u64), SampledValue>,
    event_occurrence_cache: HashMap<(SignalId, u64), bool>,
}

impl FsdbBackend {
    pub(super) fn open(path: &Path) -> Result<Self, WavepeekError> {
        Ok(Self {
            reader: FsdbReader::open(path)?,
            raw_metadata: RefCell::new(None),
            hierarchy: RefCell::new(None),
            expr_sample_cache: HashMap::new(),
            event_occurrence_cache: HashMap::new(),
        })
    }

    pub(super) fn probe(path: &Path) -> Result<bool, WavepeekError> {
        fsdb_native::probe(path)
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
        let samples = self.reader.sample_signal_values(&idcodes, query_time_raw)?;
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
            return Ok(value.clone());
        }

        let value = self.sample_expr_value_uncached(resolved, query_time_raw)?;
        self.expr_sample_cache.insert(cache_key, value.clone());
        Ok(value)
    }

    pub(super) fn expr_event_occurred(
        &mut self,
        resolved: &ExprResolvedSignal,
        query_time_raw: u64,
    ) -> Result<bool, WavepeekError> {
        let cache_key = (resolved.id, query_time_raw);
        if let Some(occurred) = self.event_occurrence_cache.get(&cache_key) {
            return Ok(*occurred);
        }

        if !matches!(resolved.expr_type.kind, ExprTypeKind::Event) {
            return Ok(false);
        }

        self.raw_metadata()?;
        let occurred = self
            .reader
            .signal_event_occurred(resolved.id.as_u64(), query_time_raw)?;
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
        self.reader
            .collect_signal_change_times(idcodes.as_slice(), from_raw, to_raw)
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
        self.reader
            .collect_signal_change_times(idcodes.as_slice(), from_raw, to_raw)
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
        Ok(SampledValue::Integral {
            bits: sample.bits,
            label: None,
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

pub(super) fn unsupported_value_sampling(path: &str) -> WavepeekError {
    WavepeekError::Signal(format!(
        "signal '{path}' has unsupported FSDB expression value encoding"
    ))
}

fn unsupported_signal_value_encoding(path: &str) -> WavepeekError {
    WavepeekError::Signal(format!(
        "signal '{path}' has unsupported non-bit-vector encoding"
    ))
}
