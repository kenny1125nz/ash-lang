pub mod client;
pub mod config;
pub mod context;
pub mod event;
pub mod filter;
pub mod offset;
pub mod pipeline;
pub mod store;

use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use std::sync::mpsc;
use std::sync::{Arc, Mutex, OnceLock};
use std::thread;

use crate::AshError;
use client::WsClient;
use config::TelemetryConfig;
use context::SpanContext;
use event::{EventKind, TelemetryEvent};
use pipeline::Pipeline;

static INSTANCE: OnceLock<Mutex<Option<Pipeline>>> = OnceLock::new();
static CAPTURE_PAYLOAD: AtomicBool = AtomicBool::new(false);
static INSTANCE_ID: OnceLock<String> = OnceLock::new();
static WORKFLOW_PATH: OnceLock<String> = OnceLock::new();
static CONTROL_STATE: OnceLock<Arc<AtomicU8>> = OnceLock::new();

pub use client::{STATE_ABORT, STATE_NORMAL, STATE_PAUSED};

pub fn instance_id() -> &'static str {
    INSTANCE_ID.get().map(|s| s.as_str()).unwrap_or("unknown")
}

pub fn set_instance_id(id: String) {
    let _ = INSTANCE_ID.set(id);
}

pub fn workflow_path() -> &'static str {
    WORKFLOW_PATH.get().map(|s| s.as_str()).unwrap_or("")
}

pub fn set_workflow_path(path: String) {
    let _ = WORKFLOW_PATH.set(path);
}

pub fn control_state() -> Arc<AtomicU8> {
    CONTROL_STATE
        .get_or_init(|| Arc::new(AtomicU8::new(STATE_NORMAL)))
        .clone()
}

pub fn init(config: TelemetryConfig) -> Result<(), AshError> {
    let (ws_tx, ws_rx) = mpsc::sync_channel::<String>(256);
    let ws_shutdown = Arc::new(AtomicBool::new(false));

    let mut pipeline = Pipeline::start(&config, Some(ws_tx), Some(ws_shutdown.clone()))?;

    let instance_id = instance_id().to_string();
    let workflow_path = workflow_path().to_string();
    let control_state = control_state();
    let ws_url = client::discover_ashd_url(&config).unwrap();
    log::info!("telemetry WebSocket URL: {}", ws_url);

    let shutdown_flag = ws_shutdown.clone();
    let handle = thread::spawn(move || {
        let client = WsClient {
            instance_id,
            workflow_path,
            control_state,
            shutdown: shutdown_flag,
        };
        client.connect_loop(&ws_url, ws_rx);
    });

    pipeline.set_ws_handle(handle);
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
