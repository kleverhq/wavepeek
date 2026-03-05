pub mod change;
pub mod info;
pub mod property;
pub mod schema;
pub mod scope;
pub mod signal;
pub mod time;
pub mod value;
mod value_format;

use serde::Serialize;

use crate::cli;
use crate::error::WavepeekError;

#[derive(Debug)]
pub enum Command {
    Schema(cli::schema::SchemaArgs),
    Info(cli::info::InfoArgs),
    Scope(cli::scope::ScopeArgs),
    Signal(cli::signal::SignalArgs),
    Value(cli::value::ValueArgs),
    Change(cli::change::ChangeArgs),
    Property(cli::property::PropertyArgs),
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandName {
    // `property` intentionally omitted while runtime remains unimplemented.
    Schema,
    Info,
    Scope,
    Signal,
    Value,
    Change,
}

impl CommandName {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Schema => "schema",
            Self::Info => "info",
            Self::Scope => "scope",
            Self::Signal => "signal",
            Self::Value => "value",
            Self::Change => "change",
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct HumanRenderOptions {
    pub scope_tree: bool,
    pub signals_abs: bool,
}

#[allow(dead_code)]
#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum CommandData {
    // `property` intentionally omitted while runtime remains unimplemented.
    Schema(String),
    Info(info::InfoData),
    Scope(Vec<scope::ScopeEntry>),
    Signal(Vec<signal::SignalEntry>),
    Value(value::ValueData),
    Change(Vec<change::ChangeSnapshot>),
}

#[derive(Debug, Serialize)]
pub struct CommandResult {
    #[serde(skip)]
    pub command: CommandName,
    #[serde(skip)]
    pub json: bool,
    #[serde(skip)]
    pub human_options: HumanRenderOptions,
    pub data: CommandData,
    pub warnings: Vec<String>,
}

pub fn run(command: Command) -> Result<CommandResult, WavepeekError> {
    match command {
        Command::Schema(args) => schema::run(args),
        Command::Info(args) => info::run(args),
        Command::Scope(args) => scope::run(args),
        Command::Signal(args) => signal::run(args),
        Command::Value(args) => value::run(args),
        Command::Change(args) => change::run(args),
        Command::Property(args) => property::run(args),
    }
}
