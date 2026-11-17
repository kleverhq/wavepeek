use std::collections::HashSet;
use std::ops::ControlFlow;

use serde::Serialize;

use crate::cli::change::{ChangeArgs, RowMode, RowValues};
use crate::cli::limits::LimitArg;
use crate::cli::sampling::SampleMode;
use crate::debug_trace::DebugTrace;
use crate::diagnostic::{Diagnostic, WarningDiagnosticCode};
use crate::engine::expr_runtime::{
    bind_waveform_event_expr, eval_with_waveform_errors, event_candidate_handles,
    event_expr_contains_wildcard, event_expr_is_edge_only, event_expr_matches, event_iff_handles,
    open_shared_waveform,
};
use crate::engine::time::{
    DumpTimeContext, ParsedTime, TimeValidationError, format_raw_timestamp,
    parse_dump_time_context, validate_time_token_to_raw,
};
use crate::engine::value_format::format_verilog_literal;
use crate::engine::{CommandData, CommandName, CommandResult, HumanRenderOptions, ResultSummary};
use crate::error::WavepeekError;
use crate::expr::{
    BoundEventExpr, EventEvalFrame, ExprTypeKind, ExpressionHost, SampledValue, SignalHandle,
};
use crate::waveform::expr_host::RuntimeExpressionHost;
use crate::waveform::expr_query::{ExpressionQuery, ExpressionQueryTick};
use crate::waveform::{ResolvedSignal, SampledSignalState, Waveform, display_signal_path};

const PRE_EDGE_REQUIRES_EDGE_ONLY_ON: &str = "--sample-mode pre-edge requires --on with only edge event terms (posedge, negedge, or edge); use --sample-mode native for wildcard or plain signal triggers";

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ChangeSignalValue {
    #[serde(skip_serializing)]
    pub display: String,
    pub path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub relative_path: Option<String>,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ChangeSnapshot {
    pub time: String,
    pub sample_time: String,
    pub signals: Vec<ChangeSignalValue>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RequestedSignal {
    display: String,
    relative_path: Option<String>,
    selected: ResolvedSignal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ChangeRunStats {
    emitted: usize,
    truncated: bool,
}

#[derive(Debug)]
struct ChangeCommandOutcome {
    human_options: HumanRenderOptions,
    diagnostics: Vec<Diagnostic>,
    summary: ResultSummary,
}

trait ChangeSnapshotSink {
    fn start(&mut self, _scope: Option<&str>) -> Result<(), WavepeekError> {
        Ok(())
    }

    fn emit(&mut self, snapshot: ChangeSnapshot) -> Result<(), WavepeekError>;
}

#[derive(Default)]
struct CollectingChangeSink {
    snapshots: Vec<ChangeSnapshot>,
}

impl ChangeSnapshotSink for CollectingChangeSink {
    fn emit(&mut self, snapshot: ChangeSnapshot) -> Result<(), WavepeekError> {
        self.snapshots.push(snapshot);
        Ok(())
    }
}

struct JsonlChangeSink<'a, W: std::io::Write> {
    writer: &'a mut crate::output::JsonlWriter<W>,
}

impl<W: std::io::Write> ChangeSnapshotSink for JsonlChangeSink<'_, W> {
    fn start(&mut self, scope: Option<&str>) -> Result<(), WavepeekError> {
        match scope {
            Some(scope) => self.writer.begin_scope(scope),
            None => self.writer.begin(),
        }
    }

    fn emit(&mut self, snapshot: ChangeSnapshot) -> Result<(), WavepeekError> {
        self.writer.data(&snapshot)
    }
}

pub fn run(args: ChangeArgs) -> Result<CommandResult, WavepeekError> {
    let output_mode = crate::output_mode::OutputMode::from_json_flags(args.json, args.jsonl);
    let scope = args.scope.clone();
    let summary_only = args.summary;
    let mut sink = CollectingChangeSink::default();
    let outcome = run_with_sink(args, &mut sink)?;

    Ok(CommandResult {
        command: CommandName::Change,
        output_mode,
        human_options: outcome.human_options,
        scope,
        summary_only,
        data: CommandData::Change(sink.snapshots),
        summary: Some(outcome.summary),
        diagnostics: outcome.diagnostics,
    })
}

pub fn run_jsonl<W: std::io::Write>(
    args: ChangeArgs,
    writer: &mut crate::output::JsonlWriter<W>,
) -> Result<(), WavepeekError> {
    writer.suppress_data(args.summary);
    let outcome = {
        let mut sink = JsonlChangeSink { writer };
        run_with_sink(args, &mut sink)?
    };

    for diagnostic in &outcome.diagnostics {
        writer.diagnostic(diagnostic)?;
    }
    writer.end_summary(&outcome.summary)
}

fn run_with_sink<S: ChangeSnapshotSink + ?Sized>(
    args: ChangeArgs,
    sink: &mut S,
) -> Result<ChangeCommandOutcome, WavepeekError> {
    let max_entries = match &args.max {
        LimitArg::Numeric(0) => {
            return Err(WavepeekError::Args(
                "--max must be greater than 0.".to_string(),
            ));
        }
        LimitArg::Numeric(value) => Some(*value),
        LimitArg::Unlimited => None,
    };

    let mut diagnostics = Vec::new();

    let debug = DebugTrace::for_command(CommandName::Change);
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

    let mut requested_signals = {
        let waveform_ref = waveform.borrow();
        resolve_requested_signals(&waveform_ref, args.scope.as_deref(), &args)?
    };
    debug.event(
        "signal.list.done",
        || serde_json::json!({"signals": requested_signals.len()}),
    );

    let event_expr_source = args.on.as_str();
    let (host, bound_event) =
        bind_waveform_event_expr(waveform.clone(), args.scope.as_deref(), event_expr_source)?;
    validate_sample_mode(args.sample_mode, &bound_event)?;
    debug.event("expression.bind.done", || serde_json::json!({}));

    let dump_time = parse_dump_time_context(&metadata)?;
    let dump_tick = dump_time.dump_tick;
    let dump_start_raw =
        u64::try_from(dump_time.dump_start_zs / dump_time.dump_tick_zs).map_err(|_| {
            WavepeekError::Internal("dump start timestamp exceeds supported range".to_string())
        })?;

    let from_raw = match args.from.as_deref() {
        Some(token) => parse_bound_time(token, "--from", dump_time, &metadata)?,
        None => dump_start_raw,
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
    debug.event("time.parse.done", || serde_json::json!({}));

    let tracked_signal_handles = requested_signals
        .iter()
        .map(|signal| {
            host.resolve_signal(&signal.selected.path)
                .map_err(|diagnostic| WavepeekError::Internal(diagnostic.message))
        })
        .collect::<Result<Vec<_>, _>>()?;
    for requested in &mut requested_signals {
        let display =
            display_signal_path(&requested.selected.path, args.scope.as_deref()).to_string();
        requested.relative_path = if (args.json || args.jsonl) && args.scope.is_some() {
            Some(display.clone())
        } else {
            None
        };
        requested.display = display;
    }

    let mut drivers = event_candidate_handles(&bound_event);
    if event_expr_contains_wildcard(&bound_event) {
        drivers.extend(&tracked_signal_handles);
    }
    let mut seen = HashSet::new();
    drivers.retain(|handle| seen.insert(*handle));
    let mut operands = event_iff_handles(&bound_event);
    operands.extend(&tracked_signal_handles);
    let query = ExpressionQuery::new(&host, &drivers, &operands)?;
    debug.event("signal.resolve.done", || {
        serde_json::json!({"requested_signals": requested_signals.len(), "drivers": drivers.len()})
    });

    sink.start(args.scope.as_deref())?;
    debug.event(
        "change.run.start",
        || serde_json::json!({"selected_engine": "query"}),
    );
    let mut stats = ChangeRunStats {
        emitted: 0,
        truncated: false,
    };
    let mut previous_values = if args.row_values == RowValues::Delta {
        vec![None; requested_signals.len()]
    } else {
        Vec::new()
    };
    waveform.borrow_mut().query_expressions(
        &query,
        from_raw..=to_raw,
        args.row_mode == RowMode::Sparse,
        |host, tick| {
            let timestamp = match tick {
                ExpressionQueryTick::Initial(timestamp) => {
                    previous_values = sample_values(&sample_requested(
                        host,
                        &requested_signals,
                        &tracked_signal_handles,
                        timestamp,
                        event_expr_source,
                    )?);
                    return Ok(ControlFlow::Continue(()));
                }
                ExpressionQueryTick::Candidate(timestamp) => timestamp,
            };
            if args.row_mode == RowMode::Sparse && timestamp <= from_raw {
                return Ok(ControlFlow::Continue(()));
            }
            let previous_timestamp = pre_edge_sample_time(timestamp, dump_start_raw);
            let frame = EventEvalFrame {
                timestamp,
                previous_timestamp,
                tracked_signals: &tracked_signal_handles,
            };
            if !event_expr_matches(event_expr_source, &bound_event, host, &frame)? {
                return Ok(ControlFlow::Continue(()));
            }
            let sample_time = match args.sample_mode {
                SampleMode::Native => timestamp,
                SampleMode::PreEdge => match previous_timestamp {
                    Some(timestamp) => timestamp,
                    None => return Ok(ControlFlow::Continue(())),
                },
            };
            let samples = sample_requested(
                host,
                &requested_signals,
                &tracked_signal_handles,
                sample_time,
                event_expr_source,
            )?;
            if emit_row(
                &requested_signals,
                &samples,
                &mut previous_values,
                args.row_mode,
                args.row_values,
                max_entries,
                &mut stats.emitted,
                timestamp,
                sample_time,
                dump_tick,
                sink,
            )? {
                stats.truncated = true;
                return Ok(ControlFlow::Break(()));
            }
            Ok(ControlFlow::Continue(()))
        },
    )?;

    debug.event("change.run.done", || {
        serde_json::json!({
            "selected_engine": "query",
            "snapshots": stats.emitted,
            "truncated": stats.truncated,
        })
    });

    if let Some(max_entries) = max_entries
        && stats.truncated
    {
        diagnostics.push(Diagnostic::warning(
            WarningDiagnosticCode::OutputTruncated,
            format!("truncated output to {max_entries} entries (use --max to increase limit)"),
        ));
    }

    Ok(ChangeCommandOutcome {
        human_options: HumanRenderOptions {
            scope_tree: false,
            signals_abs: args.abs,
        },
        diagnostics,
        summary: ResultSummary::from_run(stats.emitted, max_entries, stats.truncated),
    })
}

fn validate_sample_mode(
    sample_mode: SampleMode,
    bound_event: &BoundEventExpr,
) -> Result<(), WavepeekError> {
    if sample_mode == SampleMode::PreEdge && !event_expr_is_edge_only(bound_event) {
        return Err(WavepeekError::Args(
            PRE_EDGE_REQUIRES_EDGE_ONLY_ON.to_string(),
        ));
    }
    Ok(())
}

fn pre_edge_sample_time(trigger_timestamp: u64, dump_start_raw: u64) -> Option<u64> {
    trigger_timestamp
        .checked_sub(1)
        .filter(|query_time| *query_time >= dump_start_raw)
}

fn sample_requested(
    host: &dyn RuntimeExpressionHost,
    requested: &[RequestedSignal],
    handles: &[SignalHandle],
    timestamp: u64,
    source: &str,
) -> Result<Vec<SampledSignalState>, WavepeekError> {
    requested
        .iter()
        .zip(handles)
        .map(|(signal, &handle)| {
            let ty = host
                .signal_type(handle)
                .map_err(|diagnostic| WavepeekError::Internal(diagnostic.message))?;
            let bits = if ty.kind == ExprTypeKind::Event {
                // Raw events have a zero-width payload, including between occurrences.
                eval_with_waveform_errors(source, host, || host.event_occurred(handle, timestamp))?;
                Some(String::new())
            } else {
                match eval_with_waveform_errors(source, host, || {
                    host.sample_value(handle, timestamp)
                })? {
                    SampledValue::Integral { bits, .. } => {
                        bits.map(|bits| bits.chars().map(crate::logic::four_state).collect())
                    }
                    SampledValue::Real { value: None } | SampledValue::String { value: None } => {
                        None
                    }
                    _ => {
                        return Err(WavepeekError::Signal(format!(
                            "signal '{}' has unsupported non-bit-vector encoding",
                            signal.selected.path,
                        )));
                    }
                }
            };
            Ok(SampledSignalState {
                path: signal.selected.path.clone(),
                width: signal.selected.width,
                bits,
            })
        })
        .collect()
}

#[allow(clippy::too_many_arguments)]
fn emit_row<S: ChangeSnapshotSink + ?Sized>(
    requested_signals: &[RequestedSignal],
    current_samples: &[SampledSignalState],
    previous_values: &mut [Option<String>],
    row_mode: RowMode,
    row_values: RowValues,
    max_entries: Option<usize>,
    emitted: &mut usize,
    timestamp: u64,
    sample_time: u64,
    dump_tick: ParsedTime,
    sink: &mut S,
) -> Result<bool, WavepeekError> {
    let track_changes = row_mode == RowMode::Sparse || row_values == RowValues::Delta;
    let changed = if track_changes {
        previous_values
            .iter()
            .zip(current_samples)
            .map(|(previous, current)| *previous != current.bits)
            .collect::<Vec<_>>()
    } else {
        Vec::new()
    };
    if row_mode == RowMode::Sparse && !changed.iter().any(|changed| *changed) {
        return Ok(false);
    }
    let changed_columns = if row_values == RowValues::Delta && *emitted > 0 {
        Some(changed.as_slice())
    } else {
        None
    };
    let snapshot = build_snapshot(
        requested_signals,
        current_samples,
        changed_columns,
        timestamp,
        sample_time,
        dump_tick,
    )?;
    if max_entries == Some(*emitted) {
        return Ok(true);
    }
    sink.emit(snapshot)?;
    if track_changes {
        for (previous, current) in previous_values.iter_mut().zip(current_samples) {
            previous.clone_from(&current.bits);
        }
    }
    *emitted += 1;
    Ok(false)
}

fn sample_values(samples: &[SampledSignalState]) -> Vec<Option<String>> {
    samples.iter().map(|sample| sample.bits.clone()).collect()
}

fn build_snapshot(
    requested_signals: &[RequestedSignal],
    current_samples: &[SampledSignalState],
    changed: Option<&[bool]>,
    timestamp: u64,
    sample_timestamp: u64,
    dump_tick: ParsedTime,
) -> Result<ChangeSnapshot, WavepeekError> {
    let signals = requested_signals
        .iter()
        .zip(current_samples.iter())
        .enumerate()
        .filter(|(index, _)| changed.is_none_or(|changed| changed[*index]))
        .map(|(_, (requested, sampled))| {
            let bits = sampled.bits.as_ref().ok_or_else(|| {
                WavepeekError::Signal(format!(
                    "signal '{}' has no value at or before requested time",
                    requested.selected.path
                ))
            })?;
            Ok(ChangeSignalValue {
                display: requested.display.clone(),
                path: requested.selected.path.clone(),
                relative_path: requested.relative_path.clone(),
                value: format_verilog_literal(sampled.width, bits.as_str()),
            })
        })
        .collect::<Result<Vec<_>, WavepeekError>>()?;

    Ok(ChangeSnapshot {
        time: format_raw_timestamp(timestamp, dump_tick)?,
        sample_time: format_raw_timestamp(sample_timestamp, dump_tick)?,
        signals,
    })
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
        let normalized = crate::engine::signal_token(token);
        let display = normalized.as_str();
        if display.is_empty() {
            return Err(WavepeekError::Args(
                "signal names must not be empty. See 'wavepeek change --help'.".to_string(),
            ));
        }

        let selected = waveform.resolve_signal_with_diagnostic(display, scope, true)?;
        resolved.push(RequestedSignal {
            display: display.to_string(),
            relative_path: None,
            selected,
        });
    }

    Ok(resolved)
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
    use super::*;
    use std::fs;
    use std::path::PathBuf;
    use tempfile::NamedTempFile;

    struct FailAfterFirstEmitSink {
        emitted: usize,
    }

    impl ChangeSnapshotSink for FailAfterFirstEmitSink {
        fn emit(&mut self, _snapshot: ChangeSnapshot) -> Result<(), WavepeekError> {
            self.emitted += 1;
            Err(WavepeekError::Internal(
                "sentinel change emit failure".to_string(),
            ))
        }
    }

    #[test]
    fn request_resolution_and_snapshot_helpers_exercise_validation() {
        let fixture = write_fixture(TEST_VCD, "change-helper.vcd");
        let waveform = Waveform::open(fixture.path()).expect("waveform should open");
        let args = ChangeArgs {
            waves: PathBuf::from(fixture.path()),
            from: None,
            to: None,
            scope: Some("top".to_string()),
            signals: vec!["sig".to_string()],
            on: "*".to_string(),
            sample_mode: SampleMode::Native,
            row_mode: RowMode::Dense,
            row_values: RowValues::Full,
            max: LimitArg::Numeric(5),
            abs: false,
            json: false,
            summary: false,
            jsonl: false,
        };
        let resolved = resolve_requested_signals(&waveform, args.scope.as_deref(), &args)
            .expect("signals should resolve");
        assert_eq!(resolved[0].display, "sig");
        assert_eq!(resolved[0].selected.path, "top.sig");

        let snapshot = build_snapshot(
            &[RequestedSignal {
                display: "sig".to_string(),
                relative_path: Some("sig".to_string()),
                selected: resolved[0].selected.clone(),
            }],
            &[SampledSignalState {
                path: "top.sig".to_string(),
                width: 1,
                bits: Some("1".to_string()),
            }],
            None,
            5,
            5,
            crate::engine::time::ParsedTime {
                value: 1,
                unit: crate::engine::time::TimeUnit::Ns,
            },
        )
        .expect("snapshot should build");
        assert_eq!(snapshot.time, "5ns");
        assert_eq!(snapshot.sample_time, "5ns");
        assert_eq!(snapshot.signals[0].value, "1'h1");

        let error = build_snapshot(
            &[RequestedSignal {
                display: "sig".to_string(),
                relative_path: Some("sig".to_string()),
                selected: resolved[0].selected.clone(),
            }],
            &[SampledSignalState {
                path: "top.sig".to_string(),
                width: 1,
                bits: None,
            }],
            None,
            5,
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
    fn change_run_exercises_public_entrypoint_success_and_early_errors() {
        let fixture = write_fixture(TEST_VCD, "change-run.vcd");

        let result = run(ChangeArgs {
            waves: PathBuf::from(fixture.path()),
            from: None,
            to: None,
            scope: Some("top".to_string()),
            signals: vec!["sig".to_string()],
            on: "posedge sig".to_string(),
            sample_mode: SampleMode::Native,
            row_mode: RowMode::Dense,
            row_values: RowValues::Full,
            max: LimitArg::Unlimited,
            abs: false,
            json: true,
            summary: false,
            jsonl: false,
        })
        .expect("change run should succeed");
        assert!(result.diagnostics.is_empty());
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
            on: "*".to_string(),
            sample_mode: SampleMode::Native,
            row_mode: RowMode::Dense,
            row_values: RowValues::Full,
            max: LimitArg::Numeric(0),
            abs: false,
            json: false,
            summary: false,
            jsonl: false,
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
            on: "posedge sig".to_string(),
            sample_mode: SampleMode::Native,
            row_mode: RowMode::Dense,
            row_values: RowValues::Full,
            max: LimitArg::Numeric(5),
            abs: false,
            json: false,
            summary: false,
            jsonl: false,
        })
        .expect_err("reversed range should fail");
        assert!(reversed.to_string().contains("less than or equal to --to"));
    }

    #[test]
    fn change_streaming_sink_errors_stop_during_emission() {
        let fixture = write_fixture(TEST_VCD, "change-streaming-error.vcd");
        let mut sink = FailAfterFirstEmitSink { emitted: 0 };

        let error = run_with_sink(
            ChangeArgs {
                waves: PathBuf::from(fixture.path()),
                from: None,
                to: None,
                scope: Some("top".to_string()),
                signals: vec!["sig".to_string()],
                on: "posedge sig".to_string(),
                sample_mode: SampleMode::Native,
                row_mode: RowMode::Dense,
                row_values: RowValues::Full,
                max: LimitArg::Unlimited,
                abs: false,
                summary: false,
                json: false,
                jsonl: true,
            },
            &mut sink,
        )
        .expect_err("sink error should stop the streaming run");

        assert_eq!(sink.emitted, 1);
        assert!(error.to_string().contains("sentinel change emit failure"));
    }

    #[test]
    fn change_run_and_time_helpers_exercise_diagnostic_and_bound_error_paths() {
        let fixture = write_fixture(TEST_VCD, "change-warning-empty.vcd");
        let empty = run(ChangeArgs {
            waves: PathBuf::from(fixture.path()),
            from: None,
            to: None,
            scope: Some("top".to_string()),
            signals: vec!["sig".to_string()],
            on: "negedge sig".to_string(),
            sample_mode: SampleMode::Native,
            row_mode: RowMode::Dense,
            row_values: RowValues::Full,
            max: LimitArg::Numeric(5),
            abs: false,
            json: false,
            summary: false,
            jsonl: false,
        })
        .expect("empty result should still succeed");
        assert!(empty.diagnostics.is_empty());

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
            on: "*".to_string(),
            sample_mode: SampleMode::Native,
            row_mode: RowMode::Dense,
            row_values: RowValues::Full,
            max: LimitArg::Numeric(1),
            abs: false,
            json: false,
            summary: false,
            jsonl: false,
        })
        .expect("truncated result should succeed");
        assert!(truncated.diagnostics.iter().any(|diagnostic| {
            diagnostic.code() == Some("WPK-W0002")
                && diagnostic
                    .message()
                    .contains("truncated output to 1 entries")
        }));
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
