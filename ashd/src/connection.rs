use std::collections::HashMap;
use std::sync::Mutex;
use tokio::sync::mpsc;

pub struct ConnectionMap {
    inner: Mutex<HashMap<String, mpsc::UnboundedSender<String>>>,
}

impl ConnectionMap {
    pub fn new() -> Self {
        ConnectionMap {
            inner: Mutex::new(HashMap::new()),
        }
    }

    pub fn register(&self, instance_id: String, reply_tx: mpsc::UnboundedSender<String>) {
        let mut map = self.inner.lock().unwrap();
        map.insert(instance_id, reply_tx);
    }

    pub fn remove(&self, instance_id: &str) {
        let mut map = self.inner.lock().unwrap();
        map.remove(instance_id);
    }

    pub fn send(&self, instance_id: &str, frame: &str) -> Result<(), String> {
        let map = self.inner.lock().unwrap();
        match map.get(instance_id) {
            Some(tx) => tx.send(frame.to_string()).map_err(|_| "channel closed".to_string()),
            None => Err(format!("no connection for instance {}", instance_id)),
        }
    }

    pub fn broadcast(&self, msg: &str) {
        let map = self.inner.lock().unwrap();
        for tx in map.values() {
            let _ = tx.send(msg.to_string());
        }
    }
}
