use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

use crate::AshError;
use super::event::TelemetryEvent;

pub struct FileStore {
    path: std::path::PathBuf,
}

impl FileStore {
    pub fn new(path: &Path) -> Self {
        FileStore {
            path: path.to_path_buf(),
        }
    }

    pub fn reader(&self, offset: u64) -> Result<FileReader, AshError> {
        let file = File::open(&self.path)
            .map_err(|e| AshError::Msg(format!("failed to open telemetry file: {}", e)))?;
        let mut reader = FileReader {
            reader: BufReader::new(file),
            pos: 0,
        };
        if offset > 0 {
            reader
                .reader
                .seek_relative(offset as i64)
                .map_err(|e| AshError::Msg(format!("failed to seek: {}", e)))?;
            reader.pos = offset;
        }
        Ok(reader)
    }
}

pub struct FileReader {
    reader: BufReader<File>,
    pos: u64,
}

impl FileReader {
    pub fn read_line(&mut self) -> Result<Option<TelemetryEvent>, AshError> {
        let mut line = String::new();
        let n = self
            .reader
            .read_line(&mut line)
            .map_err(|e| AshError::Msg(format!("read error: {}", e)))?;
        if n == 0 {
            return Ok(None);
        }
        let line = line.trim_end();
        if line.is_empty() {
            return Ok(None);
        }
        self.pos += n as u64;
        let value: serde_json::Value =
            serde_json::from_str(line).map_err(|e| AshError::Msg(format!("json parse error: {}", e)))?;
        let trace_id = value["trace_id"].as_u64().unwrap_or(0);
        let span_id = value["span_id"].as_u64().unwrap_or(0);
        let parent_span_id = value["parent_span_id"].as_u64();
        let origin = value["origin"].as_str().unwrap_or("").to_string();
        let timestamp = value["timestamp"].as_u64().unwrap_or(0) as u128;
        let kind_str = value["kind"].as_str().unwrap_or("");
        let payload = value["payload"].clone();

        let ctx = crate::telemetry::context::SpanContext {
            trace_id,
            span_id,
            parent_span_id,
            origin,
        };
        let kind = crate::telemetry::event::parse_kind(kind_str);

        Ok(Some(TelemetryEvent {
            ctx,
            timestamp,
            kind,
            payload,
        }))
    }

    pub fn position(&self) -> u64 {
        self.pos
    }
}
