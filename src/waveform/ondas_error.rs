//! Classify Ondas failures before adding WavePeek operation and path context.

use std::path::Path;

use ondas::{Error, Format};

use crate::error::WavepeekError;

pub(super) fn open_error(path: &Path, error: Error) -> WavepeekError {
    let operation = match &error {
        Error::Malformed { .. } | Error::UnknownFormat => "parse",
        _ => "open",
    };
    read_error(path, operation, error)
}

pub(super) fn read_error(path: &Path, operation: &str, error: Error) -> WavepeekError {
    let category: fn(String) -> WavepeekError = match &error {
        Error::Io(_)
        | Error::UnknownFormat
        | Error::NoBackend { .. }
        | Error::BackendUnavailable { .. }
        | Error::UnsupportedInput { .. }
        | Error::UnsupportedFeature { .. }
        | Error::Malformed { .. }
        | Error::Backend { .. } => WavepeekError::File,
        Error::UnsupportedSignal { .. } | Error::NonAsciiString { .. } => WavepeekError::Signal,
        Error::UnknownBackend { .. }
        | Error::BackendDoesNotSupport { .. }
        | Error::InvalidComposition(_)
        | Error::InvalidSelectionIndex { .. }
        | Error::InvalidQueryTime { .. }
        | Error::TraceCoverage { .. }
        | Error::InvalidSignal { .. } => WavepeekError::Internal,
        // A new upstream variant needs an explicit policy before it can be
        // attributed to the user's file or signal.
        _ => WavepeekError::Internal,
    };
    let hint = match &error {
        Error::NoBackend {
            format: Format::Fsdb,
        } => "; FSDB input requires Linux x86_64, --features fsdb, and a licensed VERDI_HOME",
        Error::NoBackend {
            format: Format::Shm,
        } => "; SHM input requires Linux x86_64, --features shm, and a compatible XCELIUM_HOME",
        _ => "",
    };
    category(format!(
        "cannot {operation} '{}': {error}{hint}",
        path.display()
    ))
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use ondas::{CompositionError, Error, Format, InputKind, Time, TimeRange};

    use super::{open_error, read_error};
    use crate::error::WavepeekError;

    #[test]
    fn file_causes_are_distinct_and_sdk_text_does_not_choose_the_category() {
        let cases = [
            (
                Error::Io(std::io::Error::from(std::io::ErrorKind::NotFound)),
                "I/O error",
            ),
            (Error::UnknownFormat, "unknown waveform format"),
            (
                Error::NoBackend {
                    format: Format::Fsdb,
                },
                "no available backend for Fsdb",
            ),
            (
                Error::NoBackend {
                    format: Format::Shm,
                },
                "no available backend for Shm",
            ),
            (
                Error::NoBackend {
                    format: Format::Wlf,
                },
                "no available backend for Wlf",
            ),
            (
                Error::BackendUnavailable {
                    backend: "reader".into(),
                    message: "malformed; license missing".into(),
                },
                "is unavailable",
            ),
            (
                Error::Malformed {
                    format: Format::Vcd,
                    backend: "vcd-native".into(),
                    message: "FSDB input requires fake runtime".into(),
                },
                "malformed Vcd",
            ),
            (
                Error::UnsupportedFeature {
                    format: Format::Fsdb,
                    backend: "reader".into(),
                    operation: "open",
                    message: "newer source".into(),
                },
                "unsupported Fsdb feature",
            ),
            (
                Error::UnsupportedInput {
                    backend: "reader".into(),
                    input: InputKind::Bytes,
                },
                "does not support Bytes input",
            ),
            (
                Error::Backend {
                    backend: "reader".into(),
                    operation: "scan",
                    message: "unknown native failure".into(),
                },
                "failed during scan",
            ),
        ];
        for (error, cause) in cases {
            let result = read_error(Path::new("input.fsdb"), "read samples from", error);
            assert!(matches!(result, WavepeekError::File(_)), "{result}");
            assert_eq!(result.fatal_code(), Some("WPK-F0002"));
            assert_eq!(result.exit_code(), 2);
            let message = result.message().unwrap();
            assert!(message.starts_with("cannot read samples from 'input.fsdb': "));
            assert!(message.contains(cause), "{message}");
        }
    }

    #[test]
    fn value_support_failures_use_signal_category_even_during_open() {
        let waveform = ondas::open_bytes(
            "test.vcd",
            b"$var wire 1 ! flag $end $enddefinitions $end #0 0!"
                .as_slice()
                .into(),
        )
        .unwrap();
        let signal = waveform.hierarchy().signal("flag").unwrap();
        for error in [
            Error::UnsupportedSignal { signal },
            Error::NonAsciiString {
                format: Format::Vcd,
                backend: "vcd-native".into(),
                time: Time::from_ticks(5),
            },
        ] {
            let result = open_error(Path::new("input.fsdb"), error);
            assert!(matches!(result, WavepeekError::Signal(_)), "{result}");
            assert_eq!(result.fatal_code(), Some("WPK-F0004"));
            assert_eq!(result.exit_code(), 1);
        }
    }

    #[test]
    fn api_misuse_is_internal_in_open_and_read_paths() {
        let waveform = ondas::open_bytes(
            "test.vcd",
            b"$var wire 1 ! flag $end $enddefinitions $end #0 0!"
                .as_slice()
                .into(),
        )
        .unwrap();
        let signal = waveform.hierarchy().signal("flag").unwrap();
        let errors = || {
            [
                Error::UnknownBackend {
                    backend: "invalid".into(),
                },
                Error::BackendDoesNotSupport {
                    backend: "reader".into(),
                    format: Format::Vcd,
                },
                Error::InvalidComposition(CompositionError::Empty),
                Error::InvalidSelectionIndex { index: 1, len: 1 },
                Error::InvalidQueryTime {
                    requested: Time::ZERO,
                    current: Time::from_ticks(5),
                },
                Error::TraceCoverage {
                    requested: Time::ZERO,
                    range: TimeRange::from(Time::from_ticks(5)),
                },
                Error::InvalidSignal { signal },
            ]
        };
        for opening in [true, false] {
            for error in errors() {
                let path = Path::new("input.vcd");
                let result = if opening {
                    open_error(path, error)
                } else {
                    read_error(path, "read traces from", error)
                };
                assert!(matches!(result, WavepeekError::Internal(_)), "{result}");
                assert_eq!(result.fatal_code(), Some("WPK-F0006"));
                assert_eq!(result.exit_code(), 1);
                assert!(!result.message().unwrap().contains("cannot parse"));
            }
        }
    }
}
