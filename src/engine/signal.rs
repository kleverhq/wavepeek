use crate::cli::signal::SignalArgs;
use crate::engine::{CommandData, CommandName, CommandResult};
use crate::error::WavepeekError;
use crate::waveform::Waveform;
use regex::Regex;
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SignalEntry {
    pub name: String,
    pub path: String,
    pub kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub width: Option<u32>,
}

pub fn run(args: SignalArgs) -> Result<CommandResult, WavepeekError> {
    if args.max == 0 {
        return Err(WavepeekError::Args(
            "--max must be greater than 0. See 'wavepeek signal --help'.".to_string(),
        ));
    }

    let filter = Regex::new(args.filter.as_str()).map_err(|error| {
        WavepeekError::Args(format!(
            "invalid regex '{}': {error}. See 'wavepeek signal --help'.",
            args.filter
        ))
    })?;

    let waveform = Waveform::open(args.waves.as_path())?;
    let mut entries = waveform
        .signals_in_scope(args.scope.as_str())?
        .into_iter()
        .filter(|entry| filter.is_match(entry.name.as_str()))
        .map(|entry| SignalEntry {
            name: entry.name,
            path: entry.path,
            kind: entry.kind,
            width: entry.width,
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
        command: CommandName::Signal,
        json: args.json,
        human_options: crate::engine::HumanRenderOptions {
            scope_tree: false,
            signals_abs: args.abs,
        },
        data: CommandData::Signal(entries),
        warnings,
    })
}
