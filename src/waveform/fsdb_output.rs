//! Keep the native SDK's startup messages out of the CLI output streams.

use std::ffi::c_void;
use std::fs::File;
use std::io::{self, Write};
use std::os::fd::{AsFd, AsRawFd, OwnedFd};
use std::sync::{Mutex, MutexGuard};

use crate::error::WavepeekError;

static OUTPUT_LOCK: Mutex<()> = Mutex::new(());

unsafe extern "C" {
    fn dup2(old: i32, new: i32) -> i32;
    fn fflush(stream: *mut c_void) -> i32;
}

struct QuietOutput {
    stdout: OwnedFd,
    stderr: OwnedFd,
    _lock: MutexGuard<'static, ()>,
}

impl QuietOutput {
    fn new() -> io::Result<Self> {
        let lock = OUTPUT_LOCK
            .lock()
            .map_err(|_| io::Error::other("FSDB output lock poisoned"))?;
        io::stdout().flush()?;
        io::stderr().flush()?;
        // SAFETY: a null FILE pointer flushes all C output streams.
        unsafe {
            fflush(std::ptr::null_mut());
        }
        let null = File::options().write(true).open("/dev/null")?;
        let guard = Self {
            stdout: io::stdout().as_fd().try_clone_to_owned()?,
            stderr: io::stderr().as_fd().try_clone_to_owned()?,
            _lock: lock,
        };
        redirect(null.as_raw_fd(), 1)?;
        redirect(null.as_raw_fd(), 2)?;
        Ok(guard)
    }
}

fn redirect(source: i32, target: i32) -> io::Result<()> {
    loop {
        // SAFETY: source is an owned, live descriptor; target is a standard output descriptor.
        if unsafe { dup2(source, target) } >= 0 {
            return Ok(());
        }
        let error = io::Error::last_os_error();
        if error.kind() != io::ErrorKind::Interrupted {
            return Err(error);
        }
    }
}

impl Drop for QuietOutput {
    fn drop(&mut self) {
        // SAFETY: flush the SDK's buffered messages before restoring the descriptors.
        unsafe {
            fflush(std::ptr::null_mut());
        }
        let _ = redirect(self.stdout.as_raw_fd(), 1);
        let _ = redirect(self.stderr.as_raw_fd(), 2);
    }
}

pub(super) fn quiet<T>(run: impl FnOnce() -> T) -> Result<T, WavepeekError> {
    let _output = QuietOutput::new().map_err(|error| {
        WavepeekError::File(format!("cannot isolate FSDB reader output: {error}"))
    })?;
    Ok(run())
}
