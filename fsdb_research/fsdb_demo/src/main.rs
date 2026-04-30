use std::path::{Path, PathBuf};

use clap::{Parser, Subcommand};
use fsdb_demo::{
    BuildInfo, DisplayPath, default_fsdb_writer_path, default_verdi_bridge_path,
    format_key_value_lines, generate_fsdb_fixture, probe_with_bridge_path,
};

fn main() {
    let cli = Cli::parse();
    let build_info = BuildInfo::from_compiled_env();

    let outcome = match cli.command {
        Command::BuildInfo => build_info_output(&build_info),
        Command::Noop => noop_output(&build_info),
        Command::Generate { out, writer } => generate_output(&build_info, &out, writer.as_deref()),
        Command::Probe { waves, bridge } => probe_output(&build_info, &waves, bridge.as_deref()),
    };

    match outcome {
        Ok(stdout) => {
            print!("{stdout}");
        }
        Err(message) => {
            eprintln!("error: demo: {message}");
            std::process::exit(1);
        }
    }
}

#[derive(Debug, Parser)]
#[command(name = "fsdb_demo")]
#[command(about = "Standalone lazy-load probe for Verdi/FsdbReader feasibility")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Print the compiled bridge metadata.
    BuildInfo,
    /// Prove the binary starts without touching the Verdi bridge.
    Noop,
    /// Generate a tiny FSDB fixture using the separately built FsdbWriter tool.
    Generate {
        /// Path where the generated FSDB file should be written.
        #[arg(long)]
        out: PathBuf,

        /// Optional explicit writer path. Useful for manual experiments.
        #[arg(long)]
        writer: Option<PathBuf>,
    },
    /// Load a bridge lazily and probe one FSDB file.
    Probe {
        /// Path to the waveform file that the bridge should probe.
        #[arg(long)]
        waves: PathBuf,

        /// Optional explicit bridge path. Useful for tests or manual experiments.
        #[arg(long)]
        bridge: Option<PathBuf>,
    },
}

fn build_info_output(build_info: &BuildInfo) -> Result<String, String> {
    Ok(format_key_value_lines(&[
        (
            "mock-bridge-path",
            build_info.mock_bridge_path.display().to_string(),
        ),
        (
            "verdi-bridge-status",
            build_info.verdi_bridge_status.clone(),
        ),
        (
            "verdi-bridge-path",
            DisplayPath(&build_info.verdi_bridge_path).to_string(),
        ),
        (
            "fsdb-writer-path",
            DisplayPath(&build_info.fsdb_writer_path).to_string(),
        ),
        (
            "verdi-home",
            DisplayPath(&build_info.verdi_home).to_string(),
        ),
    ]))
}

fn noop_output(build_info: &BuildInfo) -> Result<String, String> {
    Ok(format_key_value_lines(&[
        ("command", "noop".to_string()),
        ("status", "ok".to_string()),
        (
            "verdi-bridge-status",
            build_info.verdi_bridge_status.clone(),
        ),
    ]))
}

fn generate_output(
    build_info: &BuildInfo,
    out: &Path,
    explicit_writer: Option<&Path>,
) -> Result<String, String> {
    let writer_path = match explicit_writer {
        Some(path) => path.to_path_buf(),
        None => default_fsdb_writer_path(build_info).map_err(|error| error.to_string())?,
    };

    let output_path =
        generate_fsdb_fixture(out, &writer_path).map_err(|error| error.to_string())?;
    Ok(format_key_value_lines(&[
        ("writer-path", writer_path.display().to_string()),
        ("output-path", output_path.display().to_string()),
        ("status", "ok".to_string()),
    ]))
}

fn probe_output(
    build_info: &BuildInfo,
    waves: &Path,
    explicit_bridge: Option<&Path>,
) -> Result<String, String> {
    let bridge_path = match explicit_bridge {
        Some(path) => path.to_path_buf(),
        None => default_verdi_bridge_path(build_info).map_err(|error| error.to_string())?,
    };

    let summary = probe_with_bridge_path(waves, &bridge_path).map_err(|error| error.to_string())?;
    Ok(format_key_value_lines(&[
        ("bridge-path", bridge_path.display().to_string()),
        ("bridge-kind", summary.bridge_kind),
        ("waveform-path", summary.waves_path.display().to_string()),
        ("signal-count", summary.signal_count.to_string()),
        ("end-time-raw", summary.end_time_raw.to_string()),
        ("scale-unit", summary.scale_unit),
        ("message", summary.message),
    ]))
}
