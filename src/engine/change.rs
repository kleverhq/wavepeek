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
use crate::expr::{BoundEventExpr, EventEvalFrame, ExpressionHost, SignalHandle};
use crate::waveform::{
    ChangeCandidateCollectionMode, ExprResolvedSignal, ResolvedSignal, SampledSignalState,
    SignalOffsetData, Waveform, expr_host::WaveformExprHost, should_emit_delta_and_update_baseline,
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
    entries: HashMap<(wellen::SignalRef, u32), Option<String>>,
}

impl IndexDecodeCache {
    fn bits(
        &mut self,
        waveform: &Waveform,
        resolved: &ResolvedSignal,
        time_table_idx: u32,
    ) -> Result<Option<String>, WavepeekError> {
        let key = (resolved.signal_ref, time_table_idx);
        if let Some(existing) = self.entries.get(&key) {
            return Ok(existing.clone());
        }

        let bits = waveform
            .decode_signal_at_index(resolved, time_table_idx)?
            .bits;
        self.entries.insert(key, bits.clone());
        Ok(bits)
    }
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
    candidate_sources.retain(|signal| seen.insert(signal.signal_ref));

    let candidate_mode = map_candidate_mode(args.tune_candidates);
    let window_timestamp_count = {
        let waveform_ref = waveform.borrow();
        time_window_indices(waveform_ref.timestamps_raw_slice(), from_raw, to_raw)
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
        build_candidate_schedule(waveform_ref.timestamps_raw_slice(), &candidate_times)?
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
        return run_baseline(
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
        return run_baseline(
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
        let time_table = waveform_ref.timestamps_raw_slice();
        candidate_times_to_indices(time_table, candidate_times.as_slice())?
    };

    let mut loaded_signal_refs = requested_resolved
        .iter()
        .map(|signal| signal.signal_ref)
        .collect::<HashSet<_>>();
    for signal in candidate_sources {
        loaded_signal_refs.insert(signal.signal_ref);
    }
    let loaded_signal_refs = loaded_signal_refs.into_iter().collect::<Vec<_>>();
    waveform
        .borrow_mut()
        .ensure_signals_loaded(loaded_signal_refs.as_slice());

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
                let current_offset =
                    waveform_ref.signal_offset_at_index(resolved.signal_ref, candidate_index_u32);
                let previous_offset = previous_index
                    .and_then(|idx| waveform_ref.signal_offset_at_index(resolved.signal_ref, idx));
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
            Some(waveform.borrow().timestamps_raw_slice()[candidate_index - 1])
        };
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
    let mut tracked_resolved = requested_resolved.to_vec();
    let mut tracked_seen = tracked_resolved
        .iter()
        .map(|signal| signal.signal_ref)
        .collect::<HashSet<_>>();
    for signal in candidate_sources {
        if tracked_seen.insert(signal.signal_ref) {
            tracked_resolved.push(ResolvedSignal {
                path: signal.path.clone(),
                signal_ref: signal.signal_ref,
                width: signal.expr_type.width.max(1),
            });
        }
    }

    let tracked_index_by_ref = tracked_resolved
        .iter()
        .enumerate()
        .map(|(index, signal)| (signal.signal_ref, index))
        .collect::<HashMap<_, _>>();
    let requested_tracked_indices = requested_resolved
        .iter()
        .map(|signal| {
            tracked_index_by_ref
                .get(&signal.signal_ref)
                .copied()
                .ok_or_else(|| {
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
            tracked_index_by_ref
                .get(&signal.signal_ref)
                .copied()
                .ok_or_else(|| {
                    WavepeekError::Internal(format!(
                        "candidate signal '{}' is missing from fused tracking state",
                        signal.path
                    ))
                })
        })
        .collect::<Result<Vec<_>, _>>()?;

    let all_signal_refs = tracked_resolved
        .iter()
        .map(|signal| signal.signal_ref)
        .collect::<Vec<_>>();
    waveform
        .borrow_mut()
        .ensure_signals_loaded(all_signal_refs.as_slice());

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

    let (start_idx, end_idx_exclusive) = {
        let waveform_ref = waveform.borrow();
        let Some(window) =
            time_window_indices(waveform_ref.timestamps_raw_slice(), from_raw, to_raw)
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
            waveform_ref.timestamps_raw_slice(),
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
            let offset = waveform_ref.signal_offset_at_index(signal.signal_ref, previous_idx);
            let bits = waveform_ref
                .decode_signal_at_index(signal, previous_idx)?
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
        let timestamp = waveform.borrow().timestamps_raw_slice()[idx];
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
                let current_offset =
                    waveform_ref.signal_offset_at_index(signal.signal_ref, idx_u32);
                if current_offset == rolling[tracked_index].offset {
                    continue;
                }

                changed_offsets[tracked_index] = true;
                let previous = rolling[tracked_index].bits.clone();
                previous_bits[tracked_index] = Some(previous.clone());

                rolling[tracked_index].offset = current_offset;
                rolling[tracked_index].bits =
                    waveform_ref.decode_signal_at_index(signal, idx_u32)?.bits;

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
            Some(waveform.borrow().timestamps_raw_slice()[idx - 1])
        };
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
    timestamps: &[u64],
    candidate_times: &[u64],
) -> Result<Vec<(u64, Option<u64>)>, WavepeekError> {
    candidate_times
        .iter()
        .map(|timestamp| {
            let index = timestamps.binary_search(timestamp).map_err(|_| {
                WavepeekError::Internal(format!(
                    "candidate timestamp '{timestamp}' is missing from waveform time table"
                ))
            })?;
            let previous = if index == 0 {
                None
            } else {
                Some(timestamps[index - 1])
            };
            Ok((*timestamp, previous))
        })
        .collect()
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
mod tests {
    use super::{AutoDispatchWorkEstimate, ChangeEngineMode, select_engine_mode};
    use crate::cli::change::TuneChangeEngineMode;

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
}
