use serde_json::{Map, Value, json};

use crate::contract::schema::INPUT_SCHEMA_URL;
use crate::engine::axistream::{self, AxiStreamProfileSpec};

const TREADY_MODES: &[&str] = &["mapped", "implicit-high"];

pub(super) fn apply_output_defs(defs: &mut Map<String, Value>) {
    defs.insert("axiStreamProfile".to_string(), profile_schema());
    defs.insert("treadyMode".to_string(), tready_mode_schema());
    defs.insert("extractAxiStreamData".to_string(), data_schema());
    defs.insert("extractAxiStreamTransfer".to_string(), transfer_schema());
    for profile in axistream::profile_specs() {
        defs.insert(
            profile_transfer_def_name(profile),
            profile_transfer_schema(profile),
        );
    }
}

pub(super) fn apply_stream_context_defs(defs: &mut Map<String, Value>) {
    defs.insert("extractAxiStreamContext".to_string(), context_schema());
}

pub(super) fn apply_input_defs(defs: &mut Map<String, Value>) {
    defs.insert("axiStreamProfile".to_string(), profile_schema());
    defs.insert("treadyMode".to_string(), tready_mode_schema());
    defs.insert(
        "extractAxiStreamSourceInput".to_string(),
        source_input_schema(),
    );
}

fn profile_schema() -> Value {
    json!({
        "type": "string",
        "description": "AXI-Stream profile name: axi4-stream or axi5-stream.",
        "enum": profile_names(),
    })
}

fn profile_names() -> Vec<&'static str> {
    axistream::profile_specs()
        .iter()
        .map(|profile| profile.name)
        .collect()
}

fn tready_mode_schema() -> Value {
    json!({
        "type": "string",
        "description": "Whether TREADY is mapped or physically omitted and implicitly HIGH.",
        "enum": TREADY_MODES,
    })
}

fn source_input_schema() -> Value {
    json!({
        "type": "object",
        "additionalProperties": true,
        "required": ["$schema", "kind"],
        "properties": {
            "$schema": {
                "type": "string",
                "const": INPUT_SCHEMA_URL,
                "description": "Input schema URL for this source document."
            },
            "kind": {
                "type": "string",
                "const": "extract.axistream.source",
                "description": "Input document kind discriminator."
            },
            "profile": ref_schema("axiStreamProfile"),
            "tready_mode": ref_schema("treadyMode"),
            "name": {
                "type": "string",
                "description": "AXI-Stream port name metadata. Defaults to axistream."
            },
            "includes": {
                "type": "array",
                "description": "Regexes selecting waveform signal candidates for AXI-Stream auto-mapping.",
                "items": {"type": "string"}
            },
            "maps": {
                "type": "object",
                "description": "Explicit mappings from lowercase AXI-Stream standard signal names to waveform signal names.",
                "additionalProperties": {"type": "string"}
            }
        },
        "allOf": [
            {"oneOf": profile_source_branches()},
            {"oneOf": tready_source_branches()}
        ]
    })
}

fn profile_source_branches() -> Vec<Value> {
    let default = axistream::profile_specs()
        .iter()
        .find(|profile| profile.name == "axi4-stream")
        .expect("AXI4-Stream profile must exist");
    std::iter::once(json!({
        "anyOf": [
            {"not": {"required": ["profile"]}},
            {"required": ["profile"], "properties": {"profile": {"const": default.name}}}
        ]
    }))
    .chain(
        axistream::profile_specs()
            .iter()
            .filter(|profile| profile.name != default.name)
            .map(|profile| {
                json!({
                    "required": ["profile"],
                    "properties": {"profile": {"const": profile.name}}
                })
            }),
    )
    .collect()
}

fn tready_source_branches() -> Vec<Value> {
    vec![
        json!({
            "anyOf": [
                {"not": {"required": ["tready_mode"]}},
                {"required": ["tready_mode"], "properties": {"tready_mode": {"const": "mapped"}}}
            ],
            "properties": {"maps": input_maps_schema(true)}
        }),
        json!({
            "required": ["tready_mode"],
            "properties": {
                "tready_mode": {"const": "implicit-high"},
                "maps": input_maps_schema(false)
            }
        }),
    ]
}

fn input_maps_schema(include_tready: bool) -> Value {
    let mut properties = Map::new();
    for standard in axistream::standard_signals() {
        if include_tready || *standard != "tready" {
            properties.insert(standard.to_string(), json!({"type": "string"}));
        }
    }
    json!({
        "type": "object",
        "additionalProperties": false,
        "properties": properties,
    })
}

fn data_schema() -> Value {
    json!({
        "oneOf": profile_mode_branches(data_profile_mode_schema)
    })
}

fn context_schema() -> Value {
    json!({
        "oneOf": profile_mode_branches(context_profile_mode_schema)
    })
}

fn profile_mode_branches(make: fn(&AxiStreamProfileSpec, &'static str) -> Value) -> Vec<Value> {
    axistream::profile_specs()
        .iter()
        .flat_map(|profile| TREADY_MODES.iter().map(move |mode| make(profile, mode)))
        .collect()
}

fn data_profile_mode_schema(profile: &AxiStreamProfileSpec, mode: &'static str) -> Value {
    let mut schema = context_profile_mode_schema(profile, mode);
    let object = schema
        .as_object_mut()
        .expect("AXI-Stream data schema should be object");
    object["required"] = json!([
        "name",
        "profile",
        "issue",
        "tready_mode",
        "mappings",
        "transfers"
    ]);
    object
        .get_mut("properties")
        .and_then(Value::as_object_mut)
        .expect("AXI-Stream data properties should be object")
        .insert(
            "transfers".to_string(),
            json!({
                "type": "array",
                "description": "Extracted AXI-Stream transfers in event order.",
                "items": ref_schema(&profile_transfer_def_name(profile))
            }),
        );
    schema
}

fn context_profile_mode_schema(profile: &AxiStreamProfileSpec, mode: &'static str) -> Value {
    json!({
        "type": "object",
        "additionalProperties": true,
        "required": ["name", "profile", "issue", "tready_mode", "mappings"],
        "properties": {
            "name": {
                "type": "string",
                "description": "AXI-Stream port name supplied by CLI or source JSON."
            },
            "profile": {
                "const": profile.name,
                "description": "AXI-Stream profile name used for standard signal mapping."
            },
            "issue": {
                "const": profile.issue,
                "description": "Arm IHI 0051 issue used for this profile definition."
            },
            "tready_mode": {
                "const": mode,
                "description": "Effective TREADY mapping mode."
            },
            "mappings": mappings_schema(mode == "mapped")
        }
    })
}

fn mappings_schema(include_tready: bool) -> Value {
    let mut properties = Map::new();
    for standard in axistream::standard_signals() {
        if include_tready || *standard != "tready" {
            properties.insert(standard.to_string(), ref_schema("extractAxiStreamMapping"));
        }
    }
    json!({
        "type": "object",
        "description": "Resolved waveform mappings keyed by lowercase AXI-Stream standard signal name.",
        "additionalProperties": false,
        "properties": properties,
    })
}

fn transfer_schema() -> Value {
    json!({
        "oneOf": axistream::profile_specs()
            .iter()
            .map(|profile| ref_schema(&profile_transfer_def_name(profile)))
            .collect::<Vec<_>>()
    })
}

fn profile_transfer_schema(profile: &AxiStreamProfileSpec) -> Value {
    let mut payload_properties = Map::new();
    for standard in axistream::payload_signals() {
        payload_properties.insert(standard.to_string(), ref_schema("sampledValue"));
    }
    json!({
        "type": "object",
        "additionalProperties": true,
        "required": ["time", "sample_time", "profile", "payload"],
        "properties": {
            "time": {
                "$ref": "#/$defs/normalizedTime",
                "description": "Selected AXI-Stream transfer event timestamp."
            },
            "sample_time": {
                "$ref": "#/$defs/normalizedTime",
                "description": "Pre-edge timestamp used to evaluate the handshake and sample payload values."
            },
            "profile": {
                "const": profile.name,
                "description": "AXI-Stream profile name for this transfer row."
            },
            "payload": {
                "type": "object",
                "description": "Payload values keyed by lowercase AXI-Stream standard signal name. Keys are optional because extraction emits only mapped payload signals.",
                "additionalProperties": false,
                "properties": payload_properties
            }
        }
    })
}

fn profile_transfer_def_name(profile: &AxiStreamProfileSpec) -> String {
    format!("extract{}Transfer", schema_suffix(profile.name))
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

fn ref_schema(def_name: &str) -> Value {
    json!({"$ref": format!("#/$defs/{def_name}")})
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_definitions_follow_runtime_profiles_and_signal_sets() {
        let mut defs = Map::new();
        apply_output_defs(&mut defs);
        assert_eq!(
            defs["extractAxiStreamTransfer"]["oneOf"]
                .as_array()
                .expect("transfer schema should use oneOf")
                .len(),
            axistream::profile_specs().len()
        );
        for profile in axistream::profile_specs() {
            let payload =
                &defs[&profile_transfer_def_name(profile)]["properties"]["payload"]["properties"];
            assert_eq!(
                payload
                    .as_object()
                    .expect("payload properties should be object")
                    .keys()
                    .map(String::as_str)
                    .collect::<std::collections::BTreeSet<_>>(),
                axistream::payload_signals()
                    .iter()
                    .copied()
                    .collect::<std::collections::BTreeSet<_>>()
            );
        }
    }

    #[test]
    fn implicit_high_source_maps_forbid_tready() {
        let schema = source_input_schema();
        let branches = schema["allOf"][1]["oneOf"]
            .as_array()
            .expect("mode branches should exist");
        assert!(
            branches[1]["properties"]["maps"]["properties"]
                .get("tready")
                .is_none()
        );
    }
}
