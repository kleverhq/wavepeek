use std::ffi::{CStr, CString, c_char};
use std::fmt;
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};
use std::process::Command;

use libloading::Library;
use thiserror::Error;

const BRIDGE_KIND_SYMBOL: &[u8] = b"fsdb_bridge_kind\0";
const PROBE_FILE_SYMBOL: &[u8] = b"fsdb_probe_file\0";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildInfo {
    pub mock_bridge_path: PathBuf,
    pub verdi_bridge_status: String,
    pub verdi_bridge_path: Option<PathBuf>,
    pub fsdb_writer_path: Option<PathBuf>,
    pub verdi_home: Option<PathBuf>,
}

impl BuildInfo {
    #[must_use]
    pub fn from_compiled_env() -> Self {
        Self {
            mock_bridge_path: PathBuf::from(env_or_empty("FSDB_DEMO_MOCK_BRIDGE_PATH")),
            verdi_bridge_status: env_or_empty("FSDB_DEMO_VERDI_BRIDGE_STATUS"),
            verdi_bridge_path: optional_path("FSDB_DEMO_VERDI_BRIDGE_PATH"),
            fsdb_writer_path: optional_path("FSDB_DEMO_FSDB_WRITER_PATH"),
            verdi_home: optional_path("FSDB_DEMO_VERDI_HOME"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProbeSummary {
    pub bridge_kind: String,
    pub waves_path: PathBuf,
    pub signal_count: u64,
    pub end_time_raw: u64,
    pub scale_unit: String,
    pub message: String,
}

#[derive(Debug, Error)]
pub enum ProbeError {
    #[error("no Verdi bridge was built for this crate (build status: {status})")]
    NoBuiltVerdiBridge { status: String },
    #[error("failed to open bridge '{path}': {error}")]
    OpenBridge {
        path: PathBuf,
        error: libloading::Error,
    },
    #[error("failed to resolve symbol '{symbol}' from bridge '{path}': {error}")]
    LoadSymbol {
        path: PathBuf,
        symbol: &'static str,
        error: libloading::Error,
    },
    #[error("waveform path contains an interior NUL byte: {path}")]
    InvalidWavePath { path: PathBuf },
    #[error("bridge '{path}' returned code {code}: {message}")]
    BridgeReturned {
        path: PathBuf,
        code: i32,
        message: String,
    },
    #[error("bridge '{path}' returned a null bridge-kind string")]
    NullBridgeKind { path: PathBuf },
}

#[derive(Debug, Error)]
pub enum GenerateError {
    #[error("no FSDB fixture writer was built for this crate (build status: {status})")]
    NoBuiltWriter { status: String },
    #[error("failed to run FSDB fixture writer '{path}': {error}")]
    RunWriter {
        path: PathBuf,
        error: std::io::Error,
    },
    #[error("FSDB fixture writer '{path}' failed with status {status}: {stderr}")]
    WriterFailed {
        path: PathBuf,
        status: std::process::ExitStatus,
        stderr: String,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
struct FsdbProbeResult {
    code: i32,
    signal_count: u64,
    end_time_raw: u64,
    scale_unit: [c_char; 32],
    message: [c_char; 256],
}

type BridgeKindFn = unsafe extern "C" fn() -> *const c_char;
type ProbeFileFn = unsafe extern "C" fn(*const c_char, *mut FsdbProbeResult) -> i32;

pub fn default_verdi_bridge_path(build_info: &BuildInfo) -> Result<PathBuf, ProbeError> {
    if let Some(path) = bridge_next_to_current_executable() {
        return Ok(path);
    }

    build_info
        .verdi_bridge_path
        .clone()
        .ok_or_else(|| ProbeError::NoBuiltVerdiBridge {
            status: build_info.verdi_bridge_status.clone(),
        })
}

pub fn default_fsdb_writer_path(build_info: &BuildInfo) -> Result<PathBuf, GenerateError> {
    build_info
        .fsdb_writer_path
        .clone()
        .ok_or_else(|| GenerateError::NoBuiltWriter {
            status: build_info.verdi_bridge_status.clone(),
        })
}

pub fn generate_fsdb_fixture(
    output_path: &Path,
    writer_path: &Path,
) -> Result<PathBuf, GenerateError> {
    let output = Command::new(writer_path)
        .arg(output_path)
        .output()
        .map_err(|error| GenerateError::RunWriter {
            path: writer_path.to_path_buf(),
            error,
        })?;

    if !output.status.success() {
        return Err(GenerateError::WriterFailed {
            path: writer_path.to_path_buf(),
            status: output.status,
            stderr: String::from_utf8_lossy(&output.stderr).trim().to_string(),
        });
    }

    Ok(output_path.to_path_buf())
}

pub fn probe_with_bridge_path(
    waves_path: &Path,
    bridge_path: &Path,
) -> Result<ProbeSummary, ProbeError> {
    let library = unsafe { Library::new(bridge_path) }.map_err(|error| ProbeError::OpenBridge {
        path: bridge_path.to_path_buf(),
        error,
    })?;

    let bridge_kind_fn =
        unsafe { library.get::<BridgeKindFn>(BRIDGE_KIND_SYMBOL) }.map_err(|error| {
            ProbeError::LoadSymbol {
                path: bridge_path.to_path_buf(),
                symbol: "fsdb_bridge_kind",
                error,
            }
        })?;
    let probe_file_fn =
        unsafe { library.get::<ProbeFileFn>(PROBE_FILE_SYMBOL) }.map_err(|error| {
            ProbeError::LoadSymbol {
                path: bridge_path.to_path_buf(),
                symbol: "fsdb_probe_file",
                error,
            }
        })?;

    let bridge_kind_ptr = unsafe { bridge_kind_fn() };
    if bridge_kind_ptr.is_null() {
        return Err(ProbeError::NullBridgeKind {
            path: bridge_path.to_path_buf(),
        });
    }

    let bridge_kind = unsafe { CStr::from_ptr(bridge_kind_ptr) }
        .to_string_lossy()
        .into_owned();

    let c_waves_path = CString::new(waves_path.as_os_str().as_bytes()).map_err(|_| {
        ProbeError::InvalidWavePath {
            path: waves_path.to_path_buf(),
        }
    })?;

    let mut result = FsdbProbeResult {
        code: 0,
        signal_count: 0,
        end_time_raw: 0,
        scale_unit: [0; 32],
        message: [0; 256],
    };

    let code = unsafe { probe_file_fn(c_waves_path.as_ptr(), &mut result) };
    if code != 0 {
        return Err(ProbeError::BridgeReturned {
            path: bridge_path.to_path_buf(),
            code,
            message: c_char_array_to_string(&result.message),
        });
    }

    Ok(ProbeSummary {
        bridge_kind,
        waves_path: waves_path.to_path_buf(),
        signal_count: result.signal_count,
        end_time_raw: result.end_time_raw,
        scale_unit: c_char_array_to_string(&result.scale_unit),
        message: c_char_array_to_string(&result.message),
    })
}

#[must_use]
pub fn format_key_value_lines(entries: &[(&str, String)]) -> String {
    let mut output = String::new();
    for (key, value) in entries {
        output.push_str(key);
        output.push_str(": ");
        output.push_str(value);
        output.push('\n');
    }
    output
}

fn env_or_empty(name: &str) -> String {
    match name {
        "FSDB_DEMO_MOCK_BRIDGE_PATH" => option_env!("FSDB_DEMO_MOCK_BRIDGE_PATH")
            .unwrap_or_default()
            .to_string(),
        "FSDB_DEMO_VERDI_BRIDGE_STATUS" => option_env!("FSDB_DEMO_VERDI_BRIDGE_STATUS")
            .unwrap_or_default()
            .to_string(),
        "FSDB_DEMO_VERDI_BRIDGE_PATH" => option_env!("FSDB_DEMO_VERDI_BRIDGE_PATH")
            .unwrap_or_default()
            .to_string(),
        "FSDB_DEMO_FSDB_WRITER_PATH" => option_env!("FSDB_DEMO_FSDB_WRITER_PATH")
            .unwrap_or_default()
            .to_string(),
        "FSDB_DEMO_VERDI_HOME" => option_env!("FSDB_DEMO_VERDI_HOME")
            .unwrap_or_default()
            .to_string(),
        _ => String::new(),
    }
}

fn optional_path(name: &str) -> Option<PathBuf> {
    let value = env_or_empty(name);
    (!value.is_empty()).then(|| PathBuf::from(value))
}

fn c_char_array_to_string<const N: usize>(buffer: &[c_char; N]) -> String {
    let nul_index = buffer.iter().position(|byte| *byte == 0).unwrap_or(N);
    let bytes = buffer[..nul_index]
        .iter()
        .map(|byte| *byte as u8)
        .collect::<Vec<_>>();
    String::from_utf8_lossy(&bytes).into_owned()
}

fn bridge_next_to_current_executable() -> Option<PathBuf> {
    std::env::current_exe()
        .ok()
        .and_then(|path| {
            path.parent()
                .map(|parent| parent.join("libfsdb_verdi_bridge.so"))
        })
        .filter(|path| path.exists())
}

pub struct DisplayPath<'a>(pub &'a Option<PathBuf>);

impl fmt::Display for DisplayPath<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            Some(path) => write!(formatter, "{}", path.display()),
            None => formatter.write_str("(not built)"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        BuildInfo, c_char_array_to_string, format_key_value_lines, probe_with_bridge_path,
    };
    use std::path::Path;

    #[test]
    fn build_info_exposes_mock_bridge_path() {
        let info = BuildInfo::from_compiled_env();
        assert!(info.mock_bridge_path.ends_with("libfsdb_mock_bridge.so"));
    }

    #[test]
    fn char_array_conversion_stops_at_first_nul() {
        let mut bytes = [0_i8; 8];
        bytes[0] = b'o' as i8;
        bytes[1] = b'k' as i8;
        bytes[2] = 0;
        bytes[3] = b'!' as i8;
        assert_eq!(c_char_array_to_string(&bytes), "ok");
    }

    #[test]
    fn format_key_value_lines_is_deterministic() {
        let output =
            format_key_value_lines(&[("alpha", "1".to_string()), ("beta", "2".to_string())]);
        assert_eq!(output, "alpha: 1\nbeta: 2\n");
    }

    #[test]
    fn mock_bridge_can_be_loaded_directly() {
        let bridge_path = Path::new(env!("FSDB_DEMO_MOCK_BRIDGE_PATH"));
        let summary = probe_with_bridge_path(Path::new("fixture.fsdb"), bridge_path).unwrap();
        assert_eq!(summary.bridge_kind, "mock");
        assert_eq!(summary.signal_count, 7);
        assert_eq!(summary.end_time_raw, 4242);
        assert_eq!(summary.scale_unit, "1ps");
        assert!(summary.message.contains("fixture.fsdb"));
    }
}
