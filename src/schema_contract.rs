pub const SCHEMA_URL: &str = concat!(
    "https://github.com/kleverhq/wavepeek/blob/v",
    env!("CARGO_PKG_VERSION"),
    "/schema/wavepeek.json"
);

pub const CANONICAL_SCHEMA_JSON: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/schema/wavepeek.json"));
