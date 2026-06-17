#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum DiagnosticKind {
    Info,
    Warning,
    Error,
    Debug,
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

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DebugDiagnosticCode {
    GenericMessage,
    PerformanceContext,
    PerformancePhase,
    PerformanceSummary,
}

impl DebugDiagnosticCode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::GenericMessage => "WPK-D0001",
            Self::PerformanceContext => "WPK-D1001",
            Self::PerformancePhase => "WPK-D1002",
            Self::PerformanceSummary => "WPK-D1003",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct Diagnostic {
    kind: DiagnosticKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    code: Option<&'static str>,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    details: Option<serde_json::Value>,
}

impl Diagnostic {
    #[allow(dead_code)]
    pub fn info(message: impl Into<String>) -> Self {
        Self {
            kind: DiagnosticKind::Info,
            code: None,
            message: message.into(),
            details: None,
        }
    }

    pub fn warning(code: WarningDiagnosticCode, message: impl Into<String>) -> Self {
        Self {
            kind: DiagnosticKind::Warning,
            code: Some(code.as_str()),
            message: message.into(),
            details: None,
        }
    }

    #[allow(dead_code)]
    pub fn error(code: ErrorDiagnosticCode, message: impl Into<String>) -> Self {
        Self {
            kind: DiagnosticKind::Error,
            code: Some(code.as_str()),
            message: message.into(),
            details: None,
        }
    }

    #[allow(dead_code)]
    pub fn debug(code: DebugDiagnosticCode, message: impl Into<String>) -> Self {
        Self {
            kind: DiagnosticKind::Debug,
            code: Some(code.as_str()),
            message: message.into(),
            details: None,
        }
    }

    pub fn debug_with_details(
        code: DebugDiagnosticCode,
        message: impl Into<String>,
        details: serde_json::Value,
    ) -> Self {
        Self {
            kind: DiagnosticKind::Debug,
            code: Some(code.as_str()),
            message: message.into(),
            details: Some(details),
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
            details: None,
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

    #[allow(dead_code)]
    pub const fn details(&self) -> Option<&serde_json::Value> {
        self.details.as_ref()
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

    use super::{DebugDiagnosticCode, Diagnostic, DiagnosticKind, WarningDiagnosticCode};

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
    fn debug_diagnostic_serializes_with_stable_code_and_details() {
        let diagnostic = Diagnostic::debug_with_details(
            DebugDiagnosticCode::PerformancePhase,
            "perf: backend.open 1ms",
            json!({
                "domain": "performance",
                "event": "phase",
                "phase": "backend.open",
                "duration_ns": 1_000_000,
                "status": "ok"
            }),
        );

        assert_eq!(diagnostic.kind(), DiagnosticKind::Debug);
        assert_eq!(diagnostic.code(), Some("WPK-D1002"));
        assert_eq!(
            serde_json::to_value(&diagnostic).expect("diagnostic should serialize"),
            json!({
                "kind": "debug",
                "code": "WPK-D1002",
                "message": "perf: backend.open 1ms",
                "details": {
                    "domain": "performance",
                    "event": "phase",
                    "phase": "backend.open",
                    "duration_ns": 1_000_000,
                    "status": "ok"
                }
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
