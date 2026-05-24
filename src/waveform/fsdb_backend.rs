//! FSDB-backed waveform adapter implementation.
//!
//! The implementation is filled in by the FSDB hierarchy command slice.

#![allow(dead_code)]

use std::path::Path;

use crate::error::WavepeekError;

#[derive(Debug)]
pub(super) struct FsdbBackend;

impl FsdbBackend {
    pub(super) fn open(_path: &Path) -> Result<Self, WavepeekError> {
        Err(WavepeekError::Unimplemented("FSDB backend wiring"))
    }
}
