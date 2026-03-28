use serde::Serialize;

use crate::cli::property::{CaptureMode, PropertyArgs};
use crate::engine::expr_runtime::{
    bind_waveform_event_expr, bind_waveform_logical_expr, candidate_sources_for_handles,
    eval_bound_logical_truth, event_candidate_handles, event_expr_contains_wildcard,
    event_expr_matches, open_shared_waveform, referenced_signal_handles,
};
use crate::engine::time::{
    DumpTimeContext, TimeValidationError, format_raw_timestamp, parse_dump_time_context,
    validate_time_token_to_raw,
};
use crate::engine::{CommandData, CommandName, CommandResult, HumanRenderOptions};
use crate::error::WavepeekError;
use crate::expr::EventEvalFrame;
use crate::waveform::ChangeCandidateCollectionMode;

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

    let tracked_signal_handles = if event_expr_contains_wildcard(&bound_event) {
        let handles = referenced_signal_handles(&bound_eval);
        if handles.is_empty() {
            return Err(WavepeekError::Args(
                "wildcard trigger cannot infer tracked signals from --eval; pass --on explicitly"
                    .to_string(),
            ));
        }
        handles
    } else {
        Vec::new()
    };

    let mut candidate_sources = if tracked_signal_handles.is_empty() {
        Vec::new()
    } else {
        candidate_sources_for_handles(&host, tracked_signal_handles.as_slice())?
    };
    candidate_sources.extend(candidate_sources_for_handles(
        &host,
        &event_candidate_handles(&bound_event),
    )?);
    let mut seen = std::collections::HashSet::new();
    candidate_sources.retain(|signal| seen.insert(signal.signal_ref));

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
        build_candidate_schedule(
            waveform_ref.timestamps_raw_slice(),
            candidate_times.as_slice(),
        )?
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
