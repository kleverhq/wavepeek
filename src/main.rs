use std::process::ExitCode;

fn main() -> ExitCode {
    match wavepeek::run_cli() {
        Ok(()) => ExitCode::SUCCESS,
        Err(wavepeek::WavepeekError::BrokenPipe) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::from(error.exit_code())
        }
    }
}
