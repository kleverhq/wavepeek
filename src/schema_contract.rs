pub const SCHEMA_URL: &str = concat!(
    "https://kleverhq.github.io/wavepeek/wavepeek_v",
    env!("CARGO_PKG_VERSION_MAJOR"),
    ".",
    env!("CARGO_PKG_VERSION_MINOR"),
    ".json"
);

pub const CANONICAL_SCHEMA_JSON: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/schema/wavepeek_v",
    env!("CARGO_PKG_VERSION_MAJOR"),
    ".",
    env!("CARGO_PKG_VERSION_MINOR"),
    ".json"
));

pub const STREAM_SCHEMA_URL: &str = concat!(
    "https://kleverhq.github.io/wavepeek/wavepeek-stream-v",
    env!("CARGO_PKG_VERSION_MAJOR"),
    ".",
    env!("CARGO_PKG_VERSION_MINOR"),
    ".json"
);

pub const CANONICAL_STREAM_SCHEMA_JSON: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/schema/wavepeek-stream-v",
    env!("CARGO_PKG_VERSION_MAJOR"),
    ".",
    env!("CARGO_PKG_VERSION_MINOR"),
    ".json"
));

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use super::CANONICAL_SCHEMA_JSON;
    use crate::waveform::{
        EXCLUDED_SCOPE_KIND_ALIASES, EXCLUDED_SIGNAL_KIND_ALIASES, STABLE_SCOPE_KIND_ALIASES,
        STABLE_SIGNAL_KIND_ALIASES,
    };

    #[test]
    fn schema_kind_alias_inventory_matches_canonical_schema() {
        let schema: Value =
            serde_json::from_str(CANONICAL_SCHEMA_JSON).expect("schema artifact should parse");

        let scope_aliases = schema_enum(&schema, "scopeKind");
        assert_eq!(
            scope_aliases,
            STABLE_SCOPE_KIND_ALIASES
                .iter()
                .map(|alias| alias.to_string())
                .collect::<Vec<_>>()
        );
        for alias in EXCLUDED_SCOPE_KIND_ALIASES {
            assert!(
                !scope_aliases.iter().any(|candidate| candidate == alias),
                "excluded scope alias {alias:?} leaked into canonical schema"
            );
        }

        let signal_aliases = schema_enum(&schema, "signalKind");
        assert_eq!(
            signal_aliases,
            STABLE_SIGNAL_KIND_ALIASES
                .iter()
                .map(|alias| alias.to_string())
                .collect::<Vec<_>>()
        );
        for alias in EXCLUDED_SIGNAL_KIND_ALIASES {
            assert!(
                !signal_aliases.iter().any(|candidate| candidate == alias),
                "excluded signal alias {alias:?} leaked into canonical schema"
            );
        }
    }

    fn schema_enum(schema: &Value, def_name: &str) -> Vec<String> {
        schema["$defs"][def_name]["enum"]
            .as_array()
            .unwrap_or_else(|| panic!("schema definition {def_name} must expose an enum array"))
            .iter()
            .map(|entry| {
                entry
                    .as_str()
                    .unwrap_or_else(|| panic!("schema definition {def_name} must use strings"))
                    .to_string()
            })
            .collect()
    }

    #[test]
    #[should_panic(expected = "schema definition missing must expose an enum array")]
    fn schema_enum_requires_array_definition() {
        let schema = serde_json::json!({"$defs": {"missing": {}}});
        let _ = schema_enum(&schema, "missing");
    }

    #[test]
    #[should_panic(expected = "schema definition scopeKind must use strings")]
    fn schema_enum_requires_string_entries() {
        let schema = serde_json::json!({
            "$defs": {"scopeKind": {"enum": [1, 2]}}
        });
        let _ = schema_enum(&schema, "scopeKind");
    }
}
