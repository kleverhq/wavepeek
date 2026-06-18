use crate::cli::limits::LimitArg;
use crate::cli::signal::SignalArgs;
use crate::debug_trace::DebugTrace;
use crate::diagnostic::{Diagnostic, WarningDiagnosticCode};
use crate::engine::{CommandData, CommandName, CommandResult};
use crate::error::WavepeekError;
use crate::waveform::Waveform;
use regex::Regex;
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SignalEntry {
    #[serde(skip_serializing)]
    pub display: String,
    pub name: String,
    pub path: String,
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
        json,
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
    if max.is_unlimited() {
        diagnostics.push(Diagnostic::warning(
            WarningDiagnosticCode::LimitDisabled,
            "limit disabled: --max=unlimited",
        ));
    }
    if max_depth == LimitArg::Unlimited {
        diagnostics.push(Diagnostic::warning(
            WarningDiagnosticCode::LimitDisabled,
            "limit disabled: --max-depth=unlimited",
        ));
    }

    let effective_max_depth = match max_depth {
        LimitArg::Numeric(value) => Some(value),
        LimitArg::Unlimited => None,
    };
    let scope_prefix = format!("{scope}.");

    let debug = DebugTrace::for_command(CommandName::Signal);
    debug.event("backend.open.start", || serde_json::json!({}));
    let waveform = Waveform::open(waves.as_path())?;
    debug.event("backend.open.done", || {
        serde_json::json!({
            "backend": waveform.backend_name(),
            "format": waveform.format_name(),
        })
    });
    let waveform_entries = if recursive {
        waveform.signals_in_scope_recursive(scope.as_str(), effective_max_depth)?
    } else {
        waveform.signals_in_scope(scope.as_str())?
    };
    debug.event(
        "signal.list.done",
        || serde_json::json!({"signals": waveform_entries.len()}),
    );
    let mut entries = waveform_entries
        .into_iter()
        .filter(|entry| filter.is_match(entry.name.as_str()))
        .map(|entry| SignalEntry {
            display: signal_display_name(
                recursive,
                scope_prefix.as_str(),
                entry.path.as_str(),
                entry.name.as_str(),
            ),
            name: entry.name,
            path: entry.path,
            kind: entry.kind,
            width: entry.width,
        })
        .collect::<Vec<_>>();
    debug.event(
        "signal.filter.done",
        || serde_json::json!({"signals": entries.len()}),
    );

    if let Some(max_entries) = max.numeric()
        && entries.len() > max_entries
    {
        entries.truncate(max_entries);
        diagnostics.push(Diagnostic::warning(
            WarningDiagnosticCode::OutputTruncated,
            format!("truncated output to {max_entries} entries (use --max to increase limit)"),
        ));
    }

    if entries.is_empty() {
        diagnostics.push(Diagnostic::warning(
            WarningDiagnosticCode::EmptyResult,
            "no signals found in selected scope",
        ));
    }

    Ok(CommandResult {
        command: CommandName::Signal,
        json,
        human_options: crate::engine::HumanRenderOptions {
            scope_tree: false,
            signals_abs: abs,
        },
        data: CommandData::Signal(entries),
        diagnostics,
    })
}

fn signal_display_name(recursive: bool, scope_prefix: &str, path: &str, name: &str) -> String {
    if !recursive {
        return name.to_string();
    }

    path.strip_prefix(scope_prefix).unwrap_or(name).to_string()
}
