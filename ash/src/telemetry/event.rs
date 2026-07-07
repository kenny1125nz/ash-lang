use std::time::{SystemTime, UNIX_EPOCH};

use super::context::SpanContext;

#[derive(Debug, Clone)]
pub enum EventKind {
    SessionStart,
    SessionEnd,
    AgentCall,
    AgentResponse,
    CommandExec,
    Error,
}

pub fn parse_kind(s: &str) -> EventKind {
    match s {
        "session_start" => EventKind::SessionStart,
        "session_end" => EventKind::SessionEnd,
        "agent_call" => EventKind::AgentCall,
        "agent_response" => EventKind::AgentResponse,
        "command_exec" => EventKind::CommandExec,
        "error" => EventKind::Error,
        _ => EventKind::Error,
    }
}

impl EventKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            EventKind::SessionStart => "session_start",
            EventKind::SessionEnd => "session_end",
            EventKind::AgentCall => "agent_call",
            EventKind::AgentResponse => "agent_response",
            EventKind::CommandExec => "command_exec",
            EventKind::Error => "error",
        }
    }

    /// Numeric severity: 0=debug, 1=info, 2=warn, 3=error.
    pub fn severity(&self) -> u8 {
        match self {
            EventKind::SessionStart | EventKind::SessionEnd => 1,
            EventKind::AgentCall | EventKind::AgentResponse => 1,
            EventKind::CommandExec => 1,
            EventKind::Error => 3,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TelemetryEvent {
    pub ctx: SpanContext,
    pub timestamp: u128,
    pub kind: EventKind,
    pub payload: serde_json::Value,
}

impl TelemetryEvent {
    pub fn new(ctx: SpanContext, kind: EventKind, payload: serde_json::Value) -> Self {
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis();
        TelemetryEvent {
            ctx,
            timestamp: ts,
            kind,
            payload,
        }
    }

    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "trace_id": self.ctx.trace_id,
            "span_id": self.ctx.span_id,
            "parent_span_id": self.ctx.parent_span_id,
            "origin": self.ctx.origin,
            "timestamp": self.timestamp,
            "kind": self.kind.as_str(),
            "payload": self.payload,
        })
    }
}
