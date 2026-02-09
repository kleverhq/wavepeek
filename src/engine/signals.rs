use crate::cli::signals::SignalsArgs;
use crate::error::WavepeekError;

pub fn run(_args: SignalsArgs) -> Result<(), WavepeekError> {
    Err(WavepeekError::Unimplemented(
        "`signals` command execution is not implemented yet",
    ))
}
