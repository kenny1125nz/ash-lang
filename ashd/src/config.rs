use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AshdConfig {
    pub daemon: DaemonConfig,
    #[serde(default)]
    pub sources: Vec<SourceConfig>,
    #[serde(default)]
    pub telemetry_relay: Option<TelemetryRelayConfig>,
    #[serde(default)]
    pub staging_dir: Option<PathBuf>,
}

impl AshdConfig {
    pub fn load() -> Result<Self> {
        let path = resolve_config_path()?;
        let contents = std::fs::read_to_string(&path)
            .with_context(|| format!("failed to read config from {}", path.display()))?;
        let config: AshdConfig = serde_yaml::from_str(&contents)
            .with_context(|| format!("failed to parse config from {}", path.display()))?;
        config.validate()?;
        Ok(config)
    }

    fn validate(&self) -> Result<()> {
        if self.sources.is_empty() {
            return Err(anyhow!("at least one source must be defined"));
        }

        let ws = &self.daemon.ws_listen;
        let is_tcp = ws.parse::<SocketAddr>().is_ok();
        let is_unix = ws.starts_with('/') || ws.starts_with('.');
        if !is_tcp && !is_unix {
            return Err(anyhow!(
                "invalid ws_listen '{}': must be a TCP socket address (e.g. 127.0.0.1:9877) or Unix socket path (e.g. /tmp/ashd.sock)",
                ws
            ));
        }

        if self.daemon.grace_period_secs == 0 {
            return Err(anyhow!("grace_period_secs must be > 0"));
        }

        for source in &self.sources {
            if source.source_type == "folder" && source.path.is_none() {
                return Err(anyhow!(
                    "folder source '{}' must have a 'path' field set",
                    source.name
                ));
            }
            for wf in &source.workflows {
                if !wf.path.exists() {
                    return Err(anyhow!(
                        "workflow path '{}' in source '{}' does not exist",
                        wf.path.display(),
                        source.name
                    ));
                }
            }
        }

        Ok(())
    }
}

fn resolve_config_path() -> Result<PathBuf> {
    if let Ok(path) = std::env::var("ASHD_CONFIG") {
        return Ok(PathBuf::from(path));
    }

    let local = PathBuf::from("ashd.yml");
    if local.exists() {
        return Ok(local);
    }

    if let Ok(xdg) = std::env::var("XDG_CONFIG_HOME") {
        let p = PathBuf::from(xdg).join("ash").join("ashd.yml");
        if p.exists() {
            return Ok(p);
        }
    }

    if let Ok(home) = std::env::var("HOME") {
        let p = PathBuf::from(home).join(".ash").join("ashd.yml");
        if p.exists() {
            return Ok(p);
        }
    }

    Err(anyhow!(
        "no config found: checked ASHD_CONFIG env, ./ashd.yml, ~/.ash/ashd.yml"
    ))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaemonConfig {
    #[serde(default = "default_log_level")]
    pub log_level: String,
    #[serde(default = "default_ws_listen")]
    pub ws_listen: String,
    #[serde(default = "default_grace_period_secs")]
    pub grace_period_secs: u64,
}

fn default_log_level() -> String {
    "info".into()
}

fn default_ws_listen() -> String {
    "127.0.0.1:9877".into()
}

fn default_grace_period_secs() -> u64 {
    30
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceConfig {
    #[serde(rename = "type")]
    pub source_type: String,
    pub name: String,
    pub path: Option<PathBuf>,
    #[serde(default = "default_concurrency")]
    pub concurrency: ConcurrencyMode,
    pub workflows: Vec<WorkflowConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ConcurrencyMode {
    Parallel,
    Sequential,
}

impl Default for ConcurrencyMode {
    fn default() -> Self {
        ConcurrencyMode::Parallel
    }
}

fn default_concurrency() -> ConcurrencyMode {
    ConcurrencyMode::Parallel
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowConfig {
    pub path: PathBuf,
    #[serde(default)]
    pub flags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryRelayConfig {
    #[serde(default)]
    pub kafka: Option<KafkaConfig>,
    #[serde(default)]
    pub splunk: Option<SplunkConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KafkaConfig {
    pub enabled: bool,
    pub brokers: Vec<String>,
    pub topic: String,
    #[serde(default = "default_batch_size")]
    pub batch_size: usize,
    #[serde(default = "default_flush_interval_ms")]
    pub flush_interval_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SplunkConfig {
    pub enabled: bool,
    pub endpoint: String,
    pub token: String,
    #[serde(default = "default_batch_size")]
    pub batch_size: usize,
    #[serde(default = "default_flush_interval_ms")]
    pub flush_interval_ms: u64,
}

fn default_batch_size() -> usize {
    100
}

fn default_flush_interval_ms() -> u64 {
    5000
}
