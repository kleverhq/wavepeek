pub mod change;
pub mod docs;
mod expr_runtime;
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
    Docs(cli::docs::DocsArgs),
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandName {
    Schema,
    Info,
    Scope,
    Signal,
    Value,
    Change,
    Property,
    Docs,
    DocsTopics,
    DocsShow,
    DocsSearch,
    DocsExport,
    DocsSkill,
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
            Self::Property => "property",
            Self::Docs => "docs",
            Self::DocsTopics => "docs topics",
            Self::DocsShow => "docs show",
            Self::DocsSearch => "docs search",
            Self::DocsExport => "docs export",
            Self::DocsSkill => "docs skill",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DocsTopicsData {
    pub topics: Vec<crate::docs::TopicSummary>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DocsSearchMatchData {
    pub topic: crate::docs::TopicSummary,
    pub match_kind: crate::docs::MatchKind,
    pub matched_tokens: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DocsSearchData {
    pub query: String,
    pub full_text: bool,
    pub matches: Vec<DocsSearchMatchData>,
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
    Text(String),
    Info(info::InfoData),
    Scope(Vec<scope::ScopeEntry>),
    Signal(Vec<signal::SignalEntry>),
    Value(value::ValueData),
    Change(Vec<change::ChangeSnapshot>),
    Property(Vec<property::PropertyCaptureRow>),
    DocsTopics(DocsTopicsData),
    DocsSearch(DocsSearchData),
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
        Command::Docs(args) => docs::run(args),
    }
}
