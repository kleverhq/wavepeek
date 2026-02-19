use crate::cli::change::ChangeArgs;
use crate::engine::CommandResult;
use crate::error::WavepeekError;

pub fn run(_args: ChangeArgs) -> Result<CommandResult, WavepeekError> {
    Err(WavepeekError::Unimplemented(
        "`change` command execution is not implemented yet",
    ))
}
