use std::borrow::Cow;
use std::collections::BTreeMap;

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
    ExtractAhb(ExtractAhbData<'a>),
    ExtractAxi(ExtractAxiData<'a>),
    ExtractGeneric(Vec<ExtractGenericRow<'a>>),
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
            (CommandName::ExtractAhb, CommandData::ExtractAhb(data)) => {
                Ok(Self::ExtractAhb(ExtractAhbData::from(data)))
            }
            (CommandName::ExtractAxi, CommandData::ExtractAxi(data)) => {
                Ok(Self::ExtractAxi(ExtractAxiData::from(data)))
            }
            (CommandName::ExtractGeneric, CommandData::ExtractGeneric(data)) => Ok(
                Self::ExtractGeneric(data.rows.iter().map(ExtractGenericRow::from).collect()),
            ),
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
#[schemars(rename = "extractPayloadValue")]
#[schemars(extend("additionalProperties" = true))]
pub struct ExtractPayloadValue<'a> {
    #[schemars(description = "Canonical path of the sampled payload signal.")]
    path: CanonicalPath<'a>,
    #[schemars(description = "Sampled payload value formatted as a Verilog-style literal string.")]
    value: SampledValue<'a>,
}

impl<'a> From<&'a crate::engine::extract::ExtractPayloadValue> for ExtractPayloadValue<'a> {
    fn from(value: &'a crate::engine::extract::ExtractPayloadValue) -> Self {
        Self {
            path: CanonicalPath::new(value.path.as_str()),
            value: SampledValue::new(value.value.as_str()),
        }
    }
}

#[derive(Debug, JsonSchema, Serialize)]
#[schemars(rename = "extractGenericRow")]
#[schemars(extend("additionalProperties" = true))]
pub struct ExtractGenericRow<'a> {
    #[schemars(description = "Selected event timestamp emitted by extract generic.")]
    time: NormalizedTime<'a>,
    #[schemars(
        description = "Pre-edge timestamp used to evaluate the predicate and sample payload values."
    )]
    sample_time: NormalizedTime<'a>,
    #[schemars(description = "Source name supplied by CLI flags or source JSON.")]
    source: &'a str,
    #[schemars(description = "Ordered payload values sampled for this row.")]
    payload: Vec<ExtractPayloadValue<'a>>,
}

impl<'a> From<&'a crate::engine::extract::ExtractGenericRow> for ExtractGenericRow<'a> {
    fn from(row: &'a crate::engine::extract::ExtractGenericRow) -> Self {
        Self {
            time: NormalizedTime::new(row.time.as_str()),
            sample_time: NormalizedTime::new(row.sample_time.as_str()),
            source: row.source.as_str(),
            payload: row.payload.iter().map(ExtractPayloadValue::from).collect(),
        }
    }
}

#[derive(Debug, JsonSchema, Serialize)]
#[schemars(rename = "extractAhbMapping")]
pub struct ExtractAhbMapping<'a> {
    #[schemars(description = "Canonical waveform signal path mapped to this AHB standard signal.")]
    path: CanonicalPath<'a>,
}

impl<'a> From<&'a crate::engine::ahb::AhbSignalMapping> for ExtractAhbMapping<'a> {
    fn from(mapping: &'a crate::engine::ahb::AhbSignalMapping) -> Self {
        Self {
            path: CanonicalPath::new(mapping.path.as_str()),
        }
    }
}

#[derive(Debug, JsonSchema, Serialize)]
#[schemars(rename = "extractAhbAddressSnapshot")]
pub struct ExtractAhbAddressSnapshot<'a> {
    time: NormalizedTime<'a>,
    sample_time: NormalizedTime<'a>,
    transfer: &'a str,
    direction: &'a str,
    payload: BTreeMap<&'a str, SampledValue<'a>>,
}

impl<'a> From<&'a crate::engine::ahb::AhbAddressSnapshot> for ExtractAhbAddressSnapshot<'a> {
    fn from(address: &'a crate::engine::ahb::AhbAddressSnapshot) -> Self {
        Self {
            time: NormalizedTime::new(address.time.as_str()),
            sample_time: NormalizedTime::new(address.sample_time.as_str()),
            transfer: address.transfer.as_str(),
            direction: address.direction.as_str(),
            payload: ahb_payload(&address.payload),
        }
    }
}

#[derive(Debug, JsonSchema, Serialize)]
#[schemars(rename = "extractAhbInitialDataPhase")]
pub struct ExtractAhbInitialDataPhase<'a> {
    state: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schemars(default)]
    address: Option<ExtractAhbAddressSnapshot<'a>>,
}

impl<'a> From<&'a crate::engine::ahb::AhbInitialDataPhase> for ExtractAhbInitialDataPhase<'a> {
    fn from(initial: &'a crate::engine::ahb::AhbInitialDataPhase) -> Self {
        Self {
            state: initial.state.as_str(),
            address: initial
                .address
                .as_ref()
                .map(ExtractAhbAddressSnapshot::from),
        }
    }
}

#[derive(Debug, JsonSchema, Serialize)]
#[schemars(rename = "extractAhbEvent")]
pub struct ExtractAhbEvent<'a> {
    time: NormalizedTime<'a>,
    sample_time: NormalizedTime<'a>,
    profile: &'a str,
    event: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schemars(default)]
    transfer: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schemars(default)]
    direction: Option<&'a str>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    #[schemars(default)]
    payload: BTreeMap<&'a str, SampledValue<'a>>,
}

impl<'a> From<&'a crate::engine::ahb::AhbEvent> for ExtractAhbEvent<'a> {
    fn from(event: &'a crate::engine::ahb::AhbEvent) -> Self {
        Self {
            time: NormalizedTime::new(event.time.as_str()),
            sample_time: NormalizedTime::new(event.sample_time.as_str()),
            profile: event.profile.as_str(),
            event: event.event.as_str(),
            transfer: event.transfer.as_deref(),
            direction: event.direction.as_deref(),
            payload: ahb_payload(&event.payload),
        }
    }
}

#[derive(Debug, JsonSchema, Serialize)]
#[schemars(rename = "extractAhbData")]
pub struct ExtractAhbData<'a> {
    name: &'a str,
    profile: &'a str,
    issue: &'a str,
    include_stall: bool,
    include_idle: bool,
    include_busy: bool,
    initial_data_phase: ExtractAhbInitialDataPhase<'a>,
    mappings: BTreeMap<&'a str, ExtractAhbMapping<'a>>,
    events: Vec<ExtractAhbEvent<'a>>,
}

impl<'a> From<&'a crate::engine::ahb::AhbData> for ExtractAhbData<'a> {
    fn from(data: &'a crate::engine::ahb::AhbData) -> Self {
        Self {
            name: data.name.as_str(),
            profile: data.profile.as_str(),
            issue: data.issue.as_str(),
            include_stall: data.include_stall,
            include_idle: data.include_idle,
            include_busy: data.include_busy,
            initial_data_phase: ExtractAhbInitialDataPhase::from(&data.initial_data_phase),
            mappings: data
                .mappings
                .iter()
                .map(|mapping| (mapping.standard.as_str(), ExtractAhbMapping::from(mapping)))
                .collect(),
            events: data.events.iter().map(ExtractAhbEvent::from).collect(),
        }
    }
}

fn ahb_payload<'a>(
    payload: &'a [crate::engine::ahb::AhbPayloadValue],
) -> BTreeMap<&'a str, SampledValue<'a>> {
    payload
        .iter()
        .map(|value| {
            (
                value.standard.as_str(),
                SampledValue::new(value.value.as_str()),
            )
        })
        .collect()
}

#[derive(Debug, JsonSchema, Serialize)]
#[schemars(rename = "extractAxiMapping")]
#[schemars(extend("additionalProperties" = true))]
pub struct ExtractAxiMapping<'a> {
    #[schemars(description = "Canonical waveform signal path mapped to this AXI standard signal.")]
    path: CanonicalPath<'a>,
}

impl<'a> From<&'a crate::engine::axi::AxiSignalMapping> for ExtractAxiMapping<'a> {
    fn from(mapping: &'a crate::engine::axi::AxiSignalMapping) -> Self {
        Self {
            path: CanonicalPath::new(mapping.path.as_str()),
        }
    }
}

#[derive(Debug, JsonSchema, Serialize)]
#[schemars(rename = "extractAxiTransfer")]
#[schemars(extend("additionalProperties" = true))]
pub struct ExtractAxiTransfer<'a> {
    #[schemars(description = "Selected AXI transfer event timestamp.")]
    time: NormalizedTime<'a>,
    #[schemars(
        description = "Pre-edge timestamp used to evaluate ready/valid and sample payload values."
    )]
    sample_time: NormalizedTime<'a>,
    #[schemars(
        description = "AXI profile name for this transfer row: axi3, axi4, axi4-lite, axi5, axi5-lite, ace, ace-lite, ace5, ace5-lite, ace5-lite-dvm, or ace5-lite-acp."
    )]
    profile: &'a str,
    #[schemars(
        description = "Profile-specific AXI channel name. AXI5 and ACE5-LiteDVM can add ac and cr; ACE and ACE5 can add ac, cr, and cd to the base aw, w, b, ar, and r channels."
    )]
    channel: &'a str,
    #[schemars(description = "Payload values keyed by lowercase AXI standard signal name.")]
    payload: BTreeMap<&'a str, SampledValue<'a>>,
}

impl<'a> From<&'a crate::engine::axi::AxiTransfer> for ExtractAxiTransfer<'a> {
    fn from(transfer: &'a crate::engine::axi::AxiTransfer) -> Self {
        Self {
            time: NormalizedTime::new(transfer.time.as_str()),
            sample_time: NormalizedTime::new(transfer.sample_time.as_str()),
            profile: transfer.profile.as_str(),
            channel: transfer.channel.as_str(),
            payload: transfer
                .payload
                .iter()
                .map(|value| {
                    (
                        value.standard.as_str(),
                        SampledValue::new(value.value.as_str()),
                    )
                })
                .collect(),
        }
    }
}

#[derive(Debug, JsonSchema, Serialize)]
#[schemars(rename = "extractAxiData")]
#[schemars(extend("additionalProperties" = true))]
pub struct ExtractAxiData<'a> {
    #[schemars(description = "AXI port name supplied by CLI or source JSON.")]
    name: &'a str,
    #[schemars(description = "AXI profile name used for standard signal mapping.")]
    profile: &'a str,
    #[schemars(description = "Arm IHI 0022 issue used for this profile definition.")]
    issue: &'a str,
    #[schemars(
        description = "Resolved waveform mappings keyed by lowercase AXI standard signal name."
    )]
    mappings: BTreeMap<&'a str, ExtractAxiMapping<'a>>,
    #[schemars(description = "Extracted AXI ready/valid transfers in event order.")]
    transfers: Vec<ExtractAxiTransfer<'a>>,
}

impl<'a> From<&'a crate::engine::axi::AxiData> for ExtractAxiData<'a> {
    fn from(data: &'a crate::engine::axi::AxiData) -> Self {
        Self {
            name: data.name.as_str(),
            profile: data.profile.as_str(),
            issue: data.issue.as_str(),
            mappings: data
                .mappings
                .iter()
                .map(|mapping| (mapping.standard.as_str(), ExtractAxiMapping::from(mapping)))
                .collect(),
            transfers: data
                .transfers
                .iter()
                .map(ExtractAxiTransfer::from)
                .collect(),
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
