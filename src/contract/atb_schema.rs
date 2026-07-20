use serde_json::{Map, Value, json};

use crate::contract::schema::INPUT_SCHEMA_URL;
use crate::engine::atb::{self, AtbProfile};

const ISSUE: &str = "C";
const TRANSFER_PAYLOAD_SIGNALS: &[&str] = &["atbytes", "atdata", "atid"];

pub(super) fn apply_output_defs(defs: &mut Map<String, Value>) {
    defs.insert("atbProfile".to_string(), atb_profile_schema());
    defs.insert("extractAtbData".to_string(), atb_data_schema());
    defs.insert("extractAtbEvent".to_string(), atb_event_schema());
    for profile in atb::profile_specs() {
        defs.insert(
            atb_profile_data_def_name(profile),
            atb_data_profile_schema(profile),
        );
        defs.insert(
            atb_profile_event_def_name(profile),
            atb_profile_event_schema(profile),
        );
        for event in profile_event_kinds(profile) {
            defs.insert(
                atb_event_def_name(profile, event),
                atb_event_kind_schema(profile, event),
            );
        }
    }
}

pub(super) fn apply_stream_context_defs(defs: &mut Map<String, Value>) {
    defs.insert("extractAtbContext".to_string(), atb_context_schema());
    for profile in atb::profile_specs() {
        defs.insert(
            atb_profile_context_def_name(profile),
            atb_context_profile_schema(profile),
        );
    }
}

pub(super) fn apply_input_defs(defs: &mut Map<String, Value>) {
    defs.insert("atbProfile".to_string(), atb_profile_schema());
    defs.insert(
        "extractAtbSourceInput".to_string(),
        atb_source_input_schema(),
    );
}

fn atb_profile_schema() -> Value {
    json!({
        "type": "string",
        "description": "ATB profile name: atb-a, atb-b, or atb-c.",
        "enum": atb_profile_names(),
    })
}

fn atb_profile_names() -> Vec<&'static str> {
    atb::profile_specs()
        .iter()
        .map(|profile| profile.name())
        .collect()
}

fn atb_source_input_schema() -> Value {
    let default_profile = AtbProfile::AtbC;
    let non_default_branches = atb::profile_specs()
        .into_iter()
        .filter(|profile| *profile != default_profile)
        .map(atb_source_profile_branch);
    let branches = std::iter::once(atb_source_default_profile_branch(default_profile))
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
                "const": "extract.atb.source",
                "description": "Input document kind discriminator."
            },
            "profile": ref_schema("atbProfile"),
            "name": {
                "type": "string",
                "description": "ATB interface name metadata. Defaults to atb."
            },
            "includes": {
                "type": "array",
                "description": "Regexes selecting waveform signal candidates for ATB auto-mapping.",
                "items": {"type": "string"}
            },
            "maps": {
                "type": "object",
                "description": "Explicit mappings from lowercase ATB standard signal names to waveform signal names.",
                "additionalProperties": {"type": "string"}
            }
        },
        "allOf": [{"oneOf": branches}]
    })
}

fn atb_source_default_profile_branch(profile: AtbProfile) -> Value {
    json!({
        "anyOf": [
            {"not": {"required": ["profile"]}},
            {"required": ["profile"], "properties": {"profile": {"const": profile.name()}}}
        ],
        "properties": {
            "maps": atb_input_maps_schema(profile)
        }
    })
}

fn atb_source_profile_branch(profile: AtbProfile) -> Value {
    json!({
        "required": ["profile"],
        "properties": {
            "profile": {"const": profile.name()},
            "maps": atb_input_maps_schema(profile)
        }
    })
}

fn atb_input_maps_schema(profile: AtbProfile) -> Value {
    let properties = profile
        .signals()
        .iter()
        .map(|standard| ((*standard).to_string(), json!({"type": "string"})))
        .collect::<Map<_, _>>();
    json!({
        "type": "object",
        "additionalProperties": false,
        "properties": properties,
    })
}

fn atb_data_schema() -> Value {
    json!({
        "oneOf": atb::profile_specs()
            .into_iter()
            .map(|profile| ref_schema(&atb_profile_data_def_name(profile)))
            .collect::<Vec<_>>()
    })
}

fn atb_context_schema() -> Value {
    json!({
        "oneOf": atb::profile_specs()
            .into_iter()
            .map(|profile| ref_schema(&atb_profile_context_def_name(profile)))
            .collect::<Vec<_>>()
    })
}

fn atb_data_profile_schema(profile: AtbProfile) -> Value {
    json!({
        "type": "object",
        "additionalProperties": true,
        "required": ["name", "profile", "issue", "mappings", "events"],
        "properties": {
            "name": {
                "type": "string",
                "description": "ATB interface name supplied by CLI or source JSON."
            },
            "profile": {
                "const": profile.name(),
                "description": "ATB profile name used for standard signal mapping."
            },
            "issue": {
                "const": ISSUE,
                "description": "Arm IHI 0032 issue used for this profile definition."
            },
            "mappings": atb_mappings_schema(profile),
            "events": {
                "type": "array",
                "description": "Extracted stateless ATB events in deterministic source order.",
                "items": ref_schema(&atb_profile_event_def_name(profile))
            }
        },
        "allOf": event_mapping_constraints(profile)
    })
}

fn atb_context_profile_schema(profile: AtbProfile) -> Value {
    json!({
        "type": "object",
        "additionalProperties": true,
        "required": ["name", "profile", "issue", "mappings"],
        "properties": {
            "name": {
                "type": "string",
                "description": "ATB interface name supplied by CLI or source JSON."
            },
            "profile": {
                "const": profile.name(),
                "description": "ATB profile name used for standard signal mapping."
            },
            "issue": {
                "const": ISSUE,
                "description": "Arm IHI 0032 issue used for this profile definition."
            },
            "mappings": atb_mappings_schema(profile)
        }
    })
}

fn atb_mappings_schema(profile: AtbProfile) -> Value {
    let properties = profile
        .signals()
        .iter()
        .map(|standard| ((*standard).to_string(), ref_schema("extractAtbMapping")))
        .collect::<Map<_, _>>();
    json!({
        "type": "object",
        "description": "Resolved waveform mappings keyed by lowercase ATB standard signal name.",
        "additionalProperties": false,
        "required": ["atclk"],
        "properties": properties,
        "dependentRequired": {
            "atvalid": ["atready"],
            "atready": ["atvalid"],
            "afvalid": ["afready"],
            "afready": ["afvalid"],
            "atbytes": ["atvalid", "atready"],
            "atdata": ["atvalid", "atready"],
            "atid": ["atvalid", "atready"]
        },
        "anyOf": [
            {"required": ["atvalid", "atready"]},
            {"required": ["afvalid", "afready"]}
        ]
    })
}

fn event_mapping_constraints(profile: AtbProfile) -> Vec<Value> {
    profile_event_kinds(profile)
        .iter()
        .map(|event| {
            json!({
                "if": {
                    "properties": {
                        "events": {
                            "contains": {
                                "type": "object",
                                "required": ["event"],
                                "properties": {"event": {"const": event}}
                            }
                        }
                    }
                },
                "then": {
                    "properties": {
                        "mappings": {"required": required_event_mappings(event)}
                    }
                }
            })
        })
        .collect()
}

fn required_event_mappings(event: &str) -> &'static [&'static str] {
    match event {
        "transfer" => &["atvalid", "atready"],
        "flush" => &["afvalid", "afready"],
        "sync-request" => &["syncreq"],
        _ => unreachable!("ATB event kind table is closed"),
    }
}

fn atb_event_schema() -> Value {
    json!({
        "oneOf": atb::profile_specs()
            .into_iter()
            .map(|profile| ref_schema(&atb_profile_event_def_name(profile)))
            .collect::<Vec<_>>()
    })
}

fn atb_profile_event_schema(profile: AtbProfile) -> Value {
    json!({
        "oneOf": profile_event_kinds(profile)
            .iter()
            .map(|event| ref_schema(&atb_event_def_name(profile, event)))
            .collect::<Vec<_>>()
    })
}

fn atb_event_kind_schema(profile: AtbProfile, event: &str) -> Value {
    json!({
        "type": "object",
        "additionalProperties": true,
        "required": ["time", "sample_time", "profile", "event", "payload"],
        "properties": {
            "time": {
                "$ref": "#/$defs/normalizedTime",
                "description": "Selected ATB event timestamp."
            },
            "sample_time": {
                "$ref": "#/$defs/normalizedTime",
                "description": "Pre-edge timestamp used to evaluate the ATB predicate and sample payload values."
            },
            "profile": {
                "const": profile.name(),
                "description": "ATB profile name for this event row."
            },
            "event": {
                "const": event,
                "description": "Stateless ATB event kind."
            },
            "payload": atb_event_payload_schema(event)
        }
    })
}

fn atb_event_payload_schema(event: &str) -> Value {
    let properties = if event == "transfer" {
        TRANSFER_PAYLOAD_SIGNALS
            .iter()
            .map(|standard| ((*standard).to_string(), ref_schema("sampledValue")))
            .collect::<Map<_, _>>()
    } else {
        Map::new()
    };
    json!({
        "type": "object",
        "description": if event == "transfer" {
            "Optional raw ATBYTES, ATDATA, and ATID observations keyed by lowercase standard signal name."
        } else {
            "Empty payload for a control event whose kind records the complete sampled condition."
        },
        "additionalProperties": false,
        "properties": properties,
    })
}

fn profile_event_kinds(profile: AtbProfile) -> &'static [&'static str] {
    match profile {
        AtbProfile::AtbA => &["transfer", "flush"],
        AtbProfile::AtbB | AtbProfile::AtbC => &["transfer", "flush", "sync-request"],
    }
}

fn atb_profile_data_def_name(profile: AtbProfile) -> String {
    format!("extract{}Data", schema_suffix(profile.name()))
}

fn atb_profile_context_def_name(profile: AtbProfile) -> String {
    format!("extract{}Context", schema_suffix(profile.name()))
}

fn atb_profile_event_def_name(profile: AtbProfile) -> String {
    format!("extract{}Event", schema_suffix(profile.name()))
}

fn atb_event_def_name(profile: AtbProfile, event: &str) -> String {
    format!(
        "extract{}{}Event",
        schema_suffix(profile.name()),
        schema_suffix(event)
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
    use super::*;

    #[test]
    fn profile_defs_match_runtime_signal_sets() {
        let mut output_defs = Map::new();
        apply_output_defs(&mut output_defs);

        for profile in atb::profile_specs() {
            let data_def = atb_profile_data_def_name(profile);
            let mapping_properties = output_defs[&data_def]["properties"]["mappings"]["properties"]
                .as_object()
                .expect("ATB mappings must expose exact properties");
            assert_eq!(mapping_properties.len(), profile.signals().len());
            for standard in profile.signals() {
                assert!(mapping_properties.contains_key(*standard));
            }
            assert!(!mapping_properties.contains_key("atclken"));
            assert!(!mapping_properties.contains_key("atwakeup"));
        }
    }

    #[test]
    fn event_defs_are_profile_and_kind_exact() {
        let mut output_defs = Map::new();
        apply_output_defs(&mut output_defs);

        let atb_a = &output_defs[&atb_profile_event_def_name(AtbProfile::AtbA)]["oneOf"];
        assert_eq!(atb_a.as_array().expect("ATB-A events oneOf").len(), 2);
        let atb_c = &output_defs[&atb_profile_event_def_name(AtbProfile::AtbC)]["oneOf"];
        assert_eq!(atb_c.as_array().expect("ATB-C events oneOf").len(), 3);

        let transfer = &output_defs[&atb_event_def_name(AtbProfile::AtbC, "transfer")]["properties"]
            ["payload"];
        assert_eq!(
            transfer["properties"]
                .as_object()
                .expect("transfer payload properties")
                .len(),
            3
        );
        let flush =
            &output_defs[&atb_event_def_name(AtbProfile::AtbC, "flush")]["properties"]["payload"];
        assert!(
            flush["properties"]
                .as_object()
                .expect("flush payload properties")
                .is_empty()
        );
    }
}
