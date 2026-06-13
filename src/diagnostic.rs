#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum DiagnosticKind {
    Info,
    Warning,
    Error,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WarningDiagnosticCode {
    LimitDisabled,
    OutputTruncated,
    EmptyResult,
}

impl WarningDiagnosticCode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::LimitDisabled => "WPK-W0001",
            Self::OutputTruncated => "WPK-W0002",
            Self::EmptyResult => "WPK-W0003",
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorDiagnosticCode {}

#[allow(dead_code)]
impl ErrorDiagnosticCode {
    pub const fn as_str(self) -> &'static str {
        match self {}
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct Diagnostic {
    kind: DiagnosticKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    code: Option<&'static str>,
    message: String,
}

impl Diagnostic {
    #[allow(dead_code)]
    pub fn info(message: impl Into<String>) -> Self {
        Self {
            kind: DiagnosticKind::Info,
            code: None,
            message: message.into(),
        }
    }

    pub fn warning(code: WarningDiagnosticCode, message: impl Into<String>) -> Self {
        Self {
            kind: DiagnosticKind::Warning,
            code: Some(code.as_str()),
            message: message.into(),
        }
    }

    #[allow(dead_code)]
    pub fn error(code: ErrorDiagnosticCode, message: impl Into<String>) -> Self {
        Self {
            kind: DiagnosticKind::Error,
            code: Some(code.as_str()),
            message: message.into(),
        }
    }

    #[cfg(test)]
    pub(crate) fn test_error(code: &'static str, message: impl Into<String>) -> Self {
        assert!(
            is_valid_error_code(code),
            "test diagnostic error code must match WPK-E####"
        );
        Self {
            kind: DiagnosticKind::Error,
            code: Some(code),
            message: message.into(),
        }
    }

    pub const fn kind(&self) -> DiagnosticKind {
        self.kind
    }

    pub const fn code(&self) -> Option<&'static str> {
        self.code
    }

    pub fn message(&self) -> &str {
        self.message.as_str()
    }
}

#[cfg(test)]
fn is_valid_error_code(code: &str) -> bool {
    let bytes = code.as_bytes();
    bytes.len() == 9
        && bytes[0..5] == *b"WPK-E"
        && bytes[5].is_ascii_digit()
        && bytes[6].is_ascii_digit()
        && bytes[7].is_ascii_digit()
        && bytes[8].is_ascii_digit()
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{Diagnostic, DiagnosticKind, WarningDiagnosticCode};

    #[test]
    fn warning_diagnostic_serializes_with_stable_code() {
        let diagnostic = Diagnostic::warning(
            WarningDiagnosticCode::OutputTruncated,
            "truncated output to 1 entries",
        );

        assert_eq!(diagnostic.kind(), DiagnosticKind::Warning);
        assert_eq!(diagnostic.code(), Some("WPK-W0002"));
        assert_eq!(diagnostic.message(), "truncated output to 1 entries");
        assert_eq!(
            serde_json::to_value(&diagnostic).expect("diagnostic should serialize"),
            json!({
                "kind": "warning",
                "code": "WPK-W0002",
                "message": "truncated output to 1 entries"
            })
        );
    }

    #[test]
    fn info_diagnostic_omits_code() {
        let diagnostic = Diagnostic::info("catalog loaded");

        assert_eq!(diagnostic.kind(), DiagnosticKind::Info);
        assert_eq!(diagnostic.code(), None);
        assert_eq!(
            serde_json::to_value(&diagnostic).expect("diagnostic should serialize"),
            json!({
                "kind": "info",
                "message": "catalog loaded"
            })
        );
    }

    #[test]
    fn test_error_diagnostic_requires_error_code_shape() {
        let diagnostic = Diagnostic::test_error("WPK-E0001", "partial result failed");

        assert_eq!(diagnostic.kind(), DiagnosticKind::Error);
        assert_eq!(diagnostic.code(), Some("WPK-E0001"));
    }
}
