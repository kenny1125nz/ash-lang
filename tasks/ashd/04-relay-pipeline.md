# Phase 4: ashd Telemetry Relay Pipeline

## Context

This is step 4 of 5. Phase 3 has ash processes connecting and sending `TEL:` frames. Phase 2 routed those frames to a stub `RelayPipeline`. Now we replace the stub with real Kafka and Splunk delivery. ash's `deliver_remote()` was already removed in Phase 3.

The relay pipeline in ashd is the remote delivery mechanism. It receives opaque `TEL:` body bytes from the frame router and delivers them to configured sinks.

Design principle from the protocol doc: ashd never parses TEL bodies. They are forwarded as-is. Correlation is embedded in the payload format (trace_id, span_id, etc.), not in the protocol layer.

## Task

### 1. Relay Pipeline (`ashd/src/relay.rs`) — Replace Stub

Replace the Phase 2 stub with:

```rust
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{interval, Duration};

pub struct RelayPipeline {
    sinks: Vec<Box<dyn TelemetrySink + Send + Sync>>,
    batch: Mutex<Vec<String>>,
    batch_size: usize,
    flush_interval: Duration,
}
```

**`RelayPipeline::new(config: Option<&TelemetryRelayConfig>) -> Arc<Self>`:**

1. Collect sinks from config:
   - If `kafka.enabled` → create `KafkaSink`
   - If `splunk.enabled` → create `SplunkSink`
2. Determine batch_size and flush_interval: use the minimum across enabled sinks (or aggregate). Simpler: use a single batch_size and flush_interval per pipeline, taken from the first enabled sink.
3. Spawn a background tokio task that:
   - Runs a flush interval timer
   - On tick: drain `self.batch`, deliver to all sinks
   - On drop: flush remaining events

**`RelayPipeline::forward(&self, body: &str)`:**

- Push `body.to_string()` into `self.batch.lock().await`
- If `batch.len() >= self.batch_size` → drain and deliver to all sinks
- This is async but called from the frame router. If the router is sync, use `tokio::spawn` or convert to a non-async version. Given Phase 2 uses tokio, the router can hold an `Arc<RelayPipeline>` and call `forward()` synchronously by using `tokio::task::block_in_place` or changing to `try_send` on an mpsc channel.

Recommended approach: Use an `mpsc` channel from router to pipeline:

```rust
// In RelayPipeline:
let (relay_tx, mut relay_rx) = mpsc::channel::<String>(1024);

RelayPipeline {
    relay_tx,  // clone for each forwarding call
    // ...
}

// forward() sends on the channel:
pub fn forward(&self, body: &str) {
    let _ = self.relay_tx.try_send(body.to_string());
}

// Background task processes relay_rx:
tokio::spawn(async move {
    let mut batch = vec![];
    let mut interval = interval(flush_interval);
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
        }
    }
});
```

This decouples the router from the relay — the channel absorbs bursts.

### 2. TelemetrySink Trait (`ashd/src/sinks/mod.rs`)

```rust
use async_trait::async_trait;
use anyhow::Result;

#[async_trait]
pub trait TelemetrySink: Send + Sync {
    async fn deliver(&self, events: &[String]) -> Result<()>;
    fn name(&self) -> &str;
}
```

### 3. Kafka Sink (`ashd/src/sinks/kafka.rs`)

```rust
use rdkafka::producer::{FutureProducer, FutureRecord};
use rdkafka::ClientConfig;
use std::time::Duration;

pub struct KafkaSink {
    producer: FutureProducer,
    topic: String,
}
```

**`KafkaSink::new(config: &KafkaConfig) -> Result<Self>`:**
- Build `ClientConfig` from `config.brokers`, set basic properties
- Create `FutureProducer`
- Return `KafkaSink { producer, topic: config.topic.clone() }`

**`deliver()` implementation:**
- For each event in the batch, create `FutureRecord::to(&self.topic).payload(event).key(&uuid)` (UUID key for even partition distribution)
- Send all, collect results
- Log failures, return Ok if ANY succeeded (best-effort delivery)
- Timeout per send: 5 seconds

**Add to `ashd/Cargo.toml`:**
```toml
rdkafka = { version = "0.36", features = ["tokio-compression"] }
async-trait = "0.1"
```

If `rdkafka` is too heavy to compile (native C dependency on librdkafka), use a simpler HTTP-based approach or `kafka` crate. But `rdkafka` is the production standard. Accept the build complexity.

### 4. Splunk Sink (`ashd/src/sinks/splunk.rs`)

```rust
use reqwest::Client;
use std::time::Duration;

pub struct SplunkSink {
    client: Client,
    endpoint: String,
    token: String,
}
```

**`SplunkSink::new(config: &SplunkConfig) -> Result<Self>`:**
- Build `reqwest::Client` with timeout of 10s
- Resolve `${SPLUNK_HEC_TOKEN}` from env vars (if token value is a `${VAR}` pattern)

**`deliver()` implementation:**
- Construct the Splunk HEC JSON payload: `{"event": <each event string>, "sourcetype": "ash_telemetry", "time": <current epoch>}`
- POST to `{endpoint}/services/collector/event` with `Authorization: Splunk {token}` header
- Send all events as a batch (newline-delimited or JSON array, depending on HEC endpoint type)
- Return Ok on 2xx, Err on non-2xx

**Add to `ashd/Cargo.toml`:**
```toml
reqwest = { version = "0.12", features = ["json", "rustls-tls"], default-features = false }
```

### 5. Error Handling

- If a sink is down or returns errors, log the error at `warn` level
- Do NOT crash ashd if delivery fails
- Do NOT block the frame router — the channel decouples delivery from receipt
- If the relay channel is full, `try_send` will fail — this means ashd can't keep up. Log `warn` and drop the event
- Events are NOT retried on delivery failure — ash already writes to local JSONL as the durable log

### 6. Acceptance

1. Configure Kafka in `ashd.yml`, start Kafka locally (or use `KAFKA_BROKERS` env), start ashd, run ash → events appear in Kafka topic
2. Configure Splunk HEC endpoint, start ashd, run ash → events appear in Splunk
3. Both sinks disabled → ashd runs normally, no errors
4. Kafka broker unreachable → ashd starts, logs connection error, continues running
5. Send 200 TEL frames rapidly → batched delivery (observe batch_size grouping in Kafka/Splunk)
6. Kill sink mid-batch → ashd logs warning, does not crash, subsequent events still attempt delivery

### Do NOT do

- Do not parse or validate TEL body contents — always opaque passthrough
- Do not implement persistent queueing for failed deliveries (JSONL in ash is the persistence layer)
- Do not add any new configuration fields beyond what's in the Phase 1 config structs
- Do not implement any other sink types (no HTTP webhook, no gRPC, no stdout sink)
