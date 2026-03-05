use crate::cli::property::PropertyArgs;
use crate::engine::CommandResult;
use crate::error::WavepeekError;

pub fn run(_args: PropertyArgs) -> Result<CommandResult, WavepeekError> {
    Err(WavepeekError::Unimplemented(
        "`property` command execution is not implemented yet",
    ))
}
