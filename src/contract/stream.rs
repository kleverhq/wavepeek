use schemars::{JsonSchema, Schema, SchemaGenerator, json_schema};
use serde::Serialize;

use crate::diagnostic::Diagnostic;
use crate::engine::CommandName;
use crate::error::WavepeekError;

use super::common::ContractDiagnostic;
use super::output::{
    ChangeSnapshot, ExtractAxiMapping, ExtractAxiStreamMapping, ExtractAxiStreamTransfer,
    ExtractAxiTransfer, ExtractGenericRow, InfoData, PropertyRow, ScopeEntry, SignalEntry,
    ValueSnapshot,
};
use super::schema::STREAM_SCHEMA_URL;

#[derive(Debug, JsonSchema, Serialize)]
#[schemars(rename = "beginRecord")]
#[schemars(extend("additionalProperties" = true))]
pub struct BeginRecord<'a> {
    #[serde(rename = "type")]
    #[schemars(schema_with = "begin_record_type_schema")]
    record_type: &'static str,
    #[schemars(schema_with = "sequence_ref_schema")]
    seq: usize,
    #[schemars(schema_with = "stream_command_ref_schema")]
    command: &'static str,
    #[serde(rename = "$schema")]
    #[schemars(schema_with = "stream_schema_url_schema")]
    schema: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schemars(default)]
    context: Option<StreamContextData<'a>>,
}

impl BeginRecord<'static> {
    pub fn new(seq: usize, command: CommandName) -> Result<Self, WavepeekError> {
        require_stream_command(command)?;
        Ok(Self {
            record_type: "begin",
            seq,
            command: command.as_str(),
            schema: STREAM_SCHEMA_URL,
            context: None,
        })
    }
}

impl<'a> BeginRecord<'a> {
    pub fn with_context<T: StreamContext + ?Sized>(
        seq: usize,
        command: CommandName,
        context: &'a T,
    ) -> Result<Self, WavepeekError> {
        require_stream_command(command)?;
        Ok(Self {
            record_type: "begin",
            seq,
            command: command.as_str(),
            schema: STREAM_SCHEMA_URL,
            context: Some(context.stream_context(command)?),
        })
    }
}

#[derive(Debug, JsonSchema, Serialize)]
#[schemars(rename = "itemRecord")]
#[schemars(extend("additionalProperties" = true))]
pub struct ItemRecord<'a> {
    #[serde(rename = "type")]
    #[schemars(schema_with = "item_record_type_schema")]
    record_type: &'static str,
    #[schemars(schema_with = "sequence_ref_schema")]
    seq: usize,
    #[schemars(schema_with = "stream_command_ref_schema")]
    command: &'static str,
    item: StreamItemData<'a>,
}

impl<'a> ItemRecord<'a> {
    pub fn new<T: StreamItem + ?Sized>(
        seq: usize,
        command: CommandName,
        item: &'a T,
    ) -> Result<Self, WavepeekError> {
        require_stream_command(command)?;
        Ok(Self {
            record_type: "item",
            seq,
            command: command.as_str(),
            item: item.stream_item(command)?,
        })
    }
}

#[derive(Debug, JsonSchema, Serialize)]
#[schemars(rename = "diagnosticRecord")]
#[schemars(extend("additionalProperties" = true))]
pub struct DiagnosticRecord<'a> {
    #[serde(rename = "type")]
    #[schemars(schema_with = "diagnostic_record_type_schema")]
    record_type: &'static str,
    #[schemars(schema_with = "sequence_ref_schema")]
    seq: usize,
    #[schemars(schema_with = "stream_command_ref_schema")]
    command: &'static str,
    diagnostic: ContractDiagnostic<'a>,
}

impl<'a> DiagnosticRecord<'a> {
    pub fn new(
        seq: usize,
        command: CommandName,
        diagnostic: &'a Diagnostic,
    ) -> Result<Self, WavepeekError> {
        require_stream_command(command)?;
        Ok(Self {
            record_type: "diagnostic",
            seq,
            command: command.as_str(),
            diagnostic: ContractDiagnostic::from_diagnostic(diagnostic)?,
        })
    }
}

#[derive(Debug, JsonSchema, Serialize)]
#[schemars(rename = "endRecord")]
#[schemars(extend("additionalProperties" = true))]
pub struct EndRecord {
    #[serde(rename = "type")]
    #[schemars(schema_with = "end_record_type_schema")]
    record_type: &'static str,
    #[schemars(schema_with = "sequence_ref_schema")]
    seq: usize,
    #[schemars(schema_with = "stream_command_ref_schema")]
    command: &'static str,
    summary: StreamSummary,
}

impl EndRecord {
    pub fn new(
        seq: usize,
        command: CommandName,
        items: usize,
        diagnostics: usize,
        truncated: bool,
    ) -> Result<Self, WavepeekError> {
        require_stream_command(command)?;
        Ok(Self {
            record_type: "end",
            seq,
            command: command.as_str(),
            summary: StreamSummary {
                status: "ok",
                items,
                diagnostics,
                truncated,
            },
        })
    }
}

#[derive(Debug, JsonSchema, Serialize)]
#[schemars(rename = "streamSummary")]
#[schemars(extend("additionalProperties" = true))]
struct StreamSummary {
    #[schemars(schema_with = "ok_status_schema")]
    status: &'static str,
    #[schemars(schema_with = "nonnegative_count_schema")]
    items: usize,
    #[schemars(schema_with = "nonnegative_count_schema")]
    diagnostics: usize,
    truncated: bool,
}

#[derive(Debug, JsonSchema, Serialize)]
#[schemars(rename = "streamContextData")]
#[serde(untagged)]
pub enum StreamContextData<'a> {
    ExtractAxi(ExtractAxiContext<'a>),
    ExtractAxiStream(ExtractAxiStreamContext<'a>),
}

#[derive(Debug, JsonSchema, Serialize)]
#[schemars(rename = "extractAxiContext")]
#[schemars(extend("additionalProperties" = true))]
pub struct ExtractAxiContext<'a> {
    name: &'a str,
    profile: &'a str,
    issue: &'a str,
    mappings: std::collections::BTreeMap<&'a str, ExtractAxiMapping<'a>>,
}

impl<'a> From<&'a crate::engine::axi::AxiContext> for ExtractAxiContext<'a> {
    fn from(context: &'a crate::engine::axi::AxiContext) -> Self {
        Self {
            name: context.name.as_str(),
            profile: context.profile.as_str(),
            issue: context.issue.as_str(),
            mappings: context
                .mappings
                .iter()
                .map(|mapping| (mapping.standard.as_str(), ExtractAxiMapping::from(mapping)))
                .collect(),
        }
    }
}

pub trait StreamContext {
    fn stream_context(&self, command: CommandName) -> Result<StreamContextData<'_>, WavepeekError>;
}

impl StreamContext for crate::engine::axi::AxiContext {
    fn stream_context(&self, command: CommandName) -> Result<StreamContextData<'_>, WavepeekError> {
        require_item_command(command, CommandName::ExtractAxi)?;
        Ok(StreamContextData::ExtractAxi(ExtractAxiContext::from(self)))
    }
}

#[derive(Debug, JsonSchema, Serialize)]
#[schemars(rename = "extractAxiStreamContext")]
#[schemars(extend("additionalProperties" = true))]
pub struct ExtractAxiStreamContext<'a> {
    name: &'a str,
    profile: &'a str,
    issue: &'a str,
    tready_mode: &'a str,
    mappings: std::collections::BTreeMap<&'a str, ExtractAxiStreamMapping<'a>>,
}

impl<'a> From<&'a crate::engine::axistream::AxiStreamContext> for ExtractAxiStreamContext<'a> {
    fn from(context: &'a crate::engine::axistream::AxiStreamContext) -> Self {
        Self {
            name: context.name.as_str(),
            profile: context.profile.as_str(),
            issue: context.issue.as_str(),
            tready_mode: context.tready_mode.as_str(),
            mappings: context
                .mappings
                .iter()
                .map(|mapping| {
                    (
                        mapping.standard.as_str(),
                        ExtractAxiStreamMapping::from(mapping),
                    )
                })
                .collect(),
        }
    }
}

impl StreamContext for crate::engine::axistream::AxiStreamContext {
    fn stream_context(&self, command: CommandName) -> Result<StreamContextData<'_>, WavepeekError> {
        require_item_command(command, CommandName::ExtractAxiStream)?;
        Ok(StreamContextData::ExtractAxiStream(
            ExtractAxiStreamContext::from(self),
        ))
    }
}

#[derive(Debug, JsonSchema, Serialize)]
#[schemars(rename = "streamItemData")]
#[serde(untagged)]
pub enum StreamItemData<'a> {
    Info(InfoData<'a>),
    Scope(ScopeEntry<'a>),
    Signal(SignalEntry<'a>),
    Value(ValueSnapshot<'a>),
    Change(ChangeSnapshot<'a>),
    Property(PropertyRow<'a>),
    ExtractAxi(ExtractAxiTransfer<'a>),
    ExtractAxiStream(ExtractAxiStreamTransfer<'a>),
    ExtractGeneric(ExtractGenericRow<'a>),
}

pub trait StreamItem {
    fn stream_item(&self, command: CommandName) -> Result<StreamItemData<'_>, WavepeekError>;
}

impl StreamItem for crate::engine::info::InfoData {
    fn stream_item(&self, command: CommandName) -> Result<StreamItemData<'_>, WavepeekError> {
        require_item_command(command, CommandName::Info)?;
        Ok(StreamItemData::Info(InfoData::from(self)))
    }
}

impl StreamItem for crate::engine::scope::ScopeEntry {
    fn stream_item(&self, command: CommandName) -> Result<StreamItemData<'_>, WavepeekError> {
        require_item_command(command, CommandName::Scope)?;
        Ok(StreamItemData::Scope(ScopeEntry::try_from(self)?))
    }
}

impl StreamItem for crate::engine::signal::SignalEntry {
    fn stream_item(&self, command: CommandName) -> Result<StreamItemData<'_>, WavepeekError> {
        require_item_command(command, CommandName::Signal)?;
        Ok(StreamItemData::Signal(SignalEntry::try_from(self)?))
    }
}

impl StreamItem for crate::engine::value::ValueSnapshot {
    fn stream_item(&self, command: CommandName) -> Result<StreamItemData<'_>, WavepeekError> {
        require_item_command(command, CommandName::Value)?;
        Ok(StreamItemData::Value(ValueSnapshot::from(self)))
    }
}

impl StreamItem for crate::engine::change::ChangeSnapshot {
    fn stream_item(&self, command: CommandName) -> Result<StreamItemData<'_>, WavepeekError> {
        require_item_command(command, CommandName::Change)?;
        Ok(StreamItemData::Change(ChangeSnapshot::from(self)))
    }
}

impl StreamItem for crate::engine::property::PropertyCaptureRow {
    fn stream_item(&self, command: CommandName) -> Result<StreamItemData<'_>, WavepeekError> {
        require_item_command(command, CommandName::Property)?;
        Ok(StreamItemData::Property(PropertyRow::from(self)))
    }
}

impl StreamItem for crate::engine::axi::AxiTransfer {
    fn stream_item(&self, command: CommandName) -> Result<StreamItemData<'_>, WavepeekError> {
        require_item_command(command, CommandName::ExtractAxi)?;
        Ok(StreamItemData::ExtractAxi(ExtractAxiTransfer::from(self)))
    }
}

impl StreamItem for crate::engine::axistream::AxiStreamTransfer {
    fn stream_item(&self, command: CommandName) -> Result<StreamItemData<'_>, WavepeekError> {
        require_item_command(command, CommandName::ExtractAxiStream)?;
        Ok(StreamItemData::ExtractAxiStream(
            ExtractAxiStreamTransfer::from(self),
        ))
    }
}

impl StreamItem for crate::engine::extract::ExtractGenericRow {
    fn stream_item(&self, command: CommandName) -> Result<StreamItemData<'_>, WavepeekError> {
        require_item_command(command, CommandName::ExtractGeneric)?;
        Ok(StreamItemData::ExtractGeneric(ExtractGenericRow::from(
            self,
        )))
    }
}

fn begin_record_type_schema(_: &mut SchemaGenerator) -> Schema {
    json_schema!({"const": "begin"})
}

fn item_record_type_schema(_: &mut SchemaGenerator) -> Schema {
    json_schema!({"const": "item"})
}

fn diagnostic_record_type_schema(_: &mut SchemaGenerator) -> Schema {
    json_schema!({"const": "diagnostic"})
}

fn end_record_type_schema(_: &mut SchemaGenerator) -> Schema {
    json_schema!({"const": "end"})
}

fn ok_status_schema(_: &mut SchemaGenerator) -> Schema {
    json_schema!({"const": "ok"})
}

fn sequence_ref_schema(_: &mut SchemaGenerator) -> Schema {
    json_schema!({"$ref": "#/$defs/sequence"})
}

fn nonnegative_count_schema(_: &mut SchemaGenerator) -> Schema {
    json_schema!({"type": "integer", "minimum": 0})
}

fn stream_command_ref_schema(_: &mut SchemaGenerator) -> Schema {
    json_schema!({"$ref": "#/$defs/streamCommand"})
}

fn stream_schema_url_schema(_: &mut SchemaGenerator) -> Schema {
    json_schema!({"type": "string", "const": STREAM_SCHEMA_URL})
}

fn require_item_command(actual: CommandName, expected: CommandName) -> Result<(), WavepeekError> {
    if actual == expected {
        Ok(())
    } else {
        Err(WavepeekError::Internal(format!(
            "JSONL item for {} cannot be written to {} stream",
            expected.as_str(),
            actual.as_str()
        )))
    }
}

fn require_stream_command(command: CommandName) -> Result<(), WavepeekError> {
    match command {
        CommandName::Info
        | CommandName::Scope
        | CommandName::Signal
        | CommandName::Value
        | CommandName::Change
        | CommandName::Property
        | CommandName::ExtractAxi
        | CommandName::ExtractAxiStream
        | CommandName::ExtractGeneric => Ok(()),
        _ => Err(WavepeekError::Args(
            "--jsonl is available only for waveform commands".to_string(),
        )),
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use crate::engine::CommandName;

    use super::{BeginRecord, ItemRecord};

    #[test]
    fn begin_record_uses_stream_schema_url() {
        let value = serde_json::to_value(
            BeginRecord::new(0, CommandName::Change).expect("change begin should convert"),
        )
        .expect("begin record should serialize");

        assert_eq!(value["type"], "begin");
        assert_eq!(
            value["$schema"],
            "https://kleverhq.github.io/wavepeek/schema-stream-v2.2.json"
        );
    }

    #[test]
    fn item_record_rejects_command_mismatch() {
        let item = crate::engine::info::InfoData {
            time_unit: "1ns".to_string(),
            time_start: "0ns".to_string(),
            time_end: "10ns".to_string(),
        };

        assert!(ItemRecord::new(1, CommandName::Change, &item).is_err());
    }

    #[test]
    fn item_record_uses_contract_payload_shape() {
        let item = crate::engine::signal::SignalEntry {
            display: "clk".to_string(),
            name: "clk".to_string(),
            path: "top.clk".to_string(),
            kind: "wire".to_string(),
            width: Some(1),
        };
        let value = serde_json::to_value(
            ItemRecord::new(1, CommandName::Signal, &item).expect("signal item should convert"),
        )
        .expect("item record should serialize");

        assert_eq!(
            value,
            json!({
                "type": "item",
                "seq": 1,
                "command": "signal",
                "item": {"name": "clk", "path": "top.clk", "kind": "wire", "width": 1}
            })
        );
    }
}
