use crate::cli::limits::LimitArg;
use crate::cli::signal::SignalArgs;
use crate::debug_trace::DebugTrace;
use crate::diagnostic::{Diagnostic, WarningDiagnosticCode};
use crate::engine::{CommandData, CommandName, CommandResult, ResultSummary};
use crate::error::WavepeekError;
use crate::waveform::{Waveform, display_signal_path};
use regex::Regex;
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SignalEntry {
    pub name: String,
    pub path: String,
    pub relative_path: String,
    pub kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub width: Option<u32>,
}

pub fn run(args: SignalArgs) -> Result<CommandResult, WavepeekError> {
    let SignalArgs {
        waves,
        scope,
        max,
        filter,
        recursive,
        max_depth,
        abs,
        summary,
        json,
        jsonl,
    } = args;

    if max == LimitArg::Numeric(0) {
        return Err(WavepeekError::Args(
            "--max must be greater than 0. See 'wavepeek signal --help'.".to_string(),
        ));
    }

    let filter = Regex::new(filter.as_str()).map_err(|error| {
        WavepeekError::Args(format!(
            "invalid regex '{}': {error}. See 'wavepeek signal --help'.",
            filter
        ))
    })?;

    let mut diagnostics = Vec::new();

    let effective_max_depth = match max_depth {
        LimitArg::Numeric(value) => Some(value),
        LimitArg::Unlimited => None,
    };

    let debug = DebugTrace::for_command(CommandName::Signal);
    debug.event("backend.open.start", || serde_json::json!({}));
    let waveform = Waveform::open(waves.as_path())?;
    debug.event("backend.open.done", || {
        serde_json::json!({
            "backend": waveform.backend_name(),
            "format": waveform.format_name(),
        })
    });
    let listing = if recursive {
        waveform.signals_in_scope_recursive_report(scope.as_str(), effective_max_depth)?
    } else {
        waveform.signals_in_scope_report(scope.as_str())?
    };
    debug.event(
        "signal.list.done",
        || serde_json::json!({"signals": listing.entries.len()}),
    );
    if !listing.omitted_ambiguous_paths.is_empty() {
        diagnostics.push(ambiguous_signal_warning(
            listing.omitted_ambiguous_paths.as_slice(),
        ));
    }
    let mut entries = listing
        .entries
        .into_iter()
        .filter(|entry| filter.is_match(entry.name.as_str()))
        .map(|entry| {
            let relative_path =
                display_signal_path(entry.path.as_str(), Some(scope.as_str())).to_string();
            SignalEntry {
                name: entry.name,
                path: entry.path,
                relative_path,
                kind: entry.kind,
                width: entry.width,
            }
        })
        .collect::<Vec<_>>();
    debug.event(
        "signal.filter.done",
        || serde_json::json!({"signals": entries.len()}),
    );

    let total = entries.len();
    if let Some(max_entries) = max.numeric()
        && entries.len() > max_entries
    {
        entries.truncate(max_entries);
        diagnostics.push(Diagnostic::warning(
            WarningDiagnosticCode::OutputTruncated,
            format!("truncated output to {max_entries} entries (use --max to increase limit)"),
        ));
    }

    Ok(CommandResult {
        command: CommandName::Signal,
        output_mode: crate::output_mode::OutputMode::from_json_flags(json, jsonl),
        human_options: crate::engine::HumanRenderOptions {
            scope_tree: false,
            signals_abs: abs,
        },
        scope: Some(scope),
        summary_only: summary,
        summary: Some(ResultSummary {
            complete: entries.len() == total,
            returned: entries.len(),
            limit: max.numeric(),
            total: Some(total),
        }),
        data: CommandData::Signal(entries),
        diagnostics,
    })
}

fn ambiguous_signal_warning(paths: &[String]) -> Diagnostic {
    Diagnostic::warning(
        WarningDiagnosticCode::AmbiguousSignalsOmitted,
        format!(
            "omitted ambiguous signal paths: count={}, first='{}'; multiple source declarations; no candidate was selected",
            paths.len(),
            paths[0]
        ),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ambiguous_signal_warning_is_stable_and_bounded() {
        let diagnostic =
            ambiguous_signal_warning(&["top.opcode".to_string(), "top.second_opcode".to_string()]);

        assert_eq!(diagnostic.code(), Some("WPK-W0005"));
        assert_eq!(
            diagnostic.message(),
            "omitted ambiguous signal paths: count=2, first='top.opcode'; multiple source declarations; no candidate was selected"
        );
    }
}
