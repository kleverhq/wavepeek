use serde::Serialize;

use crate::diagnostic::Diagnostic;
use crate::engine::{CommandData, CommandName, CommandResult};
use crate::error::WavepeekError;

use super::common::{ContractDiagnostic, validate_scope_kind, validate_signal_kind};
use super::schema::OUTPUT_SCHEMA_URL;

#[derive(Debug, Serialize)]
pub struct OutputEnvelope<'a> {
    #[serde(rename = "$schema")]
    schema: &'static str,
    command: &'static str,
    data: OutputData<'a>,
    diagnostics: Vec<ContractDiagnostic<'a>>,
}

impl<'a> OutputEnvelope<'a> {
    pub fn from_result(result: &'a CommandResult) -> Result<Self, WavepeekError> {
        Ok(Self {
            schema: OUTPUT_SCHEMA_URL,
            command: result.command.as_str(),
            data: OutputData::from_command_data(result.command, &result.data)?,
            diagnostics: diagnostics(&result.diagnostics)?,
        })
    }
}

fn diagnostics(diagnostics: &[Diagnostic]) -> Result<Vec<ContractDiagnostic<'_>>, WavepeekError> {
    diagnostics
        .iter()
        .map(ContractDiagnostic::from_diagnostic)
        .collect()
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum OutputData<'a> {
    Info(InfoData<'a>),
    Scope(Vec<ScopeEntry<'a>>),
    Signal(Vec<SignalEntry<'a>>),
    Value(Vec<ValueSnapshot<'a>>),
    Change(Vec<ChangeSnapshot<'a>>),
    Property(Vec<PropertyRow<'a>>),
    DocsTopics(DocsTopicsData<'a>),
    DocsSearch(DocsSearchData<'a>),
}

impl<'a> OutputData<'a> {
    pub fn from_command_data(
        command: CommandName,
        data: &'a CommandData,
    ) -> Result<Self, WavepeekError> {
        match (command, data) {
            (CommandName::Info, CommandData::Info(data)) => Ok(Self::Info(InfoData::from(data))),
            (CommandName::Scope, CommandData::Scope(entries)) => entries
                .iter()
                .map(ScopeEntry::try_from)
                .collect::<Result<Vec<_>, _>>()
                .map(Self::Scope),
            (CommandName::Signal, CommandData::Signal(entries)) => entries
                .iter()
                .map(SignalEntry::try_from)
                .collect::<Result<Vec<_>, _>>()
                .map(Self::Signal),
            (CommandName::Value, CommandData::Value(snapshots)) => Ok(Self::Value(
                snapshots.iter().map(ValueSnapshot::from).collect(),
            )),
            (CommandName::Change, CommandData::Change(snapshots)) => Ok(Self::Change(
                snapshots.iter().map(ChangeSnapshot::from).collect(),
            )),
            (CommandName::Property, CommandData::Property(rows)) => {
                Ok(Self::Property(rows.iter().map(PropertyRow::from).collect()))
            }
            (CommandName::DocsTopics, CommandData::DocsTopics(data)) => {
                Ok(Self::DocsTopics(DocsTopicsData::from(data)))
            }
            (CommandName::DocsSearch, CommandData::DocsSearch(data)) => {
                Ok(Self::DocsSearch(DocsSearchData::from(data)))
            }
            _ => Err(WavepeekError::Internal(format!(
                "command {} cannot be serialized as a JSON contract envelope",
                command.as_str()
            ))),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct InfoData<'a> {
    time_unit: &'a str,
    time_start: &'a str,
    time_end: &'a str,
}

impl<'a> From<&'a crate::engine::info::InfoData> for InfoData<'a> {
    fn from(data: &'a crate::engine::info::InfoData) -> Self {
        Self {
            time_unit: data.time_unit.as_str(),
            time_start: data.time_start.as_str(),
            time_end: data.time_end.as_str(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ScopeEntry<'a> {
    path: &'a str,
    depth: usize,
    kind: &'a str,
}

impl<'a> TryFrom<&'a crate::engine::scope::ScopeEntry> for ScopeEntry<'a> {
    type Error = WavepeekError;

    fn try_from(entry: &'a crate::engine::scope::ScopeEntry) -> Result<Self, Self::Error> {
        Ok(Self {
            path: entry.path.as_str(),
            depth: entry.depth,
            kind: validate_scope_kind(entry.kind.as_str())?,
        })
    }
}

#[derive(Debug, Serialize)]
pub struct SignalEntry<'a> {
    name: &'a str,
    path: &'a str,
    kind: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    width: Option<u32>,
}

impl<'a> TryFrom<&'a crate::engine::signal::SignalEntry> for SignalEntry<'a> {
    type Error = WavepeekError;

    fn try_from(entry: &'a crate::engine::signal::SignalEntry) -> Result<Self, Self::Error> {
        Ok(Self {
            name: entry.name.as_str(),
            path: entry.path.as_str(),
            kind: validate_signal_kind(entry.kind.as_str())?,
            width: entry.width,
        })
    }
}

#[derive(Debug, Serialize)]
pub struct SampledSignalValue<'a> {
    path: &'a str,
    value: &'a str,
}

impl<'a> From<&'a crate::engine::value::ValueSignalValue> for SampledSignalValue<'a> {
    fn from(signal: &'a crate::engine::value::ValueSignalValue) -> Self {
        Self {
            path: signal.path.as_str(),
            value: signal.value.as_str(),
        }
    }
}

impl<'a> From<&'a crate::engine::change::ChangeSignalValue> for SampledSignalValue<'a> {
    fn from(signal: &'a crate::engine::change::ChangeSignalValue) -> Self {
        Self {
            path: signal.path.as_str(),
            value: signal.value.as_str(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ValueSnapshot<'a> {
    time: &'a str,
    signals: Vec<SampledSignalValue<'a>>,
}

impl<'a> From<&'a crate::engine::value::ValueSnapshot> for ValueSnapshot<'a> {
    fn from(snapshot: &'a crate::engine::value::ValueSnapshot) -> Self {
        Self {
            time: snapshot.time.as_str(),
            signals: snapshot
                .signals
                .iter()
                .map(SampledSignalValue::from)
                .collect(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ChangeSnapshot<'a> {
    time: &'a str,
    sample_time: &'a str,
    signals: Vec<SampledSignalValue<'a>>,
}

impl<'a> From<&'a crate::engine::change::ChangeSnapshot> for ChangeSnapshot<'a> {
    fn from(snapshot: &'a crate::engine::change::ChangeSnapshot) -> Self {
        Self {
            time: snapshot.time.as_str(),
            sample_time: snapshot.sample_time.as_str(),
            signals: snapshot
                .signals
                .iter()
                .map(SampledSignalValue::from)
                .collect(),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum PropertyKind {
    Match,
    Assert,
    Deassert,
}

impl From<crate::engine::property::PropertyResultKind> for PropertyKind {
    fn from(kind: crate::engine::property::PropertyResultKind) -> Self {
        match kind {
            crate::engine::property::PropertyResultKind::Match => Self::Match,
            crate::engine::property::PropertyResultKind::Assert => Self::Assert,
            crate::engine::property::PropertyResultKind::Deassert => Self::Deassert,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct PropertyRow<'a> {
    time: &'a str,
    sample_time: &'a str,
    kind: PropertyKind,
}

impl<'a> From<&'a crate::engine::property::PropertyCaptureRow> for PropertyRow<'a> {
    fn from(row: &'a crate::engine::property::PropertyCaptureRow) -> Self {
        Self {
            time: row.time.as_str(),
            sample_time: row.sample_time.as_str(),
            kind: row.kind.into(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct TopicSummary<'a> {
    id: &'a str,
    title: &'a str,
    description: &'a str,
    section: &'a str,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    see_also: Vec<&'a str>,
}

impl<'a> From<&'a crate::docs::TopicSummary> for TopicSummary<'a> {
    fn from(topic: &'a crate::docs::TopicSummary) -> Self {
        Self {
            id: topic.id.as_str(),
            title: topic.title.as_str(),
            description: topic.description.as_str(),
            section: topic.section.as_str(),
            see_also: topic.see_also.iter().map(String::as_str).collect(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct DocsTopicsData<'a> {
    topics: Vec<TopicSummary<'a>>,
}

impl<'a> From<&'a crate::engine::DocsTopicsData> for DocsTopicsData<'a> {
    fn from(data: &'a crate::engine::DocsTopicsData) -> Self {
        Self {
            topics: data.topics.iter().map(TopicSummary::from).collect(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct DocsSearchMatch<'a> {
    topic: TopicSummary<'a>,
    match_kind: &'a crate::docs::MatchKind,
    matched_tokens: usize,
}

impl<'a> From<&'a crate::engine::DocsSearchMatchData> for DocsSearchMatch<'a> {
    fn from(entry: &'a crate::engine::DocsSearchMatchData) -> Self {
        Self {
            topic: TopicSummary::from(&entry.topic),
            match_kind: &entry.match_kind,
            matched_tokens: entry.matched_tokens,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct DocsSearchData<'a> {
    query: &'a str,
    matches: Vec<DocsSearchMatch<'a>>,
}

impl<'a> From<&'a crate::engine::DocsSearchData> for DocsSearchData<'a> {
    fn from(data: &'a crate::engine::DocsSearchData) -> Self {
        Self {
            query: data.query.as_str(),
            matches: data.matches.iter().map(DocsSearchMatch::from).collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use crate::engine::{CommandData, CommandName, CommandResult, HumanRenderOptions};
    use crate::output_mode::OutputMode;

    use super::OutputEnvelope;

    #[test]
    fn output_envelope_uses_contract_dto_not_engine_display_fields() {
        let result = CommandResult {
            command: CommandName::Value,
            output_mode: OutputMode::Json,
            human_options: HumanRenderOptions::default(),
            data: CommandData::Value(vec![crate::engine::value::ValueSnapshot {
                time: "5ns".to_string(),
                signals: vec![crate::engine::value::ValueSignalValue {
                    display: "sig".to_string(),
                    path: "top.sig".to_string(),
                    value: "1'h1".to_string(),
                }],
            }]),
            diagnostics: Vec::new(),
        };

        let value = serde_json::to_value(
            OutputEnvelope::from_result(&result).expect("result should convert to contract"),
        )
        .expect("contract envelope should serialize");
        assert_eq!(value["data"][0]["signals"][0]["path"], "top.sig");
        assert!(value["data"][0]["signals"][0].get("display").is_none());
    }

    #[test]
    fn output_envelope_rejects_non_machine_command_data() {
        let result = CommandResult {
            command: CommandName::Schema,
            output_mode: OutputMode::Json,
            human_options: HumanRenderOptions::default(),
            data: CommandData::Schema("{}".to_string()),
            diagnostics: Vec::new(),
        };

        assert!(OutputEnvelope::from_result(&result).is_err());
    }

    #[test]
    fn docs_topics_omits_empty_see_also() {
        let result = CommandResult {
            command: CommandName::DocsTopics,
            output_mode: OutputMode::Json,
            human_options: HumanRenderOptions::default(),
            data: CommandData::DocsTopics(crate::engine::DocsTopicsData {
                topics: vec![crate::docs::TopicSummary {
                    id: "intro".to_string(),
                    title: "Introduction".to_string(),
                    description: "Start here".to_string(),
                    section: "intro".to_string(),
                    see_also: Vec::new(),
                }],
            }),
            diagnostics: Vec::new(),
        };

        let value: Value = serde_json::to_value(
            OutputEnvelope::from_result(&result).expect("docs topics should convert"),
        )
        .expect("docs topics should serialize");
        assert!(value["data"]["topics"][0].get("see_also").is_none());
    }
}
