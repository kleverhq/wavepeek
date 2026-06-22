use std::io::{self, Write};

use serde::Serialize;

use crate::diagnostic::{Diagnostic, DiagnosticKind};
use crate::engine::{CommandData, CommandName, CommandResult, HumanRenderOptions};
use crate::error::WavepeekError;
use crate::output_mode::OutputMode;
use crate::schema_contract::{SCHEMA_URL, STREAM_SCHEMA_URL};

#[derive(Debug, Serialize)]
pub struct OutputEnvelope<T>
where
    T: Serialize,
{
    #[serde(rename = "$schema")]
    pub schema: &'static str,
    pub command: String,
    pub data: T,
    pub diagnostics: Vec<Diagnostic>,
}

impl<T> OutputEnvelope<T>
where
    T: Serialize,
{
    pub fn with_diagnostics(
        command: impl Into<String>,
        data: T,
        diagnostics: Vec<Diagnostic>,
    ) -> Self {
        Self {
            schema: SCHEMA_URL,
            command: command.into(),
            data,
            diagnostics,
        }
    }
}

pub struct JsonlWriter<W: Write> {
    writer: W,
    command: CommandName,
    next_seq: usize,
    items: usize,
    diagnostics: usize,
}

impl<W: Write> JsonlWriter<W> {
    pub const fn new(writer: W, command: CommandName) -> Self {
        Self {
            writer,
            command,
            next_seq: 0,
            items: 0,
            diagnostics: 0,
        }
    }

    pub fn begin(&mut self) -> Result<(), WavepeekError> {
        let record = JsonlBeginRecord {
            record_type: "begin",
            seq: self.next_seq,
            command: self.command.as_str(),
            schema: STREAM_SCHEMA_URL,
        };
        self.write_record(&record)
    }

    pub fn item<T: Serialize>(&mut self, item: &T) -> Result<(), WavepeekError> {
        let record = JsonlItemRecord {
            record_type: "item",
            seq: self.next_seq,
            command: self.command.as_str(),
            item,
        };
        self.write_record(&record)?;
        self.items += 1;
        Ok(())
    }

    pub fn diagnostic(&mut self, diagnostic: &Diagnostic) -> Result<(), WavepeekError> {
        let record = JsonlDiagnosticRecord {
            record_type: "diagnostic",
            seq: self.next_seq,
            command: self.command.as_str(),
            diagnostic,
        };
        self.write_record(&record)?;
        self.diagnostics += 1;
        Ok(())
    }

    pub fn end(&mut self, truncated: bool) -> Result<(), WavepeekError> {
        let summary = JsonlEndSummary {
            status: "ok",
            items: self.items,
            diagnostics: self.diagnostics,
            truncated,
        };
        let record = JsonlEndRecord {
            record_type: "end",
            seq: self.next_seq,
            command: self.command.as_str(),
            summary,
        };
        self.write_record(&record)
    }

    #[cfg(test)]
    pub const fn item_count(&self) -> usize {
        self.items
    }

    #[cfg(test)]
    pub const fn diagnostic_count(&self) -> usize {
        self.diagnostics
    }

    fn write_record<T: Serialize>(&mut self, record: &T) -> Result<(), WavepeekError> {
        serde_json::to_writer(&mut self.writer, record).map_err(map_jsonl_serde_error)?;
        self.writer.write_all(b"\n").map_err(map_jsonl_io_error)?;
        self.writer.flush().map_err(map_jsonl_io_error)?;
        self.next_seq += 1;
        Ok(())
    }
}

#[derive(Debug, Serialize)]
struct JsonlBeginRecord {
    #[serde(rename = "type")]
    record_type: &'static str,
    seq: usize,
    command: &'static str,
    #[serde(rename = "$schema")]
    schema: &'static str,
}

#[derive(Debug, Serialize)]
struct JsonlItemRecord<'a, T: Serialize + ?Sized> {
    #[serde(rename = "type")]
    record_type: &'static str,
    seq: usize,
    command: &'static str,
    item: &'a T,
}

#[derive(Debug, Serialize)]
struct JsonlDiagnosticRecord<'a> {
    #[serde(rename = "type")]
    record_type: &'static str,
    seq: usize,
    command: &'static str,
    diagnostic: &'a Diagnostic,
}

#[derive(Debug, Serialize)]
struct JsonlEndRecord {
    #[serde(rename = "type")]
    record_type: &'static str,
    seq: usize,
    command: &'static str,
    summary: JsonlEndSummary,
}

#[derive(Debug, Serialize)]
struct JsonlEndSummary {
    status: &'static str,
    items: usize,
    diagnostics: usize,
    truncated: bool,
}

pub fn write(result: CommandResult) -> Result<(), WavepeekError> {
    match result.output_mode {
        OutputMode::Human => {
            let output = render_human(&result.data, result.human_options);
            if !output.is_empty() {
                write_stdout(output.as_str());
            }
            emit_human_diagnostics(&result.diagnostics);
            Ok(())
        }
        OutputMode::Json => {
            let json = render_json(result)?;
            println!("{json}");
            Ok(())
        }
        OutputMode::Jsonl => {
            let stdout = io::stdout();
            let mut writer = JsonlWriter::new(stdout.lock(), result.command);
            write_jsonl_result(result, &mut writer)
        }
    }
}

pub fn write_jsonl_result<W: Write>(
    result: CommandResult,
    writer: &mut JsonlWriter<W>,
) -> Result<(), WavepeekError> {
    writer.begin()?;
    match &result.data {
        CommandData::Info(data) => writer.item(data)?,
        CommandData::Scope(entries) => {
            for entry in entries {
                writer.item(entry)?;
            }
        }
        CommandData::Signal(entries) => {
            for entry in entries {
                writer.item(entry)?;
            }
        }
        CommandData::Value(snapshots) => {
            for snapshot in snapshots {
                writer.item(snapshot)?;
            }
        }
        CommandData::Change(snapshots) => {
            for snapshot in snapshots {
                writer.item(snapshot)?;
            }
        }
        CommandData::Property(rows) => {
            for row in rows {
                writer.item(row)?;
            }
        }
        CommandData::Schema(_)
        | CommandData::Text(_)
        | CommandData::DocsTopics(_)
        | CommandData::DocsSearch(_) => {
            return Err(WavepeekError::Args(
                "--jsonl is available only for waveform commands".to_string(),
            ));
        }
    }

    let truncated = result.diagnostics.iter().any(is_truncation_diagnostic);
    for diagnostic in &result.diagnostics {
        writer.diagnostic(diagnostic)?;
    }
    writer.end(truncated)
}

fn is_truncation_diagnostic(diagnostic: &Diagnostic) -> bool {
    diagnostic.code() == Some("WPK-W0002")
}

fn map_jsonl_serde_error(error: serde_json::Error) -> WavepeekError {
    if error.io_error_kind() == Some(io::ErrorKind::BrokenPipe) {
        WavepeekError::BrokenPipe
    } else {
        WavepeekError::Internal(format!("failed to serialize JSONL output: {error}"))
    }
}

fn map_jsonl_io_error(error: io::Error) -> WavepeekError {
    if error.kind() == io::ErrorKind::BrokenPipe {
        WavepeekError::BrokenPipe
    } else {
        WavepeekError::Internal(format!("failed to write JSONL output: {error}"))
    }
}

fn render_json(result: CommandResult) -> Result<String, WavepeekError> {
    let envelope =
        OutputEnvelope::with_diagnostics(result.command.as_str(), result.data, result.diagnostics);
    serde_json::to_string(&envelope)
        .map_err(|error| WavepeekError::Internal(format!("failed to serialize output: {error}")))
}

fn render_human(data: &CommandData, options: HumanRenderOptions) -> String {
    match data {
        CommandData::Schema(schema) => schema.clone(),
        CommandData::Text(text) => text.clone(),
        CommandData::Info(info) => {
            let mut lines = Vec::new();
            lines.push(format!("time_unit: {}", info.time_unit));
            lines.push(format!("time_start: {}", info.time_start));
            lines.push(format!("time_end: {}", info.time_end));
            lines.join("\n")
        }
        CommandData::Scope(scopes) => {
            if options.scope_tree {
                render_scope_tree(scopes)
            } else {
                scopes
                    .iter()
                    .map(|entry| format!("{} {} kind={}", entry.depth, entry.path, entry.kind))
                    .collect::<Vec<_>>()
                    .join("\n")
            }
        }
        CommandData::Signal(signals) => signals
            .iter()
            .map(|entry| match entry.width {
                Some(width) => {
                    format!(
                        "{} kind={} width={width}",
                        signal_display_name(entry, options.signals_abs),
                        entry.kind
                    )
                }
                None => format!(
                    "{} kind={}",
                    signal_display_name(entry, options.signals_abs),
                    entry.kind
                ),
            })
            .collect::<Vec<_>>()
            .join("\n"),
        CommandData::Value(snapshots) => snapshots
            .iter()
            .map(|snapshot| {
                let mut parts = Vec::with_capacity(snapshot.signals.len() + 1);
                parts.push(format!("@{}", snapshot.time));
                for signal in &snapshot.signals {
                    let display = if options.signals_abs {
                        signal.path.as_str()
                    } else {
                        signal.display.as_str()
                    };
                    parts.push(format!("{display}={}", signal.value));
                }
                parts.join(" ")
            })
            .collect::<Vec<_>>()
            .join("\n"),
        CommandData::Change(snapshots) => snapshots
            .iter()
            .map(|snapshot| {
                let mut parts = Vec::with_capacity(snapshot.signals.len() + 2);
                parts.push(format!("@{}", snapshot.time));
                if snapshot.sample_time != snapshot.time {
                    parts.push(format!("sample@{}", snapshot.sample_time));
                }
                for signal in &snapshot.signals {
                    let display = if options.signals_abs {
                        signal.path.as_str()
                    } else {
                        signal.display.as_str()
                    };
                    parts.push(format!("{display}={}", signal.value));
                }
                parts.join(" ")
            })
            .collect::<Vec<_>>()
            .join("\n"),
        CommandData::Property(rows) => rows
            .iter()
            .map(|row| {
                if row.sample_time == row.time {
                    format!("@{} {}", row.time, row.kind)
                } else {
                    format!("@{} sample@{} {}", row.time, row.sample_time, row.kind)
                }
            })
            .collect::<Vec<_>>()
            .join("\n"),
        CommandData::DocsTopics(data) => data
            .topics
            .iter()
            .map(|topic| format!("{} — {}", topic.id, topic.description))
            .collect::<Vec<_>>()
            .join("\n"),
        CommandData::DocsSearch(data) => data
            .matches
            .iter()
            .map(|entry| format!("{}  {}", entry.topic.id, entry.topic.description))
            .collect::<Vec<_>>()
            .join("\n"),
    }
}

fn render_scope_tree(scopes: &[crate::engine::scope::ScopeEntry]) -> String {
    if scopes.is_empty() {
        return String::new();
    }

    let mut lines = Vec::with_capacity(scopes.len());
    let mut ancestor_last = Vec::new();

    for (index, entry) in scopes.iter().enumerate() {
        let label = entry.path.rsplit('.').next().unwrap_or(entry.path.as_str());
        let scope_label = format!("{label} kind={}", entry.kind);
        let is_last = scope_entry_is_last_sibling(scopes, index);

        if entry.depth == 0 {
            lines.push(scope_label);
        } else {
            let mut line = String::new();

            for depth in 1..entry.depth {
                let ancestor_is_last = ancestor_last.get(depth).copied().unwrap_or(true);
                if ancestor_is_last {
                    line.push_str("    ");
                } else {
                    line.push_str("│   ");
                }
            }

            line.push_str(if is_last { "└── " } else { "├── " });
            line.push_str(scope_label.as_str());
            lines.push(line);
        }

        ancestor_last.truncate(entry.depth);
        ancestor_last.push(is_last);
    }

    lines.join("\n")
}

fn scope_entry_is_last_sibling(scopes: &[crate::engine::scope::ScopeEntry], index: usize) -> bool {
    let depth = scopes[index].depth;
    for next in scopes.iter().skip(index + 1) {
        if next.depth < depth {
            return true;
        }
        if next.depth == depth {
            return false;
        }
    }

    true
}

fn signal_display_name(entry: &crate::engine::signal::SignalEntry, abs: bool) -> &str {
    if abs {
        entry.path.as_str()
    } else {
        entry.display.as_str()
    }
}

fn emit_human_diagnostics(diagnostics: &[Diagnostic]) {
    for diagnostic in diagnostics {
        match diagnostic.kind() {
            DiagnosticKind::Info => eprintln!("info: {}", diagnostic.message()),
            DiagnosticKind::Warning => eprintln!(
                "warning[{}]: {}",
                diagnostic
                    .code()
                    .expect("warning diagnostics must have stable codes"),
                diagnostic.message()
            ),
            DiagnosticKind::Error => eprintln!(
                "error[{}]: {}",
                diagnostic
                    .code()
                    .expect("error diagnostics must have stable codes"),
                diagnostic.message()
            ),
        }
    }
}

fn write_stdout(output: &str) {
    if output.ends_with('\n') {
        print!("{output}");
    } else {
        println!("{output}");
    }
}

#[cfg(test)]
#[path = "tests/output_rendering_edges.rs"]
mod output_rendering_edges;

#[cfg(test)]
mod tests {
    use std::io;

    use serde_json::{Value, json};

    use crate::diagnostic::{Diagnostic, WarningDiagnosticCode};
    use crate::engine::{CommandData, CommandName, CommandResult, HumanRenderOptions};
    use crate::output_mode::OutputMode;
    use crate::schema_contract::{SCHEMA_URL, STREAM_SCHEMA_URL};

    use super::{
        JsonlWriter, OutputEnvelope, render_human, render_json, render_scope_tree,
        scope_entry_is_last_sibling, signal_display_name, write, write_jsonl_result,
    };

    #[test]
    fn json_envelope_has_required_shape_for_info() {
        let result = CommandResult {
            command: CommandName::Info,
            output_mode: OutputMode::Json,
            human_options: HumanRenderOptions::default(),
            data: CommandData::Info(crate::engine::info::InfoData {
                time_unit: "1ns".to_string(),
                time_start: "0ns".to_string(),
                time_end: "10ns".to_string(),
            }),
            diagnostics: vec![],
        };

        let json = render_json(result).expect("json serialization should succeed");
        let value: Value = serde_json::from_str(&json).expect("json should parse");

        assert_eq!(value["$schema"], SCHEMA_URL);
        assert!(value.get("schema_version").is_none());
        assert_eq!(value["command"], "info");
        assert!(value["data"].is_object());
        assert!(value["diagnostics"].is_array());
        assert!(value.get("warnings").is_none());
    }

    #[test]
    fn json_envelope_preserves_diagnostics_for_scope() {
        let result = CommandResult {
            command: CommandName::Scope,
            output_mode: OutputMode::Json,
            human_options: HumanRenderOptions::default(),
            data: CommandData::Scope(vec![crate::engine::scope::ScopeEntry {
                path: "top.cpu".to_string(),
                depth: 1,
                kind: "module".to_string(),
            }]),
            diagnostics: vec![Diagnostic::warning(
                WarningDiagnosticCode::OutputTruncated,
                "truncated to 1 entries",
            )],
        };

        let json = render_json(result).expect("json serialization should succeed");
        let value: Value = serde_json::from_str(&json).expect("json should parse");

        assert_eq!(value["command"], "scope");
        assert_eq!(value["diagnostics"][0]["kind"], "warning");
        assert_eq!(value["diagnostics"][0]["code"], "WPK-W0002");
        assert_eq!(value["diagnostics"][0]["message"], "truncated to 1 entries");
        assert_eq!(value["data"][0]["path"], "top.cpu");
        assert_eq!(value["data"][0]["depth"], 1);
        assert_eq!(value["data"][0]["kind"], "module");
    }

    #[test]
    fn docs_topics_json_envelope_uses_nested_topics_payload() {
        let result = CommandResult {
            command: CommandName::DocsTopics,
            output_mode: OutputMode::Json,
            human_options: HumanRenderOptions::default(),
            data: CommandData::DocsTopics(crate::engine::DocsTopicsData {
                topics: vec![crate::docs::TopicSummary {
                    id: "intro".to_string(),
                    title: "Introduction".to_string(),
                    description: "Start here.".to_string(),
                    section: "intro".to_string(),
                    see_also: vec!["commands/help".to_string()],
                }],
            }),
            diagnostics: vec![],
        };

        let json = render_json(result).expect("json serialization should succeed");
        let value: Value = serde_json::from_str(&json).expect("json should parse");

        assert_eq!(value["command"], "docs topics");
        assert_eq!(value["data"]["topics"][0]["id"], "intro");
        assert_eq!(value["data"]["topics"][0]["see_also"][0], "commands/help");
    }

    #[derive(Default)]
    struct FlushCountingSink {
        bytes: Vec<u8>,
        flushes: usize,
    }

    impl io::Write for FlushCountingSink {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            self.bytes.extend_from_slice(buf);
            Ok(buf.len())
        }

        fn flush(&mut self) -> io::Result<()> {
            self.flushes += 1;
            Ok(())
        }
    }

    struct BrokenPipeSink;

    impl io::Write for BrokenPipeSink {
        fn write(&mut self, _buf: &[u8]) -> io::Result<usize> {
            Err(io::Error::from(io::ErrorKind::BrokenPipe))
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    #[test]
    fn jsonl_writer_emits_ordered_records_and_flushes_each_line() {
        let mut sink = FlushCountingSink::default();
        {
            let mut writer = JsonlWriter::new(&mut sink, CommandName::Change);
            writer.begin().expect("begin record should write");
            writer
                .item(&json!({"time": "5ns", "signals": []}))
                .expect("item record should write");
            writer
                .diagnostic(&Diagnostic::warning(
                    WarningDiagnosticCode::OutputTruncated,
                    "truncated output to 1 entries",
                ))
                .expect("diagnostic record should write");
            writer.end(true).expect("end record should write");
            assert_eq!(writer.item_count(), 1);
            assert_eq!(writer.diagnostic_count(), 1);
        }

        assert_eq!(sink.flushes, 4);
        let output = String::from_utf8(sink.bytes).expect("JSONL should be UTF-8");
        assert!(output.ends_with('\n'));
        let lines = output.lines().collect::<Vec<_>>();
        assert_eq!(lines.len(), 4);
        let records = lines
            .iter()
            .map(|line| serde_json::from_str::<Value>(line).expect("line should parse"))
            .collect::<Vec<_>>();

        assert_eq!(records[0]["type"], "begin");
        assert_eq!(records[0]["seq"], 0);
        assert_eq!(records[0]["command"], "change");
        assert_eq!(records[0]["$schema"], STREAM_SCHEMA_URL);
        assert_eq!(records[1]["type"], "item");
        assert_eq!(records[1]["seq"], 1);
        assert_eq!(records[2]["type"], "diagnostic");
        assert_eq!(records[2]["diagnostic"]["code"], "WPK-W0002");
        assert_eq!(records[3]["type"], "end");
        assert_eq!(records[3]["summary"]["items"], 1);
        assert_eq!(records[3]["summary"]["diagnostics"], 1);
        assert_eq!(records[3]["summary"]["truncated"], true);
    }

    #[test]
    fn jsonl_writer_maps_broken_pipe_without_internal_fatal() {
        let mut writer = JsonlWriter::new(BrokenPipeSink, CommandName::Info);
        let error = writer.begin().expect_err("broken pipe should be returned");
        assert!(matches!(error, crate::error::WavepeekError::BrokenPipe));
    }

    #[test]
    fn jsonl_result_adapter_emits_items_diagnostics_and_summary() {
        let result = CommandResult {
            command: CommandName::Scope,
            output_mode: OutputMode::Jsonl,
            human_options: HumanRenderOptions::default(),
            data: CommandData::Scope(vec![crate::engine::scope::ScopeEntry {
                path: "top".to_string(),
                depth: 0,
                kind: "module".to_string(),
            }]),
            diagnostics: vec![Diagnostic::warning(
                WarningDiagnosticCode::OutputTruncated,
                "truncated output to 1 entries",
            )],
        };
        let mut sink = Vec::new();
        let mut writer = JsonlWriter::new(&mut sink, CommandName::Scope);
        write_jsonl_result(result, &mut writer).expect("JSONL adapter should write");

        let output = String::from_utf8(sink).expect("JSONL should be UTF-8");
        let records = output
            .lines()
            .map(|line| serde_json::from_str::<Value>(line).expect("line should parse"))
            .collect::<Vec<_>>();
        assert_eq!(records[0]["type"], "begin");
        assert_eq!(records[1]["item"]["path"], "top");
        assert_eq!(records[2]["diagnostic"]["code"], "WPK-W0002");
        assert_eq!(records[3]["summary"]["truncated"], true);
    }

    #[test]
    fn property_rows_render_as_time_and_kind_lines() {
        let rendered = render_human(
            &CommandData::Property(vec![
                crate::engine::property::PropertyCaptureRow {
                    time: "10ns".to_string(),
                    sample_time: "10ns".to_string(),
                    kind: crate::engine::property::PropertyResultKind::Assert,
                },
                crate::engine::property::PropertyCaptureRow {
                    time: "25ns".to_string(),
                    sample_time: "24ns".to_string(),
                    kind: crate::engine::property::PropertyResultKind::Deassert,
                },
            ]),
            HumanRenderOptions::default(),
        );

        assert_eq!(rendered, "@10ns assert\n@25ns sample@24ns deassert");
    }

    #[test]
    fn scope_tree_render_matches_linux_tree_style() {
        let rendered = render_human(
            &CommandData::Scope(vec![
                crate::engine::scope::ScopeEntry {
                    path: "top".to_string(),
                    depth: 0,
                    kind: "module".to_string(),
                },
                crate::engine::scope::ScopeEntry {
                    path: "top.cpu".to_string(),
                    depth: 1,
                    kind: "module".to_string(),
                },
                crate::engine::scope::ScopeEntry {
                    path: "top.cpu.alu".to_string(),
                    depth: 2,
                    kind: "function".to_string(),
                },
                crate::engine::scope::ScopeEntry {
                    path: "top.cpu.regs".to_string(),
                    depth: 2,
                    kind: "module".to_string(),
                },
                crate::engine::scope::ScopeEntry {
                    path: "top.mem".to_string(),
                    depth: 1,
                    kind: "module".to_string(),
                },
            ]),
            HumanRenderOptions {
                scope_tree: true,
                signals_abs: false,
            },
        );

        assert_eq!(
            rendered,
            "top kind=module\n├── cpu kind=module\n│   ├── alu kind=function\n│   └── regs kind=module\n└── mem kind=module"
        );
    }

    #[test]
    fn value_human_render_is_deterministic_and_compact() {
        let rendered = render_human(
            &CommandData::Value(vec![crate::engine::value::ValueSnapshot {
                time: "10ns".to_string(),
                signals: vec![
                    crate::engine::value::ValueSignalValue {
                        display: "clk".to_string(),
                        path: "top.clk".to_string(),
                        value: "1'h1".to_string(),
                    },
                    crate::engine::value::ValueSignalValue {
                        display: "data".to_string(),
                        path: "top.data".to_string(),
                        value: "8'h0f".to_string(),
                    },
                ],
            }]),
            HumanRenderOptions::default(),
        );

        assert_eq!(rendered, "@10ns clk=1'h1 data=8'h0f");
    }

    #[test]
    fn change_human_render_is_single_line_per_snapshot() {
        let rendered = render_human(
            &CommandData::Change(vec![crate::engine::change::ChangeSnapshot {
                time: "5ns".to_string(),
                sample_time: "4ns".to_string(),
                signals: vec![
                    crate::engine::change::ChangeSignalValue {
                        display: "clk".to_string(),
                        path: "top.clk".to_string(),
                        value: "1'h1".to_string(),
                    },
                    crate::engine::change::ChangeSignalValue {
                        display: "data".to_string(),
                        path: "top.data".to_string(),
                        value: "8'h00".to_string(),
                    },
                ],
            }]),
            HumanRenderOptions::default(),
        );

        assert_eq!(rendered, "@5ns sample@4ns clk=1'h1 data=8'h00");
    }

    #[test]
    fn render_human_exercises_schema_signal_and_docs_search_variants() {
        let schema = render_human(
            &CommandData::Schema("{\"type\":\"object\"}".to_string()),
            HumanRenderOptions::default(),
        );
        assert_eq!(schema, "{\"type\":\"object\"}");

        let info = render_human(
            &CommandData::Info(crate::engine::info::InfoData {
                time_unit: "1ps".to_string(),
                time_start: "0ps".to_string(),
                time_end: "10ps".to_string(),
            }),
            HumanRenderOptions::default(),
        );
        assert_eq!(info, "time_unit: 1ps\ntime_start: 0ps\ntime_end: 10ps");

        let flat_scopes = render_human(
            &CommandData::Scope(vec![crate::engine::scope::ScopeEntry {
                path: "top.cpu".to_string(),
                depth: 1,
                kind: "module".to_string(),
            }]),
            HumanRenderOptions::default(),
        );
        assert_eq!(flat_scopes, "1 top.cpu kind=module");

        let signals = vec![
            crate::engine::signal::SignalEntry {
                display: "clk".to_string(),
                name: "clk".to_string(),
                path: "top.clk".to_string(),
                kind: "wire".to_string(),
                width: Some(1),
            },
            crate::engine::signal::SignalEntry {
                display: "status".to_string(),
                name: "status".to_string(),
                path: "top.status".to_string(),
                kind: "event".to_string(),
                width: None,
            },
        ];
        let rendered = render_human(
            &CommandData::Signal(signals.clone()),
            HumanRenderOptions {
                scope_tree: false,
                signals_abs: true,
            },
        );
        assert_eq!(rendered, "top.clk kind=wire width=1\ntop.status kind=event");
        assert_eq!(signal_display_name(&signals[0], true), "top.clk");
        assert_eq!(signal_display_name(&signals[0], false), "clk");

        let docs_search = render_human(
            &CommandData::DocsSearch(crate::engine::DocsSearchData {
                query: "change".to_string(),
                matches: vec![crate::engine::DocsSearchMatchData {
                    topic: crate::docs::TopicSummary {
                        id: "commands/change".to_string(),
                        title: "Change command".to_string(),
                        description: "Find changes.".to_string(),
                        section: "commands".to_string(),
                        see_also: vec![],
                    },
                    match_kind: crate::docs::MatchKind::IdPrefix,
                    matched_tokens: 1,
                }],
            }),
            HumanRenderOptions::default(),
        );
        assert_eq!(docs_search, "commands/change  Find changes.");
    }

    #[test]
    fn helper_renderers_exercise_empty_tree_and_sibling_detection() {
        assert_eq!(render_scope_tree(&[]), "");

        let scopes = vec![
            crate::engine::scope::ScopeEntry {
                path: "top".to_string(),
                depth: 0,
                kind: "module".to_string(),
            },
            crate::engine::scope::ScopeEntry {
                path: "top.cpu".to_string(),
                depth: 1,
                kind: "module".to_string(),
            },
            crate::engine::scope::ScopeEntry {
                path: "top.mem".to_string(),
                depth: 1,
                kind: "module".to_string(),
            },
        ];
        assert!(!scope_entry_is_last_sibling(&scopes, 1));
        assert!(scope_entry_is_last_sibling(&scopes, 2));
    }

    #[test]
    fn output_envelope_constructor_sets_schema_and_diagnostics() {
        let envelope = OutputEnvelope::with_diagnostics(
            "docs show",
            serde_json::json!({"ok": true}),
            vec![Diagnostic::warning(
                WarningDiagnosticCode::EmptyResult,
                "careful",
            )],
        );
        assert_eq!(envelope.schema, SCHEMA_URL);
        assert_eq!(envelope.command, "docs show");
        assert_eq!(envelope.diagnostics[0].code(), Some("WPK-W0003"));
    }

    #[test]
    fn write_entrypoint_exercises_json_empty_human_and_diagnostic_paths() {
        write(CommandResult {
            command: CommandName::Schema,
            output_mode: OutputMode::Json,
            human_options: HumanRenderOptions::default(),
            data: CommandData::Schema("{}".to_string()),
            diagnostics: Vec::new(),
        })
        .expect("json output should write");

        write(CommandResult {
            command: CommandName::DocsSearch,
            output_mode: OutputMode::Human,
            human_options: HumanRenderOptions::default(),
            data: CommandData::DocsSearch(crate::engine::DocsSearchData {
                query: "none".to_string(),
                matches: vec![],
            }),
            diagnostics: vec![Diagnostic::warning(
                WarningDiagnosticCode::EmptyResult,
                "nothing matched",
            )],
        })
        .expect("empty human output with diagnostics should write");

        write(CommandResult {
            command: CommandName::Info,
            output_mode: OutputMode::Human,
            human_options: HumanRenderOptions::default(),
            data: CommandData::Text("already-newline\n".to_string()),
            diagnostics: Vec::new(),
        })
        .expect("newline-terminated human output should not add a second newline");
    }
}
