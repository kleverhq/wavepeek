use crate::cli::limits::LimitArg;
use crate::cli::scope::ScopeArgs;
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

    let mut warnings = Vec::new();
    if max.is_unlimited() {
        warnings.push("limit disabled: --max=unlimited".to_string());
    }
    if max_depth.is_unlimited() {
        warnings.push("limit disabled: --max-depth=unlimited".to_string());
    }

    let waveform = Waveform::open(waves.as_path())?;
    let mut entries = waveform
        .scopes_depth_first(max_depth.numeric())
        .into_iter()
        .filter(|entry| filter.is_match(entry.path.as_str()))
        .map(|entry| ScopeEntry {
            path: entry.path,
            depth: entry.depth,
            kind: entry.kind,
        })
        .collect::<Vec<_>>();

    if let Some(max_entries) = max.numeric()
        && entries.len() > max_entries
    {
        entries.truncate(max_entries);
        warnings.push(format!(
            "truncated output to {} entries (use --max to increase limit)",
            max_entries
        ));
    }

    Ok(CommandResult {
        command: CommandName::Scope,
        json,
        human_options: crate::engine::HumanRenderOptions {
            scope_tree: tree,
            signals_abs: false,
        },
        data: CommandData::Scope(entries),
        warnings,
    })
}
