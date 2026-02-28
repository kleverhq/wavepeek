use crate::cli::limits::LimitArg;
use crate::cli::signal::SignalArgs;
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

const DEFAULT_RECURSIVE_MAX_DEPTH: usize = 5;

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

    if max_depth.is_some() && !recursive {
        return Err(WavepeekError::Args(
            "--max-depth requires --recursive. See 'wavepeek signal --help'.".to_string(),
        ));
    }

    let filter = Regex::new(filter.as_str()).map_err(|error| {
        WavepeekError::Args(format!(
            "invalid regex '{}': {error}. See 'wavepeek signal --help'.",
            filter
        ))
    })?;

    let mut warnings = Vec::new();
    if max.is_unlimited() {
        warnings.push("limit disabled: --max=unlimited".to_string());
    }
    if max_depth == Some(LimitArg::Unlimited) {
        warnings.push("limit disabled: --max-depth=unlimited".to_string());
    }

    let effective_max_depth = match max_depth {
        Some(LimitArg::Numeric(value)) => Some(value),
        Some(LimitArg::Unlimited) => None,
        None => Some(DEFAULT_RECURSIVE_MAX_DEPTH),
    };
    let scope_prefix = format!("{scope}.");

    let waveform = Waveform::open(waves.as_path())?;
    let waveform_entries = if recursive {
        waveform.signals_in_scope_recursive(scope.as_str(), effective_max_depth)?
    } else {
        waveform.signals_in_scope(scope.as_str())?
    };
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
        command: CommandName::Signal,
        json,
        human_options: crate::engine::HumanRenderOptions {
            scope_tree: false,
            signals_abs: abs,
        },
        data: CommandData::Signal(entries),
        warnings,
    })
}

fn signal_display_name(recursive: bool, scope_prefix: &str, path: &str, name: &str) -> String {
    if !recursive {
        return name.to_string();
    }

    path.strip_prefix(scope_prefix).unwrap_or(name).to_string()
}
