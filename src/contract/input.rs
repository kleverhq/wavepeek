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
