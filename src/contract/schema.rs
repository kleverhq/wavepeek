use schemars::SchemaGenerator;
use serde_json::{Map, Value, json};

use super::common::{
    CanonicalPath, ContractDiagnostic, NormalizedTime, SampledValue, ScopeKind, SignalKind,
};
use super::input::{
    ExtractAtbSourceInput, ExtractAxiSourceInput, ExtractGenericSource, ExtractGenericSourcesInput,
};
use super::output::{
    ChangeSignalValue, ChangeSnapshot, DocsSearchData, DocsSearchMatch, DocsTopicsData,
    ExtractAtbData, ExtractAtbEvent, ExtractAtbMapping, ExtractAxiData, ExtractAxiMapping,
    ExtractAxiTransfer, ExtractGenericRow, ExtractPayloadValue, InfoData, PropertyRow,
    SampledSignalValue, ScopeEntry, SignalEntry, TopicSummary, ValueSnapshot,
};
use super::stream::{
    BeginRecord, DiagnosticRecord, EndRecord, ExtractAtbContext, ExtractAxiContext,
};
use super::{atb_schema, axi_schema};

pub const OUTPUT_SCHEMA_ID: &str = "wavepeek.output";
pub const STREAM_SCHEMA_ID: &str = "wavepeek.stream-record";
pub const INPUT_SCHEMA_ID: &str = "wavepeek.input";
pub const OUTPUT_SCHEMA_VERSION_STR: &str = "2.2";
pub const STREAM_SCHEMA_VERSION_STR: &str = "2.2";
pub const INPUT_SCHEMA_VERSION_STR: &str = "2.2";
pub const OUTPUT_SCHEMA_ARTIFACT_PATH: &str = "schema/output.json";
pub const STREAM_SCHEMA_ARTIFACT_PATH: &str = "schema/stream.json";
pub const INPUT_SCHEMA_ARTIFACT_PATH: &str = "schema/input.json";
pub const SCHEMA_CATALOG_PATH: &str = "schema/catalog.json";
pub const OUTPUT_SCHEMA_ARTIFACT_NAME: &str = "schema-output-v2.2.json";
pub const STREAM_SCHEMA_ARTIFACT_NAME: &str = "schema-stream-v2.2.json";
pub const INPUT_SCHEMA_ARTIFACT_NAME: &str = "schema-input-v2.2.json";
pub const OUTPUT_SCHEMA_URL: &str = "https://kleverhq.github.io/wavepeek/schema-output-v2.2.json";
pub const STREAM_SCHEMA_URL: &str = "https://kleverhq.github.io/wavepeek/schema-stream-v2.2.json";
pub const INPUT_SCHEMA_URL: &str = "https://kleverhq.github.io/wavepeek/schema-input-v2.2.json";
pub(crate) const GENERIC_INPUT_SCHEMA_URLS: &[&str] = &[
    INPUT_SCHEMA_URL,
    "https://kleverhq.github.io/wavepeek/schema-input-v2.1.json",
];

const JSON_SCHEMA_DRAFT: &str = "https://json-schema.org/draft/2020-12/schema";

pub(crate) fn is_supported_generic_input_schema_url(schema_url: &str) -> bool {
    GENERIC_INPUT_SCHEMA_URLS.contains(&schema_url)
}

pub fn output_schema_json() -> String {
    pretty_json(&output_schema_value())
}

pub fn stream_schema_json() -> String {
    pretty_json(&stream_schema_value())
}

pub fn input_schema_json() -> String {
    pretty_json(&input_schema_value())
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
                    ref_schema("extractAtbData"),
                    ref_schema("extractAxiData"),
                    ref_schema("extractGenericData"),
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
            command_data_branch("extract atb", "extractAtbData"),
            command_data_branch("extract axi", "extractAxiData"),
            command_data_branch("extract generic", "extractGenericData"),
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

fn input_schema_value() -> Value {
    let defs = generated_input_payload_defs();
    json!({
        "$schema": JSON_SCHEMA_DRAFT,
        "$id": INPUT_SCHEMA_URL,
        "title": "wavepeek JSON input documents",
        "description": "Canonical schema for wavepeek JSON input documents.",
        "oneOf": [
            ref_schema("extractGenericSourcesInput"),
            ref_schema("extractAtbSourceInput"),
            ref_schema("extractAxiSourceInput"),
        ],
        "$defs": Value::Object(defs),
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
            {
                "id": INPUT_SCHEMA_ID,
                "version": INPUT_SCHEMA_VERSION_STR,
                "path": INPUT_SCHEMA_ARTIFACT_PATH,
                "url": INPUT_SCHEMA_URL,
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
    object.insert(
        "extractGenericData".to_string(),
        json!({"type": "array", "items": ref_schema("extractGenericRow")}),
    );
    Value::Object(object)
}

fn stream_defs() -> Value {
    let mut object = generated_waveform_payload_defs();
    object.extend(generated_stream_record_defs());
    atb_schema::apply_stream_context_defs(&mut object);
    axi_schema::apply_stream_context_defs(&mut object);
    object.insert(
        "streamCommand".to_string(),
        json!({"type": "string", "enum": stream_commands()}),
    );
    object.insert(
        "sequence".to_string(),
        json!({"type": "integer", "minimum": 0}),
    );
    object.insert("beginRecord".to_string(), begin_record_schema());
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
                ref_schema("extractAtbItemRecord"),
                ref_schema("extractAxiItemRecord"),
                ref_schema("extractGenericItemRecord"),
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
        ("extractAtbItemRecord", "itemRecordForExtractAtbEvent"),
        ("extractAxiItemRecord", "itemRecordForExtractAxiTransfer"),
        ("extractGenericItemRecord", "itemRecordForExtractGenericRow"),
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
        (
            "itemRecordForExtractAtbEvent",
            "extract atb",
            "extractAtbEvent",
        ),
        (
            "itemRecordForExtractAxiTransfer",
            "extract axi",
            "extractAxiTransfer",
        ),
        (
            "itemRecordForExtractGenericRow",
            "extract generic",
            "extractGenericRow",
        ),
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
    generator.subschema_for::<ExtractPayloadValue<'static>>();
    generator.subschema_for::<ExtractAtbMapping<'static>>();
    generator.subschema_for::<ExtractAtbEvent<'static>>();
    generator.subschema_for::<ExtractAtbData<'static>>();
    generator.subschema_for::<ExtractAxiMapping<'static>>();
    generator.subschema_for::<ExtractAxiTransfer<'static>>();
    generator.subschema_for::<ExtractAxiData<'static>>();
    generator.subschema_for::<ExtractGenericRow<'static>>();
    let mut defs = generator.take_definitions(true);
    atb_schema::apply_output_defs(&mut defs);
    axi_schema::apply_output_defs(&mut defs);
    defs
}

fn generated_input_payload_defs() -> Map<String, Value> {
    let mut generator = SchemaGenerator::default();
    generator.subschema_for::<ExtractGenericSourcesInput<'static>>();
    generator.subschema_for::<ExtractGenericSource<'static>>();
    generator.subschema_for::<ExtractAtbSourceInput<'static>>();
    generator.subschema_for::<ExtractAxiSourceInput<'static>>();
    let mut defs = generator.take_definitions(true);
    atb_schema::apply_input_defs(&mut defs);
    axi_schema::apply_input_defs(&mut defs);
    defs
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
    generator.subschema_for::<BeginRecord<'static>>();
    generator.subschema_for::<ExtractAtbContext<'static>>();
    generator.subschema_for::<ExtractAxiContext<'static>>();
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

fn begin_record_schema() -> Value {
    let context_free_commands = stream_commands()
        .into_iter()
        .filter(|command| !matches!(*command, "extract atb" | "extract axi"))
        .collect::<Vec<_>>();
    json!({
        "type": "object",
        "additionalProperties": true,
        "required": ["type", "seq", "command", "$schema"],
        "properties": {
            "type": {"const": "begin"},
            "seq": ref_schema("sequence"),
            "command": ref_schema("streamCommand"),
            "$schema": {"type": "string", "const": STREAM_SCHEMA_URL},
            "context": {
                "oneOf": [
                    ref_schema("extractAtbContext"),
                    ref_schema("extractAxiContext")
                ]
            }
        },
        "allOf": [{
            "oneOf": [
                {
                    "required": ["context"],
                    "properties": {
                        "command": {"const": "extract atb"},
                        "context": ref_schema("extractAtbContext")
                    }
                },
                {
                    "required": ["context"],
                    "properties": {
                        "command": {"const": "extract axi"},
                        "context": ref_schema("extractAxiContext")
                    }
                },
                {
                    "not": {"required": ["context"]},
                    "properties": {
                        "command": {"enum": context_free_commands}
                    }
                }
            ]
        }]
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
        "extract atb",
        "extract axi",
        "extract generic",
        "docs topics",
        "docs search",
    ]
}

fn stream_commands() -> Vec<&'static str> {
    vec![
        "info",
        "scope",
        "signal",
        "value",
        "change",
        "property",
        "extract atb",
        "extract axi",
        "extract generic",
    ]
}

#[cfg(test)]
mod tests {
    use super::{
        INPUT_SCHEMA_ARTIFACT_PATH, OUTPUT_SCHEMA_ARTIFACT_PATH, STREAM_SCHEMA_ARTIFACT_PATH,
        catalog_value, input_schema_value, output_schema_value, stream_schema_value,
    };

    #[test]
    fn catalog_uses_canonical_paths_independent_of_output_directory() {
        let catalog = catalog_value();
        let families = catalog["families"]
            .as_array()
            .expect("families should be array");
        assert_eq!(families[0]["path"], OUTPUT_SCHEMA_ARTIFACT_PATH);
        assert_eq!(families[1]["path"], STREAM_SCHEMA_ARTIFACT_PATH);
        assert_eq!(families[2]["path"], INPUT_SCHEMA_ARTIFACT_PATH);
    }

    #[test]
    fn generated_schemas_use_exact_schema_urls() {
        let output = output_schema_value();
        assert_eq!(
            output["properties"]["$schema"]["const"],
            "https://kleverhq.github.io/wavepeek/schema-output-v2.2.json"
        );
        let stream = stream_schema_value();
        assert_eq!(
            stream["$defs"]["beginRecord"]["properties"]["$schema"]["const"],
            "https://kleverhq.github.io/wavepeek/schema-stream-v2.2.json"
        );
        let input = input_schema_value();
        assert_eq!(
            input["$defs"]["extractGenericSourcesInput"]["properties"]["$schema"]["const"],
            "https://kleverhq.github.io/wavepeek/schema-input-v2.2.json"
        );
        assert_eq!(
            input["$defs"]["extractAtbSourceInput"]["properties"]["$schema"]["const"],
            "https://kleverhq.github.io/wavepeek/schema-input-v2.2.json"
        );
        assert_eq!(
            input["$defs"]["extractAxiSourceInput"]["properties"]["$schema"]["const"],
            "https://kleverhq.github.io/wavepeek/schema-input-v2.2.json"
        );
    }
}
