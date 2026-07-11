use chrono::{DateTime, Utc};
use std::path::PathBuf;

use crate::config::WorkflowConfig;

#[derive(Debug, Clone)]
pub struct WorkflowEvent {
    pub source_name: String,
    pub event_id: String,
    pub path: PathBuf,
    pub timestamp: DateTime<Utc>,
    pub workflow: WorkflowConfig,
}
