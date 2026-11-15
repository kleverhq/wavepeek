use std::collections::{HashMap, HashSet};
use std::fs;
use std::ops::ControlFlow;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

use crate::cli::extract::GenericArgs;
use crate::cli::limits::LimitArg;
use crate::debug_trace::DebugTrace;
use crate::diagnostic::{Diagnostic, WarningDiagnosticCode};
use crate::engine::expr_runtime::{
    ScopedExprHost, SharedWaveform, bind_waveform_logical_expr, eval_bound_logical_truth,
    eval_with_waveform_errors, event_candidate_handles, event_expr_is_edge_only,
    event_expr_matches, event_iff_handles, expr_diagnostic, open_shared_waveform,
    referenced_signal_handles,
};
use crate::engine::signal_projection::{ProjectedSignal, resolve_projected_signal};
use crate::engine::time::{
    DumpTimeContext, TimeValidationError, format_raw_timestamp, parse_dump_time_context,
    validate_time_token_to_raw,
};
use crate::engine::value_format::format_verilog_literal;
use crate::engine::{CommandData, CommandName, CommandResult, HumanRenderOptions, ResultSummary};
use crate::error::WavepeekError;
use crate::expr::{
    BoundEventExpr, BoundLogicalExpr, EventEvalFrame, ExpressionHost, SampledValue, SignalHandle,
    bind_event_expr_ast, parse_event_expr_ast,
};
use crate::waveform::display_signal_path;
use crate::waveform::expr_host::{RuntimeExpressionHost, WaveformExprHost};
use crate::waveform::expr_query::{ExpressionQuery, ExpressionQueryTick};

const DEFAULT_SOURCE_NAME: &str = "transfer";
const SOURCE_KIND: &str = "extract.generic.sources";
const EDGE_ONLY_ON_MESSAGE: &str = "extract generic requires --on with only edge event terms (posedge, negedge, or edge); wildcard, plain-signal, and mixed triggers are not supported";

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ExtractPayloadValue {
    #[serde(skip_serializing)]
    pub display: String,
    pub path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub relative_path: Option<String>,
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
    relative_path: Option<String>,
    selected: ProjectedSignal,
    handle: SignalHandle,
}

#[derive(Debug)]
struct BoundExtractSource {
    declaration_index: usize,
    name: String,
    on: String,
    when: String,
    payload: Vec<PayloadSignal>,
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
    pub(crate) include_relative_paths: bool,
}

struct ExtractEmitContext<'a> {
    waveform: &'a SharedWaveform,
    sources: &'a [BoundExtractSource],
    event_groups: &'a [EventGroup],
    query: &'a ExpressionQuery,
    from_raw: u64,
    to_raw: u64,
    dump_start_raw: u64,
    dump_end_raw: u64,
    dump_tick: crate::engine::time::ParsedTime,
    max_entries: Option<usize>,
    profile_timing: bool,
}

#[derive(Debug)]
struct EventGroup {
    source_indices: Vec<usize>,
}

#[derive(Debug)]
pub(crate) struct ExtractCommandOutcome {
    pub(crate) source_count: usize,
    pub(crate) diagnostics: Vec<Diagnostic>,
    pub(crate) summary: ResultSummary,
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
    scope: Option<String>,
}

impl<W: std::io::Write> ExtractRowSink for JsonlExtractSink<'_, W> {
    fn start(&mut self) -> Result<(), WavepeekError> {
        match self.scope.as_deref() {
            Some(scope) => self.writer.begin_scope(scope),
            None => self.writer.begin(),
        }
    }

    fn emit(&mut self, row: ExtractGenericRow) -> Result<(), WavepeekError> {
        self.writer.data(&row)
    }
}

#[derive(Debug, Deserialize)]
struct SourceFile {
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
    let scope = args.scope.clone();
    let summary_only = args.summary;
    let mut sink = CollectingExtractSink::default();
    let outcome = run_with_sink(args, &mut sink)?;

    Ok(CommandResult {
        command: CommandName::ExtractGeneric,
        output_mode,
        human_options: HumanRenderOptions {
            scope_tree: false,
            signals_abs,
        },
        scope,
        summary_only,
        data: CommandData::ExtractGeneric(ExtractGenericData {
            source_count: outcome.source_count,
            rows: sink.rows,
        }),
        summary: Some(outcome.summary),
        diagnostics: outcome.diagnostics,
    })
}

pub fn run_jsonl<W: std::io::Write>(
    args: GenericArgs,
    writer: &mut crate::output::JsonlWriter<W>,
) -> Result<(), WavepeekError> {
    writer.suppress_data(args.summary);
    let outcome = {
        let mut sink = JsonlExtractSink {
            writer,
            scope: args.scope.clone(),
        };
        run_with_sink(args, &mut sink)?
    };

    for diagnostic in &outcome.diagnostics {
        writer.diagnostic(diagnostic)?;
    }
    writer.end_summary(&outcome.summary)
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
            include_relative_paths: args.json || args.jsonl,
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

    run_open_plan_with_sink(args, plan, waveform, debug, max_entries, sink)
}

pub(crate) fn run_plan_with_waveform_sink<S: ExtractRowSink + ?Sized>(
    args: ExtractRunArgs,
    plan: ExtractPlan,
    waveform: SharedWaveform,
    debug: DebugTrace,
    sink: &mut S,
) -> Result<ExtractCommandOutcome, WavepeekError> {
    let max_entries = max_entries(&args.max)?;
    run_open_plan_with_sink(args, plan, waveform, debug, max_entries, sink)
}

fn run_open_plan_with_sink<S: ExtractRowSink + ?Sized>(
    args: ExtractRunArgs,
    plan: ExtractPlan,
    waveform: SharedWaveform,
    debug: DebugTrace,
    max_entries: Option<usize>,
    sink: &mut S,
) -> Result<ExtractCommandOutcome, WavepeekError> {
    let source_count = plan.source_count();
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
    debug.event("time.parse.done", || {
        serde_json::json!({
            "dump_start_raw": dump_start_raw,
            "dump_end_raw": dump_end_raw,
            "from_raw": from_raw,
            "to_raw": to_raw,
        })
    });

    let host = WaveformExprHost::from_shared_scoped(waveform.clone(), args.scope.as_deref());
    let bound_sources = bind_extract_sources(
        &waveform,
        &host,
        args.scope.as_deref(),
        args.include_relative_paths,
        plan.sources,
    )?;
    let event_groups = build_event_groups(&bound_sources);
    let mut drivers = Vec::new();
    let mut operands = Vec::new();
    for source in &bound_sources {
        drivers.extend(event_candidate_handles(&source.bound_event));
        operands.extend(event_iff_handles(&source.bound_event));
        operands.extend(referenced_signal_handles(&source.bound_when));
        operands.extend(source.payload.iter().map(|payload| payload.handle));
    }
    let mut seen = HashSet::new();
    drivers.retain(|handle| seen.insert(*handle));
    let query = ExpressionQuery::new(&host, &drivers, &operands)?;
    debug.event("extract.bind.done", || {
        serde_json::json!({
            "sources": source_count,
            "event_groups": event_groups.len(),
            "drivers": drivers.len(),
        })
    });

    sink.start()?;
    debug.event("extract.emit.start", || {
        serde_json::json!({
            "sources": bound_sources.len(),
            "event_groups": event_groups.len(),
        })
    });
    let emit_context = ExtractEmitContext {
        waveform: &waveform,
        sources: &bound_sources,
        event_groups: &event_groups,
        query: &query,
        from_raw,
        to_raw,
        dump_start_raw,
        dump_end_raw,
        dump_tick,
        max_entries,
        profile_timing: debug.is_enabled(),
    };
    let stats = emit_rows(emit_context, sink)?;
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

    let mut diagnostics = Vec::new();
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
        summary: ResultSummary::from_run(stats.emitted, max_entries, stats.truncated),
    })
}

pub(crate) fn max_entries(max: &LimitArg) -> Result<Option<usize>, WavepeekError> {
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
            let normalized = crate::engine::signal_token(&token);
            let trimmed = normalized.as_str();
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

fn bind_extract_sources(
    waveform: &SharedWaveform,
    host: &WaveformExprHost,
    scope: Option<&str>,
    include_relative_paths: bool,
    sources: Vec<ExtractSource>,
) -> Result<Vec<BoundExtractSource>, WavepeekError> {
    if let Some(scope) = scope {
        waveform.borrow().signals_in_scope(scope)?;
    }

    let mut bound_sources = Vec::with_capacity(sources.len());
    for source in sources {
        let ast = parse_event_expr_ast(&source.on)
            .map_err(|diagnostic| expr_diagnostic(&source.on, diagnostic))?;
        let bound_event = bind_event_expr_ast(&ast, &ScopedExprHost::new(host, scope))
            .map_err(|diagnostic| expr_diagnostic(&source.on, diagnostic))?;
        if !event_expr_is_edge_only(&bound_event) {
            return Err(WavepeekError::Args(EDGE_ONLY_ON_MESSAGE.to_string()));
        }
        let bound_when = bind_waveform_logical_expr(host, scope, source.when.as_str())?;
        let payload = resolve_payload_signals(
            waveform,
            host,
            scope,
            include_relative_paths,
            source.payload.as_slice(),
        )?;
        bound_sources.push(BoundExtractSource {
            declaration_index: source.declaration_index,
            name: source.name,
            on: source.on,
            when: source.when,
            payload,
            bound_event,
            bound_when,
        });
    }

    Ok(bound_sources)
}

fn resolve_payload_signals(
    waveform: &SharedWaveform,
    host: &WaveformExprHost,
    scope: Option<&str>,
    include_relative_paths: bool,
    payload: &[String],
) -> Result<Vec<PayloadSignal>, WavepeekError> {
    let selected = {
        let waveform = waveform.borrow();
        payload
            .iter()
            .map(|token| resolve_projected_signal(&waveform, token, scope))
            .collect::<Result<Vec<_>, _>>()?
    };
    selected
        .into_iter()
        .map(|selected| {
            let display = display_signal_path(selected.path.as_str(), scope).to_string();
            let relative_path = if include_relative_paths && scope.is_some() {
                Some(display.clone())
            } else {
                None
            };
            let handle = host
                .resolve_signal(&selected.source.path)
                .map_err(|diagnostic| expr_diagnostic(&selected.path, diagnostic))?;
            Ok(PayloadSignal {
                display,
                relative_path,
                selected,
                handle,
            })
        })
        .collect()
}

fn build_event_groups(sources: &[BoundExtractSource]) -> Vec<EventGroup> {
    let mut groups: Vec<EventGroup> = Vec::new();
    let mut groups_by_event = HashMap::new();
    for (source_index, source) in sources.iter().enumerate() {
        debug_assert_eq!(source.declaration_index, source_index);
        if let Some(&group_index) = groups_by_event.get(&source.on) {
            let group: &mut EventGroup = &mut groups[group_index];
            group.source_indices.push(source_index);
        } else {
            groups_by_event.insert(source.on.clone(), groups.len());
            groups.push(EventGroup {
                source_indices: vec![source_index],
            });
        }
    }
    groups
}

fn duration_ns(duration: Duration) -> u64 {
    u64::try_from(duration.as_nanos()).unwrap_or(u64::MAX)
}

fn emit_rows<S: ExtractRowSink + ?Sized>(
    context: ExtractEmitContext<'_>,
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
    let mut matched_source_indices = Vec::new();

    context.waveform.borrow_mut().query_expressions(
        context.query,
        context.from_raw..=context.to_raw,
        false,
        |host, tick| {
            let ExpressionQueryTick::Candidate(timestamp) = tick else {
                return Ok(ControlFlow::Continue(()));
            };
            candidate_count += 1;
            matched_source_indices.clear();
            let previous_timestamp = timestamp
                .checked_sub(1)
                .filter(|time| *time >= context.dump_start_raw);

            let mut matching_group_count = 0usize;
            for group in context.event_groups {
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
                    host,
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
                return Ok(ControlFlow::Continue(()));
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
                return Ok(ControlFlow::Continue(()));
            };

            for source_index in &matched_source_indices {
                let source = &context.sources[*source_index];
                debug_assert_eq!(source.declaration_index, *source_index);
                let started_at = context.profile_timing.then(Instant::now);
                let predicate_matches = eval_bound_logical_truth(
                    source.when.as_str(),
                    &source.bound_when,
                    host,
                    sample_timestamp,
                )?;
                if let Some(started_at) = started_at {
                    predicate_eval_time += started_at.elapsed();
                }
                if !predicate_matches {
                    continue;
                }
                predicate_true += 1;
                let started_at = context.profile_timing.then(Instant::now);
                let row = build_row(source, timestamp, sample_timestamp, context.dump_tick, host)?;
                if let Some(limit) = context.max_entries
                    && emitted == limit
                {
                    truncated = true;
                    return Ok(ControlFlow::Break(()));
                }
                sink.emit(row)?;
                if let Some(started_at) = started_at {
                    build_emit_time += started_at.elapsed();
                }
                emitted += 1;
            }
            Ok(ControlFlow::Continue(()))
        },
    )?;

    Ok(ExtractRunStats {
        candidate_times: candidate_count,
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
    host: &dyn RuntimeExpressionHost,
) -> Result<ExtractGenericRow, WavepeekError> {
    let payload = source
        .payload
        .iter()
        .map(|requested| build_payload_value(requested, host, sample_timestamp))
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
    host: &dyn RuntimeExpressionHost,
    timestamp: u64,
) -> Result<ExtractPayloadValue, WavepeekError> {
    let bits = if requested.selected.width() == 0 {
        // Raw-event payloads retain the existing zero-width representation.
        eval_with_waveform_errors(&requested.selected.path, host, || {
            host.event_occurred(requested.handle, timestamp)
        })?;
        Some(String::new())
    } else {
        match eval_with_waveform_errors(&requested.selected.path, host, || {
            host.sample_value(requested.handle, timestamp)
        })? {
            SampledValue::Integral { bits, .. } => bits,
            _ => {
                return Err(WavepeekError::Signal(format!(
                    "signal '{}' has unsupported non-bit-vector encoding",
                    requested.selected.path
                )));
            }
        }
    };
    let bits = bits.ok_or_else(|| {
        WavepeekError::Signal(format!(
            "signal '{}' has no value at or before requested time",
            requested.selected.path
        ))
    })?;
    let bits = bits
        .chars()
        .map(crate::logic::four_state)
        .collect::<String>();
    Ok(ExtractPayloadValue {
        display: requested.display.clone(),
        path: requested.selected.path.clone(),
        relative_path: requested.relative_path.clone(),
        value: format_verilog_literal(requested.selected.width(), &bits),
    })
}

pub(crate) fn parse_bound_time(
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
    use super::{ExtractGenericRow, ExtractRowSink, build_plan, normalize_payload, run_with_sink};
    use crate::cli::extract::GenericArgs;
    use crate::cli::limits::LimitArg;
    use crate::error::WavepeekError;

    #[test]
    fn extract_sink_failure_stops_the_query_and_preserves_its_type() {
        struct FailingSink(usize);
        impl ExtractRowSink for FailingSink {
            fn emit(&mut self, _: ExtractGenericRow) -> Result<(), WavepeekError> {
                self.0 += 1;
                Err(WavepeekError::File("sentinel extract sink failure".into()))
            }
        }
        let fixture = tempfile::NamedTempFile::with_suffix(".vcd").unwrap();
        std::fs::write(fixture.path(), "$timescale 1ns $end\n$scope module top $end\n$var wire 1 c clk $end\n$upscope $end\n$enddefinitions $end\n#0\n0c\n#5\n1c\n#10\n0c\n#15\n1c\n").unwrap();
        let mut sink = FailingSink(0);
        let error = run_with_sink(
            GenericArgs {
                waves: fixture.path().into(),
                source: None,
                from: None,
                to: None,
                scope: Some("top".into()),
                name: None,
                on: Some("posedge clk".into()),
                when: Some("1".into()),
                payload: Some(vec!["clk".into()]),
                max: LimitArg::Unlimited,
                abs: false,
                json: false,
                summary: false,
                jsonl: true,
            },
            &mut sink,
        )
        .unwrap_err();
        assert_eq!(sink.0, 1);
        assert!(
            matches!(error, WavepeekError::File(ref message) if message == "sentinel extract sink failure")
        );
    }

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
            summary: false,
            jsonl: false,
        })
        .expect("plan should build");
        assert_eq!(plan.sources[0].name, "transfer");
        assert_eq!(plan.sources[0].payload, vec!["data", "last"]);
    }

    #[test]
    fn payload_normalization_rejects_empty_and_preserves_duplicates() {
        assert!(normalize_payload(vec![" ".to_string()]).is_err());
        let payload = normalize_payload(vec![" data ".to_string(), "data".to_string()])
            .expect("trimmed payload should parse");
        assert_eq!(payload, vec!["data", "data"]);
    }
}
