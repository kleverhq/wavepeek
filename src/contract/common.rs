use std::borrow::Cow;

use schemars::{JsonSchema, Schema, SchemaGenerator, json_schema};
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

impl JsonSchema for ContractDiagnostic<'_> {
    fn schema_name() -> Cow<'static, str> {
        "diagnostic".into()
    }

    fn json_schema(_: &mut SchemaGenerator) -> Schema {
        json_schema!({
            "type": "object",
            "additionalProperties": true,
            "required": ["kind", "message"],
            "properties": {
                "kind": {"type": "string", "enum": ["info", "warning", "error"]},
                "code": {"type": "string", "pattern": "^WPK-[WE][0-9]{4}$"},
                "message": {"type": "string"},
            },
            "allOf": [
                {
                    "if": {"properties": {"kind": {"const": "warning"}}, "required": ["kind"]},
                    "then": {"required": ["code"], "properties": {"code": {"pattern": "^WPK-W[0-9]{4}$"}}},
                },
                {
                    "if": {"properties": {"kind": {"const": "error"}}, "required": ["kind"]},
                    "then": {"required": ["code"], "properties": {"code": {"pattern": "^WPK-E[0-9]{4}$"}}},
                },
                {
                    "if": {"properties": {"kind": {"const": "info"}}, "required": ["kind"]},
                    "then": {"not": {"required": ["code"]}},
                },
            ],
        })
    }
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

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(transparent)]
pub struct NormalizedTime<'a>(&'a str);

impl<'a> NormalizedTime<'a> {
    pub fn new(value: &'a str) -> Self {
        Self(value)
    }
}

impl JsonSchema for NormalizedTime<'_> {
    fn schema_name() -> Cow<'static, str> {
        "normalizedTime".into()
    }

    fn json_schema(_: &mut SchemaGenerator) -> Schema {
        json_schema!({
            "type": "string",
            "description": "Normalized timestamp rendered in the dump's time unit, for example 10ns."
        })
    }
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(transparent)]
pub struct CanonicalPath<'a>(&'a str);

impl<'a> CanonicalPath<'a> {
    pub fn new(value: &'a str) -> Self {
        Self(value)
    }
}

impl JsonSchema for CanonicalPath<'_> {
    fn schema_name() -> Cow<'static, str> {
        "canonicalPath".into()
    }

    fn json_schema(_: &mut SchemaGenerator) -> Schema {
        json_schema!({
            "type": "string",
            "description": "Canonical dot-separated hierarchy path emitted by wavepeek for a scope or signal."
        })
    }
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(transparent)]
pub struct SampledValue<'a>(&'a str);

impl<'a> SampledValue<'a> {
    pub fn new(value: &'a str) -> Self {
        Self(value)
    }
}

impl JsonSchema for SampledValue<'_> {
    fn schema_name() -> Cow<'static, str> {
        "sampledValue".into()
    }

    fn json_schema(_: &mut SchemaGenerator) -> Schema {
        json_schema!({
            "type": "string",
            "description": "Stable sampled signal value formatted as a Verilog-style literal string."
        })
    }
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(transparent)]
pub struct ScopeKind<'a>(&'a str);

impl JsonSchema for ScopeKind<'_> {
    fn schema_name() -> Cow<'static, str> {
        "scopeKind".into()
    }

    fn json_schema(_: &mut SchemaGenerator) -> Schema {
        json_schema!({
            "type": "string",
            "description": "Stable scope kind alias emitted by wavepeek for the selected scope.",
            "enum": STABLE_SCOPE_KIND_ALIASES,
        })
    }
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(transparent)]
pub struct SignalKind<'a>(&'a str);

impl JsonSchema for SignalKind<'_> {
    fn schema_name() -> Cow<'static, str> {
        "signalKind".into()
    }

    fn json_schema(_: &mut SchemaGenerator) -> Schema {
        json_schema!({
            "type": "string",
            "description": "Stable signal kind alias emitted by wavepeek for the selected signal.",
            "enum": STABLE_SIGNAL_KIND_ALIASES,
        })
    }
}

pub fn validate_scope_kind(kind: &str) -> Result<ScopeKind<'_>, WavepeekError> {
    if STABLE_SCOPE_KIND_ALIASES.contains(&kind) {
        Ok(ScopeKind(kind))
    } else {
        Err(WavepeekError::Internal(format!(
            "scope kind {kind:?} is not part of the stable machine-output contract"
        )))
    }
}

pub fn validate_signal_kind(kind: &str) -> Result<SignalKind<'_>, WavepeekError> {
    if STABLE_SIGNAL_KIND_ALIASES.contains(&kind) {
        Ok(SignalKind(kind))
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
        assert_eq!(
            serde_json::to_value(validate_scope_kind("module").unwrap()).unwrap(),
            json!("module")
        );
        assert_eq!(
            serde_json::to_value(validate_signal_kind("wire").unwrap()).unwrap(),
            json!("wire")
        );
        assert!(validate_scope_kind("vhdl_architecture").is_err());
        assert!(validate_signal_kind("std_logic").is_err());
    }
}
