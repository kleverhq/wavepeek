use crate::cli::limits::LimitArg;
use crate::cli::scope::ScopeArgs;
use crate::debug_trace::DebugTrace;
use crate::diagnostic::{Diagnostic, WarningDiagnosticCode};
use crate::engine::{CommandData, CommandName, CommandResult};
use crate::error::WavepeekError;
use crate::waveform::Waveform;
use regex::Regex;
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ScopeEntry {
    pub path: String,
    pub depth: usize,
    pub kind: String,
}

pub fn run(args: ScopeArgs) -> Result<CommandResult, WavepeekError> {
    let ScopeArgs {
        waves,
        max,
        max_depth,
        filter,
        tree,
        json,
        jsonl,
    } = args;

    if max == LimitArg::Numeric(0) {
        return Err(WavepeekError::Args(
            "--max must be greater than 0. See 'wavepeek scope --help'.".to_string(),
        ));
    }

    let filter = Regex::new(filter.as_str()).map_err(|error| {
        WavepeekError::Args(format!(
            "invalid regex '{}': {error}. See 'wavepeek scope --help'.",
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
    if max_depth.is_unlimited() {
        diagnostics.push(Diagnostic::warning(
            WarningDiagnosticCode::LimitDisabled,
            "limit disabled: --max-depth=unlimited",
        ));
    }

    let debug = DebugTrace::for_command(CommandName::Scope);
    debug.event("backend.open.start", || serde_json::json!({}));
    let waveform = Waveform::open(waves.as_path())?;
    debug.event("backend.open.done", || {
        serde_json::json!({
            "backend": waveform.backend_name(),
            "format": waveform.format_name(),
        })
    });
    let mut entries = waveform
        .scopes_depth_first(max_depth.numeric())?
        .into_iter()
        .filter(|entry| filter.is_match(entry.path.as_str()))
        .map(|entry| ScopeEntry {
            path: entry.path,
            depth: entry.depth,
            kind: entry.kind,
        })
        .collect::<Vec<_>>();
    debug.event(
        "scope.collect.done",
        || serde_json::json!({"scopes": entries.len()}),
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
            "no scopes found",
        ));
    }

    Ok(CommandResult {
        command: CommandName::Scope,
        output_mode: crate::output_mode::OutputMode::from_json_flags(json, jsonl),
        human_options: crate::engine::HumanRenderOptions {
            scope_tree: tree,
            signals_abs: false,
        },
        data: CommandData::Scope(entries),
        diagnostics,
    })
}
