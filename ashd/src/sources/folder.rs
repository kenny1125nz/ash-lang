use std::path::{Path, PathBuf};

use chrono::Utc;
use notify::{EventKind, RecursiveMode, Watcher};
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::channel::WorkflowEvent;
use crate::config::{ConcurrencyMode, SourceConfig, WorkflowConfig};

pub struct FolderSource {
    name: String,
    watch_path: PathBuf,
    _concurrency: ConcurrencyMode,
    workflows: Vec<WorkflowConfig>,
}

impl FolderSource {
    pub fn from_config(cfg: &SourceConfig) -> anyhow::Result<Self> {
        let watch_path = cfg
            .path
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("folder source '{}' has no path", cfg.name))?
            .clone();

        std::fs::create_dir_all(&watch_path)?;
        std::fs::create_dir_all(watch_path.join(".processing"))?;
        std::fs::create_dir_all(watch_path.join(".done"))?;

        Ok(FolderSource {
            name: cfg.name.clone(),
            watch_path,
            _concurrency: cfg.concurrency.clone(),
            workflows: cfg.workflows.clone(),
        })
    }

    pub async fn run(self, tx: mpsc::Sender<WorkflowEvent>) {
        let processing_dir = self.watch_path.join(".processing");
        let done_dir = self.watch_path.join(".done");

        // Ensure directories exist
        let _ = std::fs::create_dir_all(&processing_dir);
        let _ = std::fs::create_dir_all(&done_dir);
        let _ = std::fs::create_dir_all(done_dir.join("failed"));

        // Crash recovery: scan .processing/ for orphaned subdirectories
        let mut orphaned = 0u32;
        if let Ok(entries) = std::fs::read_dir(&processing_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    let name = entry.file_name();
                    let name_str = name.to_string_lossy();
                    if name_str.starts_with('.') {
                        continue;
                    }
                    let event = WorkflowEvent {
                        source_name: self.name.clone(),
                        event_id: Uuid::new_v4().to_string(),
                        path,
                        timestamp: Utc::now(),
                        workflow: self.workflows[0].clone(),
                    };
                    if tx.send(event).await.is_err() {
                        return;
                    }
                    orphaned += 1;
                }
            }
        }
        if orphaned > 0 {
            log::info!("count":% = orphaned, "source":% = self.name; "recovered orphaned event(s)");
        }

        // Initial scan: pick up any subdirectories that were placed while we were down
        if let Ok(entries) = std::fs::read_dir(&self.watch_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                let name = entry.file_name();
                let name_str = name.to_string_lossy();
                if path.is_dir() && !name_str.starts_with('.') {
                    let claimed_path = processing_dir.join(&name);
                    if let Err(e) = atomic_claim(&path, &claimed_path) {
                        log::warn!("failed to claim {} during initial scan: {}", path.display(), e);
                        continue;
                    }
                    let event = WorkflowEvent {
                        source_name: self.name.clone(),
                        event_id: Uuid::new_v4().to_string(),
                        path: claimed_path,
                        timestamp: Utc::now(),
                        workflow: self.workflows[0].clone(),
                    };
                    if tx.send(event).await.is_err() {
                        return;
                    }
                }
            }
        }

        log::info!(
            "source '{}': place completed subdirectories in '{}' — use temp-write-then-move for atomic delivery",
            self.name,
            self.watch_path.display(),
        );

        // Internal channel: notify watcher callback sends events here
        let (tx_internal, mut rx_internal) = mpsc::unbounded_channel::<notify::Event>();

        let mut watcher = match notify::recommended_watcher(
            move |res: Result<notify::Event, notify::Error>| {
                if let Ok(event) = res {
                    let _ = tx_internal.send(event);
                }
            },
        ) {
            Ok(w) => w,
            Err(e) => {
                log::error!("failed to create watcher for '{}': {}", self.name, e);
                return;
            }
        };

        if let Err(e) = watcher.watch(&self.watch_path, RecursiveMode::NonRecursive) {
            log::error!("failed to watch '{}': {}", self.watch_path.display(), e);
            return;
        }

        // Event processing loop
        loop {
            tokio::select! {
                Some(event) = rx_internal.recv() => {
                    for path in &event.paths {
                        // Skip dotfiles
                        if path
                            .file_name()
                            .and_then(|n| n.to_str())
                            .map(|n| n.starts_with('.'))
                            .unwrap_or(true)
                        {
                            continue;
                        }
                        // Only act on directories
                        if !path.is_dir() {
                            continue;
                        }

                        match event.kind {
                            EventKind::Create(_) | EventKind::Modify(_) => {
                                let dir_name = match path.file_name() {
                                    Some(n) => n.to_owned(),
                                    None => continue,
                                };
                                let claimed_path = processing_dir.join(&dir_name);

                                if let Err(e) = atomic_claim(path, &claimed_path) {
                                    log::warn!(
                                        "failed to claim {}: {}",
                                        path.display(),
                                        e
                                    );
                                    continue;
                                }

                                let event = WorkflowEvent {
                                    source_name: self.name.clone(),
                                    event_id: Uuid::new_v4().to_string(),
                                    path: claimed_path,
                                    timestamp: Utc::now(),
                                    workflow: self.workflows[0].clone(),
                                };
                                if tx.send(event).await.is_err() {
                                    return;
                                }
                            }
                            _ => {}
                        }
                    }
                }
                else => break,
            }
        }
    }
}

fn atomic_claim(src: &Path, dst: &Path) -> std::io::Result<()> {
    match std::fs::rename(src, dst) {
        Ok(()) => Ok(()),
        Err(e) => {
            log::warn!("rename failed ({}), falling back to copy-then-delete", e);
            copy_dir(src, dst)?;
            std::fs::remove_dir_all(src)?;
            Ok(())
        }
    }
}

fn copy_dir(src: &Path, dst: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        if ty.is_dir() {
            copy_dir(&src_path, &dst_path)?;
        } else {
            std::fs::copy(&src_path, &dst_path)?;
        }
    }
    Ok(())
}
