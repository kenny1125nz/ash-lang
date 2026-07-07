use std::fs::{self, File, OpenOptions};
use std::io::{BufWriter, Write};
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use super::config::TelemetryConfig;
use super::event::TelemetryEvent;
use super::filter::Filter;
use super::offset;
use super::store::FileStore;

pub struct Pipeline {
    writer: Mutex<BufWriter<File>>,
    filter: Arc<Filter>,
    path: std::path::PathBuf,
    max_size: u64,
    #[allow(dead_code)]
    max_files: u32,
    current_size: Mutex<u64>,
    pub capture_payload: bool,
    // Optional remote consumer
    remote_running: Option<Arc<AtomicBool>>,
    remote_handle: Option<thread::JoinHandle<()>>,
}

impl Pipeline {
    pub fn start(config: &TelemetryConfig) -> Result<Self, String> {
        let filter = Arc::new(Filter::new(config.filter.as_deref()));
        let file_cfg = config
            .file
            .as_ref()
            .ok_or_else(|| "file config required".to_string())?;
        let path = std::path::PathBuf::from(&file_cfg.path);
        let capture_payload = filter.capture_payload;

        // Create parent dir if needed
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| format!("create dir: {}", e))?;
        }

        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .map_err(|e| format!("open telemetry file: {}", e))?;

        let current_size = file.metadata().map(|m| m.len()).unwrap_or(0);

        // Spawn remote consumer thread if a remote adapter is enabled
        let (remote_running, remote_handle) = spawn_remote_consumer(config, &path, &filter)?;

        Ok(Pipeline {
            writer: Mutex::new(BufWriter::new(file)),
            filter,
            path,
            max_size: file_cfg.max_size_mb * 1024 * 1024,
            max_files: file_cfg.max_files,
            current_size: Mutex::new(current_size),
            capture_payload,
            remote_running,
            remote_handle,
        })
    }

    pub fn emit(&self, event: TelemetryEvent) {
        if !self.filter.accept(&event) {
            return;
        }
        let json = serde_json::to_string(&event.to_json()).unwrap_or_default();
        let bytes = json.as_bytes();
        let len = bytes.len() as u64;

        let mut writer = self.writer.lock().unwrap();
        let _ = writer.write_all(bytes);
        let _ = writer.write_all(b"\n");
        // Flush the BufWriter so data lands in the kernel buffer promptly
        let _ = writer.flush();

        let mut size = self.current_size.lock().unwrap();
        *size += len + 1;
        if *size >= self.max_size {
            drop(writer);
            drop(size);
            self.rotate();
        }
    }

    fn rotate(&self) {
        let _ = fs::rename(&self.path, self.path.with_extension("1.jsonl"));
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)
            .unwrap();
        *self.writer.lock().unwrap() = BufWriter::new(file);
        *self.current_size.lock().unwrap() = 0;
    }
}

impl Drop for Pipeline {
    fn drop(&mut self) {
        if let Some(ref r) = self.remote_running {
            r.store(false, Ordering::Relaxed);
        }
        if let Some(h) = self.remote_handle.take() {
            let _ = h.join();
        }
        let _ = self.writer.lock().unwrap().flush();
    }
}

fn spawn_remote_consumer(
    config: &TelemetryConfig,
    event_path: &Path,
    global_filter: &Arc<Filter>,
) -> Result<(Option<Arc<AtomicBool>>, Option<thread::JoinHandle<()>>), String> {
    // Find the single enabled remote adapter (kafka, splunk, etc.)
    let remote_config = if let Some(ref k) = config.kafka {
        if k.enabled { Some(k) } else { None }
    } else if let Some(ref s) = config.splunk {
        if s.enabled { Some(s) } else { None }
    } else {
        None
    };

    let remote_config = match remote_config {
        Some(c) => c.clone(),
        None => return Ok((None, None)),
    };

    let remote_filter = Arc::new(Filter::new(remote_config.filter.as_deref()));
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    let path = event_path.to_path_buf();
    let interval = Duration::from_millis(remote_config.flush_interval_ms);
    let batch_size = remote_config.batch_size;
    let gf = Arc::clone(global_filter);

    let handle = thread::spawn(move || {
        let mut batch: Vec<TelemetryEvent> = Vec::new();
        let mut last_flush = Instant::now();

        while r.load(Ordering::Relaxed) {
            let start_offset = offset::load_offset(&path);
            let store = FileStore::new(&path);
            let mut reader = match store.reader(start_offset) {
                Ok(r) => r,
                Err(_) => {
                    thread::sleep(interval);
                    continue;
                }
            };

            loop {
                match reader.read_line() {
                    Ok(Some(event)) => {
                        // Apply global filter (events that got written to file
                        // already passed the global filter, but be safe)
                        if !gf.accept(&event) {
                            continue;
                        }
                        // Apply remote-specific secondary filter
                        if remote_filter.accept(&event) {
                            batch.push(event);
                        }
                        if batch.len() >= batch_size {
                            // attempt delivery — override in real impls
                            let ok = deliver_remote(&batch);
                            if ok {
                                offset::save_offset(&path, reader.position());
                                batch.clear();
                                last_flush = Instant::now();
                            } else {
                                break; // retry next tick
                            }
                        }
                    }
                    Ok(None) => break, // EOF
                    Err(_) => break,
                }
            }

            // Flush remaining on timer
            if !batch.is_empty() && last_flush.elapsed() >= interval {
                let ok = deliver_remote(&batch);
                if ok {
                    offset::save_offset(&path, reader.position());
                    batch.clear();
                }
                last_flush = Instant::now();
            }

            thread::sleep(interval);
        }
    });

    Ok((Some(running), Some(handle)))
}

/// Placeholder delivery — replace with kafka/splunk/etc impls later.
fn deliver_remote(_batch: &[TelemetryEvent]) -> bool {
    true
}
