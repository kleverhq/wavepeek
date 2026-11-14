#[cfg(any(test, target_arch = "wasm32"))]
mod browser;
mod cli;
#[doc(hidden)]
pub mod contract;
mod debug_trace;
mod diagnostic;
mod engine;
mod error;
mod logic;
mod output;
mod output_mode;
mod skill;
mod waveform;

pub mod expr;

pub use crate::error::WavepeekError;

pub fn run_cli() -> Result<(), WavepeekError> {
    cli::run(false).map_err(|failure| failure.error)
}

#[doc(hidden)]
pub fn main_exit_code() -> std::process::ExitCode {
    match cli::run(true) {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(failure) => {
            if !failure.reported {
                eprintln!("{}", failure.error);
            }
            std::process::ExitCode::from(failure.error.exit_code())
        }
    }
}
