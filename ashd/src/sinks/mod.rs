pub mod kafka;
pub mod splunk;

use async_trait::async_trait;
use anyhow::Result;

#[async_trait]
pub trait TelemetrySink: Send + Sync {
    async fn deliver(&self, events: &[String]) -> Result<()>;
    fn name(&self) -> &str;
}
