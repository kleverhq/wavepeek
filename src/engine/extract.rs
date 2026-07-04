use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;
use std::rc::Rc;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

use crate::cli::extract::GenericArgs;
use crate::cli::limits::LimitArg;
use crate::contract::schema::INPUT_SCHEMA_URL;
use crate::debug_trace::DebugTrace;
use crate::diagnostic::{Diagnostic, WarningDiagnosticCode};
use crate::engine::expr_runtime::{
    SharedWaveform, bind_waveform_event_expr, bind_waveform_logical_expr,
    candidate_sources_for_handles, eval_bound_logical_truth, event_candidate_handles,
    event_expr_is_edge_only, event_expr_matches, event_iff_handles, open_shared_waveform,
    referenced_signal_handles,
};
use crate::engine::time::{
    DumpTimeContext, TimeValidationError, format_raw_timestamp, parse_dump_time_context,
    validate_time_token_to_raw,
};
use crate::engine::value_format::format_verilog_literal;
use crate::engine::{CommandData, CommandName, CommandResult, HumanRenderOptions};
use crate::error::WavepeekError;
use crate::expr::{BoundEventExpr, BoundLogicalExpr, EventEvalFrame};
use crate::waveform::{
    ChangeCandidateCollectionMode, ExprResolvedSignal, ResolvedSignal, SampledSignalState,
    SignalId, expr_host::WaveformExprHost,
};

const DEFAULT_SOURCE_NAME: &str = "transfer";
const SOURCE_KIND: &str = "extract.generic.sources";
const EDGE_ONLY_ON_MESSAGE: &str = "extract generic requires --on with only edge event terms (posedge, negedge, or edge); wildcard, plain-signal, and mixed triggers are not supported";

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ExtractPayloadValue {
    #[serde(skip_serializing)]
    pub display: String,
    pub path: String,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ExtractGenericRow {
    pub time: String,
    pub sample_time: String,
    pub source: String,
    pub payload: Vec<ExtractPayloadValue>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ExtractGenericData {
    #[serde(skip_serializing)]
    pub source_count: usize,
    pub rows: Vec<ExtractGenericRow>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtractPlan {
    sources: Vec<ExtractSource>,
}

impl ExtractPlan {
    pub(crate) fn new(sources: Vec<ExtractSource>) -> Self {
        Self { sources }
    }

    pub(crate) fn source_count(&self) -> usize {
        self.sources.len()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtractSource {
    declaration_index: usize,
    name: String,
    on: String,
    when: String,
    payload: Vec<String>,
}

impl ExtractSource {
    pub(crate) fn new(
        declaration_index: usize,
        name: impl Into<String>,
        on: impl Into<String>,
        when: impl Into<String>,
        payload: Vec<String>,
    ) -> Self {
        Self {
            declaration_index,
            name: name.into(),
            on: on.into(),
            when: when.into(),
            payload,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PayloadSignal {
    display: String,
    path: String,
    resolved: ResolvedSignal,
}

#[derive(Debug)]
struct BoundExtractSource {
    declaration_index: usize,
    name: String,
    on: String,
    when: String,
    payload: Vec<PayloadSignal>,
    host: WaveformExprHost,
    bound_event: BoundEventExpr,
    bound_when: BoundLogicalExpr,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ExtractRunStats {
    candidate_times: usize,
    event_checks: usize,
    event_matches: usize,
    predicate_true: usize,
    skipped_no_sample_time: usize,
    pub(crate) emitted: usize,
    pub(crate) truncated: bool,
    event_match_ns: u64,
    predicate_eval_ns: u64,
    build_emit_ns: u64,
}

#[derive(Debug, Clone)]
pub(crate) struct ExtractRunArgs {
    pub(crate) command: CommandName,
    pub(crate) help_command: &'static str,
    pub(crate) waves: PathBuf,
    pub(crate) from: Option<String>,
    pub(crate) to: Option<String>,
    pub(crate) scope: Option<String>,
    pub(crate) max: LimitArg,
}

struct ExtractEmitContext<'a> {
    waveform: &'a SharedWaveform,
    sources: &'a [BoundExtractSource],
    event_groups: &'a [EventGroup],
    dump_start_raw: u64,
    dump_end_raw: u64,
    dump_tick: crate::engine::time::ParsedTime,
    max_entries: Option<usize>,
    profile_timing: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct EventGroupKey {
    on: String,
    candidate_ids: Vec<SignalId>,
}

#[derive(Debug)]
struct EventGroup {
    source_indices: Vec<usize>,
    candidate_sources: Vec<ExprResolvedSignal>,
    candidate_key: Vec<SignalId>,
}

#[derive(Debug)]
struct EventGroupCandidateTimes {
    group_index: usize,
    times: Rc<Vec<u64>>,
    cursor: usize,
}

#[derive(Debug)]
pub(crate) struct ExtractCommandOutcome {
    pub(crate) source_count: usize,
    pub(crate) diagnostics: Vec<Diagnostic>,
    pub(crate) stats: ExtractRunStats,
}

pub(crate) trait ExtractRowSink {
    fn start(&mut self) -> Result<(), WavepeekError> {
        Ok(())
    }

    fn emit(&mut self, row: ExtractGenericRow) -> Result<(), WavepeekError>;
}

#[derive(Default)]
struct CollectingExtractSink {
    rows: Vec<ExtractGenericRow>,
}

impl ExtractRowSink for CollectingExtractSink {
    fn emit(&mut self, row: ExtractGenericRow) -> Result<(), WavepeekError> {
        self.rows.push(row);
        Ok(())
    }
}

struct JsonlExtractSink<'a, W: std::io::Write> {
    writer: &'a mut crate::output::JsonlWriter<W>,
}

impl<W: std::io::Write> ExtractRowSink for JsonlExtractSink<'_, W> {
    fn start(&mut self) -> Result<(), WavepeekError> {
        self.writer.begin()
    }

    fn emit(&mut self, row: ExtractGenericRow) -> Result<(), WavepeekError> {
        self.writer.item(&row)
    }
}

#[derive(Debug, Deserialize)]
struct SourceFile {
    #[serde(rename = "$schema")]
    schema: String,
    kind: String,
    sources: Vec<SourceFileSource>,
}

#[derive(Debug, Deserialize)]
struct SourceFileSource {
    name: String,
    on: String,
    when: String,
    payload: Vec<String>,
}

pub fn run(args: GenericArgs) -> Result<CommandResult, WavepeekError> {
    let output_mode = crate::output_mode::OutputMode::from_json_flags(args.json, args.jsonl);
    let signals_abs = args.abs;
    let mut sink = CollectingExtractSink::default();
    let outcome = run_with_sink(args, &mut sink)?;

    Ok(CommandResult {
        command: CommandName::ExtractGeneric,
        output_mode,
        human_options: HumanRenderOptions {
            scope_tree: false,
            signals_abs,
        },
        data: CommandData::ExtractGeneric(ExtractGenericData {
            source_count: outcome.source_count,
            rows: sink.rows,
        }),
        diagnostics: outcome.diagnostics,
    })
}

pub fn run_jsonl<W: std::io::Write>(
    args: GenericArgs,
    writer: &mut crate::output::JsonlWriter<W>,
) -> Result<(), WavepeekError> {
    let outcome = {
        let mut sink = JsonlExtractSink { writer };
        run_with_sink(args, &mut sink)?
    };

    for diagnostic in &outcome.diagnostics {
        writer.diagnostic(diagnostic)?;
    }
    writer.end(outcome.stats.truncated)
}

fn run_with_sink<S: ExtractRowSink + ?Sized>(
    args: GenericArgs,
    sink: &mut S,
) -> Result<ExtractCommandOutcome, WavepeekError> {
    let plan = build_plan(&args)?;
    run_plan_with_sink(
        ExtractRunArgs {
            command: CommandName::ExtractGeneric,
            help_command: "wavepeek extract generic",
            waves: args.waves,
            from: args.from,
            to: args.to,
            scope: args.scope,
            max: args.max,
        },
        plan,
        sink,
    )
}

pub(crate) fn run_plan_with_sink<S: ExtractRowSink + ?Sized>(
    args: ExtractRunArgs,
    plan: ExtractPlan,
    sink: &mut S,
) -> Result<ExtractCommandOutcome, WavepeekError> {
    let max_entries = max_entries(&args.max)?;
    let mut diagnostics = Vec::new();
    if args.max.is_unlimited() {
        diagnostics.push(Diagnostic::warning(
            WarningDiagnosticCode::LimitDisabled,
            "limit disabled: --max=unlimited",
        ));
    }

    let source_count = plan.source_count();

    let debug = DebugTrace::for_command(args.command);
    debug.event("backend.open.start", || serde_json::json!({}));
    let waveform = open_shared_waveform(args.waves.as_path())?;
    {
        let waveform_ref = waveform.borrow();
        debug.event("backend.open.done", || {
            serde_json::json!({
                "backend": waveform_ref.backend_name(),
                "format": waveform_ref.format_name(),
            })
        });
    }
    let metadata = waveform.borrow().metadata()?;
    debug.event("metadata.load.done", || serde_json::json!({}));
    let dump_time = parse_dump_time_context(&metadata)?;
    let dump_tick = dump_time.dump_tick;
    let dump_start_raw =
        u64::try_from(dump_time.dump_start_zs / dump_time.dump_tick_zs).map_err(|_| {
            WavepeekError::Internal("dump start timestamp exceeds supported range".to_string())
        })?;
    let dump_end_raw =
        u64::try_from(dump_time.dump_end_zs / dump_time.dump_tick_zs).map_err(|_| {
            WavepeekError::Internal("dump end timestamp exceeds supported range".to_string())
        })?;

    let from_raw = match args.from.as_deref() {
        Some(token) => parse_bound_time(token, "--from", dump_time, &metadata, args.help_command)?,
        None => dump_start_raw,
    };
    let to_raw = match args.to.as_deref() {
        Some(token) => parse_bound_time(token, "--to", dump_time, &metadata, args.help_command)?,
        None => dump_end_raw,
    };
    if from_raw > to_raw {
        return Err(WavepeekError::Args(format!(
            "--from must be less than or equal to --to. See '{} --help'.",
            args.help_command
        )));
    }
    let (indexed_timestamps, window_timestamps) = {
        let waveform_ref = waveform.borrow();
        waveform_ref
            .indexed_timestamps()
            .map(|timestamps| {
                (
                    Some(timestamps.len()),
                    Some(count_timestamps_in_range(timestamps, from_raw, to_raw)),
                )
            })
            .unwrap_or((None, None))
    };
    debug.event("time.parse.done", || {
        serde_json::json!({
            "dump_start_raw": dump_start_raw,
            "dump_end_raw": dump_end_raw,
            "from_raw": from_raw,
            "to_raw": to_raw,
            "indexed_timestamps": indexed_timestamps,
            "window_timestamps": window_timestamps,
        })
    });

    let bound_sources = bind_extract_sources(&waveform, args.scope.as_deref(), plan.sources)?;
    let event_groups = build_event_groups(&bound_sources)?;
    let event_group_candidate_sources = event_groups
        .iter()
        .map(|group| group.candidate_sources.len())
        .sum::<usize>();
    debug.event("extract.bind.done", || {
        serde_json::json!({
            "sources": source_count,
            "event_groups": event_groups.len(),
            "event_group_candidate_sources": event_group_candidate_sources,
        })
    });

    let group_candidate_times =
        collect_event_group_candidate_times(&waveform, &event_groups, from_raw, to_raw)?;
    let candidate_times = if debug.is_enabled() {
        count_merged_candidate_times(&group_candidate_times)
    } else {
        0
    };
    debug.event("candidate.collect.done", || {
        serde_json::json!({
            "times": candidate_times,
            "event_groups": event_groups.len(),
            "group_times": group_candidate_times
                .iter()
                .map(|group| group.times.len())
                .collect::<Vec<_>>(),
        })
    });

    if max_entries.is_none() {
        let preload_from_raw = from_raw
            .checked_sub(1)
            .filter(|value| *value >= dump_start_raw)
            .unwrap_or(from_raw);
        preload_extract_value_changes(
            &waveform,
            &bound_sources,
            &event_groups,
            preload_from_raw,
            to_raw,
        )?;
        debug.event("value.preload.done", || {
            serde_json::json!({
                "from_raw": preload_from_raw,
                "to_raw": to_raw,
                "backend_stats": waveform.borrow().debug_stats(),
            })
        });
    } else {
        debug.event("value.preload.skipped", || {
            serde_json::json!({
                "reason": "bounded_max",
                "max_entries": max_entries,
            })
        });
    }

    sink.start()?;
    debug.event("extract.emit.start", || {
        serde_json::json!({
            "candidate_times": candidate_times,
            "sources": bound_sources.len(),
            "event_groups": event_groups.len(),
        })
    });
    let emit_context = ExtractEmitContext {
        waveform: &waveform,
        sources: &bound_sources,
        event_groups: &event_groups,
        dump_start_raw,
        dump_end_raw,
        dump_tick,
        max_entries,
        profile_timing: debug.is_enabled(),
    };
    let stats = emit_rows(emit_context, group_candidate_times, candidate_times, sink)?;
    debug.event("extract.emit.done", || {
        serde_json::json!({
            "candidate_times": stats.candidate_times,
            "event_checks": stats.event_checks,
            "event_matches": stats.event_matches,
            "predicate_true": stats.predicate_true,
            "skipped_no_sample_time": stats.skipped_no_sample_time,
            "emitted": stats.emitted,
            "truncated": stats.truncated,
            "event_match_ns": stats.event_match_ns,
            "predicate_eval_ns": stats.predicate_eval_ns,
            "build_emit_ns": stats.build_emit_ns,
            "backend_stats": waveform.borrow().debug_stats(),
        })
    });

    if stats.emitted == 0 {
        diagnostics.push(Diagnostic::warning(
            WarningDiagnosticCode::EmptyResult,
            "no extract rows found in selected time range",
        ));
    }
    if let Some(max_entries) = max_entries
        && stats.truncated
    {
        diagnostics.push(Diagnostic::warning(
            WarningDiagnosticCode::OutputTruncated,
            format!("truncated output to {max_entries} entries (use --max to increase limit)"),
        ));
    }

    Ok(ExtractCommandOutcome {
        source_count,
        diagnostics,
        stats,
    })
}

fn max_entries(max: &LimitArg) -> Result<Option<usize>, WavepeekError> {
    match max {
        LimitArg::Numeric(0) => Err(WavepeekError::Args(
            "--max must be greater than 0.".to_string(),
        )),
        LimitArg::Numeric(value) => Ok(Some(*value)),
        LimitArg::Unlimited => Ok(None),
    }
}

fn build_plan(args: &GenericArgs) -> Result<ExtractPlan, WavepeekError> {
    if let Some(path) = args.source.as_ref() {
        if args.name.is_some() || args.on.is_some() || args.when.is_some() || args.payload.is_some()
        {
            return Err(WavepeekError::Args(
                "--source cannot be combined with --name, --on, --when, or --payload. See 'wavepeek extract generic --help'.".to_string(),
            ));
        }
        return plan_from_source_file(path);
    }

    let on = args.on.clone().ok_or_else(|| {
        WavepeekError::Args(
            "--on is required unless --source is used. See 'wavepeek extract generic --help'."
                .to_string(),
        )
    })?;
    let when = args.when.clone().ok_or_else(|| {
        WavepeekError::Args(
            "--when is required unless --source is used. See 'wavepeek extract generic --help'."
                .to_string(),
        )
    })?;
    let payload = args.payload.clone().ok_or_else(|| {
        WavepeekError::Args(
            "--payload is required unless --source is used. See 'wavepeek extract generic --help'."
                .to_string(),
        )
    })?;
    let payload = normalize_payload(payload)?;
    require_unique_payloads(&payload)?;

    Ok(ExtractPlan::new(vec![ExtractSource::new(
        0,
        args.name
            .clone()
            .unwrap_or_else(|| DEFAULT_SOURCE_NAME.to_string()),
        on,
        when,
        payload,
    )]))
}

fn plan_from_source_file(path: &std::path::Path) -> Result<ExtractPlan, WavepeekError> {
    let contents = fs::read_to_string(path).map_err(|error| {
        WavepeekError::File(format!(
            "failed to read extract source file '{}': {error}",
            path.display()
        ))
    })?;
    let input: SourceFile = serde_json::from_str(&contents).map_err(|error| {
        WavepeekError::Args(format!(
            "invalid extract source JSON '{}': {error}",
            path.display()
        ))
    })?;

    if input.schema != INPUT_SCHEMA_URL {
        return Err(WavepeekError::Args(format!(
            "extract source file '{}' uses unsupported $schema {}; expected {}",
            path.display(),
            input.schema,
            INPUT_SCHEMA_URL
        )));
    }
    if input.kind != SOURCE_KIND {
        return Err(WavepeekError::Args(format!(
            "extract source file '{}' has kind {}; expected {}",
            path.display(),
            input.kind,
            SOURCE_KIND
        )));
    }
    if input.sources.is_empty() {
        return Err(WavepeekError::Args(format!(
            "extract source file '{}' must contain at least one source",
            path.display()
        )));
    }

    let mut names = HashSet::new();
    let mut sources = Vec::with_capacity(input.sources.len());
    for (declaration_index, source) in input.sources.into_iter().enumerate() {
        if !names.insert(source.name.clone()) {
            return Err(WavepeekError::Args(format!(
                "extract source file '{}' contains duplicate source name '{}'",
                path.display(),
                source.name
            )));
        }
        let payload = normalize_payload(source.payload)?;
        require_unique_payloads(&payload)?;
        sources.push(ExtractSource::new(
            declaration_index,
            source.name,
            source.on,
            source.when,
            payload,
        ));
    }

    Ok(ExtractPlan::new(sources))
}

fn normalize_payload(payload: Vec<String>) -> Result<Vec<String>, WavepeekError> {
    if payload.is_empty() {
        return Err(WavepeekError::Args(
            "payload must contain at least one signal. See 'wavepeek extract generic --help'."
                .to_string(),
        ));
    }
    payload
        .into_iter()
        .map(|token| {
            let trimmed = token.trim();
            if trimmed.is_empty() {
                Err(WavepeekError::Args(
                    "payload signal names must not be empty. See 'wavepeek extract generic --help'."
                        .to_string(),
                ))
            } else {
                Ok(trimmed.to_string())
            }
        })
        .collect()
}

fn require_unique_payloads(payload: &[String]) -> Result<(), WavepeekError> {
    let mut seen = HashSet::new();
    for signal in payload {
        if !seen.insert(signal.as_str()) {
            return Err(WavepeekError::Args(format!(
                "payload contains duplicate signal '{signal}'. See 'wavepeek extract generic --help'."
            )));
        }
    }
    Ok(())
}

fn bind_extract_sources(
    waveform: &SharedWaveform,
    scope: Option<&str>,
    sources: Vec<ExtractSource>,
) -> Result<Vec<BoundExtractSource>, WavepeekError> {
    if let Some(scope) = scope {
        waveform.borrow().signals_in_scope(scope)?;
    }

    let mut bound_sources = Vec::with_capacity(sources.len());
    for source in sources {
        let (host, bound_event) =
            bind_waveform_event_expr(waveform.clone(), scope, source.on.as_str())?;
        if !event_expr_is_edge_only(&bound_event) {
            return Err(WavepeekError::Args(EDGE_ONLY_ON_MESSAGE.to_string()));
        }
        let iff_signal_handles = event_iff_handles(&bound_event);
        let iff_sources = candidate_sources_for_handles(&host, iff_signal_handles.as_slice())?;
        waveform
            .borrow()
            .validate_expr_values_supported(iff_sources.as_slice())?;
        let bound_when = bind_waveform_logical_expr(&host, scope, source.when.as_str())?;
        let eval_signal_handles = referenced_signal_handles(&bound_when);
        let eval_sources = candidate_sources_for_handles(&host, eval_signal_handles.as_slice())?;
        waveform
            .borrow()
            .validate_expr_values_supported(eval_sources.as_slice())?;
        let payload = resolve_payload_signals(waveform, scope, source.payload.as_slice())?;
        bound_sources.push(BoundExtractSource {
            declaration_index: source.declaration_index,
            name: source.name,
            on: source.on,
            when: source.when,
            payload,
            host,
            bound_event,
            bound_when,
        });
    }

    Ok(bound_sources)
}

fn resolve_payload_signals(
    waveform: &SharedWaveform,
    scope: Option<&str>,
    payload: &[String],
) -> Result<Vec<PayloadSignal>, WavepeekError> {
    let mut display_names = Vec::with_capacity(payload.len());
    let mut canonical_paths = Vec::with_capacity(payload.len());
    for token in payload {
        if scope.is_some() && token.contains('.') {
            return Err(WavepeekError::Args(format!(
                "payload signal '{token}' must be relative when --scope is set. See 'wavepeek extract generic --help'."
            )));
        }
        let path = match scope {
            Some(scope) => format!("{scope}.{token}"),
            None => token.clone(),
        };
        display_names.push(token.clone());
        canonical_paths.push(path);
    }

    let expr_resolved = waveform
        .borrow()
        .resolve_expr_signals(canonical_paths.as_slice())?;
    waveform
        .borrow()
        .validate_expr_values_supported(expr_resolved.as_slice())?;
    let resolved = waveform
        .borrow()
        .resolve_signals(canonical_paths.as_slice())?;
    Ok(display_names
        .into_iter()
        .zip(resolved)
        .map(|(display, resolved)| PayloadSignal {
            display,
            path: resolved.path.clone(),
            resolved,
        })
        .collect())
}

fn build_event_groups(sources: &[BoundExtractSource]) -> Result<Vec<EventGroup>, WavepeekError> {
    let mut groups: Vec<EventGroup> = Vec::new();
    let mut groups_by_key: HashMap<EventGroupKey, usize> = HashMap::new();

    for (source_index, source) in sources.iter().enumerate() {
        debug_assert_eq!(source.declaration_index, source_index);
        let candidate_sources = candidate_sources_for_handles(
            &source.host,
            event_candidate_handles(&source.bound_event).as_slice(),
        )?;
        let candidate_key = candidate_key_for_sources(&candidate_sources);
        let key = EventGroupKey {
            on: source.on.clone(),
            candidate_ids: candidate_key.clone(),
        };

        if let Some(group_index) = groups_by_key.get(&key).copied() {
            groups[group_index].source_indices.push(source_index);
            continue;
        }

        let group_index = groups.len();
        groups.push(EventGroup {
            source_indices: vec![source_index],
            candidate_sources,
            candidate_key,
        });
        groups_by_key.insert(key, group_index);
    }

    Ok(groups)
}

fn candidate_key_for_sources(candidate_sources: &[ExprResolvedSignal]) -> Vec<SignalId> {
    let mut candidate_ids = candidate_sources
        .iter()
        .map(|candidate| candidate.id)
        .collect::<Vec<_>>();
    candidate_ids.sort_unstable();
    candidate_ids.dedup();
    candidate_ids
}

fn collect_event_group_candidate_times(
    waveform: &SharedWaveform,
    event_groups: &[EventGroup],
    from_raw: u64,
    to_raw: u64,
) -> Result<Vec<EventGroupCandidateTimes>, WavepeekError> {
    let mut times_by_candidate_key: HashMap<Vec<SignalId>, Rc<Vec<u64>>> = HashMap::new();
    let mut group_candidate_times = Vec::with_capacity(event_groups.len());

    for (group_index, group) in event_groups.iter().enumerate() {
        let times = if let Some(times) = times_by_candidate_key.get(&group.candidate_key) {
            Rc::clone(times)
        } else {
            let times = Rc::new(
                waveform
                    .borrow_mut()
                    .collect_expr_candidate_times_with_mode(
                        group.candidate_sources.as_slice(),
                        from_raw,
                        to_raw,
                        ChangeCandidateCollectionMode::Auto,
                    )?,
            );
            times_by_candidate_key.insert(group.candidate_key.clone(), Rc::clone(&times));
            times
        };
        group_candidate_times.push(EventGroupCandidateTimes {
            group_index,
            times,
            cursor: 0,
        });
    }

    Ok(group_candidate_times)
}

fn preload_extract_value_changes(
    waveform: &SharedWaveform,
    sources: &[BoundExtractSource],
    event_groups: &[EventGroup],
    from_raw: u64,
    to_raw: u64,
) -> Result<(), WavepeekError> {
    let mut expr_sources = Vec::new();
    let mut seen_expr = HashSet::new();
    for group in event_groups {
        extend_unique_expr_sources(&mut expr_sources, &mut seen_expr, &group.candidate_sources);
    }
    for source in sources {
        let iff_handles = event_iff_handles(&source.bound_event);
        let iff_sources = candidate_sources_for_handles(&source.host, iff_handles.as_slice())?;
        extend_unique_expr_sources(&mut expr_sources, &mut seen_expr, &iff_sources);

        let when_handles = referenced_signal_handles(&source.bound_when);
        let when_sources = candidate_sources_for_handles(&source.host, when_handles.as_slice())?;
        extend_unique_expr_sources(&mut expr_sources, &mut seen_expr, &when_sources);
    }

    let mut payload_sources = Vec::new();
    let mut seen_payload = HashSet::new();
    for source in sources {
        for payload in &source.payload {
            if seen_payload.insert(payload.resolved.id) {
                payload_sources.push(payload.resolved.clone());
            }
        }
    }

    let mut waveform = waveform.borrow_mut();
    waveform.preload_expr_value_changes(expr_sources.as_slice(), from_raw, to_raw)?;
    waveform.preload_resolved_value_changes(payload_sources.as_slice(), from_raw, to_raw)
}

fn extend_unique_expr_sources(
    target: &mut Vec<ExprResolvedSignal>,
    seen: &mut HashSet<SignalId>,
    sources: &[ExprResolvedSignal],
) {
    for source in sources {
        if seen.insert(source.id) {
            target.push(source.clone());
        }
    }
}

fn count_merged_candidate_times(group_candidate_times: &[EventGroupCandidateTimes]) -> usize {
    let mut cursors = vec![0usize; group_candidate_times.len()];
    let mut count = 0usize;

    loop {
        let Some(timestamp) = group_candidate_times
            .iter()
            .enumerate()
            .filter_map(|(group_index, group)| group.times.get(cursors[group_index]).copied())
            .min()
        else {
            return count;
        };

        count += 1;
        for (group_index, group) in group_candidate_times.iter().enumerate() {
            while group
                .times
                .get(cursors[group_index])
                .is_some_and(|candidate| *candidate == timestamp)
            {
                cursors[group_index] += 1;
            }
        }
    }
}

fn next_active_event_groups(
    group_candidate_times: &mut [EventGroupCandidateTimes],
    active_groups: &mut Vec<usize>,
) -> Option<u64> {
    active_groups.clear();
    let timestamp = group_candidate_times
        .iter()
        .filter_map(|group| group.times.get(group.cursor).copied())
        .min()?;

    for group in group_candidate_times {
        while group
            .times
            .get(group.cursor)
            .is_some_and(|candidate| *candidate == timestamp)
        {
            if active_groups.last().copied() != Some(group.group_index) {
                active_groups.push(group.group_index);
            }
            group.cursor += 1;
        }
    }

    Some(timestamp)
}

fn count_timestamps_in_range(timestamps: &[u64], from_raw: u64, to_raw: u64) -> usize {
    let start_idx = timestamps.partition_point(|timestamp| *timestamp < from_raw);
    let end_idx = timestamps.partition_point(|timestamp| *timestamp <= to_raw);
    end_idx.saturating_sub(start_idx)
}

fn duration_ns(duration: Duration) -> u64 {
    u64::try_from(duration.as_nanos()).unwrap_or(u64::MAX)
}

fn emit_rows<S: ExtractRowSink + ?Sized>(
    context: ExtractEmitContext<'_>,
    mut group_candidate_times: Vec<EventGroupCandidateTimes>,
    candidate_times_total: usize,
    sink: &mut S,
) -> Result<ExtractRunStats, WavepeekError> {
    let mut candidate_count = 0usize;
    let mut event_checks = 0usize;
    let mut event_matches_count = 0usize;
    let mut predicate_true = 0usize;
    let mut skipped_no_sample_time = 0usize;
    let mut emitted = 0usize;
    let mut truncated = false;
    let mut event_match_time = Duration::ZERO;
    let mut predicate_eval_time = Duration::ZERO;
    let mut build_emit_time = Duration::ZERO;
    let mut active_groups = Vec::new();
    let mut matched_source_indices = Vec::new();

    'timestamps: while let Some(timestamp) =
        next_active_event_groups(&mut group_candidate_times, &mut active_groups)
    {
        candidate_count += 1;
        matched_source_indices.clear();
        let previous_timestamp = context.waveform.borrow().previous_sample_time(timestamp);

        let mut matching_group_count = 0usize;
        for group_index in &active_groups {
            let group = &context.event_groups[*group_index];
            let representative = &context.sources[group.source_indices[0]];
            let frame = EventEvalFrame {
                timestamp,
                previous_timestamp,
                tracked_signals: &[],
            };
            event_checks += 1;
            let started_at = context.profile_timing.then(Instant::now);
            let event_matches_row = event_expr_matches(
                representative.on.as_str(),
                &representative.bound_event,
                &representative.host,
                &frame,
            )?;
            if let Some(started_at) = started_at {
                event_match_time += started_at.elapsed();
            }
            if event_matches_row {
                event_matches_count += 1;
                matching_group_count += 1;
                matched_source_indices.extend(group.source_indices.iter().copied());
            }
        }

        if matched_source_indices.is_empty() {
            continue;
        }
        if matching_group_count > 1 {
            matched_source_indices.sort_unstable_by_key(|source_index| {
                context.sources[*source_index].declaration_index
            });
        }

        let Some(sample_timestamp) = timestamp
            .checked_sub(1)
            .filter(|value| *value >= context.dump_start_raw && *value <= context.dump_end_raw)
        else {
            skipped_no_sample_time += matched_source_indices.len();
            continue;
        };

        for source_index in &matched_source_indices {
            let source = &context.sources[*source_index];
            debug_assert_eq!(source.declaration_index, *source_index);
            let started_at = context.profile_timing.then(Instant::now);
            let predicate_matches = eval_bound_logical_truth(
                source.when.as_str(),
                &source.bound_when,
                &source.host,
                sample_timestamp,
            )?;
            if let Some(started_at) = started_at {
                predicate_eval_time += started_at.elapsed();
            }
            if !predicate_matches {
                continue;
            }
            predicate_true += 1;
            if let Some(limit) = context.max_entries
                && emitted == limit
            {
                truncated = true;
                break 'timestamps;
            }
            let started_at = context.profile_timing.then(Instant::now);
            let row = build_row(
                source,
                timestamp,
                sample_timestamp,
                context.dump_tick,
                context.waveform,
            )?;
            sink.emit(row)?;
            if let Some(started_at) = started_at {
                build_emit_time += started_at.elapsed();
            }
            emitted += 1;
        }
    }

    Ok(ExtractRunStats {
        candidate_times: if context.profile_timing {
            candidate_times_total
        } else {
            candidate_count
        },
        event_checks,
        event_matches: event_matches_count,
        predicate_true,
        skipped_no_sample_time,
        emitted,
        truncated,
        event_match_ns: duration_ns(event_match_time),
        predicate_eval_ns: duration_ns(predicate_eval_time),
        build_emit_ns: duration_ns(build_emit_time),
    })
}

fn build_row(
    source: &BoundExtractSource,
    timestamp: u64,
    sample_timestamp: u64,
    dump_tick: crate::engine::time::ParsedTime,
    waveform: &SharedWaveform,
) -> Result<ExtractGenericRow, WavepeekError> {
    let resolved = source
        .payload
        .iter()
        .map(|payload| payload.resolved.clone())
        .collect::<Vec<_>>();
    let samples = waveform
        .borrow_mut()
        .sample_resolved_optional(resolved.as_slice(), sample_timestamp)?;
    let payload = source
        .payload
        .iter()
        .zip(samples.iter())
        .map(|(requested, sampled)| build_payload_value(requested, sampled))
        .collect::<Result<Vec<_>, _>>()?;

    Ok(ExtractGenericRow {
        time: format_raw_timestamp(timestamp, dump_tick)?,
        sample_time: format_raw_timestamp(sample_timestamp, dump_tick)?,
        source: source.name.clone(),
        payload,
    })
}

fn build_payload_value(
    requested: &PayloadSignal,
    sampled: &SampledSignalState,
) -> Result<ExtractPayloadValue, WavepeekError> {
    let bits = sampled.bits.as_ref().ok_or_else(|| {
        WavepeekError::Signal(format!(
            "signal '{}' has no value at or before requested time",
            requested.path
        ))
    })?;
    Ok(ExtractPayloadValue {
        display: requested.display.clone(),
        path: requested.path.clone(),
        value: format_verilog_literal(sampled.width, bits.as_str()),
    })
}

fn parse_bound_time(
    token: &str,
    arg_name: &str,
    dump_time: DumpTimeContext,
    metadata: &crate::waveform::WaveformMetadata,
    help_command: &str,
) -> Result<u64, WavepeekError> {
    match validate_time_token_to_raw(token, dump_time, true) {
        Ok(raw) => Ok(raw),
        Err(TimeValidationError::RequiresUnits) => Err(WavepeekError::Args(format!(
            "time token '{token}' requires units. See '{help_command} --help'."
        ))),
        Err(TimeValidationError::InvalidToken) => Err(WavepeekError::Args(format!(
            "invalid time token '{token}': expected <integer><unit> (for example 10ns). See '{help_command} --help'."
        ))),
        Err(TimeValidationError::TooLarge) => Err(WavepeekError::Args(format!(
            "time '{token}' is too large to process safely. See '{help_command} --help'."
        ))),
        Err(TimeValidationError::OutOfBounds) => Err(WavepeekError::Args(format!(
            "time '{}' for {} is outside dump bounds [{}, {}]. See '{} --help'.",
            token, arg_name, metadata.time_start, metadata.time_end, help_command
        ))),
        Err(TimeValidationError::NotAligned) => {
            let dump_precision = format_raw_timestamp(1, dump_time.dump_tick)?;
            Err(WavepeekError::Args(format!(
                "time '{token}' cannot be represented exactly in dump precision '{}'. See '{} --help'.",
                dump_precision, help_command
            )))
        }
        Err(TimeValidationError::RawOutOfRange) => Err(WavepeekError::Args(format!(
            "time '{token}' exceeds supported raw timestamp range. See '{help_command} --help'."
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::{build_plan, normalize_payload, require_unique_payloads};
    use crate::cli::extract::GenericArgs;
    use crate::cli::limits::LimitArg;

    #[test]
    fn cli_plan_defaults_source_name_and_payload_order() {
        let plan = build_plan(&GenericArgs {
            waves: "dump.vcd".into(),
            source: None,
            from: None,
            to: None,
            scope: None,
            name: None,
            on: Some("posedge clk".to_string()),
            when: Some("valid".to_string()),
            payload: Some(vec!["data".to_string(), "last".to_string()]),
            max: LimitArg::Numeric(50),
            abs: false,
            json: false,
            jsonl: false,
        })
        .expect("plan should build");
        assert_eq!(plan.sources[0].name, "transfer");
        assert_eq!(plan.sources[0].payload, vec!["data", "last"]);
    }

    #[test]
    fn payload_normalization_rejects_empty_and_duplicate_names() {
        assert!(normalize_payload(vec![" ".to_string()]).is_err());
        let payload = normalize_payload(vec![" data ".to_string(), "data".to_string()])
            .expect("trimmed payload should parse");
        assert_eq!(payload, vec!["data", "data"]);
        assert!(require_unique_payloads(&payload).is_err());
    }
}
