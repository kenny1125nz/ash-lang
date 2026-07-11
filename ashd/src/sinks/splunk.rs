use async_trait::async_trait;
use anyhow::Result;
use reqwest::Client;
use serde::Serialize;
use std::time::Duration;

use crate::config::SplunkConfig;
use super::TelemetrySink;

#[derive(Serialize)]
struct SplunkEvent {
    event: String,
    sourcetype: String,
    time: u64,
}

pub struct SplunkSink {
    client: Client,
    endpoint: String,
    token: String,
}

impl SplunkSink {
    pub fn new(config: &SplunkConfig) -> Result<Self> {
        let token = resolve_token(&config.token)?;

        let client = Client::builder()
            .timeout(Duration::from_secs(10))
            .build()?;

        Ok(SplunkSink {
            client,
            endpoint: config.endpoint.trim_end_matches('/').to_string(),
            token,
        })
    }
}

fn resolve_token(raw: &str) -> Result<String> {
    if raw.starts_with("${") && raw.ends_with('}') {
        let var_name = &raw[2..raw.len() - 1];
        std::env::var(var_name).map_err(|_| {
            anyhow::anyhow!("environment variable {} not set for Splunk token", var_name)
        })
    } else {
        Ok(raw.to_string())
    }
}

#[async_trait]
impl TelemetrySink for SplunkSink {
    async fn deliver(&self, events: &[String]) -> Result<()> {
        let url = format!("{}/services/collector/event", self.endpoint);
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let mut body = String::new();
        for event in events {
            let splunk_event = SplunkEvent {
                event: event.clone(),
                sourcetype: "ash_telemetry".to_string(),
                time: now,
            };
            let json = serde_json::to_string(&splunk_event)?;
            body.push_str(&json);
            body.push('\n');
        }

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Splunk {}", self.token))
            .header("Content-Type", "application/json")
            .body(body)
            .send()
            .await?;

        if response.status().is_success() {
            Ok(())
        } else {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            Err(anyhow::anyhow!("splunk returned {}: {}", status, text))
        }
    }

    fn name(&self) -> &str {
        "splunk"
    }
}
