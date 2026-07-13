use serde_json::{Map, Value, json};

use crate::contract::schema::INPUT_SCHEMA_URL;
use crate::engine::axi::{self, AxiChannelSpec, AxiProfileSpec};

pub(super) fn apply_output_defs(defs: &mut Map<String, Value>) {
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

pub(super) fn apply_stream_context_defs(defs: &mut Map<String, Value>) {
    defs.insert("extractAxiContext".to_string(), axi_context_schema());
}

pub(super) fn apply_input_defs(defs: &mut Map<String, Value>) {
    defs.insert("axiProfile".to_string(), axi_profile_schema());
    defs.insert(
        "extractAxiSourceInput".to_string(),
        axi_source_input_schema(),
    );
}

fn axi_profile_schema() -> Value {
    json!({
        "type": "string",
        "description": "AXI profile name: axi3, axi4, axi4-lite, axi5, axi5-lite, ace, ace-lite, ace5, ace5-lite, ace5-lite-dvm, or ace5-lite-acp.",
        "enum": axi_profile_names(),
    })
}

fn axi_profile_names() -> Vec<&'static str> {
    axi::profile_specs()
        .iter()
        .map(|profile| profile.name)
        .collect()
}

fn axi_source_input_schema() -> Value {
    let default_profile = axi::profile_specs()
        .iter()
        .find(|profile| profile.name == "axi4")
        .expect("AXI4 profile must exist");
    let non_default_branches = axi::profile_specs()
        .iter()
        .filter(|profile| profile.name != default_profile.name)
        .map(axi_source_profile_branch);
    let branches = std::iter::once(axi_source_default_profile_branch(default_profile))
        .chain(non_default_branches)
        .collect::<Vec<_>>();

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
                "const": "extract.axi.source",
                "description": "Input document kind discriminator."
            },
            "profile": ref_schema("axiProfile"),
            "name": {
                "type": "string",
                "description": "AXI port name metadata. Defaults to axi."
            },
            "includes": {
                "type": "array",
                "description": "Regexes selecting waveform signal candidates for AXI auto-mapping.",
                "items": {"type": "string"}
            },
            "maps": {
                "type": "object",
                "description": "Explicit mappings from lowercase AXI standard signal names to waveform signal names.",
                "additionalProperties": {"type": "string"}
            }
        },
        "allOf": [{"oneOf": branches}]
    })
}

fn axi_source_default_profile_branch(profile: &AxiProfileSpec) -> Value {
    json!({
        "anyOf": [
            {"not": {"required": ["profile"]}},
            {"required": ["profile"], "properties": {"profile": {"const": profile.name}}}
        ],
        "properties": {
            "maps": axi_input_maps_schema(profile)
        }
    })
}

fn axi_source_profile_branch(profile: &AxiProfileSpec) -> Value {
    json!({
        "required": ["profile"],
        "properties": {
            "profile": {"const": profile.name},
            "maps": axi_input_maps_schema(profile)
        }
    })
}

fn axi_input_maps_schema(profile: &AxiProfileSpec) -> Value {
    let mut properties = Map::new();
    for standard in axi::standard_signals(profile) {
        properties.insert(standard.to_string(), json!({"type": "string"}));
    }
    json!({
        "type": "object",
        "additionalProperties": false,
        "properties": properties,
    })
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

fn ref_schema(def_name: &str) -> Value {
    json!({"$ref": format!("#/$defs/{def_name}")})
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::*;

    #[test]
    fn transfer_defs_follow_runtime_profiles_and_channels() {
        let mut defs = Map::new();
        apply_output_defs(&mut defs);

        assert_eq!(
            defs["extractAxiTransfer"]["oneOf"]
                .as_array()
                .expect("root AXI transfer schema should use oneOf")
                .len(),
            axi::profile_specs().len()
        );

        for profile in axi::profile_specs() {
            let profile_def = axi_profile_transfer_def_name(profile);
            assert_eq!(
                defs[&profile_def]["oneOf"]
                    .as_array()
                    .expect("profile transfer schema should use oneOf")
                    .len(),
                profile.channels.len()
            );

            for channel in profile.channels {
                let channel_def = axi_channel_transfer_def_name(profile, channel);
                let payload_properties = defs[&channel_def]["properties"]["payload"]["properties"]
                    .as_object()
                    .expect("channel payload schema should expose properties");
                let actual = payload_properties.keys().cloned().collect::<BTreeSet<_>>();
                let expected = axi::channel_payload_signals(channel)
                    .map(str::to_string)
                    .collect::<BTreeSet<_>>();
                assert_eq!(actual, expected, "payload keys for {channel_def}");
            }
        }
    }
}
