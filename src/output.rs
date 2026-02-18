use serde::Serialize;

use crate::engine::{CommandData, CommandResult, HumanRenderOptions};
use crate::error::WavepeekError;

pub const SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Serialize)]
pub struct OutputEnvelope<T>
where
    T: Serialize,
{
    pub schema_version: u32,
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
            schema_version: SCHEMA_VERSION,
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
            println!("{output}");
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
        CommandData::Info(info) => {
            let mut lines = Vec::new();
            lines.push(format!("time_unit: {}", info.time_unit));
            lines.push(format!("time_start: {}", info.time_start));
            lines.push(format!("time_end: {}", info.time_end));
            lines.join("\n")
        }
        CommandData::Modules(scopes) => {
            if options.modules_tree {
                render_modules_tree(scopes)
            } else {
                scopes
                    .iter()
                    .map(|entry| format!("{} {}", entry.depth, entry.path))
                    .collect::<Vec<_>>()
                    .join("\n")
            }
        }
        CommandData::Signals(signals) => signals
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

fn render_modules_tree(scopes: &[crate::engine::modules::ModulesEntry]) -> String {
    if scopes.is_empty() {
        return String::new();
    }

    let mut lines = Vec::with_capacity(scopes.len());
    let mut ancestor_last = Vec::new();

    for (index, entry) in scopes.iter().enumerate() {
        let label = entry.path.rsplit('.').next().unwrap_or(entry.path.as_str());
        let is_last = modules_entry_is_last_sibling(scopes, index);

        if entry.depth == 0 {
            lines.push(label.to_string());
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
            line.push_str(label);
            lines.push(line);
        }

        ancestor_last.truncate(entry.depth);
        ancestor_last.push(is_last);
    }

    lines.join("\n")
}

fn modules_entry_is_last_sibling(
    scopes: &[crate::engine::modules::ModulesEntry],
    index: usize,
) -> bool {
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

fn signal_display_name(entry: &crate::engine::signals::SignalEntry, abs: bool) -> &str {
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

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use crate::engine::{CommandData, CommandName, CommandResult, HumanRenderOptions};

    use super::{SCHEMA_VERSION, render_human, render_json};

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

        assert_eq!(value["schema_version"], SCHEMA_VERSION);
        assert_eq!(value["command"], "info");
        assert!(value["data"].is_object());
        assert!(value["warnings"].is_array());
    }

    #[test]
    fn json_envelope_preserves_warnings_for_modules() {
        let result = CommandResult {
            command: CommandName::Modules,
            json: true,
            human_options: HumanRenderOptions::default(),
            data: CommandData::Modules(vec![crate::engine::modules::ModulesEntry {
                path: "top.cpu".to_string(),
                depth: 1,
            }]),
            warnings: vec!["truncated to 1 entries".to_string()],
        };

        let json = render_json(result).expect("json serialization should succeed");
        let value: Value = serde_json::from_str(&json).expect("json should parse");

        assert_eq!(value["command"], "modules");
        assert_eq!(value["warnings"][0], "truncated to 1 entries");
        assert_eq!(value["data"][0]["path"], "top.cpu");
        assert_eq!(value["data"][0]["depth"], 1);
    }

    #[test]
    fn modules_tree_render_matches_linux_tree_style() {
        let rendered = render_human(
            &CommandData::Modules(vec![
                crate::engine::modules::ModulesEntry {
                    path: "top".to_string(),
                    depth: 0,
                },
                crate::engine::modules::ModulesEntry {
                    path: "top.cpu".to_string(),
                    depth: 1,
                },
                crate::engine::modules::ModulesEntry {
                    path: "top.cpu.alu".to_string(),
                    depth: 2,
                },
                crate::engine::modules::ModulesEntry {
                    path: "top.cpu.regs".to_string(),
                    depth: 2,
                },
                crate::engine::modules::ModulesEntry {
                    path: "top.mem".to_string(),
                    depth: 1,
                },
            ]),
            HumanRenderOptions {
                modules_tree: true,
                signals_abs: false,
            },
        );

        assert_eq!(rendered, "top\n├── cpu\n│   ├── alu\n│   └── regs\n└── mem");
    }
}
