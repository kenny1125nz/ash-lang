use std::fs::{self, File, OpenOptions};
use std::io::{BufWriter, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;

use crate::util::lock_guard;
use crate::AshError;
use super::config::TelemetryConfig;
use super::event::TelemetryEvent;
use super::filter::Filter;

pub struct Pipeline {
    writer: Mutex<BufWriter<File>>,
    filter: Arc<Filter>,
    path: std::path::PathBuf,
    max_size: u64,
    #[allow(dead_code)]
    max_files: u32,
    current_size: Mutex<u64>,
    pub capture_payload: bool,
    ws_tx: Option<mpsc::SyncSender<String>>,
    ws_shutdown: Option<Arc<AtomicBool>>,
    ws_handle: Option<thread::JoinHandle<()>>,
}

impl Pipeline {
    pub fn start(
        config: &TelemetryConfig,
        ws_tx: Option<mpsc::SyncSender<String>>,
        ws_shutdown: Option<Arc<AtomicBool>>,
    ) -> Result<Self, AshError> {
        let filter = Arc::new(Filter::new(config.filter.as_deref()));
        let file_cfg = config
            .file
            .as_ref()
            .ok_or_else(|| AshError::Msg("file config required".to_string()))?;
        let path = std::path::PathBuf::from(&file_cfg.path);
        let capture_payload = filter.capture_payload;

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| AshError::Msg(format!("create dir: {}", e)))?;
        }

        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .map_err(|e| AshError::Msg(format!("open telemetry file: {}", e)))?;

        let current_size = file.metadata().map(|m| m.len()).unwrap_or(0);

        Ok(Pipeline {
            writer: Mutex::new(BufWriter::new(file)),
            filter,
            path,
            max_size: file_cfg.max_size_mb * 1024 * 1024,
            max_files: file_cfg.max_files,
            current_size: Mutex::new(current_size),
            capture_payload,
            ws_tx,
            ws_shutdown,
            ws_handle: None,
        })
    }

    pub fn set_ws_handle(&mut self, handle: thread::JoinHandle<()>) {
        self.ws_handle = Some(handle);
    }

    pub fn emit(&self, event: TelemetryEvent) {
        if !self.filter.accept(&event) {
            return;
        }
        let json = serde_json::to_string(&event.to_json()).unwrap_or_default();
        let bytes = json.as_bytes();
        let len = bytes.len() as u64;

        let mut writer = lock_guard(&self.writer);
        let _ = writer.write_all(bytes);
        let _ = writer.write_all(b"\n");
        let _ = writer.flush();

        let mut size = lock_guard(&self.current_size);
        *size += len + 1;
        if *size >= self.max_size {
            drop(writer);
            drop(size);
            self.rotate();
        }

        if let Some(ref tx) = self.ws_tx {
            let _ = tx.try_send(json);
        }
    }

    fn rotate(&self) {
        let _ = fs::rename(&self.path, self.path.with_extension("1.jsonl"));
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)
            .unwrap();
        *lock_guard(&self.writer) = BufWriter::new(file);
        *lock_guard(&self.current_size) = 0;
    }
}

impl Drop for Pipeline {
    fn drop(&mut self) {
        if let Some(ref shutdown) = self.ws_shutdown {
            shutdown.store(true, Ordering::Relaxed);
        }
        drop(self.ws_tx.take());
        if let Some(h) = self.ws_handle.take() {
            let _ = h.join();
        }
        let _ = lock_guard(&self.writer).flush();
    }
}
