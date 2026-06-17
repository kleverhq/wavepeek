use crate::cli::limits::LimitArg;
use crate::cli::scope::ScopeArgs;
use crate::diagnostic::{Diagnostic, WarningDiagnosticCode};
use crate::engine::{CommandData, CommandName, CommandResult};
use crate::error::WavepeekError;
use crate::perf_diag::PerfDiagnostics;
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

    let mut perf = PerfDiagnostics::for_command(CommandName::Scope);
    let waveform = perf.time_phase("backend.open", || Waveform::open(waves.as_path()))?;
    perf.record_context(waveform.backend_name(), waveform.format_name());
    let waveform_entries = perf.time_phase_with_metrics(
        "hierarchy.load",
        || waveform.scopes_depth_first(max_depth.numeric()),
        |entries| Some(serde_json::json!({"scopes": entries.len()})),
    )?;
    let mut entries = perf.time_phase_with_metrics(
        "scope.filter",
        || {
            Ok(waveform_entries
                .into_iter()
                .filter(|entry| filter.is_match(entry.path.as_str()))
                .map(|entry| ScopeEntry {
                    path: entry.path,
                    depth: entry.depth,
                    kind: entry.kind,
                })
                .collect::<Vec<_>>())
        },
        |entries| Some(serde_json::json!({"scopes": entries.len()})),
    )?;

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

    diagnostics.extend(perf.finish());

    Ok(CommandResult {
        command: CommandName::Scope,
        json,
        human_options: crate::engine::HumanRenderOptions {
            scope_tree: tree,
            signals_abs: false,
        },
        data: CommandData::Scope(entries),
        diagnostics,
    })
}
