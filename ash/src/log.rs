use std::fs::OpenOptions;
use std::io::Write;
use std::sync::Mutex;

use log::{Level, LevelFilter, Log, Metadata, Record, SetLoggerError};

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
        let timestamp = chrono_now();
        let module = record.module_path().unwrap_or("<unknown>");
        let mut file = self.file.lock().unwrap();
        let _ = writeln!(file, "{} [{}] {} — {}", timestamp, level, module, record.args());
    }

    fn flush(&self) {
        let _ = self.file.lock().unwrap().flush();
    }
}

fn chrono_now() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let d = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let secs = d.as_secs();
    let micros = d.subsec_micros();

    let days = secs / 86400;
    let time_secs = secs % 86400;
    let hours = time_secs / 3600;
    let minutes = (time_secs % 3600) / 60;
    let seconds = time_secs % 60;

    let mut y = 1970i64;
    let mut remaining = days as i64;

    loop {
        let days_in_year = if is_leap(y) { 366 } else { 365 };
        if remaining < days_in_year {
            break;
        }
        remaining -= days_in_year;
        y += 1;
    }

    let month_days = if is_leap(y) {
        [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };

    let mut m = 0usize;
    for (i, &md) in month_days.iter().enumerate() {
        if remaining < md {
            m = i;
            break;
        }
        remaining -= md;
    }

    let day = remaining + 1;

    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}.{:06}Z",
        y,
        m + 1,
        day,
        hours,
        minutes,
        seconds,
        micros
    )
}

fn is_leap(year: i64) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
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
