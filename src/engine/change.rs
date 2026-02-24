use std::collections::HashMap;

use serde::Serialize;

use crate::cli::change::ChangeArgs;
use crate::engine::at::{
    ParsedTime, as_zeptoseconds, ensure_non_zero_dump_tick, format_raw_timestamp,
    format_verilog_literal, parse_time_token,
};
use crate::engine::{CommandData, CommandName, CommandResult, HumanRenderOptions};
use crate::error::WavepeekError;
use crate::expr::{EventKind, EventTerm, parse_event_expr};
use crate::waveform::{
    EdgeClassification, SampledSignalState, Waveform, classify_edge,
    should_emit_delta_and_update_baseline,
};

const EMPTY_WARNING: &str = "no signal changes found in selected time range";

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

#[derive(Default)]
struct SampleCache {
    entries: HashMap<(String, u64), SampledSignalState>,
}

impl SampleCache {
    fn sample(
        &mut self,
        waveform: &mut Waveform,
        path: &str,
        raw_time: u64,
    ) -> Result<SampledSignalState, WavepeekError> {
        let key = (path.to_string(), raw_time);
        if let Some(existing) = self.entries.get(&key) {
            return Ok(existing.clone());
        }

        let sampled = waveform
            .sample_signals_at_time_optional(&[path.to_string()], raw_time)?
            .pop()
            .ok_or_else(|| {
                WavepeekError::Internal(
                    "waveform backend returned no samples for requested signal".to_string(),
                )
            })?;

        self.entries.insert(key, sampled.clone());
        Ok(sampled)
    }
}

pub fn run(args: ChangeArgs) -> Result<CommandResult, WavepeekError> {
    if args.max == 0 {
        return Err(WavepeekError::Args(
            "--max must be greater than 0.".to_string(),
        ));
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
    let timestamps = waveform.timestamps_raw();
    let requested_paths = requested_signals
        .iter()
        .map(|signal| signal.path.as_str())
        .collect::<Vec<_>>();

    let mut sample_cache = SampleCache::default();

    let mut previous_sampled_values = requested_paths
        .iter()
        .map(|path| {
            sample_cache
                .sample(&mut waveform, path, baseline_raw)
                .map(|sample| sample.bits)
        })
        .collect::<Result<Vec<_>, _>>()?;

    for path in &event_signal_paths {
        sample_cache.sample(&mut waveform, path.as_str(), baseline_raw)?;
    }

    let mut snapshots = Vec::new();
    let mut truncated = false;
    for (index, timestamp) in timestamps.iter().enumerate() {
        if *timestamp < from_raw || *timestamp > to_raw {
            continue;
        }

        let previous_timestamp = if index == 0 {
            None
        } else {
            Some(timestamps[index - 1])
        };

        let event_fired = resolved_event_terms.iter().try_fold(false, |fired, term| {
            if fired {
                return Ok(true);
            }

            evaluate_event_term(
                &mut waveform,
                &mut sample_cache,
                term,
                requested_paths.as_slice(),
                previous_timestamp,
                *timestamp,
            )
        })?;

        if *timestamp <= baseline_raw {
            continue;
        }

        let current_samples = requested_paths
            .iter()
            .map(|path| sample_cache.sample(&mut waveform, path, *timestamp))
            .collect::<Result<Vec<_>, _>>()?;
        let current_values = current_samples
            .iter()
            .map(|sample| sample.bits.clone())
            .collect::<Vec<_>>();

        let should_emit =
            should_emit_delta_and_update_baseline(&mut previous_sampled_values, &current_values);
        if !event_fired || !should_emit {
            continue;
        }

        if snapshots.len() == args.max {
            truncated = true;
            break;
        }

        let signals = requested_signals
            .iter()
            .zip(current_samples)
            .map(|(requested, sampled)| {
                let bits = sampled.bits.ok_or_else(|| {
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

        snapshots.push(ChangeSnapshot {
            time: format_raw_timestamp(*timestamp, dump_tick)?,
            signals,
        });
    }

    let mut warnings = Vec::new();
    if snapshots.is_empty() {
        warnings.push(EMPTY_WARNING.to_string());
    }

    if truncated {
        warnings.push(format!(
            "truncated output to {} entries (use --max to increase limit)",
            args.max
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

fn evaluate_event_term(
    waveform: &mut Waveform,
    cache: &mut SampleCache,
    term: &ResolvedEventTerm,
    tracked_paths: &[&str],
    previous_timestamp: Option<u64>,
    timestamp: u64,
) -> Result<bool, WavepeekError> {
    match &term.event {
        ResolvedEventKind::AnyTracked => {
            for path in tracked_paths {
                if signal_changed(waveform, cache, path, previous_timestamp, timestamp)? {
                    return Ok(true);
                }
            }
            Ok(false)
        }
        ResolvedEventKind::AnyChange(path) => signal_changed(
            waveform,
            cache,
            path.as_str(),
            previous_timestamp,
            timestamp,
        ),
        ResolvedEventKind::Posedge(path) => {
            let edge = signal_edge(
                waveform,
                cache,
                path.as_str(),
                previous_timestamp,
                timestamp,
            )?;
            Ok(edge.posedge)
        }
        ResolvedEventKind::Negedge(path) => {
            let edge = signal_edge(
                waveform,
                cache,
                path.as_str(),
                previous_timestamp,
                timestamp,
            )?;
            Ok(edge.negedge)
        }
        ResolvedEventKind::Edge(path) => {
            let edge = signal_edge(
                waveform,
                cache,
                path.as_str(),
                previous_timestamp,
                timestamp,
            )?;
            Ok(edge.edge())
        }
    }
}

fn signal_changed(
    waveform: &mut Waveform,
    cache: &mut SampleCache,
    path: &str,
    previous_timestamp: Option<u64>,
    timestamp: u64,
) -> Result<bool, WavepeekError> {
    let current = cache.sample(waveform, path, timestamp)?;
    let previous = previous_timestamp
        .map(|previous_timestamp| cache.sample(waveform, path, previous_timestamp))
        .transpose()?;

    let previous_bits = previous.and_then(|sample| sample.bits);
    Ok(current.bits != previous_bits)
}

fn signal_edge(
    waveform: &mut Waveform,
    cache: &mut SampleCache,
    path: &str,
    previous_timestamp: Option<u64>,
    timestamp: u64,
) -> Result<EdgeClassification, WavepeekError> {
    let Some(previous_timestamp) = previous_timestamp else {
        return Ok(EdgeClassification {
            posedge: false,
            negedge: false,
        });
    };

    let previous = cache.sample(waveform, path, previous_timestamp)?;
    let current = cache.sample(waveform, path, timestamp)?;
    let Some(previous_bits) = previous.bits else {
        return Ok(EdgeClassification {
            posedge: false,
            negedge: false,
        });
    };
    let Some(current_bits) = current.bits else {
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
