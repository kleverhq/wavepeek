pub mod ahb;
pub mod apb;
pub mod atb;
pub mod axi;
pub mod axistream;
pub mod change;
mod expr_runtime;
pub mod extract;
pub mod info;
pub mod property;
pub mod scope;
pub mod signal;
mod signal_mapping;
mod signal_projection;
pub mod skill;
pub mod time;
pub mod value;
mod value_format;

use serde::Serialize;

use crate::cli;
use crate::diagnostic::Diagnostic;
use crate::error::WavepeekError;
use crate::output::{self, JsonlWriter};
use crate::output_mode::OutputMode;

pub(crate) fn signal_token(text: &str) -> String {
    if text.contains('\\')
        && let Ok(path) = ondas::HierarchyPath::parse(text.trim_start())
    {
        return crate::waveform::format_path(&path);
    }
    text.trim().to_string()
}

pub(crate) fn scoped_signal_path(name: &str, scope: Option<&str>) -> String {
    let canonical = |text: &str| {
        ondas::HierarchyPath::parse(text.trim_start())
            .map(|path| crate::waveform::format_path(&path))
            .unwrap_or_else(|_| text.trim().to_string())
    };
    let name = canonical(name);
    match scope.map(canonical) {
        Some(scope) if scope.is_empty() => name,
        Some(scope)
            if name
                .strip_prefix(&scope)
                .is_some_and(|suffix| suffix.starts_with('.')) =>
        {
            name
        }
        Some(scope) => format!("{scope}.{name}"),
        None => name,
    }
}

#[derive(Debug)]
pub enum Command {
    Info(cli::info::InfoArgs),
    Scope(cli::scope::ScopeArgs),
    Signal(cli::signal::SignalArgs),
    Value(cli::value::ValueArgs),
    Change(cli::change::ChangeArgs),
    Property(cli::property::PropertyArgs),
    ExtractAhb(cli::extract::AhbArgs),
    ExtractApb(cli::extract::ApbArgs),
    ExtractAtb(cli::extract::AtbArgs),
    ExtractAxi(cli::extract::AxiArgs),
    ExtractAxiStream(cli::extract::AxiStreamArgs),
    ExtractGeneric(cli::extract::GenericArgs),
    Skill(cli::skill::SkillArgs),
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandName {
    Info,
    Scope,
    Signal,
    Value,
    Change,
    Property,
    ExtractAhb,
    ExtractApb,
    ExtractAtb,
    ExtractAxi,
    ExtractAxiStream,
    ExtractGeneric,
    Skill,
}

impl Command {
    pub const fn name(&self) -> CommandName {
        match self {
            Self::Info(_) => CommandName::Info,
            Self::Scope(_) => CommandName::Scope,
            Self::Signal(_) => CommandName::Signal,
            Self::Value(_) => CommandName::Value,
            Self::Change(_) => CommandName::Change,
            Self::Property(_) => CommandName::Property,
            Self::ExtractAhb(_) => CommandName::ExtractAhb,
            Self::ExtractApb(_) => CommandName::ExtractApb,
            Self::ExtractAtb(_) => CommandName::ExtractAtb,
            Self::ExtractAxi(_) => CommandName::ExtractAxi,
            Self::ExtractAxiStream(_) => CommandName::ExtractAxiStream,
            Self::ExtractGeneric(_) => CommandName::ExtractGeneric,
            Self::Skill(_) => CommandName::Skill,
        }
    }

    pub fn set_output_mode(&mut self, mode: OutputMode) {
        let (json, jsonl) = match mode {
            OutputMode::Human => (false, false),
            OutputMode::Json => (true, false),
            OutputMode::Jsonl => (false, true),
        };
        match self {
            Self::Info(args) => (args.json, args.jsonl) = (json, jsonl),
            Self::Scope(args) => (args.json, args.jsonl) = (json, jsonl),
            Self::Signal(args) => (args.json, args.jsonl) = (json, jsonl),
            Self::Value(args) => (args.json, args.jsonl) = (json, jsonl),
            Self::Change(args) => (args.json, args.jsonl) = (json, jsonl),
            Self::Property(args) => (args.json, args.jsonl) = (json, jsonl),
            Self::ExtractAhb(args) => (args.json, args.jsonl) = (json, jsonl),
            Self::ExtractApb(args) => (args.json, args.jsonl) = (json, jsonl),
            Self::ExtractAtb(args) => (args.json, args.jsonl) = (json, jsonl),
            Self::ExtractAxi(args) => (args.json, args.jsonl) = (json, jsonl),
            Self::ExtractAxiStream(args) => (args.json, args.jsonl) = (json, jsonl),
            Self::ExtractGeneric(args) => (args.json, args.jsonl) = (json, jsonl),
            Self::Skill(_) => {}
        }
    }
}

impl CommandName {
    pub const fn supports_scope_context(self) -> bool {
        matches!(
            self,
            Self::Signal | Self::Value | Self::Change | Self::ExtractGeneric
        )
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Info => "info",
            Self::Scope => "scope",
            Self::Signal => "signal",
            Self::Value => "value",
            Self::Change => "change",
            Self::Property => "property",
            Self::ExtractAhb => "extract ahb",
            Self::ExtractApb => "extract apb",
            Self::ExtractAtb => "extract atb",
            Self::ExtractAxi => "extract axi",
            Self::ExtractAxiStream => "extract axistream",
            Self::ExtractGeneric => "extract generic",
            Self::Skill => "skill",
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
    Text(String),
    Info(info::InfoData),
    Scope(Vec<scope::ScopeEntry>),
    Signal(Vec<signal::SignalEntry>),
    Value(value::ValueData),
    Change(Vec<change::ChangeSnapshot>),
    Property(Vec<property::PropertyCaptureRow>),
    ExtractAhb(ahb::AhbData),
    ExtractApb(apb::ApbData),
    ExtractAtb(atb::AtbData),
    ExtractAxi(axi::AxiData),
    ExtractAxiStream(axistream::AxiStreamData),
    ExtractGeneric(extract::ExtractGenericData),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct ResultSummary {
    pub complete: bool,
    pub returned: usize,
    pub limit: Option<usize>,
    pub total: Option<usize>,
}

impl ResultSummary {
    pub const fn from_run(returned: usize, limit: Option<usize>, truncated: bool) -> Self {
        Self {
            complete: !truncated,
            returned,
            limit,
            total: if truncated { None } else { Some(returned) },
        }
    }
}

#[derive(Debug, Serialize)]
pub struct CommandResult {
    #[serde(skip)]
    pub command: CommandName,
    #[serde(skip)]
    pub output_mode: OutputMode,
    #[serde(skip)]
    pub human_options: HumanRenderOptions,
    #[serde(skip)]
    pub scope: Option<String>,
    #[serde(skip)]
    pub summary_only: bool,
    pub data: CommandData,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<ResultSummary>,
    pub diagnostics: Vec<Diagnostic>,
}

pub fn run(command: Command) -> Result<CommandResult, WavepeekError> {
    match command {
        Command::Info(args) => info::run(args),
        Command::Scope(args) => scope::run(args),
        Command::Signal(args) => signal::run(args),
        Command::Value(args) => value::run(args),
        Command::Change(args) => change::run(args),
        Command::Property(args) => property::run(args),
        Command::ExtractAhb(args) => ahb::run(args),
        Command::ExtractApb(args) => apb::run(args),
        Command::ExtractAtb(args) => atb::run(args),
        Command::ExtractAxi(args) => axi::run(args),
        Command::ExtractAxiStream(args) => axistream::run(args),
        Command::ExtractGeneric(args) => extract::run(args),
        Command::Skill(args) => skill::run(args),
    }
}

pub fn run_jsonl<W: std::io::Write>(
    command: Command,
    writer: &mut JsonlWriter<W>,
) -> Result<(), WavepeekError> {
    match command {
        Command::Change(args) => change::run_jsonl(args, writer),
        Command::Property(args) => property::run_jsonl(args, writer),
        Command::ExtractAhb(args) => ahb::run_jsonl(args, writer),
        Command::ExtractApb(args) => apb::run_jsonl(args, writer),
        Command::ExtractAtb(args) => atb::run_jsonl(args, writer),
        Command::ExtractAxi(args) => axi::run_jsonl(args, writer),
        Command::ExtractAxiStream(args) => axistream::run_jsonl(args, writer),
        Command::ExtractGeneric(args) => extract::run_jsonl(args, writer),
        Command::Info(_) | Command::Scope(_) | Command::Signal(_) | Command::Value(_) => {
            let result = run(command)?;
            output::write_jsonl_result(result, writer)
        }
        Command::Skill(_) => Err(WavepeekError::Args(
            "--jsonl is available only for waveform commands".to_string(),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::{CommandName, scoped_signal_path};

    #[test]
    fn scoped_signal_path_resolves_relative_names() {
        for (name, scope, expected) in [
            ("top.cpu.valid", None, "top.cpu.valid"),
            ("valid", Some("top.cpu"), "top.cpu.valid"),
            ("cpu.valid", Some("top"), "top.cpu.valid"),
            ("top.cpu.valid", Some("top"), "top.cpu.valid"),
            ("topology.valid", Some("top"), "top.topology.valid"),
        ] {
            assert_eq!(scoped_signal_path(name, scope), expected);
        }
    }

    #[test]
    fn command_name_strings_exercise_all_variants() {
        assert_eq!(CommandName::Info.as_str(), "info");
        assert_eq!(CommandName::Scope.as_str(), "scope");
        assert_eq!(CommandName::Signal.as_str(), "signal");
        assert_eq!(CommandName::Value.as_str(), "value");
        assert_eq!(CommandName::Change.as_str(), "change");
        assert_eq!(CommandName::Property.as_str(), "property");
        assert_eq!(CommandName::ExtractAhb.as_str(), "extract ahb");
        assert_eq!(CommandName::ExtractApb.as_str(), "extract apb");
        assert_eq!(CommandName::ExtractAtb.as_str(), "extract atb");
        assert_eq!(CommandName::ExtractAxi.as_str(), "extract axi");
        assert_eq!(CommandName::ExtractAxiStream.as_str(), "extract axistream");
        assert_eq!(CommandName::ExtractGeneric.as_str(), "extract generic");
        assert_eq!(CommandName::Skill.as_str(), "skill");
    }
}
