use serde_json::{Map, Value, json};

use crate::contract::schema::INPUT_SCHEMA_URL;
use crate::engine::ahb::{self, AhbProfileSpec};

const EVENT_KINDS: &[&str] = &[
    "address",
    "data-complete",
    "reset",
    "desynchronized",
    "data-stall",
    "idle",
    "busy",
];

pub(super) fn apply_output_defs(defs: &mut Map<String, Value>) {
    defs.insert("ahbProfile".to_string(), ahb_profile_schema());
    defs.insert("extractAhbMapping".to_string(), ahb_mapping_schema());
    defs.insert("extractAhbData".to_string(), ahb_data_schema());
    defs.insert("extractAhbEvent".to_string(), ahb_event_schema());
    for profile in ahb::profile_specs() {
        defs.insert(
            profile_initial_def_name(profile),
            initial_data_phase_schema(profile),
        );
        defs.insert(
            profile_event_def_name(profile),
            profile_event_schema(profile, EVENT_KINDS),
        );
        for kind in EVENT_KINDS {
            defs.insert(
                event_def_name(profile, kind),
                event_kind_schema(profile, kind),
            );
        }
    }
}

pub(super) fn apply_stream_context_defs(defs: &mut Map<String, Value>) {
    defs.insert("extractAhbContext".to_string(), ahb_context_schema());
}

pub(super) fn apply_input_defs(defs: &mut Map<String, Value>) {
    defs.insert("ahbProfile".to_string(), ahb_profile_schema());
    defs.insert(
        "extractAhbSourceInput".to_string(),
        ahb_source_input_schema(),
    );
}

fn ahb_profile_schema() -> Value {
    json!({
        "type": "string",
        "description": "AHB profile name: ahb-lite or ahb5.",
        "enum": ahb::profile_specs()
            .iter()
            .map(|profile| profile.name)
            .collect::<Vec<_>>(),
    })
}

fn ahb_mapping_schema() -> Value {
    json!({
        "type": "object",
        "additionalProperties": false,
        "required": ["path"],
        "properties": {
            "path": ref_schema("canonicalPath")
        }
    })
}

fn ahb_source_input_schema() -> Value {
    let default = ahb::profile_specs()
        .iter()
        .find(|profile| profile.name == "ahb-lite")
        .expect("AHB-Lite profile must exist");
    let mut branches = vec![json!({
        "anyOf": [
            {"not": {"required": ["profile"]}},
            {"required": ["profile"], "properties": {"profile": {"const": default.name}}}
        ],
        "properties": {"maps": input_maps_schema(default)}
    })];
    branches.extend(
        ahb::profile_specs()
            .iter()
            .filter(|profile| profile.name != default.name)
            .map(|profile| {
                json!({
                    "required": ["profile"],
                    "properties": {
                        "profile": {"const": profile.name},
                        "maps": input_maps_schema(profile)
                    }
                })
            }),
    );

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
                "const": "extract.ahb.source",
                "description": "Input document kind discriminator."
            },
            "profile": ref_schema("ahbProfile"),
            "include_stall": {"type": "boolean", "default": false},
            "include_idle": {"type": "boolean", "default": false},
            "include_busy": {"type": "boolean", "default": false},
            "name": {
                "type": "string",
                "description": "AHB interface name metadata. Defaults to ahb."
            },
            "includes": {
                "type": "array",
                "items": {"type": "string"},
                "description": "Regexes selecting waveform signal candidates for AHB auto-mapping."
            },
            "maps": {
                "type": "object",
                "description": "Explicit mappings from lowercase AHB standard signal names to waveform signal names."
            }
        },
        "allOf": [{"oneOf": branches}]
    })
}

fn input_maps_schema(profile: &AhbProfileSpec) -> Value {
    let properties = profile
        .signals
        .iter()
        .map(|standard| ((*standard).to_string(), json!({"type": "string"})))
        .collect::<Map<_, _>>();
    json!({
        "type": "object",
        "additionalProperties": false,
        "properties": properties,
    })
}

fn ahb_data_schema() -> Value {
    json!({
        "oneOf": ahb::profile_specs()
            .iter()
            .map(ahb_data_profile_schema)
            .collect::<Vec<_>>()
    })
}

fn ahb_context_schema() -> Value {
    json!({
        "oneOf": ahb::profile_specs()
            .iter()
            .map(ahb_context_profile_schema)
            .collect::<Vec<_>>()
    })
}

fn ahb_data_profile_schema(profile: &AhbProfileSpec) -> Value {
    let mut schema = context_properties(profile);
    let properties = schema["properties"]
        .as_object_mut()
        .expect("context properties object");
    properties.insert(
        "events".to_string(),
        json!({
            "type": "array",
            "items": ref_schema(&profile_event_def_name(profile)),
            "description": "Ordered public AHB pipeline events."
        }),
    );
    schema["required"] = json!([
        "name",
        "profile",
        "issue",
        "include_stall",
        "include_idle",
        "include_busy",
        "initial_data_phase",
        "mappings",
        "events"
    ]);
    schema["allOf"] = Value::Array(inclusion_branches(profile));
    schema
}

fn ahb_context_profile_schema(profile: &AhbProfileSpec) -> Value {
    context_properties(profile)
}

fn context_properties(profile: &AhbProfileSpec) -> Value {
    json!({
        "type": "object",
        "additionalProperties": false,
        "required": [
            "name",
            "profile",
            "issue",
            "include_stall",
            "include_idle",
            "include_busy",
            "initial_data_phase",
            "mappings"
        ],
        "properties": {
            "name": {"type": "string"},
            "profile": {"const": profile.name},
            "issue": {"const": profile.issue},
            "include_stall": {"type": "boolean"},
            "include_idle": {"type": "boolean"},
            "include_busy": {"type": "boolean"},
            "initial_data_phase": ref_schema(&profile_initial_def_name(profile)),
            "mappings": mappings_schema(profile)
        }
    })
}

fn mappings_schema(profile: &AhbProfileSpec) -> Value {
    let properties = profile
        .signals
        .iter()
        .map(|standard| ((*standard).to_string(), ref_schema("extractAhbMapping")))
        .collect::<Map<_, _>>();
    json!({
        "type": "object",
        "additionalProperties": false,
        "properties": properties,
        "required": ["hclk", "htrans", "hready", "hwrite"]
    })
}

fn inclusion_branches(profile: &AhbProfileSpec) -> Vec<Value> {
    let mut branches = Vec::with_capacity(8);
    for stall in [false, true] {
        for idle in [false, true] {
            for busy in [false, true] {
                let mut kinds = vec!["address", "data-complete", "reset", "desynchronized"];
                if stall {
                    kinds.push("data-stall");
                }
                if idle {
                    kinds.push("idle");
                }
                if busy {
                    kinds.push("busy");
                }
                branches.push(json!({
                    "if": {
                        "properties": {
                            "include_stall": {"const": stall},
                            "include_idle": {"const": idle},
                            "include_busy": {"const": busy}
                        },
                        "required": ["include_stall", "include_idle", "include_busy"]
                    },
                    "then": {
                        "properties": {
                            "events": {
                                "items": profile_event_schema(profile, kinds.as_slice())
                            }
                        }
                    }
                }));
            }
        }
    }
    branches
}

fn ahb_event_schema() -> Value {
    json!({
        "oneOf": ahb::profile_specs()
            .iter()
            .map(|profile| ref_schema(&profile_event_def_name(profile)))
            .collect::<Vec<_>>()
    })
}

fn profile_event_schema(profile: &AhbProfileSpec, kinds: &[&str]) -> Value {
    json!({
        "oneOf": kinds
            .iter()
            .map(|kind| ref_schema(&event_def_name(profile, kind)))
            .collect::<Vec<_>>()
    })
}

fn event_kind_schema(profile: &AhbProfileSpec, kind: &str) -> Value {
    match kind {
        "address" => address_event_schema(profile),
        "busy" => slot_event_schema(profile, "busy", true),
        "idle" => slot_event_schema(profile, "idle", false),
        "data-stall" | "data-complete" => {
            let branches = ["read", "write", "unknown"]
                .iter()
                .map(|direction| data_event_schema(profile, kind, direction))
                .collect::<Vec<_>>();
            json!({"oneOf": branches})
        }
        "reset" | "desynchronized" => boundary_event_schema(profile, kind),
        _ => unreachable!("known AHB event kind"),
    }
}

fn common_event_properties(profile: &AhbProfileSpec, kind: &str) -> Map<String, Value> {
    Map::from_iter([
        (
            "time".to_string(),
            json!({"$ref": "#/$defs/normalizedTime"}),
        ),
        (
            "sample_time".to_string(),
            json!({"$ref": "#/$defs/normalizedTime"}),
        ),
        ("profile".to_string(), json!({"const": profile.name})),
        ("event".to_string(), json!({"const": kind})),
    ])
}

fn address_event_schema(profile: &AhbProfileSpec) -> Value {
    let mut properties = common_event_properties(profile, "address");
    properties.insert(
        "transfer".to_string(),
        json!({"type": "string", "enum": ["nonseq", "seq"]}),
    );
    properties.insert(
        "direction".to_string(),
        json!({"type": "string", "enum": ["read", "write", "unknown"]}),
    );
    properties.insert(
        "payload".to_string(),
        payload_schema(
            profile,
            address_payload_signals(profile),
            &["htrans", "hwrite"],
        ),
    );
    closed_event(
        properties,
        &[
            "time",
            "sample_time",
            "profile",
            "event",
            "transfer",
            "direction",
            "payload",
        ],
    )
}

fn slot_event_schema(profile: &AhbProfileSpec, kind: &str, has_direction: bool) -> Value {
    let mut properties = common_event_properties(profile, kind);
    properties.insert("transfer".to_string(), json!({"const": kind}));
    if has_direction {
        properties.insert(
            "direction".to_string(),
            json!({"type": "string", "enum": ["read", "write", "unknown"]}),
        );
    }
    let signals = if kind == "idle" {
        vec!["htrans", "haddr", "hmastlock"]
    } else {
        address_payload_signals(profile)
    };
    let required_payload = if has_direction {
        &["htrans", "hwrite"][..]
    } else {
        &["htrans"][..]
    };
    properties.insert(
        "payload".to_string(),
        payload_schema(profile, signals, required_payload),
    );
    let mut required = vec![
        "time",
        "sample_time",
        "profile",
        "event",
        "transfer",
        "payload",
    ];
    if has_direction {
        required.push("direction");
    }
    closed_event(properties, required.as_slice())
}

fn boundary_event_schema(profile: &AhbProfileSpec, kind: &str) -> Value {
    closed_event(
        common_event_properties(profile, kind),
        &["time", "sample_time", "profile", "event"],
    )
}

fn data_event_schema(profile: &AhbProfileSpec, kind: &str, direction: &str) -> Value {
    let mut properties = common_event_properties(profile, kind);
    properties.insert("direction".to_string(), json!({"const": direction}));
    let mut signals = vec!["hresp"];
    match direction {
        "read" => signals.push("hrdata"),
        "write" => signals.extend(["hwdata", "hwstrb", "hwuser"]),
        "unknown" => signals.extend(["hwdata", "hwstrb", "hwuser", "hrdata"]),
        _ => unreachable!("known direction"),
    }
    if kind == "data-complete" {
        if matches!(direction, "read" | "unknown") {
            signals.push("hruser");
        }
        signals.extend(["hbuser", "hexokay"]);
    }
    signals.retain(|signal| profile.signals.contains(signal));
    let mut payload = payload_schema(profile, signals.clone(), &[]);
    if kind == "data-complete" {
        let success_signals = ["hruser", "hbuser", "hexokay"]
            .into_iter()
            .filter(|signal| signals.contains(signal))
            .collect::<Vec<_>>();
        let conditions = success_signals
            .into_iter()
            .map(|signal| {
                json!({
                    "if": {"required": [signal]},
                    "then": {
                        "required": ["hresp"],
                        "properties": {"hresp": {"const": "1'h0"}}
                    }
                })
            })
            .collect::<Vec<_>>();
        if !conditions.is_empty() {
            payload["allOf"] = Value::Array(conditions);
        }
    }
    properties.insert("payload".to_string(), payload);
    closed_event(
        properties,
        &["time", "sample_time", "profile", "event", "direction"],
    )
}

fn closed_event(properties: Map<String, Value>, required: &[&str]) -> Value {
    json!({
        "type": "object",
        "additionalProperties": false,
        "required": required,
        "properties": properties,
    })
}

fn initial_data_phase_schema(profile: &AhbProfileSpec) -> Value {
    json!({
        "oneOf": [
            {
                "type": "object",
                "additionalProperties": false,
                "required": ["state"],
                "properties": {"state": {"const": "empty"}}
            },
            {
                "type": "object",
                "additionalProperties": false,
                "required": ["state"],
                "properties": {"state": {"const": "desynchronized"}}
            },
            {
                "type": "object",
                "additionalProperties": false,
                "required": ["state", "address"],
                "properties": {
                    "state": {"const": "pending"},
                    "address": initial_address_schema(profile)
                }
            }
        ]
    })
}

fn initial_address_schema(profile: &AhbProfileSpec) -> Value {
    json!({
        "type": "object",
        "additionalProperties": false,
        "required": ["time", "sample_time", "transfer", "direction", "payload"],
        "properties": {
            "time": ref_schema("normalizedTime"),
            "sample_time": ref_schema("normalizedTime"),
            "transfer": {"type": "string", "enum": ["nonseq", "seq"]},
            "direction": {"type": "string", "enum": ["read", "write", "unknown"]},
            "payload": payload_schema(
                profile,
                address_payload_signals(profile),
                &["htrans", "hwrite"]
            )
        }
    })
}

fn payload_schema(
    profile: &AhbProfileSpec,
    signals: Vec<&'static str>,
    required: &[&str],
) -> Value {
    let properties = signals
        .into_iter()
        .filter(|signal| profile.signals.contains(signal))
        .map(|signal| (signal.to_string(), ref_schema("sampledValue")))
        .collect::<Map<_, _>>();
    json!({
        "type": "object",
        "additionalProperties": false,
        "properties": properties,
        "required": required,
    })
}

fn address_payload_signals(profile: &AhbProfileSpec) -> Vec<&'static str> {
    [
        "htrans",
        "hwrite",
        "haddr",
        "hburst",
        "hmastlock",
        "hprot",
        "hsize",
        "hauser",
        "hnonsec",
        "hexcl",
        "hmaster",
    ]
    .into_iter()
    .filter(|signal| profile.signals.contains(signal))
    .collect()
}

fn profile_initial_def_name(profile: &AhbProfileSpec) -> String {
    format!("extract{}InitialDataPhase", schema_suffix(profile.name))
}

fn profile_event_def_name(profile: &AhbProfileSpec) -> String {
    format!("extract{}Event", schema_suffix(profile.name))
}

fn event_def_name(profile: &AhbProfileSpec, kind: &str) -> String {
    format!(
        "extract{}{}Event",
        schema_suffix(profile.name),
        schema_suffix(kind)
    )
}

fn schema_suffix(name: &str) -> String {
    let mut suffix = String::new();
    let mut capitalize = true;
    for ch in name.chars() {
        if ch.is_ascii_alphanumeric() {
            suffix.push(if capitalize {
                capitalize = false;
                ch.to_ascii_uppercase()
            } else {
                ch
            });
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
    fn profile_event_defs_cover_every_runtime_event_kind() {
        let mut defs = Map::new();
        apply_output_defs(&mut defs);
        for profile in ahb::profile_specs() {
            let profile_def = profile_event_def_name(profile);
            assert_eq!(
                defs[&profile_def]["oneOf"]
                    .as_array()
                    .expect("profile event oneOf")
                    .len(),
                EVENT_KINDS.len()
            );
            for kind in EVENT_KINDS {
                assert!(defs.contains_key(&event_def_name(profile, kind)));
            }
        }
    }

    #[test]
    fn input_maps_are_profile_closed() {
        let source = ahb_source_input_schema();
        let branches = source["allOf"][0]["oneOf"]
            .as_array()
            .expect("profile branches");
        assert_eq!(branches.len(), 2);
        assert_eq!(
            branches[0]["properties"]["maps"]["additionalProperties"],
            false
        );
    }
}
