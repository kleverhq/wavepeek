//! FSDB-backed waveform adapter implementation.

use std::cell::{Ref, RefCell};
use std::path::Path;

use crate::error::WavepeekError;
use crate::expr::SampledValue;

use super::fsdb_hierarchy::{FsdbHierarchyIndex, FsdbValueEncoding};
use super::fsdb_native::{self, FsdbReader};
use super::fsdb_time::{normalize_raw_time, parse_scale_unit};
use super::types::{
    ChangeCandidateCollectionMode, ExprResolvedSignal, ResolvedSignal, SampledSignalState,
    ScopeEntry, SignalEntry, SignalId, SignalOffsetData, WaveformMetadata,
};

#[derive(Debug)]
pub(super) struct FsdbBackend {
    reader: FsdbReader,
    hierarchy: RefCell<Option<FsdbHierarchyIndex>>,
}

impl FsdbBackend {
    pub(super) fn open(path: &Path) -> Result<Self, WavepeekError> {
        Ok(Self {
            reader: FsdbReader::open(path)?,
            hierarchy: RefCell::new(None),
        })
    }

    pub(super) fn probe(path: &Path) -> Result<bool, WavepeekError> {
        fsdb_native::probe(path)
    }

    pub(super) fn metadata(&self) -> Result<WaveformMetadata, WavepeekError> {
        let metadata = self.reader.metadata()?;
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

    pub(super) fn previous_sample_time(&self, _raw_time: u64) -> Option<u64> {
        None
    }

    pub(super) fn sample_resolved_optional(
        &mut self,
        resolved: &[ResolvedSignal],
        query_time_raw: u64,
    ) -> Result<Vec<SampledSignalState>, WavepeekError> {
        if resolved.is_empty() {
            return Ok(Vec::new());
        }

        self.metadata()?;

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
        _resolved: &ExprResolvedSignal,
        _query_time_raw: u64,
    ) -> Result<SampledValue, WavepeekError> {
        Err(unsupported_value_sampling())
    }

    pub(super) fn expr_event_occurred(
        &mut self,
        _resolved: &ExprResolvedSignal,
        _query_time_raw: u64,
    ) -> Result<bool, WavepeekError> {
        Err(unsupported_property_evaluation())
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
        _resolved: &[ResolvedSignal],
        _from_raw: u64,
        _to_raw: u64,
        _mode: ChangeCandidateCollectionMode,
    ) -> Result<Vec<u64>, WavepeekError> {
        Err(unsupported_change_collection())
    }

    pub(super) fn collect_expr_candidate_times_with_mode(
        &mut self,
        _resolved: &[ExprResolvedSignal],
        _from_raw: u64,
        _to_raw: u64,
        _mode: ChangeCandidateCollectionMode,
    ) -> Result<Vec<u64>, WavepeekError> {
        Err(unsupported_change_collection())
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

pub(super) fn unsupported_value_sampling() -> WavepeekError {
    WavepeekError::Signal(
        "FSDB expression value sampling is not implemented yet; value command sampling supports bit-vector signals only"
            .to_string(),
    )
}

fn unsupported_signal_value_encoding(path: &str) -> WavepeekError {
    WavepeekError::Signal(format!(
        "signal '{path}' has unsupported non-bit-vector encoding"
    ))
}

pub(super) fn unsupported_change_collection() -> WavepeekError {
    WavepeekError::Signal(
        "FSDB change collection is not implemented yet; info, scope, signal, and value are supported by this build"
            .to_string(),
    )
}

pub(super) fn unsupported_property_evaluation() -> WavepeekError {
    WavepeekError::Signal(
        "FSDB property evaluation is not implemented yet; info, scope, signal, and value are supported by this build"
            .to_string(),
    )
}
