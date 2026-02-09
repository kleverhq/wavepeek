use crate::cli::schema::SchemaArgs;
use crate::error::WavepeekError;

pub fn run(_args: SchemaArgs) -> Result<(), WavepeekError> {
    Err(WavepeekError::Unimplemented(
        "`schema` command execution is not implemented yet",
    ))
}
