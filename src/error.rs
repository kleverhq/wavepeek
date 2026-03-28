#![allow(dead_code)]

use thiserror::Error;

#[derive(Debug, Error)]
pub enum WavepeekError {
    #[error("error: args: {0}")]
    Args(String),
    #[error("error: file: {0}")]
    File(String),
    #[error("error: scope: {0}")]
    Scope(String),
    #[error("error: signal: {0}")]
    Signal(String),
    #[error("error: expr: {0}")]
    Expr(String),
    #[error("error: internal: {0}")]
    Internal(String),
    #[error("error: unimplemented: {0}")]
    Unimplemented(&'static str),
}

impl WavepeekError {
    pub const fn exit_code(&self) -> u8 {
        match self {
            Self::File(_) => 2,
            Self::Args(_)
            | Self::Scope(_)
            | Self::Signal(_)
            | Self::Expr(_)
            | Self::Internal(_)
            | Self::Unimplemented(_) => 1,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::WavepeekError;

    #[test]
    fn file_errors_use_exit_code_two() {
        let error = WavepeekError::File("cannot open dump.vcd".to_string());

        assert_eq!(error.exit_code(), 2);
        assert_eq!(error.to_string(), "error: file: cannot open dump.vcd");
    }

    #[test]
    fn scope_and_signal_errors_use_exit_code_one() {
        let scope = WavepeekError::Scope("scope 'top.cpu' not found".to_string());
        let signal = WavepeekError::Signal("signal 'top.cpu.clk' not found".to_string());

        assert_eq!(scope.exit_code(), 1);
        assert_eq!(signal.exit_code(), 1);
        assert_eq!(scope.to_string(), "error: scope: scope 'top.cpu' not found");
        assert_eq!(
            signal.to_string(),
            "error: signal: signal 'top.cpu.clk' not found"
        );
    }

    #[test]
    fn expr_errors_use_exit_code_one() {
        let error = WavepeekError::Expr("parse:EXPR-PARSE-LOGICAL-UNMATCHED-OPEN".to_string());

        assert_eq!(error.exit_code(), 1);
        assert_eq!(
            error.to_string(),
            "error: expr: parse:EXPR-PARSE-LOGICAL-UNMATCHED-OPEN"
        );
    }
}
