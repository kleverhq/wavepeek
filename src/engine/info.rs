use crate::cli::info::InfoArgs;
use crate::debug_trace::DebugTrace;
use crate::engine::{CommandData, CommandName, CommandResult};
use crate::error::WavepeekError;
use crate::waveform::Waveform;
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct InfoData {
    pub time_unit: String,
    pub time_start: String,
    pub time_end: String,
}

pub fn run(args: InfoArgs) -> Result<CommandResult, WavepeekError> {
    let debug = DebugTrace::for_command(CommandName::Info);
    debug.event("backend.open.start", || serde_json::json!({}));
    let waveform = Waveform::open(args.waves.as_path())?;
    debug.event("backend.open.done", || {
        serde_json::json!({
            "backend": waveform.backend_name(),
            "format": waveform.format_name(),
        })
    });
    let metadata = waveform.metadata()?;
    debug.event("metadata.load.done", || serde_json::json!({}));

    Ok(CommandResult {
        command: CommandName::Info,
        json: args.json,
        human_options: crate::engine::HumanRenderOptions::default(),
        data: CommandData::Info(InfoData {
            time_unit: metadata.time_unit,
            time_start: metadata.time_start,
            time_end: metadata.time_end,
        }),
        diagnostics: Vec::new(),
    })
}
