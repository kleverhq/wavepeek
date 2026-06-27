mod cli;
#[doc(hidden)]
pub mod contract;
mod debug_trace;
mod diagnostic;
mod docs;
mod engine;
mod error;
mod output;
mod output_mode;
mod schema_contract;
mod waveform;

pub mod expr;

pub use crate::error::WavepeekError;

pub fn run_cli() -> Result<(), crate::error::WavepeekError> {
    cli::run()
}
