use schemars::SchemaGenerator;
use serde_json::{Map, Value, json};

use super::common::{
    CanonicalPath, ContractDiagnostic, NormalizedTime, SampledValue, ScopeKind, SignalKind,
};
use super::output::{
    ChangeSignalValue, ChangeSnapshot, DocsSearchData, DocsSearchMatch, DocsTopicsData, InfoData,
    PropertyRow, SampledSignalValue, ScopeEntry, SignalEntry, TopicSummary, ValueSnapshot,
};
use super::stream::{BeginRecord, DiagnosticRecord, EndRecord};

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
    let mut object = generated_output_payload_defs();
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
    Value::Object(object)
}

fn stream_defs() -> Value {
    let mut object = generated_waveform_payload_defs();
    object.extend(generated_stream_record_defs());
    object.insert(
        "streamCommand".to_string(),
        json!({"type": "string", "enum": stream_commands()}),
    );
    object.insert(
        "sequence".to_string(),
        json!({"type": "integer", "minimum": 0}),
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
    Value::Object(object)
}

fn generated_output_payload_defs() -> Map<String, Value> {
    let mut defs = generated_waveform_payload_defs();
    defs.extend(generated_docs_payload_defs());
    defs
}

fn generated_waveform_payload_defs() -> Map<String, Value> {
    let mut generator = SchemaGenerator::default();
    generator.subschema_for::<ContractDiagnostic<'static>>();
    generator.subschema_for::<NormalizedTime<'static>>();
    generator.subschema_for::<CanonicalPath<'static>>();
    generator.subschema_for::<SampledValue<'static>>();
    generator.subschema_for::<ScopeKind<'static>>();
    generator.subschema_for::<SignalKind<'static>>();
    generator.subschema_for::<InfoData<'static>>();
    generator.subschema_for::<ScopeEntry<'static>>();
    generator.subschema_for::<SignalEntry<'static>>();
    generator.subschema_for::<SampledSignalValue<'static>>();
    generator.subschema_for::<ChangeSignalValue<'static>>();
    generator.subschema_for::<ValueSnapshot<'static>>();
    generator.subschema_for::<ChangeSnapshot<'static>>();
    generator.subschema_for::<PropertyRow<'static>>();
    generator.take_definitions(true)
}

fn generated_docs_payload_defs() -> Map<String, Value> {
    let mut generator = SchemaGenerator::default();
    generator.subschema_for::<TopicSummary<'static>>();
    generator.subschema_for::<DocsTopicsData<'static>>();
    generator.subschema_for::<DocsSearchMatch<'static>>();
    generator.subschema_for::<DocsSearchData<'static>>();
    generator.take_definitions(true)
}

fn generated_stream_record_defs() -> Map<String, Value> {
    let mut generator = SchemaGenerator::default();
    generator.subschema_for::<BeginRecord>();
    generator.subschema_for::<DiagnosticRecord<'static>>();
    generator.subschema_for::<EndRecord>();
    generator.take_definitions(true)
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
