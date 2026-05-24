#![allow(dead_code)]

use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int, c_uint, c_void};
use std::os::unix::ffi::OsStrExt;
use std::path::Path;
use std::ptr::{self, NonNull};

use crate::error::WavepeekError;

use super::fsdb_hierarchy::{
    FsdbHierarchyBuilder, FsdbHierarchyIndex, RawDatatypeKind, RawDatatypeRecord, RawScopeKind,
    RawScopeRecord, RawSignalKind, RawSignalRecord,
};
const WP_FSDB_STATUS_OK: c_uint = 0;

#[derive(Debug)]
pub(super) struct FsdbReader {
    raw: NonNull<ffi::wp_fsdb_reader>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct FsdbNativeMetadata {
    pub(super) scale_unit: String,
    pub(super) time_start_raw: u64,
    pub(super) time_end_raw: u64,
    pub(super) xtag_type: u32,
}

impl FsdbReader {
    pub(super) fn open(path: &Path) -> Result<Self, WavepeekError> {
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

    pub(super) fn metadata(&self) -> Result<FsdbNativeMetadata, WavepeekError> {
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

    pub(super) fn read_hierarchy(&self) -> Result<FsdbHierarchyIndex, WavepeekError> {
        let mut context = HierarchyCallbackContext {
            builder: FsdbHierarchyBuilder::new(),
            error: None,
            panicked: false,
        };
        let mut error_message = ptr::null_mut();
        let status = unsafe {
            ffi::wp_fsdb_read_scope_var_tree(
                self.raw.as_ptr(),
                Some(hierarchy_callback),
                (&mut context as *mut HierarchyCallbackContext).cast::<c_void>(),
                &mut error_message,
            )
        };

        if let Some(error) = context.error.take() {
            unsafe { ffi::wp_fsdb_free_error(error_message) };
            return Err(error);
        }
        if context.panicked {
            unsafe { ffi::wp_fsdb_free_error(error_message) };
            return Err(WavepeekError::Internal(
                "FSDB hierarchy callback panicked".to_string(),
            ));
        }
        if status != WP_FSDB_STATUS_OK {
            return Err(native_error(error_message));
        }

        Ok(context.builder.finish())
    }
}

impl Drop for FsdbReader {
    fn drop(&mut self) {
        unsafe { ffi::wp_fsdb_close(self.raw.as_ptr()) };
    }
}

pub(super) fn probe(path: &Path) -> Result<bool, WavepeekError> {
    let path = c_path(path)?;
    let mut is_fsdb: c_int = 0;
    let mut error_message = ptr::null_mut();
    let status = unsafe { ffi::wp_fsdb_probe(path.as_ptr(), &mut is_fsdb, &mut error_message) };
    if status != WP_FSDB_STATUS_OK {
        return Err(native_error(error_message));
    }
    Ok(is_fsdb != 0)
}

struct HierarchyCallbackContext {
    builder: FsdbHierarchyBuilder,
    error: Option<WavepeekError>,
    panicked: bool,
}

unsafe extern "C" fn hierarchy_callback(
    event: c_uint,
    scope: *const ffi::wp_fsdb_scope_record,
    signal: *const ffi::wp_fsdb_signal_record,
    datatype: *const ffi::wp_fsdb_datatype_record,
    user: *mut c_void,
) -> c_int {
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| unsafe {
        let Some(context) = user.cast::<HierarchyCallbackContext>().as_mut() else {
            return Err(WavepeekError::Internal(
                "FSDB hierarchy callback received null user data".to_string(),
            ));
        };
        if context.error.is_some() || context.panicked {
            return Err(WavepeekError::Internal(
                "FSDB hierarchy callback called after failure".to_string(),
            ));
        }
        handle_hierarchy_event(context, event, scope, signal, datatype)
    }));

    match result {
        Ok(Ok(())) => 0,
        Ok(Err(error)) => {
            unsafe {
                if let Some(context) = user.cast::<HierarchyCallbackContext>().as_mut()
                    && context.error.is_none()
                {
                    context.error = Some(error);
                }
            }
            1
        }
        Err(_) => {
            unsafe {
                if let Some(context) = user.cast::<HierarchyCallbackContext>().as_mut() {
                    context.panicked = true;
                }
            }
            1
        }
    }
}

unsafe fn handle_hierarchy_event(
    context: &mut HierarchyCallbackContext,
    event: c_uint,
    scope: *const ffi::wp_fsdb_scope_record,
    signal: *const ffi::wp_fsdb_signal_record,
    datatype: *const ffi::wp_fsdb_datatype_record,
) -> Result<(), WavepeekError> {
    match event {
        ffi::WP_FSDB_TREE_EVENT_BEGIN_TREE => context.builder.begin_tree(),
        ffi::WP_FSDB_TREE_EVENT_SCOPE => {
            let scope = unsafe { scope.as_ref() }.ok_or_else(|| {
                WavepeekError::File(
                    "FSDB Reader emitted a scope event without scope data".to_string(),
                )
            })?;
            context.builder.scope(RawScopeRecord {
                name: unsafe { borrowed_c_string(scope.name)? },
                kind: raw_scope_kind(scope.kind),
                hidden: scope.hidden != 0,
            })?;
        }
        ffi::WP_FSDB_TREE_EVENT_SIGNAL => {
            let signal = unsafe { signal.as_ref() }.ok_or_else(|| {
                WavepeekError::File(
                    "FSDB Reader emitted a signal event without signal data".to_string(),
                )
            })?;
            let datatype_id = if signal.has_datatype_id != 0 {
                Some(u16::try_from(signal.datatype_id).map_err(|_| {
                    WavepeekError::File("FSDB datatype id exceeds supported range".to_string())
                })?)
            } else {
                None
            };
            let (left, right) = if signal.has_bit_range != 0 {
                (Some(signal.left), Some(signal.right))
            } else {
                (None, None)
            };
            context.builder.signal(RawSignalRecord {
                idcode: signal.idcode,
                name: unsafe { borrowed_c_string(signal.name)? },
                kind: raw_signal_kind(signal.kind),
                left,
                right,
                datatype_id,
            })?;
        }
        ffi::WP_FSDB_TREE_EVENT_UPSCOPE => context.builder.upscope()?,
        ffi::WP_FSDB_TREE_EVENT_END_TREE | ffi::WP_FSDB_TREE_EVENT_END_ALL_TREE => {
            context.builder.end_tree();
        }
        ffi::WP_FSDB_TREE_EVENT_DATATYPE => {
            let datatype = unsafe { datatype.as_ref() }.ok_or_else(|| {
                WavepeekError::File(
                    "FSDB Reader emitted a datatype event without datatype data".to_string(),
                )
            })?;
            context.builder.datatype(RawDatatypeRecord {
                idcode: u16::try_from(datatype.idcode).map_err(|_| {
                    WavepeekError::File("FSDB datatype id exceeds supported range".to_string())
                })?,
                kind: raw_datatype_kind(datatype.kind),
            })?;
        }
        _ => {}
    }
    Ok(())
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

unsafe fn borrowed_c_string(value: *const c_char) -> Result<String, WavepeekError> {
    if value.is_null() {
        return Err(WavepeekError::File(
            "FSDB Reader emitted a null hierarchy string".to_string(),
        ));
    }
    Ok(unsafe { CStr::from_ptr(value) }
        .to_string_lossy()
        .into_owned())
}

fn raw_scope_kind(kind: c_uint) -> RawScopeKind {
    match kind {
        ffi::WP_FSDB_SCOPE_KIND_MODULE => RawScopeKind::Module,
        ffi::WP_FSDB_SCOPE_KIND_TASK => RawScopeKind::Task,
        ffi::WP_FSDB_SCOPE_KIND_FUNCTION => RawScopeKind::Function,
        ffi::WP_FSDB_SCOPE_KIND_BEGIN => RawScopeKind::Begin,
        ffi::WP_FSDB_SCOPE_KIND_FORK => RawScopeKind::Fork,
        ffi::WP_FSDB_SCOPE_KIND_GENERATE => RawScopeKind::Generate,
        ffi::WP_FSDB_SCOPE_KIND_STRUCT => RawScopeKind::Struct,
        ffi::WP_FSDB_SCOPE_KIND_UNION => RawScopeKind::Union,
        ffi::WP_FSDB_SCOPE_KIND_CLASS => RawScopeKind::Class,
        ffi::WP_FSDB_SCOPE_KIND_INTERFACE => RawScopeKind::Interface,
        ffi::WP_FSDB_SCOPE_KIND_PACKAGE => RawScopeKind::Package,
        ffi::WP_FSDB_SCOPE_KIND_PROGRAM => RawScopeKind::Program,
        _ => RawScopeKind::Unknown,
    }
}

fn raw_signal_kind(kind: c_uint) -> RawSignalKind {
    match kind {
        ffi::WP_FSDB_SIGNAL_KIND_EVENT => RawSignalKind::Event,
        ffi::WP_FSDB_SIGNAL_KIND_INTEGER => RawSignalKind::Integer,
        ffi::WP_FSDB_SIGNAL_KIND_PARAMETER => RawSignalKind::Parameter,
        ffi::WP_FSDB_SIGNAL_KIND_REAL => RawSignalKind::Real,
        ffi::WP_FSDB_SIGNAL_KIND_REG => RawSignalKind::Reg,
        ffi::WP_FSDB_SIGNAL_KIND_SUPPLY0 => RawSignalKind::Supply0,
        ffi::WP_FSDB_SIGNAL_KIND_SUPPLY1 => RawSignalKind::Supply1,
        ffi::WP_FSDB_SIGNAL_KIND_TIME => RawSignalKind::Time,
        ffi::WP_FSDB_SIGNAL_KIND_TRI => RawSignalKind::Tri,
        ffi::WP_FSDB_SIGNAL_KIND_TRIAND => RawSignalKind::TriAnd,
        ffi::WP_FSDB_SIGNAL_KIND_TRIOR => RawSignalKind::TriOr,
        ffi::WP_FSDB_SIGNAL_KIND_TRIREG => RawSignalKind::TriReg,
        ffi::WP_FSDB_SIGNAL_KIND_TRI0 => RawSignalKind::Tri0,
        ffi::WP_FSDB_SIGNAL_KIND_TRI1 => RawSignalKind::Tri1,
        ffi::WP_FSDB_SIGNAL_KIND_WAND => RawSignalKind::WAnd,
        ffi::WP_FSDB_SIGNAL_KIND_WIRE => RawSignalKind::Wire,
        ffi::WP_FSDB_SIGNAL_KIND_WOR => RawSignalKind::WOr,
        ffi::WP_FSDB_SIGNAL_KIND_STRING => RawSignalKind::String,
        ffi::WP_FSDB_SIGNAL_KIND_PORT => RawSignalKind::Port,
        ffi::WP_FSDB_SIGNAL_KIND_SPARSE_ARRAY => RawSignalKind::SparseArray,
        ffi::WP_FSDB_SIGNAL_KIND_REAL_TIME => RawSignalKind::RealTime,
        ffi::WP_FSDB_SIGNAL_KIND_REAL_PARAMETER => RawSignalKind::RealParameter,
        ffi::WP_FSDB_SIGNAL_KIND_BIT => RawSignalKind::Bit,
        ffi::WP_FSDB_SIGNAL_KIND_LOGIC => RawSignalKind::Logic,
        ffi::WP_FSDB_SIGNAL_KIND_INT => RawSignalKind::Int,
        ffi::WP_FSDB_SIGNAL_KIND_SHORT_INT => RawSignalKind::ShortInt,
        ffi::WP_FSDB_SIGNAL_KIND_LONG_INT => RawSignalKind::LongInt,
        ffi::WP_FSDB_SIGNAL_KIND_BYTE => RawSignalKind::Byte,
        ffi::WP_FSDB_SIGNAL_KIND_ENUM => RawSignalKind::Enum,
        ffi::WP_FSDB_SIGNAL_KIND_SHORT_REAL => RawSignalKind::ShortReal,
        ffi::WP_FSDB_SIGNAL_KIND_BOOLEAN => RawSignalKind::Boolean,
        ffi::WP_FSDB_SIGNAL_KIND_BIT_VECTOR => RawSignalKind::BitVector,
        _ => RawSignalKind::Unknown,
    }
}

fn raw_datatype_kind(kind: c_uint) -> RawDatatypeKind {
    match kind {
        ffi::WP_FSDB_DATATYPE_KIND_ENUM => RawDatatypeKind::Enum,
        ffi::WP_FSDB_DATATYPE_KIND_LOGIC => RawDatatypeKind::Logic,
        ffi::WP_FSDB_DATATYPE_KIND_BIT => RawDatatypeKind::Bit,
        ffi::WP_FSDB_DATATYPE_KIND_INT => RawDatatypeKind::Int,
        ffi::WP_FSDB_DATATYPE_KIND_UINT => RawDatatypeKind::UInt,
        ffi::WP_FSDB_DATATYPE_KIND_SHORT_INT => RawDatatypeKind::ShortInt,
        ffi::WP_FSDB_DATATYPE_KIND_SHORT_UINT => RawDatatypeKind::ShortUInt,
        ffi::WP_FSDB_DATATYPE_KIND_LONG_INT => RawDatatypeKind::LongInt,
        ffi::WP_FSDB_DATATYPE_KIND_LONG_UINT => RawDatatypeKind::LongUInt,
        ffi::WP_FSDB_DATATYPE_KIND_BYTE => RawDatatypeKind::Byte,
        ffi::WP_FSDB_DATATYPE_KIND_UBYTE => RawDatatypeKind::UByte,
        ffi::WP_FSDB_DATATYPE_KIND_REAL => RawDatatypeKind::Real,
        ffi::WP_FSDB_DATATYPE_KIND_SHORT_REAL => RawDatatypeKind::ShortReal,
        ffi::WP_FSDB_DATATYPE_KIND_TIME => RawDatatypeKind::Time,
        ffi::WP_FSDB_DATATYPE_KIND_STRING => RawDatatypeKind::String,
        ffi::WP_FSDB_DATATYPE_KIND_EVENT => RawDatatypeKind::Event,
        _ => RawDatatypeKind::Unknown,
    }
}

mod ffi {
    use std::os::raw::{c_char, c_int, c_uint, c_void};

    pub(super) const WP_FSDB_TREE_EVENT_BEGIN_TREE: c_uint = 0;
    pub(super) const WP_FSDB_TREE_EVENT_SCOPE: c_uint = 1;
    pub(super) const WP_FSDB_TREE_EVENT_SIGNAL: c_uint = 2;
    pub(super) const WP_FSDB_TREE_EVENT_UPSCOPE: c_uint = 3;
    pub(super) const WP_FSDB_TREE_EVENT_END_TREE: c_uint = 4;
    pub(super) const WP_FSDB_TREE_EVENT_END_ALL_TREE: c_uint = 5;
    pub(super) const WP_FSDB_TREE_EVENT_DATATYPE: c_uint = 6;

    pub(super) const WP_FSDB_SCOPE_KIND_MODULE: c_uint = 0;
    pub(super) const WP_FSDB_SCOPE_KIND_TASK: c_uint = 1;
    pub(super) const WP_FSDB_SCOPE_KIND_FUNCTION: c_uint = 2;
    pub(super) const WP_FSDB_SCOPE_KIND_BEGIN: c_uint = 3;
    pub(super) const WP_FSDB_SCOPE_KIND_FORK: c_uint = 4;
    pub(super) const WP_FSDB_SCOPE_KIND_GENERATE: c_uint = 5;
    pub(super) const WP_FSDB_SCOPE_KIND_STRUCT: c_uint = 6;
    pub(super) const WP_FSDB_SCOPE_KIND_UNION: c_uint = 7;
    pub(super) const WP_FSDB_SCOPE_KIND_CLASS: c_uint = 8;
    pub(super) const WP_FSDB_SCOPE_KIND_INTERFACE: c_uint = 9;
    pub(super) const WP_FSDB_SCOPE_KIND_PACKAGE: c_uint = 10;
    pub(super) const WP_FSDB_SCOPE_KIND_PROGRAM: c_uint = 11;

    pub(super) const WP_FSDB_SIGNAL_KIND_EVENT: c_uint = 0;
    pub(super) const WP_FSDB_SIGNAL_KIND_INTEGER: c_uint = 1;
    pub(super) const WP_FSDB_SIGNAL_KIND_PARAMETER: c_uint = 2;
    pub(super) const WP_FSDB_SIGNAL_KIND_REAL: c_uint = 3;
    pub(super) const WP_FSDB_SIGNAL_KIND_REG: c_uint = 4;
    pub(super) const WP_FSDB_SIGNAL_KIND_SUPPLY0: c_uint = 5;
    pub(super) const WP_FSDB_SIGNAL_KIND_SUPPLY1: c_uint = 6;
    pub(super) const WP_FSDB_SIGNAL_KIND_TIME: c_uint = 7;
    pub(super) const WP_FSDB_SIGNAL_KIND_TRI: c_uint = 8;
    pub(super) const WP_FSDB_SIGNAL_KIND_TRIAND: c_uint = 9;
    pub(super) const WP_FSDB_SIGNAL_KIND_TRIOR: c_uint = 10;
    pub(super) const WP_FSDB_SIGNAL_KIND_TRIREG: c_uint = 11;
    pub(super) const WP_FSDB_SIGNAL_KIND_TRI0: c_uint = 12;
    pub(super) const WP_FSDB_SIGNAL_KIND_TRI1: c_uint = 13;
    pub(super) const WP_FSDB_SIGNAL_KIND_WAND: c_uint = 14;
    pub(super) const WP_FSDB_SIGNAL_KIND_WIRE: c_uint = 15;
    pub(super) const WP_FSDB_SIGNAL_KIND_WOR: c_uint = 16;
    pub(super) const WP_FSDB_SIGNAL_KIND_STRING: c_uint = 17;
    pub(super) const WP_FSDB_SIGNAL_KIND_PORT: c_uint = 18;
    pub(super) const WP_FSDB_SIGNAL_KIND_SPARSE_ARRAY: c_uint = 19;
    pub(super) const WP_FSDB_SIGNAL_KIND_REAL_TIME: c_uint = 20;
    pub(super) const WP_FSDB_SIGNAL_KIND_REAL_PARAMETER: c_uint = 21;
    pub(super) const WP_FSDB_SIGNAL_KIND_BIT: c_uint = 22;
    pub(super) const WP_FSDB_SIGNAL_KIND_LOGIC: c_uint = 23;
    pub(super) const WP_FSDB_SIGNAL_KIND_INT: c_uint = 24;
    pub(super) const WP_FSDB_SIGNAL_KIND_SHORT_INT: c_uint = 25;
    pub(super) const WP_FSDB_SIGNAL_KIND_LONG_INT: c_uint = 26;
    pub(super) const WP_FSDB_SIGNAL_KIND_BYTE: c_uint = 27;
    pub(super) const WP_FSDB_SIGNAL_KIND_ENUM: c_uint = 28;
    pub(super) const WP_FSDB_SIGNAL_KIND_SHORT_REAL: c_uint = 29;
    pub(super) const WP_FSDB_SIGNAL_KIND_BOOLEAN: c_uint = 30;
    pub(super) const WP_FSDB_SIGNAL_KIND_BIT_VECTOR: c_uint = 31;

    pub(super) const WP_FSDB_DATATYPE_KIND_ENUM: c_uint = 0;
    pub(super) const WP_FSDB_DATATYPE_KIND_LOGIC: c_uint = 1;
    pub(super) const WP_FSDB_DATATYPE_KIND_BIT: c_uint = 2;
    pub(super) const WP_FSDB_DATATYPE_KIND_INT: c_uint = 3;
    pub(super) const WP_FSDB_DATATYPE_KIND_UINT: c_uint = 4;
    pub(super) const WP_FSDB_DATATYPE_KIND_SHORT_INT: c_uint = 5;
    pub(super) const WP_FSDB_DATATYPE_KIND_SHORT_UINT: c_uint = 6;
    pub(super) const WP_FSDB_DATATYPE_KIND_LONG_INT: c_uint = 7;
    pub(super) const WP_FSDB_DATATYPE_KIND_LONG_UINT: c_uint = 8;
    pub(super) const WP_FSDB_DATATYPE_KIND_BYTE: c_uint = 9;
    pub(super) const WP_FSDB_DATATYPE_KIND_UBYTE: c_uint = 10;
    pub(super) const WP_FSDB_DATATYPE_KIND_REAL: c_uint = 11;
    pub(super) const WP_FSDB_DATATYPE_KIND_SHORT_REAL: c_uint = 12;
    pub(super) const WP_FSDB_DATATYPE_KIND_TIME: c_uint = 13;
    pub(super) const WP_FSDB_DATATYPE_KIND_STRING: c_uint = 14;
    pub(super) const WP_FSDB_DATATYPE_KIND_EVENT: c_uint = 15;

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

    #[repr(C)]
    pub(super) struct wp_fsdb_scope_record {
        pub(super) name: *const c_char,
        pub(super) kind: c_uint,
        pub(super) hidden: c_int,
    }

    #[repr(C)]
    pub(super) struct wp_fsdb_signal_record {
        pub(super) name: *const c_char,
        pub(super) idcode: u64,
        pub(super) has_bit_range: c_int,
        pub(super) left: i32,
        pub(super) right: i32,
        pub(super) has_datatype_id: c_int,
        pub(super) datatype_id: c_uint,
        pub(super) kind: c_uint,
    }

    #[repr(C)]
    pub(super) struct wp_fsdb_datatype_record {
        pub(super) idcode: c_uint,
        pub(super) kind: c_uint,
    }

    pub(super) type WpFsdbTreeCallback = Option<
        unsafe extern "C" fn(
            c_uint,
            *const wp_fsdb_scope_record,
            *const wp_fsdb_signal_record,
            *const wp_fsdb_datatype_record,
            *mut c_void,
        ) -> c_int,
    >;

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
        pub(super) fn wp_fsdb_read_scope_var_tree(
            reader: *mut wp_fsdb_reader,
            callback: WpFsdbTreeCallback,
            user: *mut c_void,
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
    use crate::waveform::{STABLE_SCOPE_KIND_ALIASES, STABLE_SIGNAL_KIND_ALIASES};

    #[test]
    fn fsdb_reader_metadata_smoke() {
        let path = cpu_fsdb_path();

        assert!(probe(&path).expect("FSDB probe failed"));
        let reader = FsdbReader::open(&path).expect("FSDB open failed");
        let metadata = reader.metadata().expect("FSDB metadata read failed");

        assert!(!metadata.scale_unit.is_empty());
        assert!(metadata.time_end_raw >= metadata.time_start_raw);
    }

    #[test]
    fn fsdb_reader_hierarchy_smoke() {
        let path = cpu_fsdb_path();
        let reader = FsdbReader::open(&path).expect("FSDB open failed");
        let first = reader
            .read_hierarchy()
            .expect("FSDB hierarchy read should succeed");
        let second = reader
            .read_hierarchy()
            .expect("FSDB hierarchy reread should succeed");

        let first_scopes = first.scopes_depth_first(None);
        let second_scopes = second.scopes_depth_first(None);
        assert!(
            !first_scopes.is_empty(),
            "bundled FSDB should expose scopes"
        );
        assert_eq!(first_scopes, second_scopes);
        for scope in &first_scopes {
            assert!(!scope.path.is_empty());
            assert!(!scope.path.contains('/'));
            assert!(STABLE_SCOPE_KIND_ALIASES.contains(&scope.kind.as_str()));
        }

        let mut non_empty_signal_listing = None;
        for scope in &first_scopes {
            let signals = first
                .signals_in_scope_recursive(scope.path.as_str(), None)
                .expect("scope from listing should be queryable");
            if !signals.is_empty() {
                non_empty_signal_listing = Some(signals);
                break;
            }
        }
        let signals = non_empty_signal_listing.expect("bundled FSDB should expose signals");
        for signal in signals {
            assert!(!signal.path.is_empty());
            assert!(!signal.path.contains('/'));
            assert!(STABLE_SIGNAL_KIND_ALIASES.contains(&signal.kind.as_str()));
            if let Some(width) = signal.width {
                assert!(width > 0);
            }
        }
    }

    fn cpu_fsdb_path() -> PathBuf {
        PathBuf::from(
            std::env::var_os("VERDI_HOME").expect("VERDI_HOME must be set for FSDB smoke tests"),
        )
        .join("share")
        .join("VIA")
        .join("demo")
        .join("waveform")
        .join("cpu.fsdb")
    }
}
