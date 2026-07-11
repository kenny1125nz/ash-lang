use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use std::sync::mpsc;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

pub const STATE_NORMAL: u8 = 0;
pub const STATE_ABORT: u8 = 1;
pub const STATE_PAUSED: u8 = 2;

const RECONNECT_BASE: Duration = Duration::from_secs(1);
const RECONNECT_MAX: Duration = Duration::from_secs(30);
const POLL_INTERVAL: Duration = Duration::from_millis(10);

pub struct WsClient {
    pub instance_id: String,
    pub workflow_path: String,
    pub control_state: Arc<AtomicU8>,
    pub shutdown: Arc<AtomicBool>,
}

impl WsClient {
    pub fn connect_loop(&self, url: &str, rx: mpsc::Receiver<String>) {
        let mut backoff = RECONNECT_BASE;

        loop {
            if self.shutdown.load(Ordering::Relaxed) {
                return;
            }

            match self.connect_and_run(url, &rx) {
                Ok(()) => {
                    eprintln!("[as WS] clean shutdown");
                    return;
                }
                Err(e) => {
                    eprintln!("[as WS] {} — reconnecting in {:?}", e, backoff);
                    if self.shutdown.load(Ordering::Relaxed) {
                        return;
                    }
                    thread::sleep(backoff);
                    backoff = (backoff * 2).min(RECONNECT_MAX);
                }
            }
        }
    }

    fn connect_and_run(
        &self,
        url: &str,
        rx: &mpsc::Receiver<String>,
    ) -> Result<(), String> {
        let (mut ws, _) =
            tungstenite::connect(url).map_err(|e| format!("connect: {}", e))?;

        if let tungstenite::stream::MaybeTlsStream::Plain(tcp) = ws.get_ref() {
            tcp.set_nonblocking(true)
                .map_err(|e| format!("set_nonblocking: {}", e))?;
        }

        self.send_sta(&mut ws, "running")?;

        loop {
            if self.shutdown.load(Ordering::Relaxed) {
                let _ = self.send_sta(&mut ws, "completed");
                let _ = ws.close(None);
                return Ok(());
            }

            match ws.read() {
                Ok(tungstenite::Message::Text(text)) => {
                    if text.starts_with("ABT:") {
                        self.control_state.store(STATE_ABORT, Ordering::Relaxed);
                        let _ = self.send_sta(&mut ws, "cancelled");
                        return Err("aborted by ashd".to_string());
                    } else if text.starts_with("PAU:") {
                        self.control_state.store(STATE_PAUSED, Ordering::Relaxed);
                    } else if text.starts_with("RES:") {
                        self.control_state.store(STATE_NORMAL, Ordering::Relaxed);
                        let _ = self.send_sta(&mut ws, "running");
                    }
                }
                Ok(tungstenite::Message::Ping(data)) => {
                    let _ = ws.write(tungstenite::Message::Pong(data));
                }
                Ok(tungstenite::Message::Close(_)) => {
                    return Err("connection closed by peer".to_string());
                }
                Err(tungstenite::Error::Io(ref e))
                    if e.kind() == std::io::ErrorKind::WouldBlock => {}
                Err(e) => {
                    return Err(format!("read error: {}", e));
                }
                _ => {}
            }

            match rx.try_recv() {
                Ok(msg) => {
                    let frame = format!("TEL:{}", msg);
                    if let Err(e) = ws.write(tungstenite::Message::Text(frame)) {
                        return Err(format!("write error: {}", e));
                    }
                }
                Err(mpsc::TryRecvError::Empty) => {}
                Err(mpsc::TryRecvError::Disconnected) => {
                    let _ = self.send_sta(&mut ws, "completed");
                    let _ = ws.close(None);
                    return Ok(());
                }
            }

            thread::sleep(POLL_INTERVAL);
        }
    }

    fn send_sta(
        &self,
        ws: &mut tungstenite::WebSocket<tungstenite::stream::MaybeTlsStream<std::net::TcpStream>>,
        status: &str,
    ) -> Result<(), String> {
        let frame = format!("STA:{}:{}:{}", self.instance_id, status, self.workflow_path);
        ws.write(tungstenite::Message::Text(frame))
            .map_err(|e| format!("write error: {}", e))
    }
}

pub fn discover_ashd_url(config: &super::config::TelemetryConfig) -> Option<String> {
    if let Ok(url) = std::env::var("ASHD_WS_URL") {
        return Some(url);
    }
    if let Some(ref url) = config.ws_url {
        return Some(url.clone());
    }

    let sock_path = dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join(".ash/ashd.sock");
    if let Ok(contents) = std::fs::read_to_string(&sock_path) {
        let url = contents.trim().to_string();
        if !url.is_empty() {
            return Some(url);
        }
    }

    Some("ws://127.0.0.1:9877".to_string())
}
