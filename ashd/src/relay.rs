use std::sync::Arc;
use std::time::Duration;

use tokio::sync::mpsc;
use tokio::time::interval;

use crate::config::TelemetryRelayConfig;
use crate::sinks::kafka::KafkaSink;
use crate::sinks::splunk::SplunkSink;
use crate::sinks::TelemetrySink;

pub struct RelayPipeline {
    relay_tx: mpsc::Sender<String>,
}

impl RelayPipeline {
    pub fn new(config: Option<&TelemetryRelayConfig>) -> Arc<Self> {
        let mut sinks: Vec<Box<dyn TelemetrySink + Send + Sync>> = Vec::new();
        let mut batch_size = 100usize;
        let mut flush_interval = Duration::from_millis(5000);
        let mut has_sinks = false;

        if let Some(cfg) = config {
            if let Some(kafka_cfg) = &cfg.kafka {
                if kafka_cfg.enabled {
                    match KafkaSink::new(kafka_cfg) {
                        Ok(sink) => {
                            if !has_sinks {
                                batch_size = kafka_cfg.batch_size;
                                flush_interval = Duration::from_millis(kafka_cfg.flush_interval_ms);
                            }
                            log::info!("kafka sink enabled (topic={}, brokers={:?})", kafka_cfg.topic, kafka_cfg.brokers);
                            sinks.push(Box::new(sink));
                            has_sinks = true;
                        }
                        Err(e) => log::warn!("failed to create Kafka sink: {}", e),
                    }
                }
            }

            if let Some(splunk_cfg) = &cfg.splunk {
                if splunk_cfg.enabled {
                    match SplunkSink::new(splunk_cfg) {
                        Ok(sink) => {
                            if !has_sinks {
                                batch_size = splunk_cfg.batch_size;
                                flush_interval = Duration::from_millis(splunk_cfg.flush_interval_ms);
                            }
                            log::info!("splunk sink enabled (endpoint={})", splunk_cfg.endpoint);
                            sinks.push(Box::new(sink));
                            has_sinks = true;
                        }
                        Err(e) => log::warn!("failed to create Splunk sink: {}", e),
                    }
                }
            }
        }

        let (relay_tx, mut relay_rx) = mpsc::channel::<String>(1024);

        if has_sinks {
            tokio::spawn(async move {
                let mut batch = Vec::new();
                let mut interval = interval(flush_interval);
                interval.tick().await;

                loop {
                    tokio::select! {
                        Some(body) = relay_rx.recv() => {
                            batch.push(body);
                            if batch.len() >= batch_size {
                                deliver_to_all(&batch, &sinks).await;
                                batch.clear();
                            }
                        }
                        _ = interval.tick() => {
                            if !batch.is_empty() {
                                deliver_to_all(&batch, &sinks).await;
                                batch.clear();
                            }
                        }
                        else => {
                            if !batch.is_empty() {
                                deliver_to_all(&batch, &sinks).await;
                            }
                            break;
                        }
                    }
                }
            });
        } else {
            drop(relay_rx);
        }

        Arc::new(RelayPipeline { relay_tx })
    }

    pub fn forward(&self, body: &str) {
        if let Err(e) = self.relay_tx.try_send(body.to_string()) {
            log::warn!("relay channel full or closed, dropping event: {}", e);
        }
    }
}

async fn deliver_to_all(batch: &[String], sinks: &[Box<dyn TelemetrySink + Send + Sync>]) {
    for sink in sinks {
        if let Err(e) = sink.deliver(batch).await {
            log::warn!("sink":% = sink.name(), "events_dropped":% = batch.len(), "error":% = e; "relay delivery failed");
        }
    }
}
