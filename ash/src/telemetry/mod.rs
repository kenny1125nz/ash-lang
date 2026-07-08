pub mod config;
pub mod context;
pub mod event;
pub mod filter;
pub mod offset;
pub mod pipeline;
pub mod store;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, OnceLock};

use crate::AshError;
use config::TelemetryConfig;
use context::SpanContext;
use event::{EventKind, TelemetryEvent};
use pipeline::Pipeline;

static INSTANCE: OnceLock<Mutex<Option<Pipeline>>> = OnceLock::new();
static CAPTURE_PAYLOAD: AtomicBool = AtomicBool::new(false);

pub fn init(config: TelemetryConfig) -> Result<(), AshError> {
    let pipeline = Pipeline::start(&config)?;
    CAPTURE_PAYLOAD.store(pipeline.capture_payload, Ordering::Relaxed);

    INSTANCE
        .set(Mutex::new(Some(pipeline)))
        .map_err(|_| AshError::Msg("telemetry already initialized".to_string()))
}

pub fn emit(ctx: SpanContext, kind: EventKind, payload: serde_json::Value) {
    if let Some(inst) = INSTANCE.get() {
        let event = TelemetryEvent::new(ctx, kind, payload);
        if let Ok(lock) = inst.lock() {
            if let Some(ref pipeline) = *lock {
                pipeline.emit(event);
            }
        }
    }
}

pub fn capture_payload() -> bool {
    CAPTURE_PAYLOAD.load(Ordering::Relaxed)
}

pub fn is_enabled() -> bool {
    INSTANCE
        .get()
        .and_then(|m| m.lock().ok())
        .map_or(false, |lock| lock.is_some())
}

pub fn shutdown() {
    if let Some(inst) = INSTANCE.get() {
        if let Ok(mut lock) = inst.lock() {
            if let Some(pipeline) = lock.take() {
                drop(pipeline);
            }
        }
    }
}
