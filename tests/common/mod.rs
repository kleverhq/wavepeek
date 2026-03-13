use std::path::PathBuf;
use std::process::Command;

pub mod expr_cases;
pub mod expr_runtime;

#[allow(dead_code)]
pub fn wavepeek_cmd() -> Command {
    Command::new(env!("CARGO_BIN_EXE_wavepeek"))
}

#[allow(dead_code)]
pub fn expected_schema_url() -> &'static str {
    concat!(
        "https://raw.githubusercontent.com/kleverhq/wavepeek/v",
        env!("CARGO_PKG_VERSION"),
        "/schema/wavepeek.json"
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
    let base = std::env::var("WAVEPEEK_RTL_ARTIFACTS_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/opt/rtl-artifacts"));
    base.join(filename)
}
