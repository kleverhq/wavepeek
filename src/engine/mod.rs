pub mod at;
pub mod change;
pub mod info;
pub mod schema;
pub mod scope;
pub mod signal;
pub mod when;

use serde::Serialize;

use crate::cli;
use crate::error::WavepeekError;

#[derive(Debug)]
pub enum Command {
    Schema(cli::schema::SchemaArgs),
    Info(cli::info::InfoArgs),
    Scope(cli::scope::ScopeArgs),
    Signal(cli::signal::SignalArgs),
    At(cli::at::AtArgs),
    Change(cli::change::ChangeArgs),
    When(cli::when::WhenArgs),
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandName {
    Schema,
    Info,
    Scope,
    Signal,
}

impl CommandName {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Schema => "schema",
            Self::Info => "info",
            Self::Scope => "scope",
            Self::Signal => "signal",
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
    Schema(String),
    Info(info::InfoData),
    Scope(Vec<scope::ScopeEntry>),
    Signal(Vec<signal::SignalEntry>),
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
        Command::At(args) => at::run(args),
        Command::Change(args) => change::run(args),
        Command::When(args) => when::run(args),
    }
}
