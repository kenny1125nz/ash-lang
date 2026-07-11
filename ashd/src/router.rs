use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::mpsc;

use crate::connection::ConnectionMap;
use crate::registry::ProcessRegistry;
use crate::relay::RelayPipeline;

pub struct IncomingFrame {
    pub verb: String,
    pub instance_id: Option<String>,
    pub body: String,
    pub reply_tx: mpsc::UnboundedSender<String>,
}

pub struct FrameRouter {
    relay_pipeline: Arc<RelayPipeline>,
    process_registry: Arc<ProcessRegistry>,
    connection_map: Arc<ConnectionMap>,
    queue_notify_tx: tokio::sync::broadcast::Sender<String>,
    workflow_to_source: Arc<HashMap<String, String>>,
}

impl FrameRouter {
    pub fn new(
        relay_pipeline: Arc<RelayPipeline>,
        process_registry: Arc<ProcessRegistry>,
        connection_map: Arc<ConnectionMap>,
        queue_notify_tx: tokio::sync::broadcast::Sender<String>,
        workflow_to_source: Arc<HashMap<String, String>>,
    ) -> Self {
        FrameRouter {
            relay_pipeline,
            process_registry,
            connection_map,
            queue_notify_tx,
            workflow_to_source,
        }
    }

    pub async fn run(self, mut frame_rx: mpsc::Receiver<IncomingFrame>) {
        let terminal_statuses = ["completed", "failed", "cancelled"];
        while let Some(frame) = frame_rx.recv().await {
            match frame.verb.as_str() {
                "TEL" => {
                    self.relay_pipeline.forward(&frame.body);
                }
                "STA" => {
                    let parts: Vec<&str> = frame.body.splitn(4, ':').collect();
                    if parts.len() < 3 {
                        log::warn!("malformed STA frame: insufficient fields");
                        continue;
                    }
                    let instance_id = parts[0].to_string();
                    let status = parts[1].to_string();
                    let workflow_path = parts[2..].join(":");

                    log::info!("instance":% = instance_id, "status":% = status, "path":% = workflow_path; "STA frame");

                    if let Some(source_name) = self.workflow_to_source.get(&workflow_path) {
                        self.process_registry
                            .set_source(instance_id.clone(), source_name.clone());
                    }

                    self.process_registry.update(
                        instance_id.clone(),
                        status.clone(),
                        workflow_path.clone(),
                        frame.reply_tx.clone(),
                    );
                    self.connection_map
                        .register(instance_id.clone(), frame.reply_tx);

                    if terminal_statuses.contains(&status.as_str()) {
                        if let Some(source_name) = self.workflow_to_source.get(&workflow_path) {
                            let _ = self.queue_notify_tx.send(source_name.clone());
                        }
                    }
                }
                "ABT" | "PAU" | "RES" => {
                    if let Some(ref instance_id) = frame.instance_id {
                        let cmd = frame.verb.clone();
                        let frame_str = format!("{}:", cmd);
                        if let Err(e) = self.connection_map.send(instance_id, &frame_str) {
                            log::warn!("cmd":% = cmd, "instance":% = instance_id, "error":% = e; "failed to send control frame");
                        }
                    }
                }
                "DISCONNECT" => {
                    if let Some(ref instance_id) = frame.instance_id {
                        log::info!("instance":% = instance_id; "DISCONNECT");
                        self.connection_map.remove(instance_id);
                        self.process_registry.disconnect(instance_id);
                    }
                }
                _ => {
                    log::debug!("ignoring unknown frame: {}", frame.verb);
                }
            }
        }
    }
}
