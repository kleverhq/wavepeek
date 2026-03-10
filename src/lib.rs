mod cli;
mod engine;
mod error;
mod output;
mod schema_contract;
mod waveform;

pub mod expr;

pub use crate::error::WavepeekError;

pub fn run_cli() -> Result<(), crate::error::WavepeekError> {
    cli::run()
}
