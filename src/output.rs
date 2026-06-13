use serde::Serialize;

use crate::diagnostic::{Diagnostic, DiagnosticKind};
use crate::engine::{CommandData, CommandResult, HumanRenderOptions};
use crate::error::WavepeekError;
use crate::schema_contract::SCHEMA_URL;

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

pub fn write(result: CommandResult) -> Result<(), WavepeekError> {
    if !result.json {
        let output = render_human(&result.data, result.human_options);
        if !output.is_empty() {
            write_stdout(output.as_str());
        }
        emit_human_diagnostics(&result.diagnostics);
        return Ok(());
    }

    let json = render_json(result)?;
    println!("{json}");
    Ok(())
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
        CommandData::Property(rows) => rows
            .iter()
            .map(|row| format!("@{} {}", row.time, row.kind))
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
    use serde_json::Value;

    use crate::diagnostic::{Diagnostic, WarningDiagnosticCode};
    use crate::engine::{CommandData, CommandName, CommandResult, HumanRenderOptions};
    use crate::schema_contract::SCHEMA_URL;

    use super::{
        OutputEnvelope, render_human, render_json, render_scope_tree, scope_entry_is_last_sibling,
        signal_display_name, write,
    };

    #[test]
    fn json_envelope_has_required_shape_for_info() {
        let result = CommandResult {
            command: CommandName::Info,
            json: true,
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
            json: true,
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
            json: true,
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

    #[test]
    fn property_rows_render_as_time_and_kind_lines() {
        let rendered = render_human(
            &CommandData::Property(vec![
                crate::engine::property::PropertyCaptureRow {
                    time: "10ns".to_string(),
                    kind: crate::engine::property::PropertyResultKind::Assert,
                },
                crate::engine::property::PropertyCaptureRow {
                    time: "25ns".to_string(),
                    kind: crate::engine::property::PropertyResultKind::Deassert,
                },
            ]),
            HumanRenderOptions::default(),
        );

        assert_eq!(rendered, "@10ns assert\n@25ns deassert");
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

        assert_eq!(rendered, "@5ns clk=1'h1 data=8'h00");
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
            json: true,
            human_options: HumanRenderOptions::default(),
            data: CommandData::Schema("{}".to_string()),
            diagnostics: Vec::new(),
        })
        .expect("json output should write");

        write(CommandResult {
            command: CommandName::DocsSearch,
            json: false,
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
            json: false,
            human_options: HumanRenderOptions::default(),
            data: CommandData::Text("already-newline\n".to_string()),
            diagnostics: Vec::new(),
        })
        .expect("newline-terminated human output should not add a second newline");
    }
}
