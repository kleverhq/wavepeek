use serde::Serialize;

use crate::cli::value::ValueArgs;
use crate::engine::time::{
    TimeValidationError, format_raw_timestamp, parse_dump_time_context, validate_time_token_to_raw,
};
use crate::engine::value_format::format_verilog_literal;
use crate::engine::{CommandData, CommandName, CommandResult};
use crate::error::WavepeekError;
use crate::waveform::{Waveform, WaveformMetadata};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ValueSignalValue {
    #[serde(skip_serializing)]
    pub display: String,
    pub path: String,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ValueData {
    pub time: String,
    pub signals: Vec<ValueSignalValue>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RequestedSignal {
    display: String,
    path: String,
}

pub fn run(args: ValueArgs) -> Result<CommandResult, WavepeekError> {
    let mut waveform = Waveform::open(args.waves.as_path())?;
    let metadata = waveform.metadata()?;

    let requested_signals = resolve_requested_signals(&waveform, args.scope.as_deref(), &args)?;

    let dump_time = parse_dump_time_context(&metadata)?;
    let query_time_raw = validate_time_token_to_raw(args.at.as_str(), dump_time, false)
        .map_err(|error| map_value_time_validation_error(args.at.as_str(), &metadata, error))?;

    let canonical_paths = requested_signals
        .iter()
        .map(|signal| signal.path.clone())
        .collect::<Vec<_>>();
    let sampled = waveform.sample_signals_at_time(&canonical_paths, query_time_raw)?;

    let signals = requested_signals
        .into_iter()
        .zip(sampled)
        .map(|(requested, sampled)| ValueSignalValue {
            display: requested.display,
            path: sampled.path,
            value: format_verilog_literal(sampled.width, sampled.bits.as_str()),
        })
        .collect::<Vec<_>>();

    let normalized_time = format_raw_timestamp(query_time_raw, dump_time.dump_tick)?;

    Ok(CommandResult {
        command: CommandName::Value,
        json: args.json,
        human_options: crate::engine::HumanRenderOptions {
            scope_tree: false,
            signals_abs: args.abs,
        },
        data: CommandData::Value(ValueData {
            time: normalized_time,
            signals,
        }),
        diagnostics: Vec::new(),
    })
}

fn resolve_requested_signals(
    waveform: &Waveform,
    scope: Option<&str>,
    args: &ValueArgs,
) -> Result<Vec<RequestedSignal>, WavepeekError> {
    if let Some(scope) = scope {
        waveform.signals_in_scope(scope)?;
    }

    let mut resolved = Vec::with_capacity(args.signals.len());
    for token in &args.signals {
        let display = token.trim();
        if display.is_empty() {
            return Err(WavepeekError::Args(
                "signal names must not be empty. See 'wavepeek value --help'.".to_string(),
            ));
        }

        let path = match scope {
            Some(scope) => format!("{scope}.{display}"),
            None => display.to_string(),
        };
        resolved.push(RequestedSignal {
            display: display.to_string(),
            path,
        });
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
        RequestedSignal, WaveformMetadata, map_value_time_validation_error,
        resolve_requested_signals, run,
    };
    use crate::cli::value::ValueArgs;
    use crate::engine::CommandData;
    use crate::engine::time::TimeValidationError;
    use crate::waveform::Waveform;

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

        assert_eq!(
            resolve_requested_signals(
                &waveform,
                Some("top"),
                &ValueArgs {
                    waves: PathBuf::from(fixture.path()),
                    at: "5ns".to_string(),
                    scope: Some("top".to_string()),
                    signals: vec!["sig".to_string()],
                    abs: false,
                    json: false,
                },
            )
            .expect("scoped signals should resolve"),
            vec![RequestedSignal {
                display: "sig".to_string(),
                path: "top.sig".to_string(),
            }]
        );
        assert!(
            resolve_requested_signals(
                &waveform,
                None,
                &ValueArgs {
                    waves: PathBuf::from(fixture.path()),
                    at: "5ns".to_string(),
                    scope: None,
                    signals: vec!["  ".to_string()],
                    abs: false,
                    json: false,
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

        let result = run(ValueArgs {
            waves: PathBuf::from(fixture.path()),
            at: "5ns".to_string(),
            scope: Some("top".to_string()),
            signals: vec!["sig".to_string()],
            abs: true,
            json: true,
        })
        .expect("value run should succeed");
        let CommandData::Value(payload) = result.data else {
            panic!("value command should return value data");
        };
        assert_eq!(payload.time, "5ns");
        assert_eq!(payload.signals[0].path, "top.sig");
        assert_eq!(payload.signals[0].value, "1'h1");
    }

    fn write_fixture(contents: &str, suffix: &str) -> NamedTempFile {
        let fixture = NamedTempFile::with_suffix(suffix).expect("fixture should create");
        fs::write(fixture.path(), contents).expect("fixture should write");
        fixture
    }
}
