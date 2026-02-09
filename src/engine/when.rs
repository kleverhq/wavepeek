use crate::cli::when::WhenArgs;
use crate::error::WavepeekError;

pub fn run(_args: WhenArgs) -> Result<(), WavepeekError> {
    Err(WavepeekError::Unimplemented(
        "`when` command execution is not implemented yet",
    ))
}
