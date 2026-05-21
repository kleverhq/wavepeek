use std::collections::{HashMap, HashSet};

use serde::Serialize;

use crate::cli::change::{ChangeArgs, TuneChangeCandidateMode, TuneChangeEngineMode};
use crate::cli::limits::LimitArg;
use crate::engine::expr_runtime::{
    SharedWaveform, bind_waveform_event_expr, candidate_sources_for_handles,
    event_candidate_handles, event_expr_contains_wildcard, event_expr_is_any_tracked_only,
    event_expr_is_edge_only, event_expr_matches, open_shared_waveform,
};
use crate::engine::time::{
    DumpTimeContext, ParsedTime, TimeValidationError, format_raw_timestamp,
    parse_dump_time_context, validate_time_token_to_raw,
};
use crate::engine::value_format::format_verilog_literal;
use crate::engine::{CommandData, CommandName, CommandResult, HumanRenderOptions};
use crate::error::WavepeekError;
use crate::expr::{
    BoundEventExpr, EventEvalFrame, ExprTypeKind, ExpressionHost, SampledValue, SignalHandle,
};
use crate::waveform::{
    ChangeCandidateCollectionMode, ExprResolvedSignal, ResolvedSignal, SampledSignalState,
    SignalId, SignalOffsetData, Waveform, expr_host::WaveformExprHost,
    should_emit_delta_and_update_baseline,
};

const EMPTY_WARNING: &str = "no signal changes found in selected time range";
const EDGE_FAST_MIN_WORK: usize = 1_000_000;
const AUTO_FUSED_MIN_ESTIMATED_WORK: usize = 100_000;
const AUTO_EDGE_ONLY_MIN_ESTIMATED_WORK: usize = 500_000;
const AUTO_EDGE_FAST_MIN_ESTIMATED_WORK: usize = 2_000_000;
const AUTO_FUSED_WIDE_SIGNAL_CUTOFF: usize = 32;
const AUTO_EDGE_ONLY_MIN_REQUESTED_SIGNALS: usize = 10;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct AutoDispatchWorkEstimate {
    fused_work: usize,
    edge_work: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ChangeSignalValue {
    #[serde(skip_serializing)]
    pub display: String,
    pub path: String,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ChangeSnapshot {
    pub time: String,
    pub signals: Vec<ChangeSignalValue>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RequestedSignal {
    display: String,
    path: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ChangeEngineMode {
    Baseline,
    Fused,
    EdgeFast,
}

#[derive(Debug)]
struct ChangeRunOutput {
    snapshots: Vec<ChangeSnapshot>,
    truncated: bool,
}

#[derive(Debug, Clone)]
struct RollingSignalState {
    offset: Option<SignalOffsetData>,
    bits: Option<String>,
}

#[derive(Clone)]
struct CachedEventSamples {
    current: SampledValue,
    previous: SampledValue,
}

struct FastEventEvalHost<'a> {
    base: &'a WaveformExprHost,
    current_timestamp: u64,
    cached: HashMap<SignalHandle, CachedEventSamples>,
}

impl ExpressionHost for FastEventEvalHost<'_> {
    fn resolve_signal(&self, name: &str) -> Result<SignalHandle, crate::expr::ExprDiagnostic> {
        self.base.resolve_signal(name)
    }

    fn signal_type(
        &self,
        handle: SignalHandle,
    ) -> Result<crate::expr::ExprType, crate::expr::ExprDiagnostic> {
        self.base.signal_type(handle)
    }

    fn sample_value(
        &self,
        handle: SignalHandle,
        timestamp: u64,
    ) -> Result<SampledValue, crate::expr::ExprDiagnostic> {
        if let Some(cached) = self.cached.get(&handle) {
            if timestamp >= self.current_timestamp {
                return Ok(cached.current.clone());
            }
            return Ok(cached.previous.clone());
        }
        self.base.sample_value(handle, timestamp)
    }

    fn event_occurred(
        &self,
        handle: SignalHandle,
        timestamp: u64,
    ) -> Result<bool, crate::expr::ExprDiagnostic> {
        self.base.event_occurred(handle, timestamp)
    }
}

#[derive(Default)]
struct SampleCache {
    requested_batches: HashMap<u64, Vec<SampledSignalState>>,
}

impl SampleCache {
    fn sample_requested_batch(
        &mut self,
        waveform: &SharedWaveform,
        resolved: &[ResolvedSignal],
        raw_time: u64,
    ) -> Result<Vec<SampledSignalState>, WavepeekError> {
        if let Some(existing) = self.requested_batches.get(&raw_time) {
            return Ok(existing.clone());
        }

        let sampled = waveform
            .borrow_mut()
            .sample_resolved_optional(resolved, raw_time)?;
        self.requested_batches.insert(raw_time, sampled.clone());
        Ok(sampled)
    }
}

#[derive(Default)]
struct IndexDecodeCache {
    entries: HashMap<(SignalId, u32), Option<String>>,
}

impl IndexDecodeCache {
    fn bits(
        &mut self,
        waveform: &Waveform,
        resolved: &ResolvedSignal,
        time_table_idx: u32,
    ) -> Result<Option<String>, WavepeekError> {
        let key = (resolved.id, time_table_idx);
        if let Some(existing) = self.entries.get(&key) {
            return Ok(existing.clone());
        }

        let bits = waveform
            .decode_indexed_signal_at(resolved, time_table_idx)?
            .ok_or_else(indexed_backend_unavailable)?
            .bits;
        self.entries.insert(key, bits.clone());
        Ok(bits)
    }
}

fn indexed_backend_unavailable() -> WavepeekError {
    WavepeekError::Internal("indexed waveform access is unavailable for this backend".to_string())
}

fn indexed_timestamps(waveform: &Waveform) -> Result<&[u64], WavepeekError> {
    waveform
        .indexed_timestamps()
        .ok_or_else(indexed_backend_unavailable)
}

pub fn run(args: ChangeArgs) -> Result<CommandResult, WavepeekError> {
    let max_entries = match &args.max {
        LimitArg::Numeric(0) => {
            return Err(WavepeekError::Args(
                "--max must be greater than 0.".to_string(),
            ));
        }
        LimitArg::Numeric(value) => Some(*value),
        LimitArg::Unlimited => None,
    };

    let mut warnings = Vec::new();
    if args.max.is_unlimited() {
        warnings.push("limit disabled: --max=unlimited".to_string());
    }

    let waveform = open_shared_waveform(args.waves.as_path())?;
    let metadata = waveform.borrow().metadata()?;

    let requested_signals = {
        let waveform_ref = waveform.borrow();
        resolve_requested_signals(&waveform_ref, args.scope.as_deref(), &args)?
    };

    let event_expr_source = args.on.as_deref().unwrap_or("*");
    let (host, bound_event) =
        bind_waveform_event_expr(waveform.clone(), args.scope.as_deref(), event_expr_source)?;

    let dump_time = parse_dump_time_context(&metadata)?;
    let dump_tick = dump_time.dump_tick;

    let from_raw = match args.from.as_deref() {
        Some(token) => parse_bound_time(token, "--from", dump_time, &metadata)?,
        None => u64::try_from(dump_time.dump_start_zs / dump_time.dump_tick_zs).map_err(|_| {
            WavepeekError::Internal("dump start timestamp exceeds supported range".to_string())
        })?,
    };
    let to_raw = match args.to.as_deref() {
        Some(token) => parse_bound_time(token, "--to", dump_time, &metadata)?,
        None => u64::try_from(dump_time.dump_end_zs / dump_time.dump_tick_zs).map_err(|_| {
            WavepeekError::Internal("dump end timestamp exceeds supported range".to_string())
        })?,
    };

    if from_raw > to_raw {
        return Err(WavepeekError::Args(
            "--from must be less than or equal to --to".to_string(),
        ));
    }

    let baseline_raw = from_raw;
    let requested_paths_owned = requested_signals
        .iter()
        .map(|signal| signal.path.clone())
        .collect::<Vec<_>>();
    let requested_resolved = waveform.borrow().resolve_signals(&requested_paths_owned)?;
    let requested_expr_sources = waveform
        .borrow()
        .resolve_expr_signals(&requested_paths_owned)?;
    let tracked_signal_handles = requested_paths_owned
        .iter()
        .map(|path| {
            host.resolve_signal(path.as_str())
                .map_err(|diagnostic| WavepeekError::Internal(diagnostic.message))
        })
        .collect::<Result<Vec<_>, _>>()?;

    let mut candidate_sources = Vec::new();
    if event_expr_contains_wildcard(&bound_event) {
        candidate_sources.extend(requested_expr_sources.iter().cloned());
    }
    let event_sources =
        candidate_sources_for_handles(&host, &event_candidate_handles(&bound_event))?;
    candidate_sources.extend(event_sources);
    let mut seen = HashSet::new();
    candidate_sources.retain(|signal| seen.insert(signal.id));

    let candidate_mode = map_candidate_mode(args.tune_candidates);
    let window_timestamp_count = {
        let waveform_ref = waveform.borrow();
        indexed_timestamps(&waveform_ref)
            .ok()
            .and_then(|timestamps| time_window_indices(timestamps, from_raw, to_raw))
            .map(|(start_idx, end_idx_exclusive)| end_idx_exclusive.saturating_sub(start_idx))
            .unwrap_or(0)
    };
    let estimated_work = estimate_auto_dispatch_work(
        window_timestamp_count,
        candidate_sources.len(),
        requested_resolved.len(),
    );
    let engine_mode = select_engine_mode(
        args.tune_engine,
        event_expr_is_any_tracked_only(&bound_event),
        event_expr_is_edge_only(&bound_event),
        requested_resolved.len(),
        estimated_work,
    );

    let run_output = match engine_mode {
        ChangeEngineMode::Baseline => run_baseline(
            &waveform,
            &host,
            event_expr_source,
            &bound_event,
            tracked_signal_handles.as_slice(),
            requested_signals.as_slice(),
            requested_resolved.as_slice(),
            candidate_sources.as_slice(),
            from_raw,
            to_raw,
            baseline_raw,
            dump_tick,
            max_entries,
            candidate_mode,
            None,
        )?,
        ChangeEngineMode::Fused => run_fused(
            &waveform,
            &host,
            event_expr_source,
            &bound_event,
            tracked_signal_handles.as_slice(),
            requested_signals.as_slice(),
            requested_resolved.as_slice(),
            candidate_sources.as_slice(),
            from_raw,
            to_raw,
            baseline_raw,
            dump_tick,
            max_entries,
            candidate_mode,
        )?,
        ChangeEngineMode::EdgeFast => run_edge_fast(
            &waveform,
            &host,
            event_expr_source,
            &bound_event,
            tracked_signal_handles.as_slice(),
            requested_signals.as_slice(),
            requested_resolved.as_slice(),
            candidate_sources.as_slice(),
            from_raw,
            to_raw,
            baseline_raw,
            dump_tick,
            max_entries,
            candidate_mode,
            args.tune_edge_fast_force,
        )?,
    };

    let snapshots = run_output.snapshots;
    let truncated = run_output.truncated;

    if snapshots.is_empty() {
        warnings.push(EMPTY_WARNING.to_string());
    }

    if let Some(max_entries) = max_entries
        && truncated
    {
        warnings.push(format!(
            "truncated output to {} entries (use --max to increase limit)",
            max_entries
        ));
    }

    Ok(CommandResult {
        command: CommandName::Change,
        json: args.json,
        human_options: HumanRenderOptions {
            scope_tree: false,
            signals_abs: args.abs,
        },
        data: CommandData::Change(snapshots),
        warnings,
    })
}

fn map_candidate_mode(mode: TuneChangeCandidateMode) -> ChangeCandidateCollectionMode {
    match mode {
        TuneChangeCandidateMode::Auto => ChangeCandidateCollectionMode::Auto,
        TuneChangeCandidateMode::Random => ChangeCandidateCollectionMode::Random,
        TuneChangeCandidateMode::Stream => ChangeCandidateCollectionMode::Stream,
    }
}

fn select_engine_mode(
    mode: TuneChangeEngineMode,
    any_tracked_only: bool,
    edge_only: bool,
    requested_signal_count: usize,
    estimated_work: AutoDispatchWorkEstimate,
) -> ChangeEngineMode {
    match mode {
        TuneChangeEngineMode::Baseline => ChangeEngineMode::Baseline,
        TuneChangeEngineMode::Fused => ChangeEngineMode::Fused,
        TuneChangeEngineMode::EdgeFast => ChangeEngineMode::EdgeFast,
        TuneChangeEngineMode::Auto => {
            let route_fused_for_any_tracked = any_tracked_only
                && requested_signal_count > 0
                && (requested_signal_count >= AUTO_FUSED_WIDE_SIGNAL_CUTOFF
                    || estimated_work.fused_work >= AUTO_FUSED_MIN_ESTIMATED_WORK);
            let route_edge_fast_for_edge_only = edge_only
                && requested_signal_count >= AUTO_EDGE_ONLY_MIN_REQUESTED_SIGNALS
                && estimated_work.edge_work >= AUTO_EDGE_FAST_MIN_ESTIMATED_WORK;
            let route_fused_for_edge_only = edge_only
                && requested_signal_count >= AUTO_EDGE_ONLY_MIN_REQUESTED_SIGNALS
                && estimated_work.edge_work >= AUTO_EDGE_ONLY_MIN_ESTIMATED_WORK
                && estimated_work.edge_work < AUTO_EDGE_FAST_MIN_ESTIMATED_WORK;

            if route_fused_for_any_tracked || route_fused_for_edge_only {
                ChangeEngineMode::Fused
            } else if route_edge_fast_for_edge_only {
                ChangeEngineMode::EdgeFast
            } else {
                ChangeEngineMode::Baseline
            }
        }
    }
}

fn estimate_auto_dispatch_work(
    window_timestamp_count: usize,
    candidate_signal_count: usize,
    requested_signal_count: usize,
) -> AutoDispatchWorkEstimate {
    AutoDispatchWorkEstimate {
        fused_work: window_timestamp_count
            .saturating_mul(candidate_signal_count)
            .saturating_mul(requested_signal_count),
        edge_work: window_timestamp_count.saturating_mul(requested_signal_count),
    }
}

#[allow(clippy::too_many_arguments)]
fn run_baseline(
    waveform: &SharedWaveform,
    host: &WaveformExprHost,
    event_expr_source: &str,
    bound_event: &BoundEventExpr,
    tracked_signal_handles: &[SignalHandle],
    requested_signals: &[RequestedSignal],
    requested_resolved: &[ResolvedSignal],
    candidate_sources: &[ExprResolvedSignal],
    from_raw: u64,
    to_raw: u64,
    baseline_raw: u64,
    dump_tick: ParsedTime,
    max_entries: Option<usize>,
    candidate_mode: ChangeCandidateCollectionMode,
    precomputed_candidate_times: Option<Vec<u64>>,
) -> Result<ChangeRunOutput, WavepeekError> {
    let candidate_times = if let Some(precomputed_candidate_times) = precomputed_candidate_times {
        precomputed_candidate_times
    } else {
        waveform
            .borrow_mut()
            .collect_expr_candidate_times_with_mode(
                candidate_sources,
                from_raw,
                to_raw,
                candidate_mode,
            )?
    };
    let candidate_schedule = {
        let waveform_ref = waveform.borrow();
        build_candidate_schedule(&waveform_ref, &candidate_times)?
    };

    let mut sample_cache = SampleCache::default();
    sample_cache.sample_requested_batch(waveform, requested_resolved, baseline_raw)?;

    let mut snapshots = Vec::new();
    let mut truncated = false;
    for (timestamp, previous_timestamp) in candidate_schedule {
        let current_samples =
            sample_cache.sample_requested_batch(waveform, requested_resolved, timestamp)?;
        let previous_samples = if let Some(previous) = previous_timestamp {
            sample_cache.sample_requested_batch(waveform, requested_resolved, previous)?
        } else {
            requested_resolved
                .iter()
                .map(|signal| SampledSignalState {
                    path: signal.path.clone(),
                    width: signal.width,
                    bits: None,
                })
                .collect::<Vec<_>>()
        };

        if timestamp <= baseline_raw {
            continue;
        }

        let mut previous_values = previous_samples
            .iter()
            .map(|sample| sample.bits.clone())
            .collect::<Vec<_>>();
        let current_values = current_samples
            .iter()
            .map(|sample| sample.bits.clone())
            .collect::<Vec<_>>();

        let should_emit =
            should_emit_delta_and_update_baseline(&mut previous_values, &current_values);
        if !should_emit {
            continue;
        }

        let frame = EventEvalFrame {
            timestamp,
            previous_timestamp,
            tracked_signals: tracked_signal_handles,
        };
        if !event_expr_matches(event_expr_source, bound_event, host, &frame)? {
            continue;
        }

        if let Some(limit) = max_entries
            && snapshots.len() == limit
        {
            truncated = true;
            break;
        }

        snapshots.push(build_snapshot(
            requested_signals,
            current_samples.as_slice(),
            timestamp,
            dump_tick,
        )?);
    }

    Ok(ChangeRunOutput {
        snapshots,
        truncated,
    })
}

#[allow(clippy::too_many_arguments)]
fn run_baseline_fallback(
    waveform: &SharedWaveform,
    host: &WaveformExprHost,
    event_expr_source: &str,
    bound_event: &BoundEventExpr,
    tracked_signal_handles: &[SignalHandle],
    requested_signals: &[RequestedSignal],
    requested_resolved: &[ResolvedSignal],
    candidate_sources: &[ExprResolvedSignal],
    from_raw: u64,
    to_raw: u64,
    baseline_raw: u64,
    dump_tick: ParsedTime,
    max_entries: Option<usize>,
    candidate_mode: ChangeCandidateCollectionMode,
    precomputed_candidate_times: Option<Vec<u64>>,
) -> Result<ChangeRunOutput, WavepeekError> {
    run_baseline(
        waveform,
        host,
        event_expr_source,
        bound_event,
        tracked_signal_handles,
        requested_signals,
        requested_resolved,
        candidate_sources,
        from_raw,
        to_raw,
        baseline_raw,
        dump_tick,
        max_entries,
        candidate_mode,
        precomputed_candidate_times,
    )
}

#[allow(clippy::too_many_arguments)]
fn run_edge_fast(
    waveform: &SharedWaveform,
    host: &WaveformExprHost,
    event_expr_source: &str,
    bound_event: &BoundEventExpr,
    tracked_signal_handles: &[SignalHandle],
    requested_signals: &[RequestedSignal],
    requested_resolved: &[ResolvedSignal],
    candidate_sources: &[ExprResolvedSignal],
    from_raw: u64,
    to_raw: u64,
    baseline_raw: u64,
    dump_tick: ParsedTime,
    max_entries: Option<usize>,
    candidate_mode: ChangeCandidateCollectionMode,
    force_edge_fast: bool,
) -> Result<ChangeRunOutput, WavepeekError> {
    if !event_expr_is_edge_only(bound_event) {
        return run_baseline_fallback(
            waveform,
            host,
            event_expr_source,
            bound_event,
            tracked_signal_handles,
            requested_signals,
            requested_resolved,
            candidate_sources,
            from_raw,
            to_raw,
            baseline_raw,
            dump_tick,
            max_entries,
            candidate_mode,
            None,
        );
    }

    let candidate_times = waveform
        .borrow_mut()
        .collect_expr_candidate_times_with_mode(
            candidate_sources,
            from_raw,
            to_raw,
            candidate_mode,
        )?;
    if !force_edge_fast
        && candidate_times
            .len()
            .saturating_mul(requested_resolved.len())
            < EDGE_FAST_MIN_WORK
    {
        return run_baseline_fallback(
            waveform,
            host,
            event_expr_source,
            bound_event,
            tracked_signal_handles,
            requested_signals,
            requested_resolved,
            candidate_sources,
            from_raw,
            to_raw,
            baseline_raw,
            dump_tick,
            max_entries,
            candidate_mode,
            Some(candidate_times),
        );
    }

    if waveform.borrow().indexed_timestamps().is_none() {
        return run_baseline_fallback(
            waveform,
            host,
            event_expr_source,
            bound_event,
            tracked_signal_handles,
            requested_signals,
            requested_resolved,
            candidate_sources,
            from_raw,
            to_raw,
            baseline_raw,
            dump_tick,
            max_entries,
            candidate_mode,
            Some(candidate_times),
        );
    }

    let candidate_indices = {
        let waveform_ref = waveform.borrow();
        let time_table = indexed_timestamps(&waveform_ref)?;
        candidate_times_to_indices(time_table, candidate_times.as_slice())?
    };

    let mut loaded_signal_ids = requested_resolved
        .iter()
        .map(|signal| signal.id)
        .collect::<HashSet<_>>();
    for signal in candidate_sources {
        loaded_signal_ids.insert(signal.id);
    }
    let loaded_signal_ids = loaded_signal_ids.into_iter().collect::<Vec<_>>();
    if !waveform
        .borrow_mut()
        .ensure_indexed_signals_loaded(loaded_signal_ids.as_slice())
    {
        return run_baseline_fallback(
            waveform,
            host,
            event_expr_source,
            bound_event,
            tracked_signal_handles,
            requested_signals,
            requested_resolved,
            candidate_sources,
            from_raw,
            to_raw,
            baseline_raw,
            dump_tick,
            max_entries,
            candidate_mode,
            Some(candidate_times),
        );
    }
    let cached_sources = cached_event_sources(
        host,
        cached_event_handles(bound_event, tracked_signal_handles).as_slice(),
    )?;

    let mut decode_cache = IndexDecodeCache::default();
    let mut snapshots = Vec::new();
    let mut truncated = false;

    for (candidate_index, timestamp) in candidate_indices.into_iter().zip(candidate_times) {
        let candidate_index_u32 = u32::try_from(candidate_index).map_err(|_| {
            WavepeekError::Internal("time table index exceeds u32 range".to_string())
        })?;
        let previous_index = candidate_index
            .checked_sub(1)
            .and_then(|idx| u32::try_from(idx).ok());

        if timestamp <= baseline_raw {
            continue;
        }

        let mut any_requested_offset_changed = false;
        let mut delta_confirmed = false;

        {
            let waveform_ref = waveform.borrow();
            for resolved in requested_resolved {
                let current_offset = waveform_ref
                    .indexed_signal_offset_at(resolved.id, candidate_index_u32)
                    .ok_or_else(indexed_backend_unavailable)?;
                let previous_offset = previous_index
                    .map(|idx| {
                        waveform_ref
                            .indexed_signal_offset_at(resolved.id, idx)
                            .ok_or_else(indexed_backend_unavailable)
                    })
                    .transpose()?
                    .flatten();
                if current_offset == previous_offset {
                    continue;
                }

                any_requested_offset_changed = true;
                let current_bits =
                    decode_cache.bits(&waveform_ref, resolved, candidate_index_u32)?;
                let previous_bits = previous_index
                    .map(|idx| decode_cache.bits(&waveform_ref, resolved, idx))
                    .transpose()?
                    .flatten();

                if let (Some(previous_bits), Some(current_bits)) =
                    (previous_bits.as_ref(), current_bits.as_ref())
                    && previous_bits != current_bits
                {
                    delta_confirmed = true;
                    break;
                }
            }
        }

        if !any_requested_offset_changed || !delta_confirmed {
            continue;
        }

        let previous_timestamp = if candidate_index == 0 {
            None
        } else {
            let waveform_ref = waveform.borrow();
            Some(indexed_timestamps(&waveform_ref)?[candidate_index - 1])
        };
        let fast_host = {
            let waveform_ref = waveform.borrow();
            build_edge_fast_event_eval_host(
                host,
                timestamp,
                candidate_index_u32,
                previous_index,
                &waveform_ref,
                &mut decode_cache,
                cached_sources.as_slice(),
            )?
        };
        let frame = EventEvalFrame {
            timestamp,
            previous_timestamp,
            tracked_signals: tracked_signal_handles,
        };
        if !event_expr_matches(event_expr_source, bound_event, &fast_host, &frame)? {
            continue;
        }

        if let Some(limit) = max_entries
            && snapshots.len() == limit
        {
            truncated = true;
            break;
        }

        let current_samples = requested_resolved
            .iter()
            .map(|resolved| {
                let waveform_ref = waveform.borrow();
                Ok(SampledSignalState {
                    path: resolved.path.clone(),
                    width: resolved.width,
                    bits: decode_cache.bits(&waveform_ref, resolved, candidate_index_u32)?,
                })
            })
            .collect::<Result<Vec<_>, WavepeekError>>()?;
        snapshots.push(build_snapshot(
            requested_signals,
            current_samples.as_slice(),
            timestamp,
            dump_tick,
        )?);
    }

    Ok(ChangeRunOutput {
        snapshots,
        truncated,
    })
}

#[allow(clippy::too_many_arguments)]
fn run_fused(
    waveform: &SharedWaveform,
    host: &WaveformExprHost,
    event_expr_source: &str,
    bound_event: &BoundEventExpr,
    tracked_signal_handles: &[SignalHandle],
    requested_signals: &[RequestedSignal],
    requested_resolved: &[ResolvedSignal],
    candidate_sources: &[ExprResolvedSignal],
    from_raw: u64,
    to_raw: u64,
    baseline_raw: u64,
    dump_tick: ParsedTime,
    max_entries: Option<usize>,
    candidate_mode: ChangeCandidateCollectionMode,
) -> Result<ChangeRunOutput, WavepeekError> {
    if waveform.borrow().indexed_timestamps().is_none() {
        return run_baseline_fallback(
            waveform,
            host,
            event_expr_source,
            bound_event,
            tracked_signal_handles,
            requested_signals,
            requested_resolved,
            candidate_sources,
            from_raw,
            to_raw,
            baseline_raw,
            dump_tick,
            max_entries,
            candidate_mode,
            None,
        );
    }

    let mut tracked_resolved = requested_resolved.to_vec();
    let mut tracked_seen = tracked_resolved
        .iter()
        .map(|signal| signal.id)
        .collect::<HashSet<_>>();
    for signal in candidate_sources {
        if tracked_seen.insert(signal.id) {
            tracked_resolved.push(ResolvedSignal {
                path: signal.path.clone(),
                id: signal.id,
                width: signal.expr_type.width.max(1),
            });
        }
    }

    let tracked_index_by_id = tracked_resolved
        .iter()
        .enumerate()
        .map(|(index, signal)| (signal.id, index))
        .collect::<HashMap<_, _>>();
    let requested_tracked_indices = requested_resolved
        .iter()
        .map(|signal| {
            tracked_index_by_id.get(&signal.id).copied().ok_or_else(|| {
                WavepeekError::Internal(format!(
                    "requested signal '{}' is missing from fused tracking state",
                    signal.path
                ))
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    let requested_slot_by_tracked = {
        let mut slots = vec![None; tracked_resolved.len()];
        for (requested_index, tracked_index) in
            requested_tracked_indices.iter().copied().enumerate()
        {
            slots[tracked_index] = Some(requested_index);
        }
        slots
    };
    let candidate_tracked_indices = candidate_sources
        .iter()
        .map(|signal| {
            tracked_index_by_id.get(&signal.id).copied().ok_or_else(|| {
                WavepeekError::Internal(format!(
                    "candidate signal '{}' is missing from fused tracking state",
                    signal.path
                ))
            })
        })
        .collect::<Result<Vec<_>, _>>()?;

    let all_signal_ids = tracked_resolved
        .iter()
        .map(|signal| signal.id)
        .collect::<Vec<_>>();
    if !waveform
        .borrow_mut()
        .ensure_indexed_signals_loaded(all_signal_ids.as_slice())
    {
        return run_baseline_fallback(
            waveform,
            host,
            event_expr_source,
            bound_event,
            tracked_signal_handles,
            requested_signals,
            requested_resolved,
            candidate_sources,
            from_raw,
            to_raw,
            baseline_raw,
            dump_tick,
            max_entries,
            candidate_mode,
            None,
        );
    }

    let stream_candidate_times = if should_use_stream_candidates_in_fused(candidate_mode) {
        Some(
            waveform
                .borrow_mut()
                .collect_expr_candidate_times_with_mode(
                    candidate_sources,
                    from_raw,
                    to_raw,
                    ChangeCandidateCollectionMode::Stream,
                )?,
        )
    } else {
        None
    };
    let cached_sources = cached_event_sources(
        host,
        cached_event_handles(bound_event, tracked_signal_handles).as_slice(),
    )?;

    let (start_idx, end_idx_exclusive) = {
        let waveform_ref = waveform.borrow();
        let Some(window) =
            time_window_indices(indexed_timestamps(&waveform_ref)?, from_raw, to_raw)
        else {
            return Ok(ChangeRunOutput {
                snapshots: Vec::new(),
                truncated: false,
            });
        };
        window
    };

    let stream_candidate_indices = if let Some(stream_times) = stream_candidate_times {
        let waveform_ref = waveform.borrow();
        Some(candidate_times_to_indices(
            indexed_timestamps(&waveform_ref)?,
            stream_times.as_slice(),
        )?)
    } else {
        None
    };

    let mut rolling = Vec::with_capacity(tracked_resolved.len());
    if start_idx == 0 {
        rolling.resize(
            tracked_resolved.len(),
            RollingSignalState {
                offset: None,
                bits: None,
            },
        );
    } else {
        let previous_idx = u32::try_from(start_idx - 1).map_err(|_| {
            WavepeekError::Internal("time table index exceeds u32 range".to_string())
        })?;
        let waveform_ref = waveform.borrow();
        for signal in &tracked_resolved {
            let offset = waveform_ref
                .indexed_signal_offset_at(signal.id, previous_idx)
                .ok_or_else(indexed_backend_unavailable)?;
            let bits = waveform_ref
                .decode_indexed_signal_at(signal, previous_idx)?
                .ok_or_else(indexed_backend_unavailable)?
                .bits;
            rolling.push(RollingSignalState { offset, bits });
        }
    }

    let mut changed_offsets = vec![false; tracked_resolved.len()];
    let mut previous_bits = vec![None; tracked_resolved.len()];
    let mut snapshots = Vec::new();
    let mut truncated = false;
    let mut stream_cursor = 0usize;

    for idx in start_idx..end_idx_exclusive {
        let timestamp = {
            let waveform_ref = waveform.borrow();
            indexed_timestamps(&waveform_ref)?[idx]
        };
        changed_offsets.fill(false);
        previous_bits.fill(None);

        let idx_u32 = u32::try_from(idx).map_err(|_| {
            WavepeekError::Internal("time table index exceeds u32 range".to_string())
        })?;

        let mut any_requested_offset_changed = false;
        let mut delta_confirmed = false;

        {
            let waveform_ref = waveform.borrow();
            for (tracked_index, signal) in tracked_resolved.iter().enumerate() {
                let current_offset = waveform_ref
                    .indexed_signal_offset_at(signal.id, idx_u32)
                    .ok_or_else(indexed_backend_unavailable)?;
                if current_offset == rolling[tracked_index].offset {
                    continue;
                }

                changed_offsets[tracked_index] = true;
                let previous = rolling[tracked_index].bits.clone();
                previous_bits[tracked_index] = Some(previous.clone());

                rolling[tracked_index].offset = current_offset;
                rolling[tracked_index].bits = waveform_ref
                    .decode_indexed_signal_at(signal, idx_u32)?
                    .ok_or_else(indexed_backend_unavailable)?
                    .bits;

                if requested_slot_by_tracked[tracked_index].is_some() {
                    any_requested_offset_changed = true;
                    if let (Some(previous), Some(current)) =
                        (previous.as_ref(), rolling[tracked_index].bits.as_ref())
                        && previous != current
                    {
                        delta_confirmed = true;
                    }
                }
            }
        }

        let is_candidate = if let Some(stream_candidates) = stream_candidate_indices.as_ref() {
            while stream_cursor < stream_candidates.len() && stream_candidates[stream_cursor] < idx
            {
                stream_cursor += 1;
            }
            if stream_cursor < stream_candidates.len() && stream_candidates[stream_cursor] == idx {
                stream_cursor += 1;
                true
            } else {
                false
            }
        } else {
            candidate_tracked_indices
                .iter()
                .any(|candidate_index| changed_offsets[*candidate_index])
        };
        if !is_candidate
            || timestamp <= baseline_raw
            || !any_requested_offset_changed
            || !delta_confirmed
        {
            continue;
        }

        let previous_timestamp = if idx == 0 {
            None
        } else {
            let waveform_ref = waveform.borrow();
            Some(indexed_timestamps(&waveform_ref)?[idx - 1])
        };
        let fast_host = build_fused_event_eval_host(
            host,
            timestamp,
            cached_sources.as_slice(),
            &tracked_index_by_id,
            rolling.as_slice(),
            previous_bits.as_slice(),
        );
        let frame = EventEvalFrame {
            timestamp,
            previous_timestamp,
            tracked_signals: tracked_signal_handles,
        };
        if !event_expr_matches(event_expr_source, bound_event, &fast_host, &frame)? {
            continue;
        }

        if let Some(limit) = max_entries
            && snapshots.len() == limit
        {
            truncated = true;
            break;
        }

        let current_samples = requested_tracked_indices
            .iter()
            .zip(requested_resolved.iter())
            .map(|(tracked_index, resolved)| SampledSignalState {
                path: resolved.path.clone(),
                width: resolved.width,
                bits: rolling[*tracked_index].bits.clone(),
            })
            .collect::<Vec<_>>();
        snapshots.push(build_snapshot(
            requested_signals,
            current_samples.as_slice(),
            timestamp,
            dump_tick,
        )?);
    }

    Ok(ChangeRunOutput {
        snapshots,
        truncated,
    })
}

fn should_use_stream_candidates_in_fused(mode: ChangeCandidateCollectionMode) -> bool {
    match mode {
        ChangeCandidateCollectionMode::Random => false,
        ChangeCandidateCollectionMode::Stream => true,
        ChangeCandidateCollectionMode::Auto => false,
    }
}

fn build_snapshot(
    requested_signals: &[RequestedSignal],
    current_samples: &[SampledSignalState],
    timestamp: u64,
    dump_tick: ParsedTime,
) -> Result<ChangeSnapshot, WavepeekError> {
    let signals = requested_signals
        .iter()
        .zip(current_samples.iter())
        .map(|(requested, sampled)| {
            let bits = sampled.bits.as_ref().ok_or_else(|| {
                WavepeekError::Signal(format!(
                    "signal '{}' has no value at or before requested time",
                    requested.path
                ))
            })?;
            Ok(ChangeSignalValue {
                display: requested.display.clone(),
                path: requested.path.clone(),
                value: format_verilog_literal(sampled.width, bits.as_str()),
            })
        })
        .collect::<Result<Vec<_>, WavepeekError>>()?;

    Ok(ChangeSnapshot {
        time: format_raw_timestamp(timestamp, dump_tick)?,
        signals,
    })
}

fn candidate_times_to_indices(
    timestamps: &[u64],
    candidate_times: &[u64],
) -> Result<Vec<usize>, WavepeekError> {
    candidate_times
        .iter()
        .map(|timestamp| {
            timestamps.binary_search(timestamp).map_err(|_| {
                WavepeekError::Internal(format!(
                    "candidate timestamp '{timestamp}' is missing from waveform time table"
                ))
            })
        })
        .collect()
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

fn resolve_requested_signals(
    waveform: &Waveform,
    scope: Option<&str>,
    args: &ChangeArgs,
) -> Result<Vec<RequestedSignal>, WavepeekError> {
    if let Some(scope) = scope {
        waveform.signals_in_scope(scope)?;
    }

    let mut resolved = Vec::with_capacity(args.signals.len());
    for token in &args.signals {
        let display = token.trim();
        if display.is_empty() {
            return Err(WavepeekError::Args(
                "signal names must not be empty. See 'wavepeek change --help'.".to_string(),
            ));
        }

        let path = resolve_token_to_path(display, scope)?;
        resolved.push(RequestedSignal {
            display: display.to_string(),
            path,
        });
    }

    Ok(resolved)
}

fn resolve_token_to_path(token: &str, scope: Option<&str>) -> Result<String, WavepeekError> {
    let token = token.trim();
    match scope {
        Some(scope) => {
            if token.contains('.') {
                return Err(WavepeekError::Signal(format!(
                    "signal '{token}' not found in dump"
                )));
            }
            Ok(format!("{scope}.{token}"))
        }
        None => Ok(token.to_string()),
    }
}

fn build_candidate_schedule(
    waveform: &Waveform,
    candidate_times: &[u64],
) -> Result<Vec<(u64, Option<u64>)>, WavepeekError> {
    candidate_times
        .iter()
        .map(|timestamp| {
            let previous = waveform.previous_sample_time(*timestamp);
            Ok((*timestamp, previous))
        })
        .collect()
}

fn cached_event_handles(
    bound_event: &BoundEventExpr,
    tracked_signal_handles: &[SignalHandle],
) -> Vec<SignalHandle> {
    let mut handles = event_candidate_handles(bound_event);
    let mut seen = handles.iter().copied().collect::<HashSet<_>>();
    for handle in tracked_signal_handles {
        if seen.insert(*handle) {
            handles.push(*handle);
        }
    }
    handles
}

fn cached_event_sources(
    host: &WaveformExprHost,
    handles: &[SignalHandle],
) -> Result<Vec<(SignalHandle, ExprResolvedSignal)>, WavepeekError> {
    handles
        .iter()
        .map(|handle| Ok((*handle, host.resolved_signal_for_handle(*handle)?)))
        .collect()
}

fn cached_sample_value(signal: &ExprResolvedSignal, bits: Option<String>) -> Option<SampledValue> {
    match signal.expr_type.kind {
        ExprTypeKind::BitVector | ExprTypeKind::IntegerLike(_) | ExprTypeKind::EnumCore => {
            Some(SampledValue::Integral { bits, label: None })
        }
        _ => None,
    }
}

fn build_fused_event_eval_host<'a>(
    host: &'a WaveformExprHost,
    current_timestamp: u64,
    cached_sources: &[(SignalHandle, ExprResolvedSignal)],
    tracked_index_by_id: &HashMap<SignalId, usize>,
    rolling: &[RollingSignalState],
    previous_bits: &[Option<Option<String>>],
) -> FastEventEvalHost<'a> {
    let mut cached = HashMap::new();
    for (handle, signal) in cached_sources {
        let Some(&tracked_index) = tracked_index_by_id.get(&signal.id) else {
            continue;
        };
        let current_bits = rolling[tracked_index].bits.clone();
        let previous = previous_bits[tracked_index]
            .clone()
            .unwrap_or_else(|| current_bits.clone());
        let Some(current) = cached_sample_value(signal, current_bits) else {
            continue;
        };
        let Some(previous) = cached_sample_value(signal, previous) else {
            continue;
        };
        cached.insert(*handle, CachedEventSamples { current, previous });
    }
    FastEventEvalHost {
        base: host,
        current_timestamp,
        cached,
    }
}

fn build_edge_fast_event_eval_host<'a>(
    host: &'a WaveformExprHost,
    current_timestamp: u64,
    current_index: u32,
    previous_index: Option<u32>,
    waveform: &Waveform,
    decode_cache: &mut IndexDecodeCache,
    cached_sources: &[(SignalHandle, ExprResolvedSignal)],
) -> Result<FastEventEvalHost<'a>, WavepeekError> {
    let mut cached = HashMap::new();
    for (handle, signal) in cached_sources {
        let resolved = ResolvedSignal {
            path: signal.path.clone(),
            id: signal.id,
            width: signal.expr_type.width.max(1),
        };
        let current_bits = decode_cache.bits(waveform, &resolved, current_index)?;
        let previous_bits = previous_index
            .map(|index| decode_cache.bits(waveform, &resolved, index))
            .transpose()?
            .flatten();
        let Some(current) = cached_sample_value(signal, current_bits) else {
            continue;
        };
        let Some(previous) = cached_sample_value(signal, previous_bits) else {
            continue;
        };
        cached.insert(*handle, CachedEventSamples { current, previous });
    }
    Ok(FastEventEvalHost {
        base: host,
        current_timestamp,
        cached,
    })
}

fn parse_bound_time(
    token: &str,
    arg_name: &str,
    dump_time: DumpTimeContext,
    metadata: &crate::waveform::WaveformMetadata,
) -> Result<u64, WavepeekError> {
    match validate_time_token_to_raw(token, dump_time, true) {
        Ok(raw) => Ok(raw),
        Err(TimeValidationError::RequiresUnits) => Err(WavepeekError::Args(format!(
            "time token '{token}' requires units. See 'wavepeek change --help'."
        ))),
        Err(TimeValidationError::InvalidToken) => Err(WavepeekError::Args(format!(
            "invalid time token '{token}': expected <integer><unit> (for example 10ns). See 'wavepeek change --help'."
        ))),
        Err(TimeValidationError::TooLarge) => Err(WavepeekError::Args(format!(
            "time '{token}' is too large to process safely. See 'wavepeek change --help'."
        ))),
        Err(TimeValidationError::OutOfBounds) => Err(WavepeekError::Args(format!(
            "time '{}' for {} is outside dump bounds [{}, {}]. See 'wavepeek change --help'.",
            token, arg_name, metadata.time_start, metadata.time_end
        ))),
        Err(TimeValidationError::NotAligned) => {
            let dump_precision = format_raw_timestamp(1, dump_time.dump_tick)?;
            Err(WavepeekError::Args(format!(
                "time '{token}' cannot be represented exactly in dump precision '{}'. See 'wavepeek change --help'.",
                dump_precision
            )))
        }
        Err(TimeValidationError::RawOutOfRange) => Err(WavepeekError::Args(format!(
            "time '{token}' exceeds supported raw timestamp range. See 'wavepeek change --help'."
        ))),
    }
}

#[cfg(test)]
#[path = "../tests/change_private_helpers.rs"]
mod change_private_helpers;

#[cfg(test)]
mod change_inline_derive_tests {
    use super::*;

    #[test]
    fn inline_derive_calls_are_attributed_to_change_module() {
        let signal = ChangeSignalValue {
            display: "sig".to_string(),
            path: "top.sig".to_string(),
            value: "1'b1".to_string(),
        };
        assert_eq!(signal.clone(), signal);
        assert!(format!("{signal:?}").contains("top.sig"));
        assert!(serde_json::to_string(&signal).unwrap().contains("top.sig"));
        let snapshot = ChangeSnapshot {
            time: "1ns".to_string(),
            signals: vec![signal],
        };
        assert_eq!(snapshot.clone(), snapshot);
        assert!(format!("{snapshot:?}").contains("1ns"));
        assert!(serde_json::to_string(&snapshot).unwrap().contains("1ns"));
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;

    use tempfile::NamedTempFile;

    use super::{
        AutoDispatchWorkEstimate, CachedEventSamples, ChangeEngineMode, FastEventEvalHost,
        IndexDecodeCache, RequestedSignal, RollingSignalState, SampleCache,
        build_candidate_schedule, build_edge_fast_event_eval_host, build_fused_event_eval_host,
        build_snapshot, cached_event_handles, cached_event_sources, cached_sample_value,
        candidate_times_to_indices, parse_bound_time, resolve_requested_signals,
        resolve_token_to_path, run, run_baseline, run_edge_fast, run_fused, select_engine_mode,
        should_use_stream_candidates_in_fused, time_window_indices,
    };
    use crate::cli::change::{ChangeArgs, TuneChangeCandidateMode, TuneChangeEngineMode};
    use crate::cli::limits::LimitArg;
    use crate::engine::CommandData;
    use crate::engine::expr_runtime::bind_waveform_event_expr;
    use crate::expr::SignalHandle;
    use crate::expr::host::{ExprStorage, ExprType, ExprTypeKind, ExpressionHost, SampledValue};
    use crate::expr::sema::{BoundEventExpr, BoundEventKind, BoundEventTerm};
    use crate::waveform::{
        ChangeCandidateCollectionMode, SampledSignalState, Waveform, expr_host::WaveformExprHost,
    };

    fn select_auto_mode_for_profile(
        any_tracked_only: bool,
        edge_only: bool,
        requested_signal_count: usize,
        fused_work: usize,
        edge_work: usize,
    ) -> ChangeEngineMode {
        select_engine_mode(
            TuneChangeEngineMode::Auto,
            any_tracked_only,
            edge_only,
            requested_signal_count,
            AutoDispatchWorkEstimate {
                fused_work,
                edge_work,
            },
        )
    }

    #[test]
    fn auto_engine_mode_uses_fused_for_wide_any_tracked_only() {
        let selected = select_auto_mode_for_profile(true, false, 64, 500_000, 0);

        assert_eq!(selected, ChangeEngineMode::Fused);
    }

    #[test]
    fn auto_engine_mode_uses_fused_for_mid_any_tracked_only() {
        let selected = select_auto_mode_for_profile(true, false, 10, 250_000, 0);

        assert_eq!(selected, ChangeEngineMode::Fused);
    }

    #[test]
    fn auto_engine_mode_uses_fused_for_wide_selective_edge_events() {
        let selected = select_auto_mode_for_profile(false, true, 128, 0, 1_500_000);

        assert_eq!(selected, ChangeEngineMode::Fused);
    }

    #[test]
    fn auto_engine_mode_uses_edge_fast_for_ultra_wide_selective_edge_events() {
        let selected = select_auto_mode_for_profile(false, true, 128, 0, 2_500_000);

        assert_eq!(selected, ChangeEngineMode::EdgeFast);
    }

    #[test]
    fn auto_engine_mode_uses_fused_for_mid_edge_only_terms() {
        let selected = select_auto_mode_for_profile(false, true, 10, 0, 1_500_000);

        assert_eq!(selected, ChangeEngineMode::Fused);
    }

    #[test]
    fn auto_engine_mode_keeps_baseline_for_narrow_selective_edge_events() {
        let selected = select_auto_mode_for_profile(false, true, 16, 2_000, 2_000);

        assert_eq!(selected, ChangeEngineMode::Baseline);
    }

    #[test]
    fn auto_engine_mode_keeps_baseline_for_low_signal_count_edge_only_terms() {
        let selected = select_auto_mode_for_profile(false, true, 1, 0, 2_000_000);

        assert_eq!(selected, ChangeEngineMode::Baseline);
    }

    #[test]
    fn auto_engine_mode_keeps_baseline_for_low_work_any_tracked_only() {
        let selected = select_auto_mode_for_profile(true, false, 10, 2_000, 2_000);

        assert_eq!(selected, ChangeEngineMode::Baseline);
    }

    #[test]
    fn auto_engine_mode_keeps_baseline_below_fused_threshold() {
        let selected = select_auto_mode_for_profile(true, false, 31, 2_000, 2_000);

        assert_eq!(selected, ChangeEngineMode::Baseline);
    }

    #[test]
    fn auto_engine_mode_uses_fused_at_threshold_for_any_tracked() {
        let selected = select_auto_mode_for_profile(true, false, 32, 2_000, 2_000);

        assert_eq!(selected, ChangeEngineMode::Fused);
    }

    #[test]
    fn candidate_and_window_helpers_exercise_success_and_error_paths() {
        assert_eq!(time_window_indices(&[], 0, 1), None);
        assert_eq!(time_window_indices(&[0, 5, 10], 7, 6), None);
        assert_eq!(time_window_indices(&[0, 5, 10], 5, 10), Some((1, 3)));

        assert_eq!(
            candidate_times_to_indices(&[0, 5, 10], &[0, 10]).expect("indices"),
            vec![0, 2]
        );
        let fixture = write_fixture(TEST_VCD, "change-schedule.vcd");
        let waveform = Waveform::open(fixture.path()).expect("waveform should open");
        assert_eq!(
            build_candidate_schedule(&waveform, &[5, 10]).expect("schedule"),
            vec![(5, Some(0)), (10, Some(5))]
        );
        assert!(
            candidate_times_to_indices(&[0, 5], &[1])
                .expect_err("missing candidate should fail")
                .to_string()
                .contains("missing from waveform time table")
        );
    }

    #[test]
    fn request_resolution_and_snapshot_helpers_exercise_validation() {
        assert_eq!(
            resolve_token_to_path("sig", Some("top")).expect("scoped token"),
            "top.sig"
        );
        assert_eq!(
            resolve_token_to_path("top.sig", None).expect("canonical token"),
            "top.sig"
        );
        assert!(
            resolve_token_to_path("top.sig", Some("top"))
                .expect_err("dotted scoped token should fail")
                .to_string()
                .contains("signal 'top.sig' not found in dump")
        );

        let fixture = write_fixture(TEST_VCD, "change-helper.vcd");
        let waveform = Waveform::open(fixture.path()).expect("waveform should open");
        let args = ChangeArgs {
            waves: PathBuf::from(fixture.path()),
            from: None,
            to: None,
            scope: Some("top".to_string()),
            signals: vec!["sig".to_string(), "msg".to_string()],
            on: None,
            max: LimitArg::Numeric(5),
            abs: false,
            json: false,
            tune_engine: TuneChangeEngineMode::Auto,
            tune_candidates: TuneChangeCandidateMode::Auto,
            tune_edge_fast_force: false,
        };
        let resolved = resolve_requested_signals(&waveform, args.scope.as_deref(), &args)
            .expect("signals should resolve");
        assert_eq!(resolved[0].display, "sig");
        assert_eq!(resolved[0].path, "top.sig");

        let snapshot = build_snapshot(
            &[RequestedSignal {
                display: "sig".to_string(),
                path: "top.sig".to_string(),
            }],
            &[SampledSignalState {
                path: "top.sig".to_string(),
                width: 1,
                bits: Some("1".to_string()),
            }],
            5,
            crate::engine::time::ParsedTime {
                value: 1,
                unit: crate::engine::time::TimeUnit::Ns,
            },
        )
        .expect("snapshot should build");
        assert_eq!(snapshot.time, "5ns");
        assert_eq!(snapshot.signals[0].value, "1'h1");

        let error = build_snapshot(
            &[RequestedSignal {
                display: "sig".to_string(),
                path: "top.sig".to_string(),
            }],
            &[SampledSignalState {
                path: "top.sig".to_string(),
                width: 1,
                bits: None,
            }],
            5,
            crate::engine::time::ParsedTime {
                value: 1,
                unit: crate::engine::time::TimeUnit::Ns,
            },
        )
        .expect_err("missing value should fail");
        assert!(
            error
                .to_string()
                .contains("has no value at or before requested time")
        );
    }

    #[test]
    fn cached_event_helpers_exercise_dedup_and_sample_filtering() {
        let bound_event = BoundEventExpr {
            terms: vec![
                BoundEventTerm {
                    event: BoundEventKind::Posedge(SignalHandle(1)),
                    iff: None,
                },
                BoundEventTerm {
                    event: BoundEventKind::Named(SignalHandle(2)),
                    iff: None,
                },
                BoundEventTerm {
                    event: BoundEventKind::Named(SignalHandle(1)),
                    iff: None,
                },
            ],
        };
        assert_eq!(
            cached_event_handles(&bound_event, &[SignalHandle(2), SignalHandle(3)]),
            vec![SignalHandle(1), SignalHandle(2), SignalHandle(3)]
        );

        let integral = crate::waveform::ExprResolvedSignal {
            path: "top.sig".to_string(),
            id: crate::waveform::SignalId::from_test_index(1),
            expr_type: ExprType {
                kind: ExprTypeKind::BitVector,
                storage: ExprStorage::Scalar,
                width: 1,
                is_four_state: true,
                is_signed: false,
                enum_type_id: None,
                enum_labels: None,
            },
        };
        let real = crate::waveform::ExprResolvedSignal {
            path: "top.temp".to_string(),
            id: crate::waveform::SignalId::from_test_index(2),
            expr_type: ExprType {
                kind: ExprTypeKind::Real,
                storage: ExprStorage::Scalar,
                width: 64,
                is_four_state: false,
                is_signed: false,
                enum_type_id: None,
                enum_labels: None,
            },
        };
        assert_eq!(
            cached_sample_value(&integral, Some("1".to_string())),
            Some(SampledValue::Integral {
                bits: Some("1".to_string()),
                label: None,
            })
        );
        assert_eq!(cached_sample_value(&real, Some("1".to_string())), None);
        assert!(should_use_stream_candidates_in_fused(
            ChangeCandidateCollectionMode::Stream
        ));
        assert!(!should_use_stream_candidates_in_fused(
            ChangeCandidateCollectionMode::Random
        ));
    }

    #[test]
    fn fast_event_eval_host_prefers_cached_samples_over_waveform_queries() {
        let fixture = write_fixture(TEST_VCD, "change-fast-host.vcd");
        let host = WaveformExprHost::open(fixture.path()).expect("host should open");
        let cached = std::collections::HashMap::from([(
            SignalHandle(9),
            CachedEventSamples {
                current: SampledValue::Integral {
                    bits: Some("1".to_string()),
                    label: None,
                },
                previous: SampledValue::Integral {
                    bits: Some("0".to_string()),
                    label: None,
                },
            },
        )]);
        let fast = FastEventEvalHost {
            base: &host,
            current_timestamp: 5,
            cached,
        };

        assert_eq!(
            fast.resolve_signal("top.sig")
                .expect("resolve should forward"),
            host.resolve_signal("top.sig").expect("base resolve")
        );
        assert!(matches!(
            fast.signal_type(
                host.resolve_signal("top.sig")
                    .expect("signal should resolve")
            )
            .expect("type should forward")
            .kind,
            ExprTypeKind::BitVector
        ));
        assert!(
            fast.event_occurred(
                host.resolve_signal("top.sig")
                    .expect("signal should resolve"),
                5
            )
            .is_err()
        );
        assert_eq!(
            fast.sample_value(SignalHandle(9), 5)
                .expect("current sample"),
            SampledValue::Integral {
                bits: Some("1".to_string()),
                label: None,
            }
        );
        assert_eq!(
            fast.sample_value(SignalHandle(9), 4)
                .expect("previous sample"),
            SampledValue::Integral {
                bits: Some("0".to_string()),
                label: None,
            }
        );
        assert!(
            fast.sample_value(
                host.resolve_signal("top.sig")
                    .expect("signal should resolve"),
                5,
            )
            .is_ok()
        );

        let sources = cached_event_sources(&host, &[host.resolve_signal("top.sig").unwrap()])
            .expect("cached event sources should resolve");
        assert_eq!(sources.len(), 1);
        assert_eq!(sources[0].1.path, "top.sig");
    }

    #[test]
    fn fast_event_eval_builders_exercise_skip_and_missing_previous_paths() {
        let fixture = write_fixture(TEST_VCD, "change-fast-builders.vcd");
        let host = WaveformExprHost::open(fixture.path()).expect("host should open");
        let handle = host
            .resolve_signal("top.sig")
            .expect("signal should resolve");
        let source = host
            .resolved_signal_for_handle(handle)
            .expect("source should resolve");

        let mut tracked_index_by_id = std::collections::HashMap::new();
        tracked_index_by_id.insert(source.id, 0usize);
        let rolling = vec![RollingSignalState {
            offset: None,
            bits: Some("1".to_string()),
        }];
        let previous_bits = vec![None];
        let missing_id_source = crate::waveform::ExprResolvedSignal {
            id: crate::waveform::SignalId::from_test_index(999),
            ..source.clone()
        };
        let real_source = crate::waveform::ExprResolvedSignal {
            expr_type: ExprType {
                kind: ExprTypeKind::Real,
                storage: ExprStorage::Scalar,
                width: 64,
                is_four_state: false,
                is_signed: false,
                enum_type_id: None,
                enum_labels: None,
            },
            ..source.clone()
        };
        let fused_host = build_fused_event_eval_host(
            &host,
            7,
            &[
                (SignalHandle(21), source.clone()),
                (SignalHandle(22), missing_id_source),
                (SignalHandle(23), real_source),
            ],
            &tracked_index_by_id,
            &rolling,
            &previous_bits,
        );
        assert_eq!(
            fused_host
                .sample_value(SignalHandle(21), 7)
                .expect("current cached sample"),
            SampledValue::Integral {
                bits: Some("1".to_string()),
                label: None,
            }
        );
        assert_eq!(
            fused_host
                .sample_value(SignalHandle(21), 6)
                .expect("previous defaults to current when unchanged"),
            SampledValue::Integral {
                bits: Some("1".to_string()),
                label: None,
            }
        );
        assert!(fused_host.sample_value(SignalHandle(22), 7).is_err());
        assert!(fused_host.sample_value(SignalHandle(23), 7).is_err());

        let mut waveform = Waveform::open(fixture.path()).expect("waveform should open");
        waveform.ensure_indexed_signals_loaded(&[source.id]);
        let mut decode_cache = IndexDecodeCache::default();
        let edge_host = build_edge_fast_event_eval_host(
            &host,
            5,
            1,
            None,
            &waveform,
            &mut decode_cache,
            &[(SignalHandle(31), source)],
        )
        .expect("edge-fast host should build");
        assert_eq!(
            edge_host
                .sample_value(SignalHandle(31), 5)
                .expect("current edge-fast sample"),
            SampledValue::Integral {
                bits: Some("1".to_string()),
                label: None,
            }
        );
        assert_eq!(
            edge_host
                .sample_value(SignalHandle(31), 4)
                .expect("missing previous index should become empty sample"),
            SampledValue::Integral {
                bits: None,
                label: None,
            }
        );
    }

    #[test]
    fn change_cache_helpers_exercise_request_reuse_and_schedule_edges() {
        let fixture = write_fixture(TEST_VCD, "change-cache-helpers.vcd");
        let waveform = std::rc::Rc::new(std::cell::RefCell::new(
            Waveform::open(fixture.path()).expect("waveform should open"),
        ));
        let resolved = waveform
            .borrow()
            .resolve_signals(&["top.sig".to_string()])
            .expect("signal should resolve");

        let mut sample_cache = SampleCache::default();
        let first = sample_cache
            .sample_requested_batch(&waveform, &resolved, 5)
            .expect("first sample should work");
        let second = sample_cache
            .sample_requested_batch(&waveform, &resolved, 5)
            .expect("cached sample should work");
        assert_eq!(first, second);

        let mut decode_cache = IndexDecodeCache::default();
        let waveform_ref = waveform.borrow();
        let first_bits = decode_cache
            .bits(&waveform_ref, &resolved[0], 0)
            .expect("decode should work");
        let second_bits = decode_cache
            .bits(&waveform_ref, &resolved[0], 0)
            .expect("cached decode should work");
        assert_eq!(first_bits, second_bits);
        drop(waveform_ref);

        let waveform_ref = waveform.borrow();
        assert_eq!(
            build_candidate_schedule(&waveform_ref, &[0, 5]).expect("schedule should build"),
            vec![(0, None), (5, Some(0))]
        );
    }

    #[test]
    fn change_run_exercises_public_entrypoint_success_and_early_errors() {
        let fixture = write_fixture(TEST_VCD, "change-run.vcd");

        let result = run(ChangeArgs {
            waves: PathBuf::from(fixture.path()),
            from: None,
            to: None,
            scope: Some("top".to_string()),
            signals: vec!["sig".to_string()],
            on: Some("posedge sig".to_string()),
            max: LimitArg::Unlimited,
            abs: false,
            json: true,
            tune_engine: TuneChangeEngineMode::Auto,
            tune_candidates: TuneChangeCandidateMode::Auto,
            tune_edge_fast_force: false,
        })
        .expect("change run should succeed");
        assert_eq!(result.warnings, vec!["limit disabled: --max=unlimited"]);
        let CommandData::Change(rows) = result.data else {
            panic!("change command should return change rows");
        };
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].time, "5ns");
        assert_eq!(rows[0].signals[0].path, "top.sig");
        assert_eq!(rows[0].signals[0].value, "1'h1");

        let zero_max = run(ChangeArgs {
            waves: PathBuf::from(fixture.path()),
            from: None,
            to: None,
            scope: Some("top".to_string()),
            signals: vec!["sig".to_string()],
            on: None,
            max: LimitArg::Numeric(0),
            abs: false,
            json: false,
            tune_engine: TuneChangeEngineMode::Auto,
            tune_candidates: TuneChangeCandidateMode::Auto,
            tune_edge_fast_force: false,
        })
        .expect_err("zero max should fail");
        assert!(
            zero_max
                .to_string()
                .contains("--max must be greater than 0")
        );

        let reversed = run(ChangeArgs {
            waves: PathBuf::from(fixture.path()),
            from: Some("5ns".to_string()),
            to: Some("0ns".to_string()),
            scope: Some("top".to_string()),
            signals: vec!["sig".to_string()],
            on: Some("posedge sig".to_string()),
            max: LimitArg::Numeric(5),
            abs: false,
            json: false,
            tune_engine: TuneChangeEngineMode::Auto,
            tune_candidates: TuneChangeCandidateMode::Auto,
            tune_edge_fast_force: false,
        })
        .expect_err("reversed range should fail");
        assert!(reversed.to_string().contains("less than or equal to --to"));
    }

    #[test]
    fn change_run_and_time_helpers_exercise_warning_and_bound_error_paths() {
        let fixture = write_fixture(TEST_VCD, "change-warning-empty.vcd");
        let empty = run(ChangeArgs {
            waves: PathBuf::from(fixture.path()),
            from: None,
            to: None,
            scope: Some("top".to_string()),
            signals: vec!["sig".to_string()],
            on: Some("negedge sig".to_string()),
            max: LimitArg::Numeric(5),
            abs: false,
            json: false,
            tune_engine: TuneChangeEngineMode::Baseline,
            tune_candidates: TuneChangeCandidateMode::Auto,
            tune_edge_fast_force: false,
        })
        .expect("empty result should still succeed");
        assert_eq!(empty.warnings, vec![super::EMPTY_WARNING.to_string()]);

        const MULTI_CHANGE_VCD: &str = concat!(
            "$date\n  today\n$end\n",
            "$version\n  wavepeek-test\n$end\n",
            "$timescale 1ns $end\n",
            "$scope module top $end\n",
            "$var wire 1 ! sig $end\n",
            "$upscope $end\n",
            "$enddefinitions $end\n",
            "#0\n0!\n#5\n1!\n#10\n0!\n#15\n1!\n"
        );
        let multi = write_fixture(MULTI_CHANGE_VCD, "change-warning-truncated.vcd");
        let truncated = run(ChangeArgs {
            waves: PathBuf::from(multi.path()),
            from: None,
            to: None,
            scope: Some("top".to_string()),
            signals: vec!["sig".to_string()],
            on: Some("*".to_string()),
            max: LimitArg::Numeric(1),
            abs: false,
            json: false,
            tune_engine: TuneChangeEngineMode::Baseline,
            tune_candidates: TuneChangeCandidateMode::Auto,
            tune_edge_fast_force: false,
        })
        .expect("truncated result should succeed");
        assert!(
            truncated
                .warnings
                .iter()
                .any(|warning| { warning.contains("truncated output to 1 entries") })
        );
        let CommandData::Change(rows) = truncated.data else {
            panic!("change command should return rows");
        };
        assert_eq!(rows.len(), 1);

        let waveform = Waveform::open(fixture.path()).expect("waveform should open");
        let metadata = waveform.metadata().expect("metadata should load");
        let dump_time = crate::engine::time::parse_dump_time_context(&metadata)
            .expect("dump time should parse");
        for (token, expected) in [
            ("5", "requires units"),
            ("abc", "invalid time token"),
            ("10ns", "outside dump bounds"),
            ("1ps", "cannot be represented exactly"),
        ] {
            assert!(
                parse_bound_time(token, "--from", dump_time, &metadata)
                    .expect_err("time token should fail")
                    .to_string()
                    .contains(expected),
                "{token} should contain {expected}"
            );
        }
    }

    #[test]
    fn run_edge_fast_falls_back_to_baseline_for_small_edge_workloads() {
        let fixture = write_fixture(TEST_VCD, "change-edge-fast-fallback.vcd");
        let waveform = std::rc::Rc::new(std::cell::RefCell::new(
            Waveform::open(fixture.path()).expect("waveform should open"),
        ));
        let (host, bound_event) =
            bind_waveform_event_expr(waveform.clone(), Some("top"), "posedge sig")
                .expect("event expression should bind");
        let requested_signals = vec![RequestedSignal {
            display: "sig".to_string(),
            path: "top.sig".to_string(),
        }];
        let requested_resolved = waveform
            .borrow()
            .resolve_signals(&["top.sig".to_string()])
            .expect("signal should resolve");
        let requested_expr_sources = waveform
            .borrow()
            .resolve_expr_signals(&["top.sig".to_string()])
            .expect("expr signal should resolve");
        let tracked_signal_handles = vec![host.resolve_signal("top.sig").expect("handle")];
        let dump_tick = crate::engine::time::ParsedTime {
            value: 1,
            unit: crate::engine::time::TimeUnit::Ns,
        };

        let baseline = run_baseline(
            &waveform,
            &host,
            "posedge sig",
            &bound_event,
            &tracked_signal_handles,
            &requested_signals,
            &requested_resolved,
            &requested_expr_sources,
            0,
            5,
            0,
            dump_tick,
            None,
            ChangeCandidateCollectionMode::Random,
            None,
        )
        .expect("baseline should succeed");
        let edge_fast = run_edge_fast(
            &waveform,
            &host,
            "posedge sig",
            &bound_event,
            &tracked_signal_handles,
            &requested_signals,
            &requested_resolved,
            &requested_expr_sources,
            0,
            5,
            0,
            dump_tick,
            None,
            ChangeCandidateCollectionMode::Random,
            false,
        )
        .expect("edge-fast fallback should succeed");
        assert_eq!(edge_fast.snapshots, baseline.snapshots);
        assert_eq!(edge_fast.truncated, baseline.truncated);

        let forced_edge_fast = run_edge_fast(
            &waveform,
            &host,
            "posedge sig",
            &bound_event,
            &tracked_signal_handles,
            &requested_signals,
            &requested_resolved,
            &requested_expr_sources,
            0,
            5,
            0,
            dump_tick,
            Some(0),
            ChangeCandidateCollectionMode::Random,
            true,
        )
        .expect("forced edge-fast path should run directly");
        assert!(forced_edge_fast.truncated);
        assert!(forced_edge_fast.snapshots.is_empty());

        let fused = run_fused(
            &waveform,
            &host,
            "posedge sig",
            &bound_event,
            &tracked_signal_handles,
            &requested_signals,
            &requested_resolved,
            &requested_expr_sources,
            0,
            5,
            0,
            dump_tick,
            None,
            ChangeCandidateCollectionMode::Random,
        )
        .expect("fused should succeed");
        assert_eq!(fused.snapshots, baseline.snapshots);
        assert_eq!(fused.truncated, baseline.truncated);

        let truncated_fused = run_fused(
            &waveform,
            &host,
            "posedge sig",
            &bound_event,
            &tracked_signal_handles,
            &requested_signals,
            &requested_resolved,
            &requested_expr_sources,
            0,
            5,
            0,
            dump_tick,
            Some(0),
            ChangeCandidateCollectionMode::Random,
        )
        .expect("fused truncation branch should run");
        assert!(truncated_fused.truncated);
        assert!(truncated_fused.snapshots.is_empty());
    }

    const TEST_VCD: &str = concat!(
        "$date\n  today\n$end\n",
        "$version\n  wavepeek-test\n$end\n",
        "$timescale 1ns $end\n",
        "$scope module top $end\n",
        "$var wire 1 ! sig $end\n",
        "$var string 1 \" msg $end\n",
        "$upscope $end\n",
        "$enddefinitions $end\n",
        "#0\n",
        "0!\n",
        "shello \"\n",
        "#5\n",
        "1!\n",
        "sworld \"\n"
    );

    fn write_fixture(contents: &str, suffix: &str) -> NamedTempFile {
        let fixture = NamedTempFile::with_suffix(suffix).expect("fixture should create");
        fs::write(fixture.path(), contents).expect("fixture should write");
        fixture
    }
}
