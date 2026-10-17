use std::ffi::OsString;
use std::path::PathBuf;
use std::sync::Arc;

use serde::Serialize;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

use crate::cli;
use crate::waveform;

#[derive(Debug, Serialize)]
pub(crate) struct BrowserResult {
    pub(crate) stdout: String,
    pub(crate) stderr: String,
    pub(crate) status: u8,
}

impl BrowserResult {
    fn unsupported(message: &str) -> Self {
        Self {
            stdout: String::new(),
            stderr: format!("fatal: args: {message}\n"),
            status: 1,
        }
    }
}

pub(crate) fn invoke(argv: Vec<String>, filename: String, bytes: &[u8]) -> BrowserResult {
    if argv.is_empty() {
        return BrowserResult::unsupported("a wavepeek command is required");
    }
    if browser_command(&argv) == Some("skill")
        || argv
            .windows(2)
            .any(|args| args[0] == "help" && args[1] == "skill")
    {
        return BrowserResult::unsupported("skill is not supported in the browser");
    }
    if argv
        .iter()
        .any(|arg| arg == "--source" || arg.starts_with("--source="))
    {
        return BrowserResult::unsupported("--source is not supported in the browser");
    }
    let paths = std::iter::once(filename.as_str())
        .chain(
            argv.windows(2)
                .filter_map(|args| (args[0] == "--waves").then_some(args[1].as_str())),
        )
        .chain(argv.iter().filter_map(|arg| arg.strip_prefix("--waves=")));
    for path in paths {
        if let Some(format) = native_format(path) {
            return BrowserResult::unsupported(&format!(
                "{format} is not supported in the browser"
            ));
        }
    }

    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let result = waveform::with_waveform_bytes(PathBuf::from(filename), Arc::from(bytes), || {
        cli::run_from(
            argv.into_iter().map(OsString::from).collect(),
            &mut stdout,
            &mut stderr,
            true,
            true,
        )
    });
    let status = match result {
        Ok(()) => 0,
        Err(failure) => {
            if !failure.reported {
                stderr.extend_from_slice(failure.error.to_string().as_bytes());
                stderr.push(b'\n');
            }
            failure.error.exit_code()
        }
    };
    BrowserResult {
        stdout: String::from_utf8_lossy(&stdout).into_owned(),
        stderr: String::from_utf8_lossy(&stderr).into_owned(),
        status,
    }
}

fn native_format(filename: &str) -> Option<&'static str> {
    let filename = filename.to_ascii_lowercase();
    if filename.ends_with(".fsdb") || filename.ends_with(".fsdb.gz") {
        Some("FSDB")
    } else if filename.ends_with(".shm") {
        Some("SHM")
    } else {
        None
    }
}

fn browser_command(argv: &[String]) -> Option<&str> {
    argv.iter()
        .skip(1)
        .map(String::as_str)
        .find(|arg| !matches!(*arg, "--json" | "--jsonl"))
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn run_wavepeek(argv_json: &str, filename: String, bytes: &[u8]) -> String {
    let result = match serde_json::from_str(argv_json) {
        Ok(argv) => invoke(argv, filename, bytes),
        Err(error) => BrowserResult::unsupported(&format!("invalid argv JSON: {error}")),
    };
    serde_json::to_string(&result).unwrap_or_else(|error| {
        format!(
            r#"{{"stdout":"","stderr":"fatal: internal: failed to serialize browser result: {error}\n","status":1}}"#
        )
    })
}

#[cfg(test)]
mod tests {
    use super::invoke;

    const VCD: &[u8] = br#"$date today $end
$version test $end
$timescale 1ns $end
$scope module top $end
$var wire 1 ! clk $end
$upscope $end
$enddefinitions $end
#0
0!
#5
1!
"#;

    fn run(args: &[&str]) -> super::BrowserResult {
        invoke(
            args.iter().map(|arg| (*arg).to_string()).collect(),
            "demo.vcd".to_string(),
            VCD,
        )
    }

    #[test]
    fn browser_invocation_runs_waveform_bytes_and_keeps_machine_channels() {
        let human = run(&["wavepeek", "info", "--waves", "demo.vcd"]);
        assert_eq!(human.status, 0);
        assert!(human.stdout.contains("time_unit: 1ns"));
        assert!(human.stderr.is_empty());

        let json = run(&["wavepeek", "info", "--waves", "demo.vcd", "--json"]);
        assert_eq!(json.status, 0);
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&json.stdout)
                .expect("browser JSON should parse")["command"],
            "info"
        );
        assert!(json.stderr.is_empty());
    }

    #[test]
    fn browser_invocation_rejects_unsupported_inputs_clearly() {
        for (args, filename, expected) in [
            (
                vec!["wavepeek", "skill", "out"],
                "demo.vcd",
                "skill is not supported",
            ),
            (
                vec!["wavepeek", "help", "skill"],
                "demo.vcd",
                "skill is not supported",
            ),
            (
                vec!["wavepeek", "extract", "axi", "--source", "map.json"],
                "demo.vcd",
                "--source is not supported",
            ),
            (
                vec!["wavepeek", "info", "--waves", "demo.fsdb"],
                "demo.fsdb",
                "FSDB is not supported",
            ),
            (
                vec!["wavepeek", "info", "--waves", "demo.fsdb.gz"],
                "demo.fsdb.gz",
                "FSDB is not supported",
            ),
            (
                vec!["wavepeek", "info", "--waves=other.fsdb.gz"],
                "demo.vcd",
                "FSDB is not supported",
            ),
            (
                vec!["wavepeek", "info", "--waves=other.SHM"],
                "demo.vcd",
                "SHM is not supported",
            ),
        ] {
            let result = invoke(
                args.into_iter().map(str::to_string).collect(),
                filename.to_string(),
                VCD,
            );
            assert_eq!(result.status, 1);
            assert!(result.stdout.is_empty());
            assert!(result.stderr.contains(expected));
        }
    }

    #[test]
    fn browser_help_is_truthful() {
        let root = run(&["wavepeek", "--help"]);
        assert_eq!(root.status, 0);
        assert!(!root.stdout.contains("Extract the packaged agent skill"));
        assert!(root.stdout.contains("FSDB/SHM input, skill"));

        for command in ["ahb", "apb", "atb", "axi", "axistream", "generic"] {
            let extract = run(&["wavepeek", "extract", command, "--help"]);
            assert_eq!(extract.status, 0);
            assert!(!extract.stdout.contains("source-file"));
            assert!(
                !extract
                    .stdout
                    .lines()
                    .any(|line| line.trim_start().starts_with("--source"))
            );
        }
    }
}
