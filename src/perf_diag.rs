use std::time::{Duration, Instant};

use serde_json::{Map, Value, json};

use crate::diagnostic::{DebugDiagnosticCode, Diagnostic};
use crate::engine::CommandName;
use crate::error::WavepeekError;

#[derive(Debug)]
pub(crate) struct PerfDiagnostics {
    enabled: bool,
    command: CommandName,
    started_at: Option<Instant>,
    diagnostics: Vec<Diagnostic>,
}

impl PerfDiagnostics {
    pub(crate) fn for_command(command: CommandName) -> Self {
        Self::with_enabled(command, std::env::var("DEBUG").as_deref() == Ok("1"))
    }

    fn with_enabled(command: CommandName, enabled: bool) -> Self {
        Self {
            enabled,
            command,
            started_at: enabled.then(Instant::now),
            diagnostics: Vec::new(),
        }
    }

    #[cfg(test)]
    pub(crate) const fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub(crate) fn record_context(&mut self, backend: &'static str, format: &'static str) {
        if !self.enabled {
            return;
        }

        self.diagnostics.push(Diagnostic::debug_with_details(
            DebugDiagnosticCode::PerformanceContext,
            format!(
                "perf: context command={} backend={backend} format={format}",
                self.command.as_str()
            ),
            json!({
                "domain": "performance",
                "event": "context",
                "command": self.command.as_str(),
                "backend": backend,
                "format": format,
            }),
        ));
    }

    pub(crate) fn time_phase<T>(
        &mut self,
        phase: &'static str,
        work: impl FnOnce() -> Result<T, WavepeekError>,
    ) -> Result<T, WavepeekError> {
        self.time_phase_with_metrics(phase, work, |_| None)
    }

    pub(crate) fn time_phase_with_metrics<T>(
        &mut self,
        phase: &'static str,
        work: impl FnOnce() -> Result<T, WavepeekError>,
        metrics: impl FnOnce(&T) -> Option<Value>,
    ) -> Result<T, WavepeekError> {
        if !self.enabled {
            return work();
        }

        let started_at = Instant::now();
        let result = work();
        let duration = started_at.elapsed();
        let status = if result.is_ok() { "ok" } else { "error" };
        let metrics = result.as_ref().ok().and_then(metrics);
        self.record_phase(phase, duration, status, metrics);
        result
    }

    pub(crate) fn finish(mut self) -> Vec<Diagnostic> {
        if !self.enabled {
            return Vec::new();
        }

        let total_duration_ns = self
            .started_at
            .map(|started_at| duration_ns(started_at.elapsed()))
            .unwrap_or(0);
        let phase_count = self
            .diagnostics
            .iter()
            .filter(|diagnostic| {
                diagnostic.code() == Some(DebugDiagnosticCode::PerformancePhase.as_str())
            })
            .count();
        self.diagnostics.push(Diagnostic::debug_with_details(
            DebugDiagnosticCode::PerformanceSummary,
            format!(
                "perf: total {}",
                format_duration(Duration::from_nanos(total_duration_ns))
            ),
            json!({
                "domain": "performance",
                "event": "summary",
                "command": self.command.as_str(),
                "total_duration_ns": total_duration_ns,
                "phase_count": phase_count,
            }),
        ));
        self.diagnostics
    }

    fn record_phase(
        &mut self,
        phase: &'static str,
        duration: Duration,
        status: &'static str,
        metrics: Option<Value>,
    ) {
        let duration_ns = duration_ns(duration);
        let mut details = Map::new();
        details.insert("domain".to_string(), json!("performance"));
        details.insert("event".to_string(), json!("phase"));
        details.insert("phase".to_string(), json!(phase));
        details.insert("duration_ns".to_string(), json!(duration_ns));
        details.insert("status".to_string(), json!(status));
        if let Some(metrics) = metrics {
            details.insert("metrics".to_string(), metrics);
        }

        self.diagnostics.push(Diagnostic::debug_with_details(
            DebugDiagnosticCode::PerformancePhase,
            format!("perf: {phase} {}", format_duration(duration)),
            Value::Object(details),
        ));
    }
}

fn duration_ns(duration: Duration) -> u64 {
    u64::try_from(duration.as_nanos()).unwrap_or(u64::MAX)
}

fn format_duration(duration: Duration) -> String {
    let nanos = duration_ns(duration);
    if nanos >= 1_000_000_000 {
        format!("{:.3}s", nanos as f64 / 1_000_000_000.0)
    } else if nanos >= 1_000_000 {
        format!("{:.3}ms", nanos as f64 / 1_000_000.0)
    } else if nanos >= 1_000 {
        format!("{:.3}us", nanos as f64 / 1_000.0)
    } else {
        format!("{nanos}ns")
    }
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use super::PerfDiagnostics;
    use crate::engine::CommandName;

    #[test]
    fn disabled_recorder_does_not_emit_or_time() {
        let mut recorder = PerfDiagnostics::with_enabled(CommandName::Value, false);
        let result = recorder
            .time_phase("backend.open", || Ok::<_, crate::error::WavepeekError>(42))
            .expect("disabled phase should return work result");

        assert_eq!(result, 42);
        assert!(!recorder.is_enabled());
        assert!(recorder.finish().is_empty());
    }

    #[test]
    fn enabled_recorder_emits_context_phase_and_summary() {
        let mut recorder = PerfDiagnostics::with_enabled(CommandName::Value, true);
        recorder.record_context("wellen", "vcd");
        let result = recorder
            .time_phase_with_metrics(
                "value.sample",
                || Ok::<_, crate::error::WavepeekError>(vec![1, 2]),
                |values| Some(serde_json::json!({"samples": values.len()})),
            )
            .expect("enabled phase should return work result");
        assert_eq!(result, vec![1, 2]);

        let diagnostics = recorder.finish();
        assert_eq!(diagnostics.len(), 3);
        assert_eq!(diagnostics[0].code(), Some("WPK-D1001"));
        assert_eq!(diagnostics[1].code(), Some("WPK-D1002"));
        assert_eq!(diagnostics[2].code(), Some("WPK-D1003"));
        let phase = diagnostics[1].details().expect("phase should have details");
        assert_eq!(phase["phase"], Value::String("value.sample".to_string()));
        assert_eq!(phase["status"], Value::String("ok".to_string()));
        assert_eq!(phase["metrics"]["samples"], Value::from(2));
    }
}
