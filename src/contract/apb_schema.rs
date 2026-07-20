use serde_json::{Map, Value, json};

use crate::contract::schema::INPUT_SCHEMA_URL;
use crate::engine::apb::{self, ApbProfileSpec};

const EVENTS: &[&str] = &["setup", "access-wait", "access-complete"];
const DIRECTIONS: &[&str] = &["read", "write", "unknown"];

pub(super) fn apply_output_defs(defs: &mut Map<String, Value>) {
    defs.insert("apbProfile".to_string(), apb_profile_schema());
    defs.insert("apbPreadyMode".to_string(), pready_mode_schema());
    defs.insert("apbEventKind".to_string(), event_kind_schema());
    defs.insert("apbDirection".to_string(), direction_schema());
    defs.insert("extractApbData".to_string(), apb_data_schema());
    defs.insert("extractApbEvent".to_string(), apb_event_schema());
    for profile in apb::profile_specs() {
        for event in EVENTS {
            for direction in DIRECTIONS {
                defs.insert(
                    apb_event_def_name(profile, event, direction),
                    apb_profile_event_schema(profile, event, direction),
                );
            }
        }
    }
}

pub(super) fn apply_stream_context_defs(defs: &mut Map<String, Value>) {
    defs.insert("extractApbContext".to_string(), apb_context_schema());
}

pub(super) fn apply_input_defs(defs: &mut Map<String, Value>) {
    defs.insert("apbProfile".to_string(), apb_profile_schema());
    defs.insert("apbPreadyMode".to_string(), pready_mode_schema());
    defs.insert(
        "extractApbSourceInput".to_string(),
        apb_source_input_schema(),
    );
}

fn apb_profile_schema() -> Value {
    json!({
        "type": "string",
        "description": "APB profile name: apb3, apb4, or apb5.",
        "enum": apb::profile_specs()
            .iter()
            .map(|profile| profile.name)
            .collect::<Vec<_>>(),
    })
}

fn pready_mode_schema() -> Value {
    json!({
        "type": "string",
        "description": "APB PREADY handling mode.",
        "enum": ["mapped", "implicit-high"],
    })
}

fn event_kind_schema() -> Value {
    json!({
        "type": "string",
        "description": "Sampled APB event kind.",
        "enum": EVENTS,
    })
}

fn direction_schema() -> Value {
    json!({
        "type": "string",
        "description": "Direction derived from the sampled pwrite value.",
        "enum": DIRECTIONS,
    })
}

fn apb_source_input_schema() -> Value {
    let branches = apb::profile_specs()
        .iter()
        .map(|profile| {
            let profile_condition = if profile.name == "apb4" {
                json!({
                    "anyOf": [
                        {"not": {"required": ["profile"]}},
                        {"required": ["profile"], "properties": {"profile": {"const": profile.name}}}
                    ]
                })
            } else {
                json!({
                    "required": ["profile"],
                    "properties": {"profile": {"const": profile.name}}
                })
            };
            json!({
                "allOf": [
                    profile_condition,
                    {
                        "oneOf": [
                            {
                                "anyOf": [
                                    {"not": {"required": ["pready_mode"]}},
                                    {"required": ["pready_mode"], "properties": {"pready_mode": {"const": "mapped"}}}
                                ],
                                "properties": {
                                    "maps": apb_input_maps_schema(profile, true)
                                }
                            },
                            {
                                "required": ["pready_mode"],
                                "properties": {
                                    "pready_mode": {"const": "implicit-high"},
                                    "include_wait": {"const": false},
                                    "maps": apb_input_maps_schema(profile, false)
                                }
                            }
                        ]
                    }
                ]
            })
        })
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
                "const": "extract.apb.source",
                "description": "Input document kind discriminator."
            },
            "profile": ref_schema("apbProfile"),
            "pready_mode": ref_schema("apbPreadyMode"),
            "include_wait": {
                "type": "boolean",
                "description": "Whether to emit one access-wait row per waited Access cycle."
            },
            "name": {
                "type": "string",
                "description": "APB port name metadata. Defaults to apb."
            },
            "includes": {
                "type": "array",
                "description": "Regexes selecting waveform signal candidates for APB auto-mapping.",
                "items": {"type": "string"}
            },
            "maps": {
                "type": "object",
                "description": "Explicit mappings from lowercase APB standard signal names to waveform signal names.",
                "additionalProperties": {"type": "string"}
            }
        },
        "allOf": [{"oneOf": branches}]
    })
}

fn apb_input_maps_schema(profile: &ApbProfileSpec, allow_pready: bool) -> Value {
    let properties = apb::standard_signals(profile)
        .filter(|standard| allow_pready || *standard != "pready")
        .map(|standard| (standard.to_string(), json!({"type": "string"})))
        .collect::<Map<_, _>>();
    json!({
        "type": "object",
        "additionalProperties": false,
        "properties": properties,
    })
}

fn apb_data_schema() -> Value {
    json!({
        "oneOf": apb::profile_specs()
            .iter()
            .flat_map(|profile| [
                apb_data_branch(profile, "mapped", false),
                apb_data_branch(profile, "mapped", true),
                apb_data_branch(profile, "implicit-high", false),
            ])
            .collect::<Vec<_>>()
    })
}

fn apb_context_schema() -> Value {
    json!({
        "oneOf": apb::profile_specs()
            .iter()
            .flat_map(|profile| [
                apb_context_branch(profile, "mapped", false),
                apb_context_branch(profile, "mapped", true),
                apb_context_branch(profile, "implicit-high", false),
            ])
            .collect::<Vec<_>>()
    })
}

fn apb_data_branch(profile: &ApbProfileSpec, mode: &str, include_wait: bool) -> Value {
    let mut context = apb_context_properties(profile, mode, include_wait);
    context.insert(
        "events".to_string(),
        json!({
            "type": "array",
            "description": "Extracted APB sampled events in event order.",
            "items": {
                "oneOf": allowed_events(mode, include_wait)
                    .iter()
                    .flat_map(|event| DIRECTIONS.iter().map(move |direction| {
                        ref_schema(&apb_event_def_name(profile, event, direction))
                    }))
                    .collect::<Vec<_>>()
            }
        }),
    );
    json!({
        "type": "object",
        "additionalProperties": true,
        "required": [
            "name", "profile", "issue", "pready_mode", "include_wait", "mappings", "events"
        ],
        "properties": context,
    })
}

fn apb_context_branch(profile: &ApbProfileSpec, mode: &str, include_wait: bool) -> Value {
    json!({
        "type": "object",
        "additionalProperties": true,
        "required": [
            "name", "profile", "issue", "pready_mode", "include_wait", "mappings"
        ],
        "properties": apb_context_properties(profile, mode, include_wait),
    })
}

fn apb_context_properties(
    profile: &ApbProfileSpec,
    mode: &str,
    include_wait: bool,
) -> Map<String, Value> {
    Map::from_iter([
        (
            "name".to_string(),
            json!({
                "type": "string",
                "description": "APB port name supplied by CLI or source JSON."
            }),
        ),
        (
            "profile".to_string(),
            json!({
                "const": profile.name,
                "description": "APB profile name used for standard signal mapping."
            }),
        ),
        (
            "issue".to_string(),
            json!({
                "const": profile.issue,
                "description": "Arm IHI 0024 issue used for this profile definition."
            }),
        ),
        (
            "pready_mode".to_string(),
            json!({
                "const": mode,
                "description": "PREADY handling mode."
            }),
        ),
        (
            "include_wait".to_string(),
            json!({
                "const": include_wait,
                "description": "Whether waited Access cycles are emitted."
            }),
        ),
        (
            "mappings".to_string(),
            apb_mappings_schema(profile, mode == "mapped"),
        ),
    ])
}

fn apb_mappings_schema(profile: &ApbProfileSpec, mapped_pready: bool) -> Value {
    let properties = apb::standard_signals(profile)
        .filter(|standard| mapped_pready || *standard != "pready")
        .map(|standard| (standard.to_string(), ref_schema("extractApbMapping")))
        .collect::<Map<_, _>>();
    let mut required = vec!["pclk", "psel", "penable", "pwrite"];
    if mapped_pready {
        required.push("pready");
    }
    json!({
        "type": "object",
        "description": "Resolved waveform mappings keyed by lowercase APB standard signal name.",
        "additionalProperties": false,
        "required": required,
        "properties": properties,
    })
}

fn apb_event_schema() -> Value {
    json!({
        "oneOf": apb::profile_specs()
            .iter()
            .flat_map(|profile| EVENTS.iter().flat_map(move |event| {
                DIRECTIONS.iter().map(move |direction| {
                    ref_schema(&apb_event_def_name(profile, event, direction))
                })
            }))
            .collect::<Vec<_>>()
    })
}

fn apb_profile_event_schema(profile: &ApbProfileSpec, event: &str, direction: &str) -> Value {
    json!({
        "type": "object",
        "additionalProperties": true,
        "required": ["time", "sample_time", "profile", "event", "direction", "payload"],
        "properties": {
            "time": {
                "$ref": "#/$defs/normalizedTime",
                "description": "Selected APB event timestamp."
            },
            "sample_time": {
                "$ref": "#/$defs/normalizedTime",
                "description": "Pre-edge timestamp used to classify and sample the APB event."
            },
            "profile": {
                "const": profile.name,
                "description": "APB profile name for this event row."
            },
            "event": {
                "const": event,
                "description": "Sampled APB event kind."
            },
            "direction": {
                "const": direction,
                "description": "Direction derived from the sampled pwrite value."
            },
            "payload": apb_payload_schema(profile, event, direction)
        }
    })
}

fn apb_payload_schema(profile: &ApbProfileSpec, event: &str, direction: &str) -> Value {
    let properties = apb::event_payload_signals(profile, event, direction)
        .map(|standard| (standard.to_string(), ref_schema("sampledValue")))
        .collect::<Map<_, _>>();
    json!({
        "type": "object",
        "description": "Observed APB values keyed by lowercase standard signal name.",
        "additionalProperties": false,
        "required": ["pwrite"],
        "properties": properties,
    })
}

fn allowed_events(mode: &str, include_wait: bool) -> &'static [&'static str] {
    if mode == "mapped" && include_wait {
        EVENTS
    } else {
        &["setup", "access-complete"]
    }
}

fn apb_event_def_name(profile: &ApbProfileSpec, event: &str, direction: &str) -> String {
    format!(
        "extract{}{}{}Event",
        schema_suffix(profile.name),
        schema_suffix(event),
        schema_suffix(direction)
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
    fn implicit_high_context_forbids_pready_and_wait_capture() {
        let context = apb_context_branch(&apb::profile_specs()[1], "implicit-high", false);
        assert_eq!(
            context["properties"]["include_wait"]["const"],
            Value::Bool(false)
        );
        assert!(
            context["properties"]["mappings"]["properties"]
                .get("pready")
                .is_none()
        );
    }

    #[test]
    fn event_payloads_are_closed_and_event_specific() {
        let profile = &apb::profile_specs()[2];
        let setup = apb_payload_schema(profile, "setup", "unknown");
        assert_eq!(setup["additionalProperties"], Value::Bool(false));
        assert!(setup["properties"].get("prdata").is_none());
        let complete = apb_payload_schema(profile, "access-complete", "unknown");
        assert!(complete["properties"].get("prdata").is_some());
        assert!(complete["properties"].get("pwdata").is_some());
    }
}
