use std::collections::HashSet;
use std::fs;

use serde::{Deserialize, Serialize};

use crate::cli::extract::GenericArgs;
use crate::cli::limits::LimitArg;
use crate::contract::schema::INPUT_SCHEMA_URL;
use crate::debug_trace::DebugTrace;
use crate::diagnostic::{Diagnostic, WarningDiagnosticCode};
use crate::engine::expr_runtime::{
    SharedWaveform, bind_waveform_event_expr, bind_waveform_logical_expr,
    candidate_sources_for_handles, eval_bound_logical_truth, event_candidate_handles,
    event_expr_is_edge_only, event_expr_matches, open_shared_waveform, referenced_signal_handles,
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
    expr_host::WaveformExprHost,
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtractSource {
    declaration_index: usize,
    name: String,
    on: String,
    when: String,
    payload: Vec<String>,
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
struct ExtractRunStats {
    emitted: usize,
    truncated: bool,
}

struct ExtractEmitContext<'a> {
    waveform: &'a SharedWaveform,
    sources: &'a [BoundExtractSource],
    dump_start_raw: u64,
    dump_end_raw: u64,
    dump_tick: crate::engine::time::ParsedTime,
    max_entries: Option<usize>,
}

#[derive(Debug)]
struct ExtractCommandOutcome {
    source_count: usize,
    diagnostics: Vec<Diagnostic>,
    stats: ExtractRunStats,
}

trait ExtractRowSink {
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
    let max_entries = max_entries(&args.max)?;
    let mut diagnostics = Vec::new();
    if args.max.is_unlimited() {
        diagnostics.push(Diagnostic::warning(
            WarningDiagnosticCode::LimitDisabled,
            "limit disabled: --max=unlimited",
        ));
    }

    let plan = build_plan(&args)?;
    let source_count = plan.sources.len();

    let debug = DebugTrace::for_command(CommandName::ExtractGeneric);
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
        Some(token) => parse_bound_time(token, "--from", dump_time, &metadata)?,
        None => dump_start_raw,
    };
    let to_raw = match args.to.as_deref() {
        Some(token) => parse_bound_time(token, "--to", dump_time, &metadata)?,
        None => dump_end_raw,
    };
    if from_raw > to_raw {
        return Err(WavepeekError::Args(
            "--from must be less than or equal to --to".to_string(),
        ));
    }

    let bound_sources = bind_extract_sources(&waveform, args.scope.as_deref(), plan.sources)?;
    let candidate_sources = candidate_sources_for_bound_sources(&bound_sources)?;
    debug.event("extract.bind.done", || {
        serde_json::json!({
            "sources": source_count,
            "candidate_sources": candidate_sources.len(),
        })
    });

    let candidate_times = waveform
        .borrow_mut()
        .collect_expr_candidate_times_with_mode(
            candidate_sources.as_slice(),
            from_raw,
            to_raw,
            ChangeCandidateCollectionMode::Auto,
        )?;
    debug.event(
        "candidate.collect.done",
        || serde_json::json!({"times": candidate_times.len()}),
    );

    sink.start()?;
    let emit_context = ExtractEmitContext {
        waveform: &waveform,
        sources: &bound_sources,
        dump_start_raw,
        dump_end_raw,
        dump_tick,
        max_entries,
    };
    let stats = emit_rows(emit_context, candidate_times, sink)?;

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

    Ok(ExtractPlan {
        sources: vec![ExtractSource {
            declaration_index: 0,
            name: args
                .name
                .clone()
                .unwrap_or_else(|| DEFAULT_SOURCE_NAME.to_string()),
            on,
            when,
            payload,
        }],
    })
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
        sources.push(ExtractSource {
            declaration_index,
            name: source.name,
            on: source.on,
            when: source.when,
            payload,
        });
    }

    Ok(ExtractPlan { sources })
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

fn candidate_sources_for_bound_sources(
    sources: &[BoundExtractSource],
) -> Result<Vec<ExprResolvedSignal>, WavepeekError> {
    let mut seen = HashSet::new();
    let mut result = Vec::new();
    for source in sources {
        let mut event_sources = candidate_sources_for_handles(
            &source.host,
            event_candidate_handles(&source.bound_event).as_slice(),
        )?;
        for candidate in event_sources.drain(..) {
            if seen.insert(candidate.id) {
                result.push(candidate);
            }
        }
    }
    Ok(result)
}

fn emit_rows<S: ExtractRowSink + ?Sized>(
    context: ExtractEmitContext<'_>,
    candidate_times: Vec<u64>,
    sink: &mut S,
) -> Result<ExtractRunStats, WavepeekError> {
    let mut emitted = 0usize;
    let mut truncated = false;

    'timestamps: for timestamp in candidate_times {
        let previous_timestamp = context.waveform.borrow().previous_sample_time(timestamp);
        for (source_order, source) in context.sources.iter().enumerate() {
            debug_assert_eq!(source.declaration_index, source_order);
            let frame = EventEvalFrame {
                timestamp,
                previous_timestamp,
                tracked_signals: &[],
            };
            if !event_expr_matches(
                source.on.as_str(),
                &source.bound_event,
                &source.host,
                &frame,
            )? {
                continue;
            }
            let Some(sample_timestamp) = timestamp
                .checked_sub(1)
                .filter(|value| *value >= context.dump_start_raw && *value <= context.dump_end_raw)
            else {
                continue;
            };
            if !eval_bound_logical_truth(
                source.when.as_str(),
                &source.bound_when,
                &source.host,
                sample_timestamp,
            )? {
                continue;
            }
            if let Some(limit) = context.max_entries
                && emitted == limit
            {
                truncated = true;
                break 'timestamps;
            }
            let row = build_row(
                source,
                timestamp,
                sample_timestamp,
                context.dump_tick,
                context.waveform,
            )?;
            sink.emit(row)?;
            emitted += 1;
        }
    }

    Ok(ExtractRunStats { emitted, truncated })
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
) -> Result<u64, WavepeekError> {
    match validate_time_token_to_raw(token, dump_time, true) {
        Ok(raw) => Ok(raw),
        Err(TimeValidationError::RequiresUnits) => Err(WavepeekError::Args(format!(
            "time token '{token}' requires units. See 'wavepeek extract generic --help'."
        ))),
        Err(TimeValidationError::InvalidToken) => Err(WavepeekError::Args(format!(
            "invalid time token '{token}': expected <integer><unit> (for example 10ns). See 'wavepeek extract generic --help'."
        ))),
        Err(TimeValidationError::TooLarge) => Err(WavepeekError::Args(format!(
            "time '{token}' is too large to process safely. See 'wavepeek extract generic --help'."
        ))),
        Err(TimeValidationError::OutOfBounds) => Err(WavepeekError::Args(format!(
            "time '{}' for {} is outside dump bounds [{}, {}]. See 'wavepeek extract generic --help'.",
            token, arg_name, metadata.time_start, metadata.time_end
        ))),
        Err(TimeValidationError::NotAligned) => {
            let dump_precision = format_raw_timestamp(1, dump_time.dump_tick)?;
            Err(WavepeekError::Args(format!(
                "time '{token}' cannot be represented exactly in dump precision '{}'. See 'wavepeek extract generic --help'.",
                dump_precision
            )))
        }
        Err(TimeValidationError::RawOutOfRange) => Err(WavepeekError::Args(format!(
            "time '{token}' exceeds supported raw timestamp range. See 'wavepeek extract generic --help'."
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
