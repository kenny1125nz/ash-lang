use futures_util::stream::SplitSink;
use futures_util::{SinkExt, StreamExt};
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::WebSocketStream;

use crate::router::IncomingFrame;

pub const CLOSE_SENTINEL: &str = "\x00__CLOSE__\x00";

pub struct WsServer {
    listen_addr: String,
    frame_tx: mpsc::Sender<IncomingFrame>,
}

impl WsServer {
    pub fn new(listen_addr: String, frame_tx: mpsc::Sender<IncomingFrame>) -> Self {
        WsServer {
            listen_addr,
            frame_tx,
        }
    }

    pub async fn start(self) -> anyhow::Result<()> {
        if self.listen_addr.starts_with('/') || self.listen_addr.starts_with('.') {
            self.start_unix().await
        } else {
            self.start_tcp().await
        }
    }

    async fn start_tcp(self) -> anyhow::Result<()> {
        let listener = tokio::net::TcpListener::bind(&self.listen_addr).await?;
        log::info!("WebSocket server listening on {}", self.listen_addr);

        loop {
            match listener.accept().await {
                Ok((stream, peer)) => {
                    log::debug!("new connection from {}", peer);
                    let frame_tx = self.frame_tx.clone();
                    tokio::spawn(async move {
                        if let Err(e) = handle_tcp_connection(stream, frame_tx).await {
                            log::warn!("connection handler error: {}", e);
                        }
                    });
                }
                Err(e) => {
                    log::error!("accept error: {}", e);
                    return Err(e.into());
                }
            }
        }
    }

    async fn start_unix(self) -> anyhow::Result<()> {
        let _ = std::fs::remove_file(&self.listen_addr);
        let listener = tokio::net::UnixListener::bind(&self.listen_addr)?;
        log::info!("WebSocket server listening on Unix socket {}", self.listen_addr);

        loop {
            match listener.accept().await {
                Ok((stream, _)) => {
                    log::debug!("new unix connection");
                    let frame_tx = self.frame_tx.clone();
                    tokio::spawn(async move {
                        if let Err(e) = handle_unix_connection(stream, frame_tx).await {
                            log::warn!("unix connection handler error: {}", e);
                        }
                    });
                }
                Err(e) => {
                    log::error!("unix accept error: {}", e);
                    return Err(e.into());
                }
            }
        }
    }
}

async fn handle_tcp_connection(
    stream: TcpStream,
    frame_tx: mpsc::Sender<IncomingFrame>,
) -> anyhow::Result<()> {
    let ws_stream = tokio_tungstenite::accept_async(stream).await?;
    handle_ws_stream(ws_stream, frame_tx).await
}

async fn handle_unix_connection(
    stream: tokio::net::UnixStream,
    frame_tx: mpsc::Sender<IncomingFrame>,
) -> anyhow::Result<()> {
    let ws_stream = tokio_tungstenite::accept_async(stream).await?;
    handle_ws_stream(ws_stream, frame_tx).await
}

async fn handle_ws_stream<S>(
    ws_stream: WebSocketStream<S>,
    frame_tx: mpsc::Sender<IncomingFrame>,
) -> anyhow::Result<()>
where
    S: AsyncRead + AsyncWrite + Unpin + Send + 'static,
{
    let (ws_writer, mut ws_reader) = ws_stream.split();

    let (reply_tx, reply_rx) = mpsc::unbounded_channel::<String>();
    let write_handle = tokio::spawn(writer_task::<S>(ws_writer, reply_rx));

    let mut connection_instance: Option<String> = None;

    while let Some(msg) = ws_reader.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                let (verb, body) = match text.split_once(':') {
                    Some((v, b)) => (v.to_uppercase(), b.to_string()),
                    None => (text.to_uppercase(), String::new()),
                };

                let instance_id = if verb == "STA" {
                    let parts: Vec<&str> = body.splitn(4, ':').collect();
                    if parts.len() >= 3 {
                        let id = parts[0].to_string();
                        connection_instance = Some(id.clone());
                        Some(id)
                    } else {
                        log::warn!("malformed STA frame: insufficient fields");
                        None
                    }
                } else {
                    connection_instance.clone()
                };

                let frame = IncomingFrame {
                    verb,
                    instance_id,
                    body,
                    reply_tx: reply_tx.clone(),
                };

                if frame_tx.send(frame).await.is_err() {
                    log::warn!("frame channel closed, ending connection");
                    break;
                }
            }
            Ok(Message::Close(_)) => {
                log::debug!("connection closed");
                break;
            }
            Ok(Message::Ping(data)) => {
                let _ = reply_tx
                    .send(format!("PONG:{}", String::from_utf8_lossy(&data)))
                    .ok();
            }
            Ok(_) => {}
            Err(e) => {
                log::warn!("ws read error: {}", e);
                break;
            }
        }
    }

    if let Some(ref instance_id) = connection_instance {
        let frame = IncomingFrame {
            verb: "DISCONNECT".to_string(),
            instance_id: Some(instance_id.clone()),
            body: String::new(),
            reply_tx: reply_tx.clone(),
        };
        let _ = frame_tx.send(frame).await;
    }

    write_handle.abort();

    Ok(())
}

async fn writer_task<S>(
    mut ws_writer: SplitSink<WebSocketStream<S>, Message>,
    mut reply_rx: mpsc::UnboundedReceiver<String>,
) where
    S: AsyncRead + AsyncWrite + Unpin + Send + 'static,
{
    while let Some(msg) = reply_rx.recv().await {
        if msg == CLOSE_SENTINEL {
            let _ = ws_writer.send(Message::Close(None)).await;
            break;
        }
        if let Err(e) = ws_writer.send(Message::Text(msg)).await {
            log::warn!("ws write error: {}", e);
            break;
        }
    }
}
