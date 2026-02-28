pub const SCHEMA_URL: &str = concat!(
    "https://raw.githubusercontent.com/kleverhq/wavepeek/v",
    env!("CARGO_PKG_VERSION"),
    "/schema/wavepeek.json"
);

pub const CANONICAL_SCHEMA_JSON: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/schema/wavepeek.json"));
