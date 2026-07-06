use schemars::SchemaGenerator;
use serde_json::{Map, Value, json};

use crate::engine::axi::{self, AxiChannelSpec, AxiProfileSpec};

use super::common::{
    CanonicalPath, ContractDiagnostic, NormalizedTime, SampledValue, ScopeKind, SignalKind,
};
use super::input::{ExtractAxiSourceInput, ExtractGenericSource, ExtractGenericSourcesInput};
use super::output::{
    ChangeSignalValue, ChangeSnapshot, DocsSearchData, DocsSearchMatch, DocsTopicsData,
    ExtractAxiData, ExtractAxiMapping, ExtractAxiTransfer, ExtractGenericRow, ExtractPayloadValue,
    InfoData, PropertyRow, SampledSignalValue, ScopeEntry, SignalEntry, TopicSummary,
    ValueSnapshot,
};
use super::stream::{BeginRecord, DiagnosticRecord, EndRecord, ExtractAxiContext};

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

const JSON_SCHEMA_DRAFT: &str = "https://json-schema.org/draft/2020-12/schema";

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
    apply_axi_stream_context_defs(&mut object);
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
    generator.subschema_for::<ExtractAxiMapping<'static>>();
    generator.subschema_for::<ExtractAxiTransfer<'static>>();
    generator.subschema_for::<ExtractAxiData<'static>>();
    generator.subschema_for::<ExtractGenericRow<'static>>();
    let mut defs = generator.take_definitions(true);
    apply_axi_output_defs(&mut defs);
    defs
}

fn apply_axi_output_defs(defs: &mut Map<String, Value>) {
    defs.insert("axiProfile".to_string(), axi_profile_schema());
    defs.insert("extractAxiData".to_string(), axi_data_schema());
    defs.insert("extractAxiTransfer".to_string(), axi_transfer_schema());
    for profile in axi::profile_specs() {
        defs.insert(
            axi_profile_transfer_def_name(profile),
            axi_profile_transfer_schema(profile),
        );
        for channel in profile.channels {
            defs.insert(
                axi_channel_transfer_def_name(profile, channel),
                axi_channel_transfer_schema(profile, channel),
            );
        }
    }
}

fn apply_axi_stream_context_defs(defs: &mut Map<String, Value>) {
    defs.insert("extractAxiContext".to_string(), axi_context_schema());
}

fn axi_profile_schema() -> Value {
    json!({
        "type": "string",
        "description": "AXI profile name: axi3, axi4, or axi4-lite.",
        "enum": axi_profile_names(),
    })
}

fn axi_profile_names() -> Vec<&'static str> {
    axi::profile_specs()
        .iter()
        .map(|profile| profile.name)
        .collect()
}

fn axi_data_schema() -> Value {
    json!({
        "oneOf": axi::profile_specs()
            .iter()
            .map(axi_data_profile_schema)
            .collect::<Vec<_>>()
    })
}

fn axi_context_schema() -> Value {
    json!({
        "oneOf": axi::profile_specs()
            .iter()
            .map(axi_context_profile_schema)
            .collect::<Vec<_>>()
    })
}

fn axi_data_profile_schema(profile: &AxiProfileSpec) -> Value {
    json!({
        "type": "object",
        "additionalProperties": true,
        "required": ["name", "profile", "issue", "mappings", "transfers"],
        "properties": {
            "name": {
                "type": "string",
                "description": "AXI port name supplied by CLI or source JSON."
            },
            "profile": {
                "const": profile.name,
                "description": "AXI profile name used for standard signal mapping."
            },
            "issue": {
                "const": profile.issue,
                "description": "Arm IHI 0022 issue used for this profile definition."
            },
            "mappings": axi_mappings_schema(profile),
            "transfers": {
                "type": "array",
                "description": "Extracted AXI ready/valid transfers in event order.",
                "items": ref_schema(&axi_profile_transfer_def_name(profile))
            }
        }
    })
}

fn axi_context_profile_schema(profile: &AxiProfileSpec) -> Value {
    json!({
        "type": "object",
        "additionalProperties": true,
        "required": ["name", "profile", "issue", "mappings"],
        "properties": {
            "name": {
                "type": "string",
                "description": "AXI port name supplied by CLI or source JSON."
            },
            "profile": {
                "const": profile.name,
                "description": "AXI profile name used for standard signal mapping."
            },
            "issue": {
                "const": profile.issue,
                "description": "Arm IHI 0022 issue used for this profile definition."
            },
            "mappings": axi_mappings_schema(profile)
        }
    })
}

fn axi_mappings_schema(profile: &AxiProfileSpec) -> Value {
    let mut properties = Map::new();
    for standard in axi::standard_signals(profile) {
        properties.insert(standard.to_string(), ref_schema("extractAxiMapping"));
    }
    json!({
        "type": "object",
        "description": "Resolved waveform mappings keyed by lowercase AXI standard signal name.",
        "additionalProperties": false,
        "properties": properties,
    })
}

fn axi_transfer_schema() -> Value {
    json!({
        "oneOf": axi::profile_specs()
            .iter()
            .map(|profile| ref_schema(&axi_profile_transfer_def_name(profile)))
            .collect::<Vec<_>>()
    })
}

fn axi_profile_transfer_schema(profile: &AxiProfileSpec) -> Value {
    json!({
        "oneOf": profile
            .channels
            .iter()
            .map(|channel| ref_schema(&axi_channel_transfer_def_name(profile, channel)))
            .collect::<Vec<_>>()
    })
}

fn axi_channel_transfer_schema(profile: &AxiProfileSpec, channel: &AxiChannelSpec) -> Value {
    json!({
        "type": "object",
        "additionalProperties": true,
        "required": ["time", "sample_time", "profile", "channel", "payload"],
        "properties": {
            "time": {
                "$ref": "#/$defs/normalizedTime",
                "description": "Selected AXI transfer event timestamp."
            },
            "sample_time": {
                "$ref": "#/$defs/normalizedTime",
                "description": "Pre-edge timestamp used to evaluate ready/valid and sample payload values."
            },
            "profile": {
                "const": profile.name,
                "description": "AXI profile name for this transfer row."
            },
            "channel": {
                "const": channel.name,
                "description": "AXI ready/valid channel name."
            },
            "payload": axi_payload_schema(channel)
        }
    })
}

fn axi_payload_schema(channel: &AxiChannelSpec) -> Value {
    let mut properties = Map::new();
    for standard in axi::channel_payload_signals(channel) {
        properties.insert(standard.to_string(), ref_schema("sampledValue"));
    }
    json!({
        "type": "object",
        "description": "Payload values keyed by lowercase AXI standard signal name. Keys are optional because extraction emits only mapped payload signals.",
        "additionalProperties": false,
        "properties": properties,
    })
}

fn axi_profile_transfer_def_name(profile: &AxiProfileSpec) -> String {
    format!("extract{}Transfer", schema_suffix(profile.name))
}

fn axi_channel_transfer_def_name(profile: &AxiProfileSpec, channel: &AxiChannelSpec) -> String {
    format!(
        "extract{}{}Transfer",
        schema_suffix(profile.name),
        schema_suffix(channel.name)
    )
}

fn schema_suffix(name: &str) -> String {
    let mut suffix = String::new();
    let mut capitalize = true;
    for ch in name.chars() {
        if ch.is_ascii_alphanumeric() {
            if capitalize {
                suffix.push(ch.to_ascii_uppercase());
                capitalize = false;
            } else {
                suffix.push(ch);
            }
        } else {
            capitalize = true;
        }
    }
    suffix
}

fn generated_input_payload_defs() -> Map<String, Value> {
    let mut generator = SchemaGenerator::default();
    generator.subschema_for::<ExtractGenericSourcesInput<'static>>();
    generator.subschema_for::<ExtractGenericSource<'static>>();
    generator.subschema_for::<ExtractAxiSourceInput<'static>>();
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
    generator.subschema_for::<BeginRecord<'static>>();
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
            input["$defs"]["extractAxiSourceInput"]["properties"]["$schema"]["const"],
            "https://kleverhq.github.io/wavepeek/schema-input-v2.2.json"
        );
    }
}
