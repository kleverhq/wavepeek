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
    if args.max == 0 {
        return Err(WavepeekError::Args(
            "--max must be greater than 0. See 'wavepeek scope --help'.".to_string(),
        ));
    }

    let filter = Regex::new(args.filter.as_str()).map_err(|error| {
        WavepeekError::Args(format!(
            "invalid regex '{}': {error}. See 'wavepeek scope --help'.",
            args.filter
        ))
    })?;

    let waveform = Waveform::open(args.waves.as_path())?;
    let mut entries = waveform
        .scopes_depth_first(args.max_depth)
        .into_iter()
        .filter(|entry| filter.is_match(entry.path.as_str()))
        .map(|entry| ScopeEntry {
            path: entry.path,
            depth: entry.depth,
            kind: entry.kind,
        })
        .collect::<Vec<_>>();

    let mut warnings = Vec::new();
    if entries.len() > args.max {
        entries.truncate(args.max);
        warnings.push(format!(
            "truncated output to {} entries (use --max to increase limit)",
            args.max
        ));
    }

    Ok(CommandResult {
        command: CommandName::Scope,
        json: args.json,
        human_options: crate::engine::HumanRenderOptions {
            scope_tree: args.tree,
            signals_abs: false,
        },
        data: CommandData::Scope(entries),
        warnings,
    })
}
