use std::time::{Duration, Instant};

use serde::Serialize;
use serde_json::Value;

use crate::engine::CommandName;

#[derive(Debug)]
pub(crate) struct DebugTrace {
    enabled: bool,
    command: CommandName,
    started_at: Option<Instant>,
}

impl DebugTrace {
    pub(crate) fn for_command(command: CommandName) -> Self {
        let enabled = std::env::var("DEBUG").as_deref() == Ok("1");
        Self {
            enabled,
            command,
            started_at: enabled.then(Instant::now),
        }
    }

    pub(crate) fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub(crate) fn event(&self, message: &'static str, details: impl FnOnce() -> Value) {
        if !self.enabled {
            return;
        }

        let Some(started_at) = self.started_at else {
            return;
        };
        let mut details = match details() {
            Value::Object(map) => map,
            _ => serde_json::Map::new(),
        };
        details
            .entry("command".to_string())
            .or_insert_with(|| Value::String(self.command.as_str().to_string()));

        let event = DebugEvent {
            kind: "debug",
            message,
            timestamp_ns: duration_ns(started_at.elapsed()),
            details: Value::Object(details),
        };
        if let Ok(line) = serde_json::to_string(&event) {
            eprintln!("{line}");
        }
    }
}

#[derive(Serialize)]
struct DebugEvent {
    kind: &'static str,
    message: &'static str,
    timestamp_ns: u64,
    details: Value,
}

fn duration_ns(duration: Duration) -> u64 {
    u64::try_from(duration.as_nanos()).unwrap_or(u64::MAX)
}

#[cfg(test)]
mod tests {
    use super::DebugTrace;
    use crate::engine::CommandName;

    #[test]
    fn disabled_trace_does_not_evaluate_details() {
        let trace = DebugTrace {
            enabled: false,
            command: CommandName::Value,
            started_at: None,
        };

        trace.event("value.sample.done", || panic!("details must stay lazy"));
    }
}
