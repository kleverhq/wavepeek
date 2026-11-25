use serde::Serialize;

use crate::cli::value::ValueArgs;
use crate::debug_trace::DebugTrace;
use crate::engine::output_paths::OutputPaths;
use crate::engine::time::{
    DumpTimeContext, TimeValidationError, format_raw_timestamp, parse_dump_time_context,
    validate_time_token_to_raw,
};
use crate::engine::value_format::format_verilog_literal;
use crate::engine::{CommandData, CommandName, CommandResult, OPENING_WAVEFORM, ReportProgress};
use crate::error::WavepeekError;
use crate::waveform::{ResolvedSignal, Waveform, WaveformMetadata};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ValueSignalValue {
    #[serde(skip_serializing)]
    pub display: String,
    pub path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub relative_path: Option<String>,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ValueSnapshot {
    pub time: String,
    pub signals: Vec<ValueSignalValue>,
}

pub type ValueData = Vec<ValueSnapshot>;

pub fn run(args: ValueArgs, report: ReportProgress<'_>) -> Result<CommandResult, WavepeekError> {
    let debug = DebugTrace::for_command(CommandName::Value);
    debug.event("backend.open.start", || serde_json::json!({}));
    report(OPENING_WAVEFORM);
    let mut waveform = Waveform::open_with_progress(args.waves.as_path(), report)?;
    debug.event("backend.open.done", || {
        serde_json::json!({
            "backend": waveform.backend_name(),
            "format": waveform.format_name(),
        })
    });
    let metadata = waveform.metadata()?;
    debug.event("metadata.load.done", || serde_json::json!({}));

    let requested_signals = resolve_requested_signals(&waveform, args.scope.as_deref(), &args)?;
    let machine = args.json || args.jsonl;
    let paths = OutputPaths::new(
        args.path_output.style(),
        args.scope.as_deref(),
        args.abs || machine,
    )?;
    let output_paths = requested_signals
        .iter()
        .map(|signal| paths.signal(signal))
        .collect::<Result<Vec<_>, _>>()?;
    debug.event(
        "signal.select.done",
        || serde_json::json!({"signals": requested_signals.len()}),
    );

    let dump_time = parse_dump_time_context(&metadata)?;
    let query_times_raw = parse_at_tokens(&args.at, &metadata, dump_time)?;
    debug.event(
        "time.parse.done",
        || serde_json::json!({"times": query_times_raw.len()}),
    );

    let mut snapshots = Vec::with_capacity(query_times_raw.len());

    report("sampling values");
    let sampled_rows = waveform.sample_resolved_at_times(&requested_signals, &query_times_raw)?;
    for (query_time_raw, sampled) in query_times_raw.into_iter().zip(sampled_rows) {
        let signals = sampled
            .into_iter()
            .zip(&output_paths)
            .map(|(sampled, (path, display))| {
                let relative_path = if machine && args.scope.is_some() {
                    Some(display.clone())
                } else {
                    None
                };
                ValueSignalValue {
                    display: display.clone(),
                    path: path.clone(),
                    relative_path,
                    value: format_verilog_literal(sampled.width, &sampled.bits),
                }
            })
            .collect();

        snapshots.push(ValueSnapshot {
            time: format_raw_timestamp(query_time_raw, dump_time.dump_tick)?,
            signals,
        });
    }
    debug.event("value.sample.done", || {
        serde_json::json!({
            "snapshots": snapshots.len(),
            "signals": requested_signals.len(),
        })
    });

    Ok(CommandResult {
        command: CommandName::Value,
        output_mode: crate::output_mode::OutputMode::from_json_flags(args.json, args.jsonl),
        human_options: crate::engine::HumanRenderOptions {
            scope_tree: false,
            signals_abs: args.abs,
        },
        scope: paths.context_scope(&waveform, machine)?,
        summary_only: false,
        data: CommandData::Value(snapshots),
        summary: None,
        diagnostics: Vec::new(),
    })
}

fn parse_at_tokens(
    at: &[String],
    metadata: &WaveformMetadata,
    dump_time: DumpTimeContext,
) -> Result<Vec<u64>, WavepeekError> {
    let mut raw_times = Vec::new();
    for token in at {
        let token = token.trim();
        if token.is_empty() {
            return Err(WavepeekError::Args(
                "time list in --at must not contain empty entries. See 'wavepeek value --help'."
                    .to_string(),
            ));
        }
        let raw_time = validate_time_token_to_raw(token, dump_time, false)
            .map_err(|error| map_value_time_validation_error(token, metadata, error))?;
        raw_times.push(raw_time);
    }

    if raw_times.is_empty() {
        return Err(WavepeekError::Args(
            "time list in --at must not be empty. See 'wavepeek value --help'.".to_string(),
        ));
    }

    Ok(raw_times)
}

fn resolve_requested_signals(
    waveform: &Waveform<'_>,
    scope: Option<&str>,
    args: &ValueArgs,
) -> Result<Vec<ResolvedSignal>, WavepeekError> {
    if let Some(scope) = scope {
        waveform.validate_scope(scope)?;
    }

    let mut resolved = Vec::with_capacity(args.signals.len());
    for token in &args.signals {
        let normalized = crate::cli::signal_token(token);
        let display = normalized.as_str();
        if display.is_empty() {
            return Err(WavepeekError::Args(
                "signal names must not be empty. See 'wavepeek value --help'.".to_string(),
            ));
        }

        resolved.push(waveform.resolve_signal_with_diagnostic(display, scope, true)?);
    }

    Ok(resolved)
}

fn map_value_time_validation_error(
    token: &str,
    metadata: &WaveformMetadata,
    error: TimeValidationError,
) -> WavepeekError {
    match error {
        TimeValidationError::RequiresUnits | TimeValidationError::InvalidToken => {
            WavepeekError::Args(format!(
                "invalid time token '{token}': expected <integer><unit> (for example 10ns). See 'wavepeek value --help'."
            ))
        }
        TimeValidationError::TooLarge => WavepeekError::Args(format!(
            "time '{token}' is too large to process safely. See 'wavepeek value --help'."
        )),
        TimeValidationError::OutOfBounds => WavepeekError::Args(format!(
            "time '{token}' is outside dump bounds [{}, {}]. See 'wavepeek value --help'.",
            metadata.time_start, metadata.time_end
        )),
        TimeValidationError::NotAligned => WavepeekError::Args(format!(
            "time '{token}' is not aligned to dump resolution '{}'. See 'wavepeek value --help'.",
            metadata.time_unit
        )),
        TimeValidationError::RawOutOfRange => WavepeekError::Args(format!(
            "time '{token}' exceeds supported raw timestamp range. See 'wavepeek value --help'."
        )),
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;

    use tempfile::NamedTempFile;

    use super::{
        WaveformMetadata, map_value_time_validation_error, parse_at_tokens,
        resolve_requested_signals,
    };
    use crate::cli::value::ValueArgs;
    use crate::engine::CommandData;
    use crate::engine::time::{TimeValidationError, parse_dump_time_context};
    use crate::error::WavepeekError;
    use crate::waveform::Waveform;

    fn run(args: ValueArgs) -> Result<super::CommandResult, WavepeekError> {
        super::run(args, &|_| {})
    }

    const TEST_VCD: &str = concat!(
        "$date\n  today\n$end\n",
        "$version\n  value-test\n$end\n",
        "$timescale 1ns $end\n",
        "$scope module top $end\n",
        "$var wire 1 ! sig $end\n",
        "$upscope $end\n",
        "$enddefinitions $end\n",
        "#0\n0!\n#5\n1!\n"
    );

    #[test]
    fn value_helpers_exercise_resolution_time_errors_and_public_run() {
        let fixture = write_fixture(TEST_VCD, ".value-run.vcd");
        let waveform = Waveform::open(fixture.path()).expect("waveform should open");

        let resolved = resolve_requested_signals(
            &waveform,
            Some("top"),
            &ValueArgs {
                path_output: Default::default(),
                waves: PathBuf::from(fixture.path()),
                at: vec!["5ns".to_string()],
                scope: Some("top".to_string()),
                signals: vec!["sig".to_string()],
                abs: false,
                json: false,
                jsonl: false,
            },
        )
        .expect("scoped signals should resolve");
        assert_eq!(resolved[0].path, "top.sig");
        assert!(
            resolve_requested_signals(
                &waveform,
                None,
                &ValueArgs {
                    path_output: Default::default(),
                    waves: PathBuf::from(fixture.path()),
                    at: vec!["5ns".to_string()],
                    scope: None,
                    signals: vec!["  ".to_string()],
                    abs: false,
                    json: false,
                    jsonl: false,
                },
            )
            .expect_err("empty signal names should fail")
            .to_string()
            .contains("signal names must not be empty")
        );

        let metadata = WaveformMetadata {
            time_unit: "1ns".to_string(),
            time_start: "0ns".to_string(),
            time_end: "5ns".to_string(),
        };
        for error in [
            TimeValidationError::RequiresUnits,
            TimeValidationError::InvalidToken,
        ] {
            assert!(
                map_value_time_validation_error("10", &metadata, error)
                    .to_string()
                    .contains("expected <integer><unit>")
            );
        }
        assert!(
            map_value_time_validation_error("10ns", &metadata, TimeValidationError::TooLarge)
                .to_string()
                .contains("too large")
        );
        assert!(
            map_value_time_validation_error("10ns", &metadata, TimeValidationError::OutOfBounds)
                .to_string()
                .contains("outside dump bounds [0ns, 5ns]")
        );
        assert!(
            map_value_time_validation_error("10ps", &metadata, TimeValidationError::NotAligned)
                .to_string()
                .contains("not aligned to dump resolution '1ns'")
        );
        assert!(
            map_value_time_validation_error(
                "9999999999999999999ns",
                &metadata,
                TimeValidationError::RawOutOfRange
            )
            .to_string()
            .contains("supported raw timestamp range")
        );

        let dump_time = parse_dump_time_context(&metadata).expect("dump time should parse");
        assert_eq!(
            parse_at_tokens(
                &["5ns".to_string(), " 0ns ".to_string(), "5ns".to_string()],
                &metadata,
                dump_time,
            )
            .expect("time list should parse"),
            vec![5, 0, 5]
        );
        assert!(
            parse_at_tokens(
                &["5ns".to_string(), "".to_string(), "0ns".to_string()],
                &metadata,
                dump_time,
            )
            .expect_err("empty time list entries should fail")
            .to_string()
            .contains("time list in --at must not contain empty entries")
        );
        assert!(
            parse_at_tokens(&[], &metadata, dump_time)
                .expect_err("empty time lists should fail")
                .to_string()
                .contains("time list in --at must not be empty")
        );

        let result = run(ValueArgs {
            path_output: Default::default(),
            waves: PathBuf::from(fixture.path()),
            at: vec!["5ns".to_string()],
            scope: Some("top".to_string()),
            signals: vec!["sig".to_string()],
            abs: true,
            json: true,
            jsonl: false,
        })
        .expect("value run should succeed");
        let CommandData::Value(payload) = result.data else {
            panic!("value command should return value data");
        };
        assert_eq!(payload.len(), 1);
        assert_eq!(payload[0].time, "5ns");
        assert_eq!(payload[0].signals[0].path, "top.sig");
        assert_eq!(payload[0].signals[0].value, "1'h1");
    }

    fn write_fixture(contents: &str, suffix: &str) -> NamedTempFile {
        let fixture = NamedTempFile::with_suffix(suffix).expect("fixture should create");
        fs::write(fixture.path(), contents).expect("fixture should write");
        fixture
    }
}
