use std::collections::BTreeMap;

use schemars::{JsonSchema, Schema, SchemaGenerator, json_schema};
use serde::Serialize;

#[derive(Debug, JsonSchema, Serialize)]
#[schemars(rename = "extractGenericSourcesInput")]
#[schemars(extend("additionalProperties" = true))]
pub struct ExtractGenericSourcesInput<'a> {
    #[serde(rename = "$schema")]
    #[schemars(schema_with = "input_schema_url_schema")]
    #[schemars(description = "Input schema URL for this source document.")]
    schema: &'a str,
    #[schemars(schema_with = "extract_generic_kind_schema")]
    #[schemars(description = "Input document kind discriminator.")]
    kind: &'a str,
    #[schemars(schema_with = "sources_schema")]
    #[schemars(description = "Ordered extraction sources.")]
    sources: Vec<ExtractGenericSource<'a>>,
}

#[derive(Debug, JsonSchema, Serialize)]
#[schemars(rename = "extractAtbSourceInput")]
#[schemars(extend("additionalProperties" = true))]
pub struct ExtractAtbSourceInput<'a> {
    #[serde(rename = "$schema")]
    #[schemars(schema_with = "input_schema_url_schema")]
    #[schemars(description = "Input schema URL for this source document.")]
    schema: &'a str,
    #[schemars(schema_with = "extract_atb_kind_schema")]
    #[schemars(description = "Input document kind discriminator.")]
    kind: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schemars(default)]
    #[schemars(schema_with = "atb_profile_schema")]
    #[schemars(description = "ATB profile. Defaults to atb-c.")]
    profile: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schemars(default)]
    #[schemars(description = "ATB interface name metadata. Defaults to atb.")]
    name: Option<&'a str>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[schemars(default)]
    #[schemars(schema_with = "includes_schema")]
    #[schemars(description = "Regexes selecting waveform signal candidates for ATB auto-mapping.")]
    includes: Vec<&'a str>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    #[schemars(default)]
    #[schemars(schema_with = "maps_schema")]
    #[schemars(
        description = "Explicit mappings from lowercase ATB standard signal names to waveform signal names."
    )]
    maps: BTreeMap<&'a str, &'a str>,
}

#[derive(Debug, JsonSchema, Serialize)]
#[schemars(rename = "extractAxiSourceInput")]
#[schemars(extend("additionalProperties" = true))]
pub struct ExtractAxiSourceInput<'a> {
    #[serde(rename = "$schema")]
    #[schemars(schema_with = "input_schema_url_schema")]
    #[schemars(description = "Input schema URL for this source document.")]
    schema: &'a str,
    #[schemars(schema_with = "extract_axi_kind_schema")]
    #[schemars(description = "Input document kind discriminator.")]
    kind: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schemars(default)]
    #[schemars(schema_with = "axi_profile_schema")]
    #[schemars(description = "AXI profile. Defaults to axi4.")]
    profile: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schemars(default)]
    #[schemars(description = "AXI port name metadata. Defaults to axi.")]
    name: Option<&'a str>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[schemars(default)]
    #[schemars(schema_with = "includes_schema")]
    #[schemars(description = "Regexes selecting waveform signal candidates for AXI auto-mapping.")]
    includes: Vec<&'a str>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    #[schemars(default)]
    #[schemars(schema_with = "maps_schema")]
    #[schemars(
        description = "Explicit mappings from lowercase AXI standard signal names to waveform signal names."
    )]
    maps: BTreeMap<&'a str, &'a str>,
}

#[derive(Debug, JsonSchema, Serialize)]
#[schemars(rename = "extractGenericSource")]
#[schemars(extend("additionalProperties" = true))]
pub struct ExtractGenericSource<'a> {
    #[schemars(description = "Source name emitted in extract rows.")]
    name: &'a str,
    #[schemars(description = "Edge-only event expression that selects row candidate times.")]
    on: &'a str,
    #[schemars(description = "Boolean expression evaluated at the pre-edge sample time.")]
    when: &'a str,
    #[schemars(schema_with = "payload_schema")]
    #[schemars(description = "Ordered payload signal names for this source.")]
    payload: Vec<&'a str>,
}

fn input_schema_url_schema(_: &mut SchemaGenerator) -> Schema {
    json_schema!({"type": "string", "const": crate::contract::schema::INPUT_SCHEMA_URL})
}

fn extract_generic_kind_schema(_: &mut SchemaGenerator) -> Schema {
    json_schema!({"type": "string", "const": "extract.generic.sources"})
}

fn extract_atb_kind_schema(_: &mut SchemaGenerator) -> Schema {
    json_schema!({"type": "string", "const": "extract.atb.source"})
}

fn extract_axi_kind_schema(_: &mut SchemaGenerator) -> Schema {
    json_schema!({"type": "string", "const": "extract.axi.source"})
}

fn atb_profile_schema(_: &mut SchemaGenerator) -> Schema {
    json_schema!({
        "type": "string",
        "enum": ["atb-a", "atb-b", "atb-c"]
    })
}

fn axi_profile_schema(_: &mut SchemaGenerator) -> Schema {
    json_schema!({
        "type": "string",
        "enum": [
            "axi3", "axi4", "axi4-lite", "axi5", "axi5-lite", "ace", "ace-lite", "ace5",
            "ace5-lite", "ace5-lite-dvm", "ace5-lite-acp"
        ]
    })
}

fn sources_schema(_: &mut SchemaGenerator) -> Schema {
    json_schema!({
        "type": "array",
        "minItems": 1,
        "items": {"$ref": "#/$defs/extractGenericSource"}
    })
}

fn payload_schema(_: &mut SchemaGenerator) -> Schema {
    json_schema!({
        "type": "array",
        "minItems": 1,
        "items": {"type": "string"}
    })
}

fn includes_schema(_: &mut SchemaGenerator) -> Schema {
    json_schema!({
        "type": "array",
        "items": {"type": "string"}
    })
}

fn maps_schema(_: &mut SchemaGenerator) -> Schema {
    json_schema!({
        "type": "object",
        "additionalProperties": {"type": "string"}
    })
}
