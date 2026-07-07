use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Debug, Clone)]
pub struct SpanContext {
    pub trace_id: u64,
    pub span_id: u64,
    pub parent_span_id: Option<u64>,
    pub origin: String,
}

fn next_id() -> u64 {
    static COUNTER: AtomicU64 = AtomicU64::new(1);
    COUNTER.fetch_add(1, Ordering::Relaxed)
}

fn host_and_user() -> String {
    let user = std::env::var("USER").unwrap_or_else(|_| String::new());
    let host = std::env::var("HOSTNAME").unwrap_or_else(|_| String::new());
    if host.is_empty() && user.is_empty() {
        "unknown".to_string()
    } else if host.is_empty() {
        user
    } else if user.is_empty() {
        host
    } else {
        format!("{}@{}", user, host)
    }
}

impl SpanContext {
    pub fn root() -> Self {
        let id = next_id();
        SpanContext {
            trace_id: id,
            span_id: id,
            parent_span_id: None,
            origin: host_and_user(),
        }
    }

    pub fn child(&self) -> Self {
        SpanContext {
            trace_id: self.trace_id,
            span_id: next_id(),
            parent_span_id: Some(self.span_id),
            origin: self.origin.clone(),
        }
    }
}
