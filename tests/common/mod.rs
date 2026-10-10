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
pub fn ondas_fixture_path(fixture: &str) -> PathBuf {
    PathBuf::from(
        std::env::var("ONDAS_FIXTURES_DIR")
            .expect("ONDAS_FIXTURES_DIR must be set by the wavepeek container"),
    )
    .join(fixture)
    .join("waveform.fst")
}
