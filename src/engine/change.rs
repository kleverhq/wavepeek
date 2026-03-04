use std::collections::{HashMap, HashSet};

use serde::Serialize;

use crate::cli::change::{ChangeArgs, TuneChangeCandidateMode, TuneChangeEngineMode};
use crate::cli::limits::LimitArg;
use crate::engine::at::{
    ParsedTime, as_zeptoseconds, ensure_non_zero_dump_tick, format_raw_timestamp,
    format_verilog_literal, parse_time_token,
};
use crate::engine::{CommandData, CommandName, CommandResult, HumanRenderOptions};
use crate::error::WavepeekError;
use crate::expr::{EventKind, EventTerm, parse_event_expr};
use crate::waveform::{
    ChangeCandidateCollectionMode, EdgeClassification, ResolvedSignal, SampledSignalState,
    SignalOffsetData, Waveform, classify_edge, should_emit_delta_and_update_baseline,
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

#[derive(Debug, Clone, PartialEq, Eq)]
enum ResolvedEventKind {
    AnyTracked,
    AnyChange(String),
    Posedge(String),
    Negedge(String),
    Edge(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ResolvedEventTerm {
    event: ResolvedEventKind,
    iff_expr: Option<String>,
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
    entries: HashMap<(wellen::SignalRef, u64), SampledSignalState>,
    requested_batches: HashMap<u64, Vec<SampledSignalState>>,
}

impl SampleCache {
    fn sample(
        &mut self,
        waveform: &mut Waveform,
        resolved_by_path: &HashMap<String, ResolvedSignal>,
        path: &str,
        raw_time: u64,
    ) -> Result<SampledSignalState, WavepeekError> {
        let resolved = resolved_by_path.get(path).ok_or_else(|| {
            WavepeekError::Internal(format!("internal resolved signal is missing for '{path}'"))
        })?;

        let key = (resolved.signal_ref, raw_time);
        if let Some(existing) = self.entries.get(&key) {
            return Ok(existing.clone());
        }

        let sampled = waveform
            .sample_resolved_optional(std::slice::from_ref(resolved), raw_time)?
            .pop()
            .ok_or_else(|| {
                WavepeekError::Internal(
                    "waveform backend returned no samples for requested signal".to_string(),
                )
            })?;

        self.entries.insert(key, sampled.clone());
        Ok(sampled)
    }

    fn sample_requested_batch(
        &mut self,
        waveform: &mut Waveform,
        resolved: &[ResolvedSignal],
        raw_time: u64,
    ) -> Result<Vec<SampledSignalState>, WavepeekError> {
        if let Some(existing) = self.requested_batches.get(&raw_time) {
            return Ok(existing.clone());
        }

        let sampled = waveform.sample_resolved_optional(resolved, raw_time)?;
        for (resolved_signal, sampled_signal) in resolved.iter().zip(sampled.iter()) {
            self.entries.insert(
                (resolved_signal.signal_ref, raw_time),
                sampled_signal.clone(),
            );
        }
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

    let mut waveform = Waveform::open(args.waves.as_path())?;
    let metadata = waveform.metadata()?;

    let requested_signals = resolve_requested_signals(&waveform, args.scope.as_deref(), &args)?;

    let event_expr_source = args.when.as_deref().unwrap_or("*");
    let event_expr = parse_event_expr(event_expr_source)?;

    if event_expr.terms.iter().any(|term| term.iff_expr.is_some()) {
        return Err(WavepeekError::Args(
            "iff logical expressions are not implemented yet".to_string(),
        ));
    }

    let resolved_event_terms = event_expr
        .terms
        .iter()
        .map(|term| resolve_event_term(term, args.scope.as_deref()))
        .collect::<Result<Vec<_>, _>>()?;
    let event_signal_paths = collect_event_signal_paths(resolved_event_terms.as_slice());

    let dump_tick = parse_time_token(metadata.time_unit.as_str()).ok_or_else(|| {
        WavepeekError::Internal(format!(
            "waveform metadata contains invalid time_unit '{}': expected <integer><unit>",
            metadata.time_unit
        ))
    })?;
    let dump_tick_zs = as_zeptoseconds(dump_tick).ok_or_else(|| {
        WavepeekError::Internal(
            "waveform metadata time_unit overflowed during conversion".to_string(),
        )
    })?;
    ensure_non_zero_dump_tick(dump_tick_zs)?;

    let dump_start = parse_time_token(metadata.time_start.as_str()).ok_or_else(|| {
        WavepeekError::Internal(format!(
            "waveform metadata contains invalid time_start '{}': expected <integer><unit>",
            metadata.time_start
        ))
    })?;
    let dump_start_zs = as_zeptoseconds(dump_start).ok_or_else(|| {
        WavepeekError::Internal(
            "waveform metadata time_start overflowed during conversion".to_string(),
        )
    })?;

    let dump_end = parse_time_token(metadata.time_end.as_str()).ok_or_else(|| {
        WavepeekError::Internal(format!(
            "waveform metadata contains invalid time_end '{}': expected <integer><unit>",
            metadata.time_end
        ))
    })?;
    let dump_end_zs = as_zeptoseconds(dump_end).ok_or_else(|| {
        WavepeekError::Internal(
            "waveform metadata time_end overflowed during conversion".to_string(),
        )
    })?;

    let from_raw = match args.from.as_deref() {
        Some(token) => parse_bound_time(token, "--from", dump_tick, dump_tick_zs, &metadata)?,
        None => u64::try_from(dump_start_zs / dump_tick_zs).map_err(|_| {
            WavepeekError::Internal("dump start timestamp exceeds supported range".to_string())
        })?,
    };
    let to_raw = match args.to.as_deref() {
        Some(token) => parse_bound_time(token, "--to", dump_tick, dump_tick_zs, &metadata)?,
        None => u64::try_from(dump_end_zs / dump_tick_zs).map_err(|_| {
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
    let requested_paths = requested_signals
        .iter()
        .map(|signal| signal.path.as_str())
        .collect::<Vec<_>>();
    let requested_resolved = waveform.resolve_signals(&requested_paths_owned)?;

    let mut resolved_by_path = HashMap::new();
    for signal in &requested_resolved {
        resolved_by_path.insert(signal.path.clone(), signal.clone());
    }

    let unresolved_event_paths = event_signal_paths
        .iter()
        .filter(|path| !resolved_by_path.contains_key(path.as_str()))
        .cloned()
        .collect::<Vec<_>>();
    if !unresolved_event_paths.is_empty() {
        let extra_resolved = waveform.resolve_signals(unresolved_event_paths.as_slice())?;
        for signal in extra_resolved {
            resolved_by_path.insert(signal.path.clone(), signal);
        }
    }

    let mut candidate_resolved = Vec::new();
    if resolved_event_terms
        .iter()
        .any(|term| matches!(term.event, ResolvedEventKind::AnyTracked))
    {
        candidate_resolved.extend(requested_resolved.iter().cloned());
    }
    for path in &event_signal_paths {
        if let Some(signal) = resolved_by_path.get(path.as_str()) {
            candidate_resolved.push(signal.clone());
        }
    }
    let mut seen = HashSet::new();
    candidate_resolved.retain(|signal| seen.insert(signal.signal_ref));
    let requested_index_by_path =
        requested_resolved
            .iter()
            .enumerate()
            .fold(HashMap::new(), |mut acc, (index, signal)| {
                acc.entry(signal.path.clone()).or_insert(index);
                acc
            });

    let candidate_mode = map_candidate_mode(args.tune_candidates);
    let window_timestamp_count =
        time_window_indices(waveform.timestamps_raw_slice(), from_raw, to_raw)
            .map(|(start_idx, end_idx_exclusive)| end_idx_exclusive.saturating_sub(start_idx))
            .unwrap_or(0);
    let estimated_work = estimate_auto_dispatch_work(
        window_timestamp_count,
        candidate_resolved.len(),
        requested_resolved.len(),
    );
    let engine_mode = select_engine_mode(
        args.tune_engine,
        resolved_event_terms.as_slice(),
        requested_resolved.len(),
        estimated_work,
    );

    let run_output = match engine_mode {
        ChangeEngineMode::Baseline => run_baseline(
            &mut waveform,
            requested_signals.as_slice(),
            requested_resolved.as_slice(),
            resolved_by_path,
            resolved_event_terms.as_slice(),
            event_signal_paths.as_slice(),
            requested_paths.as_slice(),
            &requested_index_by_path,
            candidate_resolved.as_slice(),
            from_raw,
            to_raw,
            baseline_raw,
            dump_tick,
            max_entries,
            candidate_mode,
            None,
        )?,
        ChangeEngineMode::Fused => run_fused(
            &mut waveform,
            requested_signals.as_slice(),
            requested_resolved.as_slice(),
            resolved_by_path,
            resolved_event_terms.as_slice(),
            event_signal_paths.as_slice(),
            candidate_resolved.as_slice(),
            from_raw,
            to_raw,
            baseline_raw,
            dump_tick,
            max_entries,
            candidate_mode,
        )?,
        ChangeEngineMode::EdgeFast => run_edge_fast(
            &mut waveform,
            requested_signals.as_slice(),
            requested_resolved.as_slice(),
            resolved_by_path,
            resolved_event_terms.as_slice(),
            event_signal_paths.as_slice(),
            requested_paths.as_slice(),
            &requested_index_by_path,
            candidate_resolved.as_slice(),
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
    resolved_event_terms: &[ResolvedEventTerm],
    requested_signal_count: usize,
    estimated_work: AutoDispatchWorkEstimate,
) -> ChangeEngineMode {
    match mode {
        TuneChangeEngineMode::Baseline => ChangeEngineMode::Baseline,
        TuneChangeEngineMode::Fused => ChangeEngineMode::Fused,
        TuneChangeEngineMode::EdgeFast => ChangeEngineMode::EdgeFast,
        TuneChangeEngineMode::Auto => {
            let any_tracked_only = !resolved_event_terms.is_empty()
                && resolved_event_terms
                    .iter()
                    .all(|term| matches!(term.event, ResolvedEventKind::AnyTracked));
            let edge_only = is_edge_only_terms(resolved_event_terms);

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

fn is_edge_only_terms(terms: &[ResolvedEventTerm]) -> bool {
    !terms.is_empty()
        && terms.iter().all(|term| {
            matches!(
                term.event,
                ResolvedEventKind::Posedge(_)
                    | ResolvedEventKind::Negedge(_)
                    | ResolvedEventKind::Edge(_)
            )
        })
}

#[allow(clippy::too_many_arguments)]
fn run_baseline(
    waveform: &mut Waveform,
    requested_signals: &[RequestedSignal],
    requested_resolved: &[ResolvedSignal],
    resolved_by_path: HashMap<String, ResolvedSignal>,
    resolved_event_terms: &[ResolvedEventTerm],
    event_signal_paths: &[String],
    requested_paths: &[&str],
    requested_index_by_path: &HashMap<String, usize>,
    candidate_resolved: &[ResolvedSignal],
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
        waveform.collect_change_times_with_mode(
            candidate_resolved,
            from_raw,
            to_raw,
            candidate_mode,
        )?
    };
    let candidate_schedule =
        build_candidate_schedule(waveform.timestamps_raw_slice(), &candidate_times)?;

    let mut sample_cache = SampleCache::default();
    sample_cache.sample_requested_batch(waveform, requested_resolved, baseline_raw)?;
    for path in event_signal_paths {
        sample_cache.sample(waveform, &resolved_by_path, path.as_str(), baseline_raw)?;
    }

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

        let event_eval = EventEvaluation {
            resolved_by_path: &resolved_by_path,
            requested_index_by_path,
            current_requested: current_samples.as_slice(),
            previous_requested: previous_samples.as_slice(),
            previous_timestamp,
            timestamp,
        };

        let event_fired = resolved_event_terms.iter().try_fold(false, |fired, term| {
            if fired {
                return Ok(true);
            }

            evaluate_event_term(
                waveform,
                &mut sample_cache,
                term,
                requested_paths,
                &event_eval,
            )
        })?;

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
        if !event_fired || !should_emit {
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
    waveform: &mut Waveform,
    requested_signals: &[RequestedSignal],
    requested_resolved: &[ResolvedSignal],
    resolved_by_path: HashMap<String, ResolvedSignal>,
    resolved_event_terms: &[ResolvedEventTerm],
    event_signal_paths: &[String],
    requested_paths: &[&str],
    requested_index_by_path: &HashMap<String, usize>,
    candidate_resolved: &[ResolvedSignal],
    from_raw: u64,
    to_raw: u64,
    baseline_raw: u64,
    dump_tick: ParsedTime,
    max_entries: Option<usize>,
    candidate_mode: ChangeCandidateCollectionMode,
    force_edge_fast: bool,
) -> Result<ChangeRunOutput, WavepeekError> {
    if !is_edge_only_terms(resolved_event_terms) {
        return run_baseline(
            waveform,
            requested_signals,
            requested_resolved,
            resolved_by_path,
            resolved_event_terms,
            event_signal_paths,
            requested_paths,
            requested_index_by_path,
            candidate_resolved,
            from_raw,
            to_raw,
            baseline_raw,
            dump_tick,
            max_entries,
            candidate_mode,
            None,
        );
    }

    let candidate_times = waveform.collect_change_times_with_mode(
        candidate_resolved,
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
            requested_signals,
            requested_resolved,
            resolved_by_path,
            resolved_event_terms,
            event_signal_paths,
            requested_paths,
            requested_index_by_path,
            candidate_resolved,
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
        let time_table = waveform.timestamps_raw_slice();
        candidate_times_to_indices(time_table, candidate_times.as_slice())?
    };

    let mut loaded_signal_refs = requested_resolved
        .iter()
        .map(|signal| signal.signal_ref)
        .collect::<HashSet<_>>();
    for path in event_signal_paths {
        let resolved = resolved_by_path.get(path).ok_or_else(|| {
            WavepeekError::Internal(format!(
                "internal resolved signal is missing for event path '{path}'"
            ))
        })?;
        loaded_signal_refs.insert(resolved.signal_ref);
    }
    let loaded_signal_refs = loaded_signal_refs.into_iter().collect::<Vec<_>>();
    waveform.ensure_signals_loaded(loaded_signal_refs.as_slice());

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

        let event_fired = resolved_event_terms.iter().try_fold(false, |fired, term| {
            if fired {
                return Ok(true);
            }

            evaluate_edge_event_term_fast(
                waveform,
                &mut decode_cache,
                &resolved_by_path,
                term,
                candidate_index_u32,
                previous_index,
            )
        })?;

        if timestamp <= baseline_raw {
            continue;
        }

        let mut any_requested_offset_changed = false;
        let mut delta_confirmed = false;

        for resolved in requested_resolved {
            let current_offset =
                waveform.signal_offset_at_index(resolved.signal_ref, candidate_index_u32);
            let previous_offset = previous_index
                .and_then(|idx| waveform.signal_offset_at_index(resolved.signal_ref, idx));
            if current_offset == previous_offset {
                continue;
            }

            any_requested_offset_changed = true;
            let current_bits = decode_cache.bits(waveform, resolved, candidate_index_u32)?;
            let previous_bits = previous_index
                .map(|idx| decode_cache.bits(waveform, resolved, idx))
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

        if !event_fired || !any_requested_offset_changed || !delta_confirmed {
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
                Ok(SampledSignalState {
                    path: resolved.path.clone(),
                    width: resolved.width,
                    bits: decode_cache.bits(waveform, resolved, candidate_index_u32)?,
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
    waveform: &mut Waveform,
    requested_signals: &[RequestedSignal],
    requested_resolved: &[ResolvedSignal],
    resolved_by_path: HashMap<String, ResolvedSignal>,
    resolved_event_terms: &[ResolvedEventTerm],
    event_signal_paths: &[String],
    candidate_resolved: &[ResolvedSignal],
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
    for path in event_signal_paths {
        let signal = resolved_by_path.get(path).ok_or_else(|| {
            WavepeekError::Internal(format!(
                "internal resolved signal is missing for event path '{path}'"
            ))
        })?;
        if tracked_seen.insert(signal.signal_ref) {
            tracked_resolved.push(signal.clone());
        }
    }

    let tracked_index_by_ref = tracked_resolved
        .iter()
        .enumerate()
        .map(|(index, signal)| (signal.signal_ref, index))
        .collect::<HashMap<_, _>>();
    let tracked_index_by_path = tracked_resolved
        .iter()
        .enumerate()
        .map(|(index, signal)| (signal.path.clone(), index))
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
    let candidate_tracked_indices = candidate_resolved
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
    waveform.ensure_signals_loaded(all_signal_refs.as_slice());

    let stream_candidate_times = if should_use_stream_candidates_in_fused(candidate_mode) {
        Some(waveform.collect_change_times_with_mode(
            candidate_resolved,
            from_raw,
            to_raw,
            ChangeCandidateCollectionMode::Stream,
        )?)
    } else {
        None
    };

    let time_table = waveform.timestamps_raw_slice();
    let Some((start_idx, end_idx_exclusive)) = time_window_indices(time_table, from_raw, to_raw)
    else {
        return Ok(ChangeRunOutput {
            snapshots: Vec::new(),
            truncated: false,
        });
    };

    let stream_candidate_indices = if let Some(stream_times) = stream_candidate_times {
        Some(candidate_times_to_indices(
            time_table,
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
        for signal in &tracked_resolved {
            let offset = waveform.signal_offset_at_index(signal.signal_ref, previous_idx);
            let bits = waveform.decode_signal_at_index(signal, previous_idx)?.bits;
            rolling.push(RollingSignalState { offset, bits });
        }
    }

    let mut changed_offsets = vec![false; tracked_resolved.len()];
    let mut previous_bits = vec![None; tracked_resolved.len()];
    let mut snapshots = Vec::new();
    let mut truncated = false;
    let mut stream_cursor = 0usize;

    for (idx, timestamp) in time_table
        .iter()
        .copied()
        .enumerate()
        .take(end_idx_exclusive)
        .skip(start_idx)
    {
        changed_offsets.fill(false);
        previous_bits.fill(None);

        let idx_u32 = u32::try_from(idx).map_err(|_| {
            WavepeekError::Internal("time table index exceeds u32 range".to_string())
        })?;

        let mut any_requested_offset_changed = false;
        let mut delta_confirmed = false;

        for (tracked_index, signal) in tracked_resolved.iter().enumerate() {
            let current_offset = waveform.signal_offset_at_index(signal.signal_ref, idx_u32);
            if current_offset == rolling[tracked_index].offset {
                continue;
            }

            changed_offsets[tracked_index] = true;
            let previous = rolling[tracked_index].bits.clone();
            previous_bits[tracked_index] = Some(previous.clone());

            rolling[tracked_index].offset = current_offset;
            rolling[tracked_index].bits = waveform.decode_signal_at_index(signal, idx_u32)?.bits;

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
        if !is_candidate {
            continue;
        }

        if timestamp <= baseline_raw {
            continue;
        }

        let event_fired = resolved_event_terms.iter().try_fold(false, |fired, term| {
            if fired {
                return Ok(true);
            }

            evaluate_event_term_fused(
                term,
                &tracked_index_by_path,
                changed_offsets.as_slice(),
                previous_bits.as_slice(),
                rolling.as_slice(),
                any_requested_offset_changed,
            )
        })?;
        if !event_fired {
            continue;
        }

        if !any_requested_offset_changed || !delta_confirmed {
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

fn evaluate_edge_event_term_fast(
    waveform: &Waveform,
    decode_cache: &mut IndexDecodeCache,
    resolved_by_path: &HashMap<String, ResolvedSignal>,
    term: &ResolvedEventTerm,
    current_index: u32,
    previous_index: Option<u32>,
) -> Result<bool, WavepeekError> {
    let Some(previous_index) = previous_index else {
        return Ok(false);
    };

    let path = match &term.event {
        ResolvedEventKind::Posedge(path)
        | ResolvedEventKind::Negedge(path)
        | ResolvedEventKind::Edge(path) => path,
        _ => {
            return Err(WavepeekError::Internal(
                "edge-fast engine encountered a non-edge event term".to_string(),
            ));
        }
    };
    let resolved = resolved_by_path.get(path).ok_or_else(|| {
        WavepeekError::Internal(format!(
            "internal resolved signal is missing for event path '{path}'"
        ))
    })?;

    let current_offset = waveform.signal_offset_at_index(resolved.signal_ref, current_index);
    let previous_offset = waveform.signal_offset_at_index(resolved.signal_ref, previous_index);
    if current_offset == previous_offset {
        return Ok(false);
    }

    let previous_bits = decode_cache.bits(waveform, resolved, previous_index)?;
    let current_bits = decode_cache.bits(waveform, resolved, current_index)?;
    let Some(previous_bits) = previous_bits.as_ref() else {
        return Ok(false);
    };
    let Some(current_bits) = current_bits.as_ref() else {
        return Ok(false);
    };

    let edge = classify_edge(previous_bits.as_str(), current_bits.as_str());
    match &term.event {
        ResolvedEventKind::Posedge(_) => Ok(edge.posedge),
        ResolvedEventKind::Negedge(_) => Ok(edge.negedge),
        ResolvedEventKind::Edge(_) => Ok(edge.edge()),
        _ => Ok(false),
    }
}

fn evaluate_event_term_fused(
    term: &ResolvedEventTerm,
    tracked_index_by_path: &HashMap<String, usize>,
    changed_offsets: &[bool],
    previous_bits: &[Option<Option<String>>],
    rolling: &[RollingSignalState],
    any_requested_offset_changed: bool,
) -> Result<bool, WavepeekError> {
    match &term.event {
        ResolvedEventKind::AnyTracked => Ok(any_requested_offset_changed),
        ResolvedEventKind::AnyChange(path) => {
            let tracked_index = tracked_index_by_path.get(path).copied().ok_or_else(|| {
                WavepeekError::Internal(format!(
                    "event signal '{path}' is missing from fused tracking state"
                ))
            })?;
            if !changed_offsets[tracked_index] {
                return Ok(false);
            }
            let previous = previous_bits[tracked_index].as_ref();
            let Some(previous) = previous else {
                return Ok(false);
            };
            Ok(previous != &rolling[tracked_index].bits)
        }
        ResolvedEventKind::Posedge(path)
        | ResolvedEventKind::Negedge(path)
        | ResolvedEventKind::Edge(path) => {
            let tracked_index = tracked_index_by_path.get(path).copied().ok_or_else(|| {
                WavepeekError::Internal(format!(
                    "event signal '{path}' is missing from fused tracking state"
                ))
            })?;
            if !changed_offsets[tracked_index] {
                return Ok(false);
            }
            let previous = previous_bits[tracked_index]
                .as_ref()
                .and_then(|value| value.as_ref());
            let current = rolling[tracked_index].bits.as_ref();
            let Some(previous) = previous else {
                return Ok(false);
            };
            let Some(current) = current else {
                return Ok(false);
            };

            let edge = classify_edge(previous.as_str(), current.as_str());
            match &term.event {
                ResolvedEventKind::Posedge(_) => Ok(edge.posedge),
                ResolvedEventKind::Negedge(_) => Ok(edge.negedge),
                ResolvedEventKind::Edge(_) => Ok(edge.edge()),
                _ => Ok(false),
            }
        }
    }
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

fn collect_event_signal_paths(terms: &[ResolvedEventTerm]) -> Vec<String> {
    let mut paths = Vec::new();

    for term in terms {
        let maybe_path = match &term.event {
            ResolvedEventKind::AnyTracked => None,
            ResolvedEventKind::AnyChange(path)
            | ResolvedEventKind::Posedge(path)
            | ResolvedEventKind::Negedge(path)
            | ResolvedEventKind::Edge(path) => Some(path.clone()),
        };

        if let Some(path) = maybe_path
            && !paths.contains(&path)
        {
            paths.push(path);
        }
    }

    paths
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

fn resolve_event_term(
    term: &EventTerm,
    scope: Option<&str>,
) -> Result<ResolvedEventTerm, WavepeekError> {
    let event = match &term.event {
        EventKind::AnyTracked => ResolvedEventKind::AnyTracked,
        EventKind::AnyChange(name) => {
            let path = validate_event_name(name.as_str())?;
            ResolvedEventKind::AnyChange(resolve_token_to_path(path, scope)?)
        }
        EventKind::Posedge(name) => {
            let path = validate_event_name(name.as_str())?;
            ResolvedEventKind::Posedge(resolve_token_to_path(path, scope)?)
        }
        EventKind::Negedge(name) => {
            let path = validate_event_name(name.as_str())?;
            ResolvedEventKind::Negedge(resolve_token_to_path(path, scope)?)
        }
        EventKind::Edge(name) => {
            let path = validate_event_name(name.as_str())?;
            ResolvedEventKind::Edge(resolve_token_to_path(path, scope)?)
        }
    };

    Ok(ResolvedEventTerm {
        event,
        iff_expr: term.iff_expr.clone(),
    })
}

fn validate_event_name(name: &str) -> Result<&str, WavepeekError> {
    let name = name.trim();
    if name.is_empty() {
        return Err(WavepeekError::Args(
            "invalid --when expression: missing signal name. See 'wavepeek change --help'."
                .to_string(),
        ));
    }
    Ok(name)
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

struct EventEvaluation<'a> {
    resolved_by_path: &'a HashMap<String, ResolvedSignal>,
    requested_index_by_path: &'a HashMap<String, usize>,
    current_requested: &'a [SampledSignalState],
    previous_requested: &'a [SampledSignalState],
    previous_timestamp: Option<u64>,
    timestamp: u64,
}

fn evaluate_event_term(
    waveform: &mut Waveform,
    cache: &mut SampleCache,
    term: &ResolvedEventTerm,
    tracked_paths: &[&str],
    eval: &EventEvaluation<'_>,
) -> Result<bool, WavepeekError> {
    match &term.event {
        ResolvedEventKind::AnyTracked => {
            for path in tracked_paths {
                let Some(&index) = eval.requested_index_by_path.get(*path) else {
                    return Err(WavepeekError::Internal(format!(
                        "tracked signal '{path}' is missing from requested sample map"
                    )));
                };
                if eval.current_requested[index].bits != eval.previous_requested[index].bits {
                    return Ok(true);
                }
            }
            Ok(false)
        }
        ResolvedEventKind::AnyChange(path) => {
            let (previous_bits, current_bits) =
                sample_event_bits(waveform, cache, path.as_str(), eval)?;
            Ok(current_bits != previous_bits)
        }
        ResolvedEventKind::Posedge(path) => {
            let edge = signal_edge(waveform, cache, path.as_str(), eval)?;
            Ok(edge.posedge)
        }
        ResolvedEventKind::Negedge(path) => {
            let edge = signal_edge(waveform, cache, path.as_str(), eval)?;
            Ok(edge.negedge)
        }
        ResolvedEventKind::Edge(path) => {
            let edge = signal_edge(waveform, cache, path.as_str(), eval)?;
            Ok(edge.edge())
        }
    }
}

fn sample_event_bits(
    waveform: &mut Waveform,
    cache: &mut SampleCache,
    path: &str,
    eval: &EventEvaluation<'_>,
) -> Result<(Option<String>, Option<String>), WavepeekError> {
    if let Some(&index) = eval.requested_index_by_path.get(path) {
        return Ok((
            eval.previous_requested[index].bits.clone(),
            eval.current_requested[index].bits.clone(),
        ));
    }

    let current = cache.sample(waveform, eval.resolved_by_path, path, eval.timestamp)?;
    let previous = eval
        .previous_timestamp
        .map(|previous_timestamp| {
            cache.sample(waveform, eval.resolved_by_path, path, previous_timestamp)
        })
        .transpose()?;

    Ok((previous.and_then(|sample| sample.bits), current.bits))
}

fn signal_edge(
    waveform: &mut Waveform,
    cache: &mut SampleCache,
    path: &str,
    eval: &EventEvaluation<'_>,
) -> Result<EdgeClassification, WavepeekError> {
    let Some(_) = eval.previous_timestamp else {
        return Ok(EdgeClassification {
            posedge: false,
            negedge: false,
        });
    };

    let (previous_bits, current_bits) = sample_event_bits(waveform, cache, path, eval)?;
    let Some(previous_bits) = previous_bits else {
        return Ok(EdgeClassification {
            posedge: false,
            negedge: false,
        });
    };
    let Some(current_bits) = current_bits else {
        return Ok(EdgeClassification {
            posedge: false,
            negedge: false,
        });
    };

    Ok(classify_edge(previous_bits.as_str(), current_bits.as_str()))
}

fn parse_bound_time(
    token: &str,
    arg_name: &str,
    dump_tick: ParsedTime,
    dump_tick_zs: u128,
    metadata: &crate::waveform::WaveformMetadata,
) -> Result<u64, WavepeekError> {
    if token.chars().all(|ch| ch.is_ascii_digit()) {
        return Err(WavepeekError::Args(format!(
            "time token '{token}' requires units. See 'wavepeek change --help'."
        )));
    }

    let parsed = parse_time_token(token).ok_or_else(|| {
        WavepeekError::Args(format!(
            "invalid time token '{token}': expected <integer><unit> (for example 10ns). See 'wavepeek change --help'."
        ))
    })?;
    let parsed_zs = as_zeptoseconds(parsed).ok_or_else(|| {
        WavepeekError::Args(format!(
            "time '{token}' is too large to process safely. See 'wavepeek change --help'."
        ))
    })?;

    let dump_start = parse_time_token(metadata.time_start.as_str()).ok_or_else(|| {
        WavepeekError::Internal(format!(
            "waveform metadata contains invalid time_start '{}': expected <integer><unit>",
            metadata.time_start
        ))
    })?;
    let dump_start_zs = as_zeptoseconds(dump_start).ok_or_else(|| {
        WavepeekError::Internal(
            "waveform metadata time_start overflowed during conversion".to_string(),
        )
    })?;

    let dump_end = parse_time_token(metadata.time_end.as_str()).ok_or_else(|| {
        WavepeekError::Internal(format!(
            "waveform metadata contains invalid time_end '{}': expected <integer><unit>",
            metadata.time_end
        ))
    })?;
    let dump_end_zs = as_zeptoseconds(dump_end).ok_or_else(|| {
        WavepeekError::Internal(
            "waveform metadata time_end overflowed during conversion".to_string(),
        )
    })?;

    if parsed_zs < dump_start_zs || parsed_zs > dump_end_zs {
        return Err(WavepeekError::Args(format!(
            "time '{}' for {} is outside dump bounds [{}, {}]. See 'wavepeek change --help'.",
            token, arg_name, metadata.time_start, metadata.time_end
        )));
    }

    if parsed_zs % dump_tick_zs != 0 {
        return Err(WavepeekError::Args(format!(
            "time '{token}' cannot be represented exactly in dump precision '{}'.",
            format_raw_timestamp(1, dump_tick)?
        )));
    }

    let raw = parsed_zs / dump_tick_zs;
    u64::try_from(raw).map_err(|_| {
        WavepeekError::Args(format!(
            "time '{token}' exceeds supported raw timestamp range. See 'wavepeek change --help'."
        ))
    })
}

#[cfg(test)]
mod tests {
    use super::{
        AutoDispatchWorkEstimate, ChangeEngineMode, ResolvedEventKind, ResolvedEventTerm,
        select_engine_mode,
    };
    use crate::cli::change::TuneChangeEngineMode;

    fn term(event: ResolvedEventKind) -> ResolvedEventTerm {
        ResolvedEventTerm {
            event,
            iff_expr: None,
        }
    }

    fn select_auto_mode_for_profile(
        terms: &[ResolvedEventTerm],
        requested_signal_count: usize,
        fused_work: usize,
        edge_work: usize,
    ) -> ChangeEngineMode {
        select_engine_mode(
            TuneChangeEngineMode::Auto,
            terms,
            requested_signal_count,
            AutoDispatchWorkEstimate {
                fused_work,
                edge_work,
            },
        )
    }

    #[test]
    fn auto_engine_mode_uses_fused_for_wide_any_tracked_only() {
        let terms = vec![term(ResolvedEventKind::AnyTracked)];

        let selected = select_auto_mode_for_profile(terms.as_slice(), 64, 500_000, 0);

        assert_eq!(selected, ChangeEngineMode::Fused);
    }

    #[test]
    fn auto_engine_mode_uses_fused_for_mid_any_tracked_only() {
        let terms = vec![term(ResolvedEventKind::AnyTracked)];

        let selected = select_auto_mode_for_profile(terms.as_slice(), 10, 250_000, 0);

        assert_eq!(selected, ChangeEngineMode::Fused);
    }

    #[test]
    fn auto_engine_mode_uses_fused_for_wide_selective_edge_events() {
        let terms = vec![term(ResolvedEventKind::Posedge("top.clk".to_string()))];

        let selected = select_auto_mode_for_profile(terms.as_slice(), 128, 0, 1_500_000);

        assert_eq!(selected, ChangeEngineMode::Fused);
    }

    #[test]
    fn auto_engine_mode_uses_edge_fast_for_ultra_wide_selective_edge_events() {
        let terms = vec![term(ResolvedEventKind::Posedge("top.clk".to_string()))];

        let selected = select_auto_mode_for_profile(terms.as_slice(), 128, 0, 2_500_000);

        assert_eq!(selected, ChangeEngineMode::EdgeFast);
    }

    #[test]
    fn auto_engine_mode_uses_fused_for_mid_edge_only_terms() {
        let terms = vec![term(ResolvedEventKind::Posedge("top.clk".to_string()))];

        let selected = select_auto_mode_for_profile(terms.as_slice(), 10, 0, 1_500_000);

        assert_eq!(selected, ChangeEngineMode::Fused);
    }

    #[test]
    fn auto_engine_mode_keeps_baseline_for_narrow_selective_edge_events() {
        let terms = vec![term(ResolvedEventKind::Posedge("top.clk".to_string()))];

        let selected = select_auto_mode_for_profile(terms.as_slice(), 16, 2_000, 2_000);

        assert_eq!(selected, ChangeEngineMode::Baseline);
    }

    #[test]
    fn auto_engine_mode_keeps_baseline_for_low_signal_count_edge_only_terms() {
        let terms = vec![term(ResolvedEventKind::Posedge("top.clk".to_string()))];

        let selected = select_auto_mode_for_profile(terms.as_slice(), 1, 0, 2_000_000);

        assert_eq!(selected, ChangeEngineMode::Baseline);
    }

    #[test]
    fn auto_engine_mode_keeps_baseline_for_low_work_any_tracked_only() {
        let terms = vec![term(ResolvedEventKind::AnyTracked)];

        let selected = select_auto_mode_for_profile(terms.as_slice(), 10, 2_000, 2_000);

        assert_eq!(selected, ChangeEngineMode::Baseline);
    }

    #[test]
    fn auto_engine_mode_keeps_baseline_below_fused_threshold() {
        let terms = vec![term(ResolvedEventKind::AnyTracked)];

        let selected = select_auto_mode_for_profile(terms.as_slice(), 31, 2_000, 2_000);

        assert_eq!(selected, ChangeEngineMode::Baseline);
    }

    #[test]
    fn auto_engine_mode_uses_fused_at_threshold_for_any_tracked() {
        let terms = vec![term(ResolvedEventKind::AnyTracked)];

        let selected = select_auto_mode_for_profile(terms.as_slice(), 32, 2_000, 2_000);

        assert_eq!(selected, ChangeEngineMode::Fused);
    }
}
