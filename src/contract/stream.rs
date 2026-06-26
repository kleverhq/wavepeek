use serde::Serialize;

use crate::diagnostic::Diagnostic;
use crate::engine::CommandName;
use crate::error::WavepeekError;

use super::common::ContractDiagnostic;
use super::output::{
    ChangeSnapshot, InfoData, PropertyRow, ScopeEntry, SignalEntry, ValueSnapshot,
};
use super::schema::STREAM_SCHEMA_URL;

#[derive(Debug, Serialize)]
pub struct BeginRecord {
    #[serde(rename = "type")]
    record_type: &'static str,
    seq: usize,
    command: &'static str,
    #[serde(rename = "$schema")]
    schema: &'static str,
}

impl BeginRecord {
    pub fn new(seq: usize, command: CommandName) -> Result<Self, WavepeekError> {
        require_stream_command(command)?;
        Ok(Self {
            record_type: "begin",
            seq,
            command: command.as_str(),
            schema: STREAM_SCHEMA_URL,
        })
    }
}

#[derive(Debug, Serialize)]
pub struct ItemRecord<'a> {
    #[serde(rename = "type")]
    record_type: &'static str,
    seq: usize,
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

#[derive(Debug, Serialize)]
pub struct DiagnosticRecord<'a> {
    #[serde(rename = "type")]
    record_type: &'static str,
    seq: usize,
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

#[derive(Debug, Serialize)]
pub struct EndRecord {
    #[serde(rename = "type")]
    record_type: &'static str,
    seq: usize,
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

#[derive(Debug, Serialize)]
struct StreamSummary {
    status: &'static str,
    items: usize,
    diagnostics: usize,
    truncated: bool,
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum StreamItemData<'a> {
    Info(InfoData<'a>),
    Scope(ScopeEntry<'a>),
    Signal(SignalEntry<'a>),
    Value(ValueSnapshot<'a>),
    Change(ChangeSnapshot<'a>),
    Property(PropertyRow<'a>),
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
        | CommandName::Property => Ok(()),
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
            "https://kleverhq.github.io/wavepeek/schema-stream-v2.0.json"
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
