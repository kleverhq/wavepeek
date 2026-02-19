use serde::Serialize;

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
    pub warnings: Vec<String>,
}

impl<T> OutputEnvelope<T>
where
    T: Serialize,
{
    pub fn with_warnings(command: impl Into<String>, data: T, warnings: Vec<String>) -> Self {
        Self {
            schema: SCHEMA_URL,
            command: command.into(),
            data,
            warnings,
        }
    }
}

pub fn write(result: CommandResult) -> Result<(), WavepeekError> {
    if !result.json {
        let output = render_human(&result.data, result.human_options);
        if !output.is_empty() {
            write_stdout(output.as_str());
        }
        emit_human_warnings(&result.warnings);
        return Ok(());
    }

    let json = render_json(result)?;
    println!("{json}");
    Ok(())
}

fn render_json(result: CommandResult) -> Result<String, WavepeekError> {
    let envelope =
        OutputEnvelope::with_warnings(result.command.as_str(), result.data, result.warnings);
    serde_json::to_string(&envelope)
        .map_err(|error| WavepeekError::Internal(format!("failed to serialize output: {error}")))
}

fn render_human(data: &CommandData, options: HumanRenderOptions) -> String {
    match data {
        CommandData::Schema(schema) => schema.clone(),
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
        entry.name.as_str()
    }
}

fn emit_human_warnings(warnings: &[String]) {
    for warning in warnings {
        eprintln!("warning: {warning}");
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
mod tests {
    use serde_json::Value;

    use crate::engine::{CommandData, CommandName, CommandResult, HumanRenderOptions};
    use crate::schema_contract::SCHEMA_URL;

    use super::{render_human, render_json};

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
            warnings: vec![],
        };

        let json = render_json(result).expect("json serialization should succeed");
        let value: Value = serde_json::from_str(&json).expect("json should parse");

        assert_eq!(value["$schema"], SCHEMA_URL);
        assert!(value.get("schema_version").is_none());
        assert_eq!(value["command"], "info");
        assert!(value["data"].is_object());
        assert!(value["warnings"].is_array());
    }

    #[test]
    fn json_envelope_preserves_warnings_for_scope() {
        let result = CommandResult {
            command: CommandName::Scope,
            json: true,
            human_options: HumanRenderOptions::default(),
            data: CommandData::Scope(vec![crate::engine::scope::ScopeEntry {
                path: "top.cpu".to_string(),
                depth: 1,
                kind: "module".to_string(),
            }]),
            warnings: vec!["truncated to 1 entries".to_string()],
        };

        let json = render_json(result).expect("json serialization should succeed");
        let value: Value = serde_json::from_str(&json).expect("json should parse");

        assert_eq!(value["command"], "scope");
        assert_eq!(value["warnings"][0], "truncated to 1 entries");
        assert_eq!(value["data"][0]["path"], "top.cpu");
        assert_eq!(value["data"][0]["depth"], 1);
        assert_eq!(value["data"][0]["kind"], "module");
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
}
