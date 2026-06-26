use serde::Serialize;

use crate::diagnostic::{Diagnostic, DiagnosticKind};
use crate::error::WavepeekError;
use crate::waveform::{STABLE_SCOPE_KIND_ALIASES, STABLE_SIGNAL_KIND_ALIASES};

#[derive(Debug, Serialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum ContractDiagnostic<'a> {
    Info {
        message: &'a str,
    },
    Warning {
        code: &'static str,
        message: &'a str,
    },
    Error {
        code: &'static str,
        message: &'a str,
    },
}

impl<'a> ContractDiagnostic<'a> {
    pub fn from_diagnostic(diagnostic: &'a Diagnostic) -> Result<Self, WavepeekError> {
        match diagnostic.kind() {
            DiagnosticKind::Info => Ok(Self::Info {
                message: diagnostic.message(),
            }),
            DiagnosticKind::Warning => Ok(Self::Warning {
                code: diagnostic.code().ok_or_else(|| {
                    WavepeekError::Internal(
                        "warning diagnostic is missing a stable code".to_string(),
                    )
                })?,
                message: diagnostic.message(),
            }),
            DiagnosticKind::Error => Ok(Self::Error {
                code: diagnostic.code().ok_or_else(|| {
                    WavepeekError::Internal("error diagnostic is missing a stable code".to_string())
                })?,
                message: diagnostic.message(),
            }),
        }
    }
}

pub fn validate_scope_kind(kind: &str) -> Result<&str, WavepeekError> {
    if STABLE_SCOPE_KIND_ALIASES.contains(&kind) {
        Ok(kind)
    } else {
        Err(WavepeekError::Internal(format!(
            "scope kind {kind:?} is not part of the stable machine-output contract"
        )))
    }
}

pub fn validate_signal_kind(kind: &str) -> Result<&str, WavepeekError> {
    if STABLE_SIGNAL_KIND_ALIASES.contains(&kind) {
        Ok(kind)
    } else {
        Err(WavepeekError::Internal(format!(
            "signal kind {kind:?} is not part of the stable machine-output contract"
        )))
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use crate::diagnostic::{Diagnostic, WarningDiagnosticCode};

    use super::{ContractDiagnostic, validate_scope_kind, validate_signal_kind};

    #[test]
    fn contract_diagnostic_preserves_wire_shape() {
        let diagnostic = Diagnostic::warning(WarningDiagnosticCode::OutputTruncated, "truncated");
        let contract = ContractDiagnostic::from_diagnostic(&diagnostic)
            .expect("warning diagnostic should convert");

        assert_eq!(
            serde_json::to_value(contract).expect("contract diagnostic should serialize"),
            json!({"kind": "warning", "code": "WPK-W0002", "message": "truncated"})
        );
    }

    #[test]
    fn kind_validation_rejects_backend_specific_aliases() {
        assert_eq!(validate_scope_kind("module").unwrap(), "module");
        assert_eq!(validate_signal_kind("wire").unwrap(), "wire");
        assert!(validate_scope_kind("vhdl_architecture").is_err());
        assert!(validate_signal_kind("std_logic").is_err());
    }
}
