use std::collections::HashMap;
use std::sync::Mutex;
use std::time::Instant;
use tokio::sync::mpsc;

#[derive(Clone)]
pub struct ProcessState {
    pub status: String,
    pub workflow_path: String,
    pub last_seen: Instant,
    pub reply_tx: Option<mpsc::UnboundedSender<String>>,
}

pub struct ProcessRegistry {
    inner: Mutex<HashMap<String, ProcessState>>,
    source_names: Mutex<HashMap<String, String>>,
}

impl ProcessRegistry {
    pub fn new() -> Self {
        ProcessRegistry {
            inner: Mutex::new(HashMap::new()),
            source_names: Mutex::new(HashMap::new()),
        }
    }

    pub fn update(
        &self,
        instance_id: String,
        status: String,
        workflow_path: String,
        reply_tx: mpsc::UnboundedSender<String>,
    ) {
        let mut map = self.inner.lock().unwrap();
        map.insert(
            instance_id,
            ProcessState {
                status,
                workflow_path,
                last_seen: Instant::now(),
                reply_tx: Some(reply_tx),
            },
        );
    }

    pub fn disconnect(&self, instance_id: &str) {
        let mut map = self.inner.lock().unwrap();
        if let Some(state) = map.get_mut(instance_id) {
            state.reply_tx = None;
        }
    }

    pub fn get(&self, instance_id: &str) -> Option<ProcessState> {
        let map = self.inner.lock().unwrap();
        map.get(instance_id).cloned()
    }

    pub fn is_source_busy(&self, source_name: &str) -> bool {
        let sources = self.source_names.lock().unwrap();
        let inner = self.inner.lock().unwrap();
        for (instance_id, state) in inner.iter() {
            if let Some(src) = sources.get(instance_id) {
                if src == source_name && (state.status == "running" || state.status == "paused") {
                    return true;
                }
            }
        }
        false
    }

    pub fn source_from_instance(&self, instance_id: &str) -> Option<String> {
        let sources = self.source_names.lock().unwrap();
        sources.get(instance_id).cloned()
    }

    pub fn set_source(&self, instance_id: String, source_name: String) {
        let mut sources = self.source_names.lock().unwrap();
        sources.insert(instance_id, source_name);
    }
}
