use serde_json::{Value, json};

use crate::waveform::{STABLE_SCOPE_KIND_ALIASES, STABLE_SIGNAL_KIND_ALIASES};

pub const OUTPUT_SCHEMA_ID: &str = "wavepeek.output";
pub const STREAM_SCHEMA_ID: &str = "wavepeek.stream-record";
pub const OUTPUT_SCHEMA_VERSION_STR: &str = "2.0";
pub const STREAM_SCHEMA_VERSION_STR: &str = "2.0";
pub const OUTPUT_SCHEMA_ARTIFACT_PATH: &str = "schema/output.json";
pub const STREAM_SCHEMA_ARTIFACT_PATH: &str = "schema/stream.json";
pub const SCHEMA_CATALOG_PATH: &str = "schema/catalog.json";
pub const OUTPUT_SCHEMA_ARTIFACT_NAME: &str = "schema-output-v2.0.json";
pub const STREAM_SCHEMA_ARTIFACT_NAME: &str = "schema-stream-v2.0.json";
pub const OUTPUT_SCHEMA_URL: &str = "https://kleverhq.github.io/wavepeek/schema-output-v2.0.json";
pub const STREAM_SCHEMA_URL: &str = "https://kleverhq.github.io/wavepeek/schema-stream-v2.0.json";

const JSON_SCHEMA_DRAFT: &str = "https://json-schema.org/draft/2020-12/schema";

pub fn output_schema_json() -> String {
    pretty_json(&output_schema_value())
}

pub fn stream_schema_json() -> String {
    pretty_json(&stream_schema_value())
}

pub fn catalog_json() -> String {
    pretty_json(&catalog_value())
}

fn pretty_json(value: &Value) -> String {
    let mut output = serde_json::to_string_pretty(value).expect("schema JSON should serialize");
    output.push('\n');
    output
}

fn output_schema_value() -> Value {
    json!({
        "$schema": JSON_SCHEMA_DRAFT,
        "$id": OUTPUT_SCHEMA_URL,
        "title": "wavepeek JSON output envelope",
        "description": "Canonical schema for wavepeek --json command outputs.",
        "type": "object",
        "additionalProperties": true,
        "required": ["$schema", "command", "data", "diagnostics"],
        "properties": {
            "$schema": {"type": "string", "const": OUTPUT_SCHEMA_URL},
            "command": {
                "type": "string",
                "enum": output_commands(),
            },
            "data": {
                "anyOf": [
                    ref_schema("infoData"),
                    ref_schema("scopeData"),
                    ref_schema("signalData"),
                    ref_schema("valueData"),
                    ref_schema("changeData"),
                    ref_schema("propertyData"),
                    ref_schema("docsTopicsData"),
                    ref_schema("docsSearchData"),
                ],
            },
            "diagnostics": {"type": "array", "items": ref_schema("diagnostic")},
        },
        "allOf": [
            command_data_branch("info", "infoData"),
            command_data_branch("scope", "scopeData"),
            command_data_branch("signal", "signalData"),
            command_data_branch("value", "valueData"),
            command_data_branch("change", "changeData"),
            command_data_branch("property", "propertyData"),
            command_data_branch("docs topics", "docsTopicsData"),
            command_data_branch("docs search", "docsSearchData"),
        ],
        "$defs": output_defs(),
    })
}

fn stream_schema_value() -> Value {
    json!({
        "$schema": JSON_SCHEMA_DRAFT,
        "$id": STREAM_SCHEMA_URL,
        "title": "wavepeek JSONL stream record",
        "description": "Canonical schema for one wavepeek --jsonl stream record. Validate each JSONL line independently; stream order and summary counts are stateful invariants.",
        "oneOf": [
            ref_schema("beginRecord"),
            ref_schema("itemRecord"),
            ref_schema("diagnosticRecord"),
            ref_schema("endRecord"),
        ],
        "$defs": stream_defs(),
    })
}

fn catalog_value() -> Value {
    json!({
        "families": [
            {
                "id": OUTPUT_SCHEMA_ID,
                "version": OUTPUT_SCHEMA_VERSION_STR,
                "path": OUTPUT_SCHEMA_ARTIFACT_PATH,
                "url": OUTPUT_SCHEMA_URL,
            },
            {
                "id": STREAM_SCHEMA_ID,
                "version": STREAM_SCHEMA_VERSION_STR,
                "path": STREAM_SCHEMA_ARTIFACT_PATH,
                "url": STREAM_SCHEMA_URL,
            },
        ],
    })
}

fn output_defs() -> Value {
    let mut defs = common_payload_defs();
    let object = defs
        .as_object_mut()
        .expect("common payload defs should be an object");
    object.insert(
        "scopeData".to_string(),
        json!({"type": "array", "items": ref_schema("scopeEntry")}),
    );
    object.insert(
        "signalData".to_string(),
        json!({"type": "array", "items": ref_schema("signalEntry")}),
    );
    object.insert(
        "valueData".to_string(),
        json!({
            "type": "array",
            "description": "Ordered snapshots sampled by the value command.",
            "items": ref_schema("valueSnapshot"),
        }),
    );
    object.insert(
        "changeData".to_string(),
        json!({"type": "array", "items": ref_schema("changeSnapshot")}),
    );
    object.insert(
        "propertyData".to_string(),
        json!({"type": "array", "items": ref_schema("propertyRow")}),
    );
    object.insert("topicSummary".to_string(), topic_summary_def());
    object.insert(
        "docsTopicsData".to_string(),
        json!({
            "type": "object",
            "additionalProperties": true,
            "required": ["topics"],
            "properties": {
                "topics": {
                    "type": "array",
                    "items": ref_schema("topicSummary"),
                    "description": "Available embedded documentation topics."
                }
            }
        }),
    );
    object.insert("docsSearchMatch".to_string(), docs_search_match_def());
    object.insert(
        "docsSearchData".to_string(),
        json!({
            "type": "object",
            "additionalProperties": true,
            "required": ["query", "matches"],
            "properties": {
                "query": {"type": "string", "description": "Normalized search query used for the docs search."},
                "matches": {"type": "array", "items": ref_schema("docsSearchMatch")}
            }
        }),
    );
    defs
}

fn stream_defs() -> Value {
    let mut defs = common_payload_defs();
    let object = defs
        .as_object_mut()
        .expect("common payload defs should be an object");
    object.insert(
        "streamCommand".to_string(),
        json!({"type": "string", "enum": stream_commands()}),
    );
    object.insert(
        "sequence".to_string(),
        json!({"type": "integer", "minimum": 0}),
    );
    object.insert(
        "beginRecord".to_string(),
        json!({
            "type": "object",
            "additionalProperties": true,
            "required": ["type", "seq", "command", "$schema"],
            "properties": {
                "type": {"const": "begin"},
                "seq": ref_schema("sequence"),
                "command": ref_schema("streamCommand"),
                "$schema": {"type": "string", "const": STREAM_SCHEMA_URL}
            }
        }),
    );
    object.insert(
        "itemRecord".to_string(),
        json!({
            "oneOf": [
                ref_schema("infoItemRecord"),
                ref_schema("scopeItemRecord"),
                ref_schema("signalItemRecord"),
                ref_schema("valueItemRecord"),
                ref_schema("changeItemRecord"),
                ref_schema("propertyItemRecord"),
            ]
        }),
    );
    for (alias, wrapper) in [
        ("infoItemRecord", "itemRecordForInfoData"),
        ("scopeItemRecord", "itemRecordForScopeEntry"),
        ("signalItemRecord", "itemRecordForSignalEntry"),
        ("valueItemRecord", "itemRecordForValueSnapshot"),
        ("changeItemRecord", "itemRecordForChangeSnapshot"),
        ("propertyItemRecord", "itemRecordForPropertyRow"),
    ] {
        object.insert(alias.to_string(), ref_schema(wrapper));
    }
    for (name, command, item_ref) in [
        ("itemRecordForInfoData", "info", "infoData"),
        ("itemRecordForScopeEntry", "scope", "scopeEntry"),
        ("itemRecordForSignalEntry", "signal", "signalEntry"),
        ("itemRecordForValueSnapshot", "value", "valueSnapshot"),
        ("itemRecordForChangeSnapshot", "change", "changeSnapshot"),
        ("itemRecordForPropertyRow", "property", "propertyRow"),
    ] {
        object.insert(name.to_string(), item_record_for(command, item_ref));
    }
    object.insert(
        "diagnosticRecord".to_string(),
        json!({
            "type": "object",
            "additionalProperties": true,
            "required": ["type", "seq", "command", "diagnostic"],
            "properties": {
                "type": {"const": "diagnostic"},
                "seq": ref_schema("sequence"),
                "command": ref_schema("streamCommand"),
                "diagnostic": ref_schema("diagnostic"),
            }
        }),
    );
    object.insert(
        "endRecord".to_string(),
        json!({
            "type": "object",
            "additionalProperties": true,
            "required": ["type", "seq", "command", "summary"],
            "properties": {
                "type": {"const": "end"},
                "seq": ref_schema("sequence"),
                "command": ref_schema("streamCommand"),
                "summary": ref_schema("streamSummary"),
            }
        }),
    );
    object.insert(
        "streamSummary".to_string(),
        json!({
            "type": "object",
            "additionalProperties": true,
            "required": ["status", "items", "diagnostics", "truncated"],
            "properties": {
                "status": {"const": "ok"},
                "items": {"type": "integer", "minimum": 0},
                "diagnostics": {"type": "integer", "minimum": 0},
                "truncated": {"type": "boolean"},
            }
        }),
    );
    defs
}

fn common_payload_defs() -> Value {
    json!({
        "diagnostic": diagnostic_def(),
        "normalizedTime": {
            "type": "string",
            "description": "Normalized timestamp rendered in the dump's time unit, for example 10ns."
        },
        "canonicalPath": {
            "type": "string",
            "description": "Canonical dot-separated hierarchy path emitted by wavepeek for a scope or signal."
        },
        "sampledValue": {
            "type": "string",
            "description": "Stable sampled signal value formatted as a Verilog-style literal string."
        },
        "scopeKind": {
            "type": "string",
            "description": "Stable scope kind alias emitted by wavepeek for the selected scope.",
            "enum": STABLE_SCOPE_KIND_ALIASES,
        },
        "signalKind": {
            "type": "string",
            "description": "Stable signal kind alias emitted by wavepeek for the selected signal.",
            "enum": STABLE_SIGNAL_KIND_ALIASES,
        },
        "infoData": {
            "type": "object",
            "additionalProperties": true,
            "required": ["time_unit", "time_start", "time_end"],
            "properties": {
                "time_unit": {"type": "string", "description": "Dump time unit used to normalize timestamps in this waveform, for example 1ns."},
                "time_start": {"$ref": "#/$defs/normalizedTime", "description": "Earliest timestamp present in the waveform."},
                "time_end": {"$ref": "#/$defs/normalizedTime", "description": "Latest timestamp present in the waveform."},
            }
        },
        "scopeEntry": {
            "type": "object",
            "additionalProperties": true,
            "required": ["path", "depth", "kind"],
            "properties": {
                "path": {"$ref": "#/$defs/canonicalPath", "description": "Canonical hierarchy path for this scope entry."},
                "depth": {"type": "integer", "minimum": 0, "description": "Zero-based scope depth from the dump root used by list and tree renderers."},
                "kind": {"$ref": "#/$defs/scopeKind", "description": "Stable scope kind alias for this scope entry."},
            }
        },
        "signalEntry": {
            "type": "object",
            "additionalProperties": true,
            "required": ["name", "path", "kind"],
            "properties": {
                "name": {"type": "string", "description": "Signal name as declared inside its immediate parent scope."},
                "path": {"$ref": "#/$defs/canonicalPath", "description": "Canonical hierarchy path for this signal entry."},
                "kind": {"$ref": "#/$defs/signalKind", "description": "Stable signal kind alias for this signal entry."},
                "width": {"type": "integer", "minimum": 1, "description": "Declared packed bit width when the waveform backend reports one."},
            }
        },
        "sampledSignalValue": {
            "type": "object",
            "additionalProperties": true,
            "required": ["path", "value"],
            "properties": {
                "path": {"$ref": "#/$defs/canonicalPath", "description": "Canonical path of the sampled signal."},
                "value": {"$ref": "#/$defs/sampledValue", "description": "Sampled value for this signal in the selected timestamp snapshot."},
            }
        },
        "valueSnapshot": {
            "type": "object",
            "additionalProperties": true,
            "required": ["time", "signals"],
            "properties": {
                "time": {"$ref": "#/$defs/normalizedTime", "description": "Timestamp requested by the value command."},
                "signals": {"type": "array", "items": ref_schema("sampledSignalValue"), "description": "Signal values sampled at this timestamp."},
            }
        },
        "changeSignalValue": {
            "type": "object",
            "additionalProperties": true,
            "required": ["path", "value"],
            "properties": {
                "path": {"$ref": "#/$defs/canonicalPath", "description": "Canonical path of the changed signal."},
                "value": {"$ref": "#/$defs/sampledValue", "description": "Changed signal value at the reported sample point."},
            }
        },
        "changeSnapshot": {
            "type": "object",
            "additionalProperties": true,
            "required": ["time", "sample_time", "signals"],
            "properties": {
                "time": {"$ref": "#/$defs/normalizedTime", "description": "Trigger timestamp emitted by the change command."},
                "sample_time": {"$ref": "#/$defs/normalizedTime", "description": "Timestamp used to sample values for this change row."},
                "signals": {"type": "array", "items": ref_schema("changeSignalValue"), "description": "Changed signal values for this row."},
            }
        },
        "propertyRow": {
            "type": "object",
            "additionalProperties": true,
            "required": ["time", "sample_time", "kind"],
            "properties": {
                "time": {"$ref": "#/$defs/normalizedTime", "description": "Trigger timestamp emitted by the property command."},
                "sample_time": {"$ref": "#/$defs/normalizedTime", "description": "Timestamp used to evaluate the property expression."},
                "kind": {"type": "string", "enum": ["match", "assert", "deassert"], "description": "Property result kind captured for this row."},
            }
        },
    })
}

fn diagnostic_def() -> Value {
    json!({
        "type": "object",
        "additionalProperties": true,
        "required": ["kind", "message"],
        "properties": {
            "kind": {"type": "string", "enum": ["info", "warning", "error"]},
            "code": {"type": "string", "pattern": "^WPK-[WE][0-9]{4}$"},
            "message": {"type": "string"},
        },
        "allOf": [
            {
                "if": {"properties": {"kind": {"const": "warning"}}, "required": ["kind"]},
                "then": {"required": ["code"], "properties": {"code": {"pattern": "^WPK-W[0-9]{4}$"}}},
            },
            {
                "if": {"properties": {"kind": {"const": "error"}}, "required": ["kind"]},
                "then": {"required": ["code"], "properties": {"code": {"pattern": "^WPK-E[0-9]{4}$"}}},
            },
            {
                "if": {"properties": {"kind": {"const": "info"}}, "required": ["kind"]},
                "then": {"not": {"required": ["code"]}},
            },
        ],
    })
}

fn topic_summary_def() -> Value {
    json!({
        "type": "object",
        "additionalProperties": true,
        "required": ["id", "title", "description", "section"],
        "properties": {
            "id": {"type": "string", "description": "Stable documentation topic identifier."},
            "title": {"type": "string", "description": "Human-readable documentation topic title."},
            "description": {"type": "string", "description": "Short documentation topic description."},
            "section": {"type": "string", "description": "Documentation section that contains this topic."},
            "see_also": {"type": "array", "items": {"type": "string"}, "description": "Related documentation topic identifiers."},
        }
    })
}

fn docs_search_match_def() -> Value {
    json!({
        "type": "object",
        "additionalProperties": true,
        "required": ["topic", "match_kind", "matched_tokens"],
        "properties": {
            "topic": ref_schema("topicSummary"),
            "match_kind": {
                "type": "string",
                "enum": ["id_exact", "id_prefix", "title_exact", "title_or_description", "heading", "body"],
                "description": "How the topic matched the normalized search query."
            },
            "matched_tokens": {"type": "integer", "minimum": 1, "description": "Number of query tokens matched by this topic."},
        }
    })
}

fn command_data_branch(command: &str, data_def: &str) -> Value {
    json!({
        "if": {"properties": {"command": {"const": command}}, "required": ["command"]},
        "then": {"properties": {"data": ref_schema(data_def)}},
    })
}

fn item_record_for(command: &str, item_def: &str) -> Value {
    json!({
        "type": "object",
        "additionalProperties": true,
        "required": ["type", "seq", "command", "item"],
        "properties": {
            "type": {"const": "item"},
            "seq": ref_schema("sequence"),
            "command": {"const": command},
            "item": ref_schema(item_def),
        }
    })
}

fn ref_schema(def_name: &str) -> Value {
    json!({"$ref": format!("#/$defs/{def_name}")})
}

fn output_commands() -> Vec<&'static str> {
    vec![
        "info",
        "scope",
        "signal",
        "value",
        "change",
        "property",
        "docs topics",
        "docs search",
    ]
}

fn stream_commands() -> Vec<&'static str> {
    vec!["info", "scope", "signal", "value", "change", "property"]
}

#[cfg(test)]
mod tests {
    use super::{
        OUTPUT_SCHEMA_ARTIFACT_PATH, STREAM_SCHEMA_ARTIFACT_PATH, catalog_value,
        output_schema_value, stream_schema_value,
    };

    #[test]
    fn catalog_uses_canonical_paths_independent_of_output_directory() {
        let catalog = catalog_value();
        let families = catalog["families"]
            .as_array()
            .expect("families should be array");
        assert_eq!(families[0]["path"], OUTPUT_SCHEMA_ARTIFACT_PATH);
        assert_eq!(families[1]["path"], STREAM_SCHEMA_ARTIFACT_PATH);
    }

    #[test]
    fn generated_schemas_use_exact_schema_urls() {
        let output = output_schema_value();
        assert_eq!(
            output["properties"]["$schema"]["const"],
            "https://kleverhq.github.io/wavepeek/schema-output-v2.0.json"
        );
        let stream = stream_schema_value();
        assert_eq!(
            stream["$defs"]["beginRecord"]["properties"]["$schema"]["const"],
            "https://kleverhq.github.io/wavepeek/schema-stream-v2.0.json"
        );
    }
}
