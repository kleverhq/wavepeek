#![allow(dead_code)]

use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int, c_uint};
use std::os::unix::ffi::OsStrExt;
use std::path::Path;
use std::ptr::{self, NonNull};

use crate::error::WavepeekError;

const WP_FSDB_STATUS_OK: c_uint = 0;

#[derive(Debug)]
struct FsdbReader {
    raw: NonNull<ffi::wp_fsdb_reader>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct FsdbNativeMetadata {
    scale_unit: String,
    time_start_raw: u64,
    time_end_raw: u64,
    xtag_type: u32,
}

impl FsdbReader {
    fn open(path: &Path) -> Result<Self, WavepeekError> {
        let path = c_path(path)?;
        let mut raw = ptr::null_mut();
        let mut error_message = ptr::null_mut();
        let status = unsafe { ffi::wp_fsdb_open(path.as_ptr(), &mut raw, &mut error_message) };
        if status != WP_FSDB_STATUS_OK {
            return Err(native_error(error_message));
        }

        let raw = NonNull::new(raw).ok_or_else(|| {
            WavepeekError::File("FSDB Reader returned a null reader handle".to_string())
        })?;
        Ok(Self { raw })
    }

    fn metadata(&self) -> Result<FsdbNativeMetadata, WavepeekError> {
        let mut raw_metadata = ffi::wp_fsdb_metadata {
            scale_unit: ptr::null_mut(),
            time_start_raw: 0,
            time_end_raw: 0,
            xtag_type: 0,
        };
        let mut error_message = ptr::null_mut();
        let status = unsafe {
            ffi::wp_fsdb_read_metadata(self.raw.as_ptr(), &mut raw_metadata, &mut error_message)
        };
        if status != WP_FSDB_STATUS_OK {
            return Err(native_error(error_message));
        }

        let scale_unit = unsafe { take_native_string(raw_metadata.scale_unit)? };
        Ok(FsdbNativeMetadata {
            scale_unit,
            time_start_raw: raw_metadata.time_start_raw,
            time_end_raw: raw_metadata.time_end_raw,
            xtag_type: raw_metadata.xtag_type,
        })
    }
}

impl Drop for FsdbReader {
    fn drop(&mut self) {
        unsafe { ffi::wp_fsdb_close(self.raw.as_ptr()) };
    }
}

fn probe(path: &Path) -> Result<bool, WavepeekError> {
    let path = c_path(path)?;
    let mut is_fsdb: c_int = 0;
    let mut error_message = ptr::null_mut();
    let status = unsafe { ffi::wp_fsdb_probe(path.as_ptr(), &mut is_fsdb, &mut error_message) };
    if status != WP_FSDB_STATUS_OK {
        return Err(native_error(error_message));
    }
    Ok(is_fsdb != 0)
}

fn c_path(path: &Path) -> Result<CString, WavepeekError> {
    CString::new(path.as_os_str().as_bytes()).map_err(|_| {
        WavepeekError::File(format!(
            "FSDB Reader path contains an interior NUL byte: '{}'",
            path.display()
        ))
    })
}

fn native_error(error_message: *mut c_char) -> WavepeekError {
    let message = unsafe { take_error_string(error_message) };
    if message.starts_with("FSDB Reader") {
        WavepeekError::File(message)
    } else {
        WavepeekError::File(format!("FSDB Reader: {message}"))
    }
}

unsafe fn take_error_string(error_message: *mut c_char) -> String {
    if error_message.is_null() {
        return "FSDB Reader native call failed without an error message".to_string();
    }

    let message = unsafe { CStr::from_ptr(error_message) }
        .to_string_lossy()
        .into_owned();
    unsafe { ffi::wp_fsdb_free_error(error_message) };
    message
}

unsafe fn take_native_string(value: *mut c_char) -> Result<String, WavepeekError> {
    if value.is_null() {
        return Err(WavepeekError::File(
            "FSDB Reader returned a null string".to_string(),
        ));
    }

    let result = unsafe { CStr::from_ptr(value) }
        .to_string_lossy()
        .into_owned();
    unsafe { ffi::wp_fsdb_free_string(value) };
    Ok(result)
}

mod ffi {
    use std::os::raw::{c_char, c_int, c_uint};

    #[repr(C)]
    pub(super) struct wp_fsdb_reader {
        _private: [u8; 0],
    }

    #[repr(C)]
    pub(super) struct wp_fsdb_metadata {
        pub(super) scale_unit: *mut c_char,
        pub(super) time_start_raw: u64,
        pub(super) time_end_raw: u64,
        pub(super) xtag_type: u32,
    }

    unsafe extern "C" {
        pub(super) fn wp_fsdb_probe(
            path: *const c_char,
            is_fsdb: *mut c_int,
            error_message: *mut *mut c_char,
        ) -> c_uint;
        pub(super) fn wp_fsdb_open(
            path: *const c_char,
            out: *mut *mut wp_fsdb_reader,
            error_message: *mut *mut c_char,
        ) -> c_uint;
        pub(super) fn wp_fsdb_close(reader: *mut wp_fsdb_reader);
        pub(super) fn wp_fsdb_read_metadata(
            reader: *mut wp_fsdb_reader,
            out: *mut wp_fsdb_metadata,
            error_message: *mut *mut c_char,
        ) -> c_uint;
        pub(super) fn wp_fsdb_free_string(value: *mut c_char);
        pub(super) fn wp_fsdb_free_error(value: *mut c_char);
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{FsdbReader, probe};

    #[test]
    fn fsdb_reader_metadata_smoke() {
        let path = PathBuf::from(
            std::env::var_os("VERDI_HOME").expect("VERDI_HOME must be set for FSDB smoke tests"),
        )
        .join("share")
        .join("VIA")
        .join("demo")
        .join("waveform")
        .join("cpu.fsdb");

        assert!(probe(&path).expect("FSDB probe failed"));
        let reader = FsdbReader::open(&path).expect("FSDB open failed");
        let metadata = reader.metadata().expect("FSDB metadata read failed");

        assert!(!metadata.scale_unit.is_empty());
        assert!(metadata.time_end_raw >= metadata.time_start_raw);
    }
}
