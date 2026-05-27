use serde::Serialize;

use crate::cli::property::{CaptureMode, PropertyArgs};
use crate::engine::expr_runtime::{
    bind_waveform_event_expr, bind_waveform_logical_expr, candidate_sources_for_handles,
    eval_bound_logical_truth, event_candidate_handles, event_expr_contains_wildcard,
    event_expr_is_any_tracked_only, event_expr_matches, open_shared_waveform,
    referenced_signal_handles,
};
use crate::engine::time::{
    DumpTimeContext, TimeValidationError, format_raw_timestamp, parse_dump_time_context,
    validate_time_token_to_raw,
};
use crate::engine::{CommandData, CommandName, CommandResult, HumanRenderOptions};
use crate::error::WavepeekError;
use crate::expr::EventEvalFrame;
use crate::waveform::{ChangeCandidateCollectionMode, Waveform};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum PropertyResultKind {
    Match,
    Assert,
    Deassert,
}

impl std::fmt::Display for PropertyResultKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Match => f.write_str("match"),
            Self::Assert => f.write_str("assert"),
            Self::Deassert => f.write_str("deassert"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PropertyCaptureRow {
    pub time: String,
    pub kind: PropertyResultKind,
}

pub fn run(args: PropertyArgs) -> Result<CommandResult, WavepeekError> {
    let waveform = open_shared_waveform(args.waves.as_path())?;
    if let Some(error) = waveform.borrow().unsupported_fsdb_command_error("property") {
        return Err(error);
    }
    let metadata = waveform.borrow().metadata()?;
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

    let event_expr_source = args.on.as_deref().unwrap_or("*");
    let (host, bound_event) =
        bind_waveform_event_expr(waveform.clone(), args.scope.as_deref(), event_expr_source)?;
    let bound_eval = bind_waveform_logical_expr(&host, args.scope.as_deref(), args.eval.as_str())?;
    let eval_signal_handles = referenced_signal_handles(&bound_eval);
    let eval_sources = candidate_sources_for_handles(&host, eval_signal_handles.as_slice())?;
    waveform
        .borrow()
        .validate_expr_values_supported(eval_sources.as_slice())?;

    let tracked_signal_handles = if event_expr_contains_wildcard(&bound_event) {
        if eval_signal_handles.is_empty() && event_expr_is_any_tracked_only(&bound_event) {
            return Err(WavepeekError::Args(
                "wildcard trigger cannot infer tracked signals from --eval; pass --on explicitly"
                    .to_string(),
            ));
        }
        eval_signal_handles.clone()
    } else {
        Vec::new()
    };

    let mut candidate_sources = if tracked_signal_handles.is_empty() {
        Vec::new()
    } else {
        eval_sources
    };
    candidate_sources.extend(candidate_sources_for_handles(
        &host,
        &event_candidate_handles(&bound_event),
    )?);
    let mut seen = std::collections::HashSet::new();
    candidate_sources.retain(|signal| seen.insert(signal.id));

    let candidate_times = waveform
        .borrow_mut()
        .collect_expr_candidate_times_with_mode(
            candidate_sources.as_slice(),
            from_raw,
            to_raw,
            ChangeCandidateCollectionMode::Auto,
        )?;
    let candidate_schedule = {
        let waveform_ref = waveform.borrow();
        build_candidate_schedule(&waveform_ref, candidate_times.as_slice())?
    };

    let mut rows = Vec::new();
    let mut previous_state = match args.capture {
        CaptureMode::Match => None,
        CaptureMode::Switch | CaptureMode::Assert | CaptureMode::Deassert => Some(
            eval_bound_logical_truth(args.eval.as_str(), &bound_eval, &host, from_raw)?,
        ),
    };

    for (timestamp, previous_timestamp) in candidate_schedule {
        let frame = EventEvalFrame {
            timestamp,
            previous_timestamp,
            tracked_signals: tracked_signal_handles.as_slice(),
        };
        if !event_expr_matches(event_expr_source, &bound_event, &host, &frame)? {
            continue;
        }

        let decision = eval_bound_logical_truth(args.eval.as_str(), &bound_eval, &host, timestamp)?;
        match args.capture {
            CaptureMode::Match => {
                if decision {
                    rows.push(PropertyCaptureRow {
                        time: format_raw_timestamp(timestamp, dump_tick)?,
                        kind: PropertyResultKind::Match,
                    });
                }
            }
            CaptureMode::Switch | CaptureMode::Assert | CaptureMode::Deassert => {
                if timestamp == from_raw {
                    previous_state = Some(decision);
                    continue;
                }

                let prior = previous_state.unwrap_or(false);
                let transition = match (prior, decision) {
                    (false, true) => Some(PropertyResultKind::Assert),
                    (true, false) => Some(PropertyResultKind::Deassert),
                    _ => None,
                };
                previous_state = Some(decision);

                let Some(kind) = transition else {
                    continue;
                };
                if !capture_allows_kind(args.capture, kind) {
                    continue;
                }

                rows.push(PropertyCaptureRow {
                    time: format_raw_timestamp(timestamp, dump_tick)?,
                    kind,
                });
            }
        }
    }

    Ok(CommandResult {
        command: CommandName::Property,
        json: args.json,
        human_options: HumanRenderOptions::default(),
        data: CommandData::Property(rows),
        warnings: Vec::new(),
    })
}

fn capture_allows_kind(capture: CaptureMode, kind: PropertyResultKind) -> bool {
    match capture {
        CaptureMode::Match | CaptureMode::Switch => true,
        CaptureMode::Assert => kind == PropertyResultKind::Assert,
        CaptureMode::Deassert => kind == PropertyResultKind::Deassert,
    }
}

fn build_candidate_schedule(
    waveform: &Waveform,
    candidate_times: &[u64],
) -> Result<Vec<(u64, Option<u64>)>, WavepeekError> {
    candidate_times
        .iter()
        .map(|timestamp| Ok((*timestamp, waveform.previous_sample_time(*timestamp))))
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
            "time token '{token}' requires units. See 'wavepeek property --help'."
        ))),
        Err(TimeValidationError::InvalidToken) => Err(WavepeekError::Args(format!(
            "invalid time token '{token}': expected <integer><unit> (for example 10ns). See 'wavepeek property --help'."
        ))),
        Err(TimeValidationError::TooLarge) => Err(WavepeekError::Args(format!(
            "time '{token}' is too large to process safely. See 'wavepeek property --help'."
        ))),
        Err(TimeValidationError::OutOfBounds) => Err(WavepeekError::Args(format!(
            "time '{}' for {} is outside dump bounds [{}, {}]. See 'wavepeek property --help'.",
            token, arg_name, metadata.time_start, metadata.time_end
        ))),
        Err(TimeValidationError::NotAligned) => {
            let dump_precision = format_raw_timestamp(1, dump_time.dump_tick)?;
            Err(WavepeekError::Args(format!(
                "time '{token}' cannot be represented exactly in dump precision '{}'. See 'wavepeek property --help'.",
                dump_precision
            )))
        }
        Err(TimeValidationError::RawOutOfRange) => Err(WavepeekError::Args(format!(
            "time '{token}' exceeds supported raw timestamp range. See 'wavepeek property --help'."
        ))),
    }
}

#[cfg(test)]
#[path = "../tests/property_capture_types.rs"]
mod property_capture_types;

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;

    use tempfile::NamedTempFile;

    use super::{
        DumpTimeContext, PropertyResultKind, build_candidate_schedule, capture_allows_kind,
        parse_bound_time, run,
    };
    use crate::cli::property::{CaptureMode, PropertyArgs};
    use crate::engine::CommandData;
    use crate::engine::time::{ParsedTime, TimeUnit, as_zeptoseconds};
    use crate::waveform::{Waveform, WaveformMetadata};

    const TEST_VCD: &str = concat!(
        "$date\n  today\n$end\n",
        "$version\n  property-test\n$end\n",
        "$timescale 1ns $end\n",
        "$scope module top $end\n",
        "$var wire 1 ! sig $end\n",
        "$upscope $end\n",
        "$enddefinitions $end\n",
        "#0\n",
        "0!\n",
        "#5\n",
        "1!\n",
        "#10\n",
        "0!\n"
    );

    fn metadata() -> WaveformMetadata {
        WaveformMetadata {
            time_unit: "1ns".to_string(),
            time_start: "0ns".to_string(),
            time_end: "10ns".to_string(),
        }
    }

    fn dump_time() -> DumpTimeContext {
        DumpTimeContext {
            dump_tick: ParsedTime {
                value: 1,
                unit: TimeUnit::Ns,
            },
            dump_tick_zs: 1_000_000_000_000,
            dump_start_zs: 0,
            dump_end_zs: 10_000_000_000_000,
        }
    }

    #[test]
    fn property_result_kind_display_is_stable() {
        assert_eq!(PropertyResultKind::Match.to_string(), "match");
        assert_eq!(PropertyResultKind::Assert.to_string(), "assert");
        assert_eq!(PropertyResultKind::Deassert.to_string(), "deassert");
    }

    #[test]
    fn capture_mode_filtering_matches_expected_transition_kinds() {
        assert!(capture_allows_kind(
            CaptureMode::Match,
            PropertyResultKind::Assert
        ));
        assert!(capture_allows_kind(
            CaptureMode::Switch,
            PropertyResultKind::Deassert
        ));
        assert!(capture_allows_kind(
            CaptureMode::Assert,
            PropertyResultKind::Assert
        ));
        assert!(!capture_allows_kind(
            CaptureMode::Assert,
            PropertyResultKind::Deassert
        ));
        assert!(capture_allows_kind(
            CaptureMode::Deassert,
            PropertyResultKind::Deassert
        ));
        assert!(!capture_allows_kind(
            CaptureMode::Deassert,
            PropertyResultKind::Assert
        ));
    }

    #[test]
    fn candidate_schedule_tracks_previous_timestamp_for_indexed_and_between_times() {
        let fixture = write_fixture(TEST_VCD, "property-schedule.vcd");
        let waveform = Waveform::open(fixture.path()).expect("waveform should open");

        let schedule = build_candidate_schedule(&waveform, &[0, 7, 10]).expect("schedule");
        assert_eq!(schedule, vec![(0, None), (7, Some(5)), (10, Some(5))]);
    }

    #[test]
    fn parse_bound_time_maps_validation_failures_to_argument_errors() {
        let md = metadata();
        let context = dump_time();

        assert_eq!(
            parse_bound_time("10ns", "--from", context, &md).expect("valid time"),
            10
        );

        for (token, expected) in [
            ("10", "requires units"),
            ("1.5ns", "expected <integer><unit>"),
            ("11ns", "outside dump bounds [0ns, 10ns]"),
            (
                "15ps",
                "cannot be represented exactly in dump precision '1ns'",
            ),
        ] {
            let error = parse_bound_time(token, "--from", context, &md).expect_err("bad time");
            assert!(error.to_string().contains(expected), "{token}: {error}");
        }

        let huge_context = DumpTimeContext {
            dump_tick: ParsedTime {
                value: 1,
                unit: TimeUnit::Zs,
            },
            dump_tick_zs: 1,
            dump_start_zs: 0,
            dump_end_zs: as_zeptoseconds(ParsedTime {
                value: u64::MAX,
                unit: TimeUnit::Ns,
            })
            .expect("end should convert"),
        };
        let error = parse_bound_time(&format!("{}ns", u64::MAX), "--to", huge_context, &md)
            .expect_err("raw out-of-range should fail");
        assert!(error.to_string().contains("supported raw timestamp range"));

        let error = parse_bound_time(&format!("{}s", u64::MAX), "--to", context, &md)
            .expect_err("too-large time should fail");
        assert!(error.to_string().contains("too large to process safely"));
    }

    #[test]
    fn property_run_captures_match_and_switch_rows_through_public_entrypoint() {
        let fixture = write_fixture(TEST_VCD, ".property-run.vcd");

        let matched = run(PropertyArgs {
            waves: PathBuf::from(fixture.path()),
            from: None,
            to: None,
            scope: Some("top".to_string()),
            on: Some("posedge sig".to_string()),
            eval: "sig".to_string(),
            capture: CaptureMode::Match,
            json: false,
        })
        .expect("match capture should succeed");
        let CommandData::Property(rows) = matched.data else {
            panic!("property command should return property rows");
        };
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].time, "5ns");
        assert_eq!(rows[0].kind, PropertyResultKind::Match);

        let switched = run(PropertyArgs {
            waves: PathBuf::from(fixture.path()),
            from: None,
            to: None,
            scope: Some("top".to_string()),
            on: None,
            eval: "sig".to_string(),
            capture: CaptureMode::Switch,
            json: true,
        })
        .expect("switch capture should succeed");
        let CommandData::Property(rows) = switched.data else {
            panic!("property command should return property rows");
        };
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].kind, PropertyResultKind::Assert);
        assert_eq!(rows[0].time, "5ns");
        assert_eq!(rows[1].kind, PropertyResultKind::Deassert);
        assert_eq!(rows[1].time, "10ns");

        let assert_only = run(PropertyArgs {
            waves: PathBuf::from(fixture.path()),
            from: Some("0ns".to_string()),
            to: Some("10ns".to_string()),
            scope: Some("top".to_string()),
            on: None,
            eval: "sig".to_string(),
            capture: CaptureMode::Assert,
            json: false,
        })
        .expect("assert capture should succeed");
        let CommandData::Property(rows) = assert_only.data else {
            panic!("property command should return property rows");
        };
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].kind, PropertyResultKind::Assert);

        let error = run(PropertyArgs {
            waves: PathBuf::from(fixture.path()),
            from: Some("10ns".to_string()),
            to: Some("0ns".to_string()),
            scope: Some("top".to_string()),
            on: Some("posedge sig".to_string()),
            eval: "sig".to_string(),
            capture: CaptureMode::Match,
            json: false,
        })
        .expect_err("reversed time bounds should fail");
        assert!(
            error
                .to_string()
                .contains("--from must be less than or equal to --to")
        );
    }

    #[test]
    fn property_run_rejects_signal_free_wildcard_inference() {
        let fixture = write_fixture(TEST_VCD, ".property-run-error.vcd");
        let error = run(PropertyArgs {
            waves: PathBuf::from(fixture.path()),
            from: None,
            to: None,
            scope: Some("top".to_string()),
            on: None,
            eval: "1'b1".to_string(),
            capture: CaptureMode::Match,
            json: false,
        })
        .expect_err("signal-free wildcard trigger should fail");
        assert!(
            error
                .to_string()
                .contains("wildcard trigger cannot infer tracked signals")
        );
    }

    fn write_fixture(contents: &str, suffix: &str) -> NamedTempFile {
        let fixture = NamedTempFile::with_suffix(suffix).expect("fixture should create");
        fs::write(fixture.path(), contents).expect("fixture should write");
        fixture
    }
}
