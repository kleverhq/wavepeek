use crate::cli::info::InfoArgs;
use crate::error::WavepeekError;

pub fn run(_args: InfoArgs) -> Result<(), WavepeekError> {
    Err(WavepeekError::Unimplemented(
        "`info` command execution is not implemented yet",
    ))
}
