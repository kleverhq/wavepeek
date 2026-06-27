use std::borrow::Cow;

use schemars::{JsonSchema, Schema, SchemaGenerator, json_schema};
use serde::Serialize;

use crate::diagnostic::Diagnostic;
use crate::engine::{CommandData, CommandName, CommandResult};
use crate::error::WavepeekError;

use super::common::{
    CanonicalPath, ContractDiagnostic, NormalizedTime, SampledValue, ScopeKind, SignalKind,
    validate_scope_kind, validate_signal_kind,
};
use super::schema::OUTPUT_SCHEMA_URL;

#[derive(Debug, JsonSchema, Serialize)]
#[schemars(rename = "outputEnvelope")]
#[schemars(extend("additionalProperties" = true))]
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

#[derive(Debug, JsonSchema, Serialize)]
#[schemars(rename = "outputData")]
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

#[derive(Debug, JsonSchema, Serialize)]
#[schemars(rename = "infoData")]
#[schemars(extend("additionalProperties" = true))]
pub struct InfoData<'a> {
    #[schemars(
        description = "Dump time unit used to normalize timestamps in this waveform, for example 1ns."
    )]
    time_unit: &'a str,
    #[schemars(description = "Earliest timestamp present in the waveform.")]
    time_start: NormalizedTime<'a>,
    #[schemars(description = "Latest timestamp present in the waveform.")]
    time_end: NormalizedTime<'a>,
}

impl<'a> From<&'a crate::engine::info::InfoData> for InfoData<'a> {
    fn from(data: &'a crate::engine::info::InfoData) -> Self {
        Self {
            time_unit: data.time_unit.as_str(),
            time_start: NormalizedTime::new(data.time_start.as_str()),
            time_end: NormalizedTime::new(data.time_end.as_str()),
        }
    }
}

#[derive(Debug, JsonSchema, Serialize)]
#[schemars(rename = "scopeEntry")]
#[schemars(extend("additionalProperties" = true))]
pub struct ScopeEntry<'a> {
    #[schemars(description = "Canonical hierarchy path for this scope entry.")]
    path: CanonicalPath<'a>,
    #[schemars(schema_with = "nonnegative_integer_schema")]
    #[schemars(
        description = "Zero-based scope depth from the dump root used by list and tree renderers."
    )]
    depth: usize,
    #[schemars(description = "Stable scope kind alias for this scope entry.")]
    kind: ScopeKind<'a>,
}

impl<'a> TryFrom<&'a crate::engine::scope::ScopeEntry> for ScopeEntry<'a> {
    type Error = WavepeekError;

    fn try_from(entry: &'a crate::engine::scope::ScopeEntry) -> Result<Self, Self::Error> {
        Ok(Self {
            path: CanonicalPath::new(entry.path.as_str()),
            depth: entry.depth,
            kind: validate_scope_kind(entry.kind.as_str())?,
        })
    }
}

#[derive(Debug, JsonSchema, Serialize)]
#[schemars(rename = "signalEntry")]
#[schemars(extend("additionalProperties" = true))]
pub struct SignalEntry<'a> {
    #[schemars(description = "Signal name as declared inside its immediate parent scope.")]
    name: &'a str,
    #[schemars(description = "Canonical hierarchy path for this signal entry.")]
    path: CanonicalPath<'a>,
    #[schemars(description = "Stable signal kind alias for this signal entry.")]
    kind: SignalKind<'a>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schemars(default)]
    #[schemars(schema_with = "signal_width_schema")]
    #[schemars(description = "Declared packed bit width when the waveform backend reports one.")]
    width: Option<u32>,
}

impl<'a> TryFrom<&'a crate::engine::signal::SignalEntry> for SignalEntry<'a> {
    type Error = WavepeekError;

    fn try_from(entry: &'a crate::engine::signal::SignalEntry) -> Result<Self, Self::Error> {
        Ok(Self {
            name: entry.name.as_str(),
            path: CanonicalPath::new(entry.path.as_str()),
            kind: validate_signal_kind(entry.kind.as_str())?,
            width: entry.width,
        })
    }
}

fn signal_width_schema(generator: &mut SchemaGenerator) -> Schema {
    positive_integer_schema(generator)
}

fn positive_integer_schema(_: &mut SchemaGenerator) -> Schema {
    json_schema!({
        "type": "integer",
        "minimum": 1
    })
}

fn nonnegative_integer_schema(_: &mut SchemaGenerator) -> Schema {
    json_schema!({
        "type": "integer",
        "minimum": 0
    })
}

#[derive(Debug, JsonSchema, Serialize)]
#[schemars(rename = "sampledSignalValue")]
#[schemars(extend("additionalProperties" = true))]
pub struct SampledSignalValue<'a> {
    #[schemars(description = "Canonical path of the sampled signal.")]
    path: CanonicalPath<'a>,
    #[schemars(description = "Sampled value for this signal in the selected timestamp snapshot.")]
    value: SampledValue<'a>,
}

impl<'a> From<&'a crate::engine::value::ValueSignalValue> for SampledSignalValue<'a> {
    fn from(signal: &'a crate::engine::value::ValueSignalValue) -> Self {
        Self {
            path: CanonicalPath::new(signal.path.as_str()),
            value: SampledValue::new(signal.value.as_str()),
        }
    }
}

#[derive(Debug, JsonSchema, Serialize)]
#[schemars(rename = "changeSignalValue")]
#[schemars(extend("additionalProperties" = true))]
pub struct ChangeSignalValue<'a> {
    #[schemars(description = "Canonical path of the changed signal.")]
    path: CanonicalPath<'a>,
    #[schemars(description = "Changed signal value at the reported sample point.")]
    value: SampledValue<'a>,
}

impl<'a> From<&'a crate::engine::change::ChangeSignalValue> for ChangeSignalValue<'a> {
    fn from(signal: &'a crate::engine::change::ChangeSignalValue) -> Self {
        Self {
            path: CanonicalPath::new(signal.path.as_str()),
            value: SampledValue::new(signal.value.as_str()),
        }
    }
}

#[derive(Debug, JsonSchema, Serialize)]
#[schemars(rename = "valueSnapshot")]
#[schemars(extend("additionalProperties" = true))]
pub struct ValueSnapshot<'a> {
    #[schemars(description = "Timestamp requested by the value command.")]
    time: NormalizedTime<'a>,
    #[schemars(description = "Signal values sampled at this timestamp.")]
    signals: Vec<SampledSignalValue<'a>>,
}

impl<'a> From<&'a crate::engine::value::ValueSnapshot> for ValueSnapshot<'a> {
    fn from(snapshot: &'a crate::engine::value::ValueSnapshot) -> Self {
        Self {
            time: NormalizedTime::new(snapshot.time.as_str()),
            signals: snapshot
                .signals
                .iter()
                .map(SampledSignalValue::from)
                .collect(),
        }
    }
}

#[derive(Debug, JsonSchema, Serialize)]
#[schemars(rename = "changeSnapshot")]
#[schemars(extend("additionalProperties" = true))]
pub struct ChangeSnapshot<'a> {
    #[schemars(description = "Trigger timestamp emitted by the change command.")]
    time: NormalizedTime<'a>,
    #[schemars(description = "Timestamp used to sample values for this change row.")]
    sample_time: NormalizedTime<'a>,
    #[schemars(description = "Changed signal values for this row.")]
    signals: Vec<ChangeSignalValue<'a>>,
}

impl<'a> From<&'a crate::engine::change::ChangeSnapshot> for ChangeSnapshot<'a> {
    fn from(snapshot: &'a crate::engine::change::ChangeSnapshot) -> Self {
        Self {
            time: NormalizedTime::new(snapshot.time.as_str()),
            sample_time: NormalizedTime::new(snapshot.sample_time.as_str()),
            signals: snapshot
                .signals
                .iter()
                .map(ChangeSignalValue::from)
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

impl JsonSchema for PropertyKind {
    fn inline_schema() -> bool {
        true
    }

    fn schema_name() -> Cow<'static, str> {
        "propertyKind".into()
    }

    fn json_schema(_: &mut SchemaGenerator) -> Schema {
        json_schema!({"type": "string", "enum": ["match", "assert", "deassert"]})
    }
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

#[derive(Debug, JsonSchema, Serialize)]
#[schemars(rename = "propertyRow")]
#[schemars(extend("additionalProperties" = true))]
pub struct PropertyRow<'a> {
    #[schemars(description = "Trigger timestamp emitted by the property command.")]
    time: NormalizedTime<'a>,
    #[schemars(description = "Timestamp used to evaluate the property expression.")]
    sample_time: NormalizedTime<'a>,
    #[schemars(description = "Property result kind captured for this row.")]
    kind: PropertyKind,
}

impl<'a> From<&'a crate::engine::property::PropertyCaptureRow> for PropertyRow<'a> {
    fn from(row: &'a crate::engine::property::PropertyCaptureRow) -> Self {
        Self {
            time: NormalizedTime::new(row.time.as_str()),
            sample_time: NormalizedTime::new(row.sample_time.as_str()),
            kind: row.kind.into(),
        }
    }
}

#[derive(Debug, JsonSchema, Serialize)]
#[schemars(rename = "topicSummary")]
#[schemars(extend("additionalProperties" = true))]
pub struct TopicSummary<'a> {
    #[schemars(description = "Stable documentation topic identifier.")]
    id: &'a str,
    #[schemars(description = "Human-readable documentation topic title.")]
    title: &'a str,
    #[schemars(description = "Short documentation topic description.")]
    description: &'a str,
    #[schemars(description = "Documentation section that contains this topic.")]
    section: &'a str,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[schemars(default)]
    #[schemars(description = "Related documentation topic identifiers.")]
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

#[derive(Debug, JsonSchema, Serialize)]
#[schemars(rename = "docsTopicsData")]
#[schemars(extend("additionalProperties" = true))]
pub struct DocsTopicsData<'a> {
    #[schemars(description = "Available embedded documentation topics.")]
    topics: Vec<TopicSummary<'a>>,
}

impl<'a> From<&'a crate::engine::DocsTopicsData> for DocsTopicsData<'a> {
    fn from(data: &'a crate::engine::DocsTopicsData) -> Self {
        Self {
            topics: data.topics.iter().map(TopicSummary::from).collect(),
        }
    }
}

#[derive(Debug, JsonSchema, Serialize)]
#[schemars(rename = "docsSearchMatch")]
#[schemars(extend("additionalProperties" = true))]
pub struct DocsSearchMatch<'a> {
    topic: TopicSummary<'a>,
    #[schemars(description = "How the topic matched the normalized search query.")]
    match_kind: DocsMatchKind,
    #[schemars(schema_with = "positive_integer_schema")]
    #[schemars(description = "Number of query tokens matched by this topic.")]
    matched_tokens: usize,
}

impl<'a> From<&'a crate::engine::DocsSearchMatchData> for DocsSearchMatch<'a> {
    fn from(entry: &'a crate::engine::DocsSearchMatchData) -> Self {
        Self {
            topic: TopicSummary::from(&entry.topic),
            match_kind: entry.match_kind.into(),
            matched_tokens: entry.matched_tokens,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
enum DocsMatchKind {
    IdExact,
    IdPrefix,
    TitleExact,
    TitleOrDescription,
    Heading,
    Body,
}

impl JsonSchema for DocsMatchKind {
    fn inline_schema() -> bool {
        true
    }

    fn schema_name() -> Cow<'static, str> {
        "docsMatchKind".into()
    }

    fn json_schema(_: &mut SchemaGenerator) -> Schema {
        json_schema!({
            "type": "string",
            "enum": ["id_exact", "id_prefix", "title_exact", "title_or_description", "heading", "body"],
        })
    }
}

impl From<crate::docs::MatchKind> for DocsMatchKind {
    fn from(kind: crate::docs::MatchKind) -> Self {
        match kind {
            crate::docs::MatchKind::IdExact => Self::IdExact,
            crate::docs::MatchKind::IdPrefix => Self::IdPrefix,
            crate::docs::MatchKind::TitleExact => Self::TitleExact,
            crate::docs::MatchKind::TitleOrDescription => Self::TitleOrDescription,
            crate::docs::MatchKind::Heading => Self::Heading,
            crate::docs::MatchKind::Body => Self::Body,
        }
    }
}

#[derive(Debug, JsonSchema, Serialize)]
#[schemars(rename = "docsSearchData")]
#[schemars(extend("additionalProperties" = true))]
pub struct DocsSearchData<'a> {
    #[schemars(description = "Normalized search query used for the docs search.")]
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
