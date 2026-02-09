use crate::cli::info::InfoArgs;
use crate::engine::{CommandData, CommandName, CommandResult};
use crate::error::WavepeekError;
use crate::waveform::Waveform;
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct InfoData {
    pub time_unit: String,
    pub time_precision: String,
    pub time_start: String,
    pub time_end: String,
}

pub fn run(args: InfoArgs) -> Result<CommandResult, WavepeekError> {
    let waveform = Waveform::open(args.waves.as_path())?;
    let metadata = waveform.metadata()?;

    Ok(CommandResult {
        command: CommandName::Info,
        human: args.human,
        data: CommandData::Info(InfoData {
            time_unit: metadata.time_unit,
            time_precision: metadata.time_precision,
            time_start: metadata.time_start,
            time_end: metadata.time_end,
        }),
        warnings: Vec::new(),
    })
}
