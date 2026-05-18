use std::path::{Path, PathBuf};
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
        .or_else(|_| std::env::var("RTL_ARTIFACTS_DIR"))
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            let opt_dir = Path::new("/opt/rtl-artifacts");
            if opt_dir.is_dir() && std::fs::read_dir(opt_dir).is_ok() {
                return opt_dir.to_path_buf();
            }

            std::env::var("HOME")
                .map(PathBuf::from)
                .map(|home| home.join(".cache").join("wavepeek").join("rtl-artifacts"))
                .unwrap_or_else(|_| opt_dir.to_path_buf())
        });
    base.join(filename)
}
