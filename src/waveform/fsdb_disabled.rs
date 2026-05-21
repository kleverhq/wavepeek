//! Default-build handling for FSDB-looking inputs.

use std::path::Path;

use crate::error::WavepeekError;

pub(crate) const FSDB_DISABLED_MESSAGE: &str = "FSDB input requires a wavepeek binary built with FSDB support; reinstall with --features fsdb and provide a licensed VERDI_HOME";

pub(crate) fn looks_like_fsdb_path(path: &Path) -> bool {
    let Some(file_name) = path.file_name() else {
        return false;
    };
    let file_name = file_name.to_string_lossy().to_lowercase();
    file_name.ends_with(".fsdb") || file_name.ends_with(".fsdb.gz")
}

pub(crate) fn disabled_support_error() -> WavepeekError {
    WavepeekError::File(FSDB_DISABLED_MESSAGE.to_string())
}

pub(crate) fn should_report_disabled_support(path: &Path, error: &WavepeekError) -> bool {
    if !looks_like_fsdb_path(path) {
        return false;
    }

    let WavepeekError::File(message) = error else {
        return false;
    };

    // Wellen open errors currently preserve the public file category as plain text.
    // Keep this narrow so missing files, permission failures, directories, and other
    // read/open failures remain ordinary `cannot open` errors.
    let parse_prefix = format!("cannot parse '{}': ", path.display());
    message.starts_with(&parse_prefix)
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use crate::error::WavepeekError;

    use super::{
        FSDB_DISABLED_MESSAGE, disabled_support_error, looks_like_fsdb_path,
        should_report_disabled_support,
    };

    #[test]
    fn suffix_detection_accepts_fsdb_names_case_insensitively() {
        for path in [
            "dump.fsdb",
            "dump.FSDB",
            "dump.fsdb.gz",
            "dump.FsDb.Gz",
            "/tmp/archive.dump.fsdb.gz",
        ] {
            assert!(looks_like_fsdb_path(Path::new(path)), "{path}");
        }
    }

    #[test]
    fn suffix_detection_rejects_near_misses_and_parent_directories() {
        for path in [
            "dump.fsdbx",
            "dump.fsdb.gz.tmp",
            "dump.fsdb/good.vcd",
            "fsdb",
            "",
        ] {
            assert!(!looks_like_fsdb_path(Path::new(path)), "{path}");
        }
    }

    #[test]
    fn disabled_error_preserves_file_category_and_message() {
        let error = disabled_support_error();

        assert_eq!(error.exit_code(), 2);
        assert_eq!(
            error.to_string(),
            format!("error: file: {FSDB_DISABLED_MESSAGE}")
        );
    }

    #[test]
    fn report_only_parse_failures_for_fsdb_looking_paths() {
        let path = Path::new("dump.fsdb");
        let parse_error = WavepeekError::File(
            "cannot parse 'dump.fsdb': unknown file format, only GHW, FST and VCD are supported"
                .to_string(),
        );
        let open_error =
            WavepeekError::File("cannot open 'dump.fsdb': No such file or directory".to_string());
        let signal_error = WavepeekError::Signal("signal missing".to_string());
        let unrelated_parse_error = WavepeekError::File(
            "cannot parse 'dump.notfsdb': unknown file format, only GHW, FST and VCD are supported"
                .to_string(),
        );

        assert!(should_report_disabled_support(path, &parse_error));
        assert!(!should_report_disabled_support(path, &open_error));
        assert!(!should_report_disabled_support(path, &signal_error));
        assert!(!should_report_disabled_support(
            Path::new("dump.notfsdb"),
            &unrelated_parse_error
        ));
    }
}
