use crate::cli::limits::LimitArg;
use crate::cli::scope::ScopeArgs;
use crate::debug_trace::DebugTrace;
use crate::diagnostic::{Diagnostic, WarningDiagnosticCode};
use crate::engine::output_paths::OutputPaths;
use crate::engine::{CommandData, CommandName, CommandResult, ResultSummary};
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
        summary,
        path_output,
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

    let debug = DebugTrace::for_command(CommandName::Scope);
    debug.event("backend.open.start", || serde_json::json!({}));
    let waveform = Waveform::open(waves.as_path())?;
    debug.event("backend.open.done", || {
        serde_json::json!({
            "backend": waveform.backend_name(),
            "format": waveform.format_name(),
        })
    });
    let scopes = waveform.scopes_depth_first(max_depth.numeric())?;
    let include_ancestors = tree && !json && !jsonl;
    let mut included = vec![false; scopes.len()];
    let mut ancestors = Vec::new();
    let mut total = 0;

    for (index, entry) in scopes.iter().enumerate() {
        ancestors.truncate(entry.depth);
        if filter.is_match(entry.path.as_str()) {
            total += 1;
            if max.numeric().is_none_or(|limit| total <= limit) {
                included[index] = true;
                if include_ancestors {
                    for &ancestor in &ancestors {
                        included[ancestor] = true;
                    }
                }
            }
        }
        ancestors.push(index);
    }

    let paths = OutputPaths::new(path_output.style(), None, true)?;
    let entries = scopes
        .into_iter()
        .zip(included)
        .filter(|(_, included)| *included)
        .map(|(entry, _)| {
            Ok(ScopeEntry {
                path: if summary {
                    entry.path
                } else {
                    paths.exact(entry.path)?
                },
                depth: entry.depth,
                kind: entry.kind,
            })
        })
        .collect::<Result<Vec<_>, WavepeekError>>()?;
    debug.event(
        "scope.collect.done",
        || serde_json::json!({"scopes": entries.len()}),
    );

    let returned = max.numeric().map_or(total, |limit| total.min(limit));
    if returned < total {
        let entries = if include_ancestors {
            "matching entries"
        } else {
            "entries"
        };
        diagnostics.push(Diagnostic::warning(
            WarningDiagnosticCode::OutputTruncated,
            format!("truncated output to {returned} {entries} (use --max to increase limit)"),
        ));
    }

    Ok(CommandResult {
        command: CommandName::Scope,
        output_mode: crate::output_mode::OutputMode::from_json_flags(json, jsonl),
        human_options: crate::engine::HumanRenderOptions {
            scope_tree: tree,
            signals_abs: false,
        },
        scope: None,
        summary_only: summary,
        summary: Some(ResultSummary {
            complete: returned == total,
            returned,
            limit: max.numeric(),
            total: Some(total),
        }),
        data: CommandData::Scope(entries),
        diagnostics,
    })
}
