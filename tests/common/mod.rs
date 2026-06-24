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
    concat!(
        "https://kleverhq.github.io/wavepeek/wavepeek_v",
        env!("CARGO_PKG_VERSION_MAJOR"),
        ".",
        env!("CARGO_PKG_VERSION_MINOR"),
        ".json"
    )
}

#[allow(dead_code)]
pub fn expected_stream_schema_url() -> &'static str {
    concat!(
        "https://kleverhq.github.io/wavepeek/wavepeek-stream-v",
        env!("CARGO_PKG_VERSION_MAJOR"),
        ".",
        env!("CARGO_PKG_VERSION_MINOR"),
        ".json"
    )
}

#[allow(dead_code)]
pub fn fixture_path(filename: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("hand")
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
