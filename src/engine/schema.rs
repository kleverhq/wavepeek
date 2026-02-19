use crate::cli::schema::SchemaArgs;
use crate::engine::{CommandData, CommandName, CommandResult, HumanRenderOptions};
use crate::error::WavepeekError;
use crate::schema_contract::CANONICAL_SCHEMA_JSON;

pub fn run(_args: SchemaArgs) -> Result<CommandResult, WavepeekError> {
    Ok(CommandResult {
        command: CommandName::Schema,
        json: false,
        human_options: HumanRenderOptions::default(),
        data: CommandData::Schema(CANONICAL_SCHEMA_JSON.to_string()),
        warnings: Vec::new(),
    })
}
