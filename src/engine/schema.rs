use crate::cli::schema::SchemaArgs;
use crate::engine::{CommandData, CommandName, CommandResult, HumanRenderOptions};
use crate::error::WavepeekError;
use crate::schema_contract::{CANONICAL_SCHEMA_JSON, CANONICAL_STREAM_SCHEMA_JSON};

pub fn run(args: SchemaArgs) -> Result<CommandResult, WavepeekError> {
    let schema = if args.stream {
        CANONICAL_STREAM_SCHEMA_JSON
    } else {
        CANONICAL_SCHEMA_JSON
    };

    Ok(CommandResult {
        command: CommandName::Schema,
        output_mode: crate::output_mode::OutputMode::Human,
        human_options: HumanRenderOptions::default(),
        data: CommandData::Schema(schema.to_string()),
        diagnostics: Vec::new(),
    })
}
