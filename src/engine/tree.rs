use crate::cli::tree::TreeArgs;
use crate::error::WavepeekError;

pub fn run(_args: TreeArgs) -> Result<(), WavepeekError> {
    Err(WavepeekError::Unimplemented(
        "`tree` command execution is not implemented yet",
    ))
}
