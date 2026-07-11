use async_trait::async_trait;
use anyhow::Result;
use rdkafka::producer::{FutureProducer, FutureRecord};
use rdkafka::ClientConfig;
use std::time::Duration;
use uuid::Uuid;

use crate::config::KafkaConfig;
use super::TelemetrySink;

pub struct KafkaSink {
    producer: FutureProducer,
    topic: String,
}

impl KafkaSink {
    pub fn new(config: &KafkaConfig) -> Result<Self> {
        let mut client_config = ClientConfig::new();
        client_config.set("bootstrap.servers", config.brokers.join(","));
        client_config.set("message.timeout.ms", "5000");
        client_config.set("queue.buffering.max.ms", "10");

        let producer: FutureProducer = client_config.create()?;

        Ok(KafkaSink {
            producer,
            topic: config.topic.clone(),
        })
    }
}

#[async_trait]
impl TelemetrySink for KafkaSink {
    async fn deliver(&self, events: &[String]) -> Result<()> {
        let timeout = Duration::from_secs(5);
        let mut any_success = false;

        for event in events {
            let key = Uuid::new_v4().to_string();
            let record = FutureRecord::to(&self.topic)
                .payload(event)
                .key(&key);

            match self.producer.send(record, timeout).await {
                Ok((_partition, _offset)) => {
                    any_success = true;
                }
                Err((e, _)) => {
                    log::warn!("kafka send failed: {}", e);
                }
            }
        }

        if any_success {
            Ok(())
        } else {
            Err(anyhow::anyhow!("all kafka sends failed"))
        }
    }

    fn name(&self) -> &str {
        "kafka"
    }
}
