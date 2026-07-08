use std::fs::OpenOptions;
use std::io::Write;
use std::sync::Mutex;

use log::{Level, LevelFilter, Log, Metadata, Record, SetLoggerError};

use crate::util::lock_guard;

struct AshLogger {
    file: Mutex<std::fs::File>,
    level: LevelFilter,
}

impl Log for AshLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= self.level
    }

    fn log(&self, record: &Record) {
        if !self.enabled(record.metadata()) {
            return;
        }
        let level = match record.level() {
            Level::Error => "error",
            Level::Warn => "warn",
            Level::Info => "info",
            Level::Debug => "debug",
            Level::Trace => "trace",
        };
        let timestamp = crate::runtime::date::timestamp_now();
        let module = record.module_path().unwrap_or("<unknown>");
        let mut file = lock_guard(&self.file);
        let _ = writeln!(file, "{} [{}] {} — {}", timestamp, level, module, record.args());
    }

    fn flush(&self) {
        let _ = lock_guard(&self.file).flush();
    }
}

fn parse_level(s: &str) -> LevelFilter {
    match s.trim().to_lowercase().as_str() {
        "error" => LevelFilter::Error,
        "warn" | "warning" => LevelFilter::Warn,
        "info" => LevelFilter::Info,
        "debug" => LevelFilter::Debug,
        "trace" => LevelFilter::Trace,
        _ => LevelFilter::Warn,
    }
}

pub fn init() -> Result<(), SetLoggerError> {
    let level_str = std::env::var("ASH_LOG").unwrap_or_default();
    let level = parse_level(&level_str);
    let log_file = std::env::var("ASH_LOG_FILE").unwrap_or_else(|_| "ash.log".to_string());
    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_file)
        .expect("failed to open log file");

    let logger = AshLogger {
        file: Mutex::new(file),
        level,
    };

    log::set_boxed_logger(Box::new(logger))?;
    log::set_max_level(level);
    Ok(())
}
