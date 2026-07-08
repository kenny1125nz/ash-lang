use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct TelemetryConfig {
    /// Global filter applied before writing to disk.
    pub filter: Option<String>,
    /// Local file configuration. Required for telemetry to be active.
    pub file: Option<FileConfig>,
    /// At most one remote adapter.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kafka: Option<RemoteAdapterConfig>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub splunk: Option<RemoteAdapterConfig>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct FileConfig {
    #[serde(default = "default_file_path")]
    pub path: String,
    #[serde(default = "default_max_size_mb", skip_serializing_if = "is_default_max_size_mb")]
    pub max_size_mb: u64,
    #[serde(default = "default_max_files", skip_serializing_if = "is_default_max_files")]
    pub max_files: u32,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct RemoteAdapterConfig {
    #[serde(default)]
    pub enabled: bool,
    pub filter: Option<String>,
    /// Target-specific endpoint / brokers / topic.
    #[serde(default)]
    pub options: std::collections::HashMap<String, String>,
    #[serde(default = "default_batch_size")]
    pub batch_size: usize,
    #[serde(default = "default_flush_interval")]
    pub flush_interval_ms: u64,
}

impl Default for FileConfig {
    fn default() -> Self {
        FileConfig {
            path: default_file_path(),
            max_size_mb: default_max_size_mb(),
            max_files: default_max_files(),
        }
    }
}

fn default_file_path() -> String {
    "ash-telemetry.jsonl".to_string()
}

fn default_max_size_mb() -> u64 {
    100
}

fn default_max_files() -> u32 {
    7
}

fn default_batch_size() -> usize {
    100
}

fn default_flush_interval() -> u64 {
    5000
}

fn is_default_max_size_mb(v: &u64) -> bool {
    *v == 100
}

fn is_default_max_files(v: &u32) -> bool {
    *v == 7
}
