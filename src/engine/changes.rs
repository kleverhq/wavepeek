use crate::cli::changes::ChangesArgs;
use crate::error::WavepeekError;

pub fn run(_args: ChangesArgs) -> Result<(), WavepeekError> {
    Err(WavepeekError::Unimplemented(
        "`changes` command execution is not implemented yet",
    ))
}
