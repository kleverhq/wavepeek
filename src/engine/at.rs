use crate::cli::at::AtArgs;
use crate::error::WavepeekError;

pub fn run(_args: AtArgs) -> Result<(), WavepeekError> {
    Err(WavepeekError::Unimplemented(
        "`at` command execution is not implemented yet",
    ))
}
