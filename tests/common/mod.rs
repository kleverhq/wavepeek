use std::path::PathBuf;
use std::process::Command;

pub mod command_cases;
pub mod expr_cases;
pub mod expr_runtime;

#[allow(dead_code)]
pub fn wavepeek_cmd() -> Command {
    Command::new(env!("CARGO_BIN_EXE_wavepeek"))
}

#[allow(dead_code)]
pub fn expected_schema_url() -> &'static str {
    "https://kleverhq.github.io/wavepeek/schema-output-v2.2.json"
}

#[allow(dead_code)]
pub fn expected_stream_schema_url() -> &'static str {
    "https://kleverhq.github.io/wavepeek/schema-stream-v2.2.json"
}

#[allow(dead_code)]
pub fn expected_input_schema_url() -> &'static str {
    "https://kleverhq.github.io/wavepeek/schema-input-v2.2.json"
}

#[allow(dead_code)]
pub fn fixture_path(filename: &str) -> PathBuf {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    for directory in ["generated", "hand"] {
        let path = root
            .join("tests")
            .join("fixtures")
            .join(directory)
            .join(filename);
        if path.exists() {
            return path;
        }
    }
    root.join("tests")
        .join("fixtures")
        .join("generated")
        .join(filename)
}

#[allow(dead_code)]
pub fn rtl_fixture_path(filename: &str) -> PathBuf {
    PathBuf::from(
        std::env::var("RTL_ARTIFACTS_DIR")
            .expect("RTL_ARTIFACTS_DIR must be set by the wavepeek container"),
    )
    .join(filename)
}
