use crate::cli::info::InfoArgs;
use crate::engine::{CommandData, CommandName, CommandResult};
use crate::error::WavepeekError;
use crate::perf_diag::PerfDiagnostics;
use crate::waveform::Waveform;
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct InfoData {
    pub time_unit: String,
    pub time_start: String,
    pub time_end: String,
}

pub fn run(args: InfoArgs) -> Result<CommandResult, WavepeekError> {
    let mut perf = PerfDiagnostics::for_command(CommandName::Info);
    let waveform = perf.time_phase("backend.open", || Waveform::open(args.waves.as_path()))?;
    perf.record_context(waveform.backend_name(), waveform.format_name());
    let metadata = perf.time_phase("metadata.load", || waveform.metadata())?;

    Ok(CommandResult {
        command: CommandName::Info,
        json: args.json,
        human_options: crate::engine::HumanRenderOptions::default(),
        data: CommandData::Info(InfoData {
            time_unit: metadata.time_unit,
            time_start: metadata.time_start,
            time_end: metadata.time_end,
        }),
        diagnostics: perf.finish(),
    })
}
