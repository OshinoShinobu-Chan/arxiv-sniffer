//! Simple colored logger.

use chrono::Local;
use std::sync::{Mutex, OnceLock};

fn log_output_lock() -> &'static Mutex<()> {
    static LOG_OUTPUT_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOG_OUTPUT_LOCK.get_or_init(|| Mutex::new(()))
}

/// Log level for output.
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

impl LogLevel {
    fn label(&self) -> &'static str {
        match self {
            Self::Debug => "DEBUG",
            Self::Info => "INFO",
            Self::Warn => "WARN",
            Self::Error => "ERROR",
        }
    }

    fn ansi_color(&self) -> &'static str {
        match self {
            Self::Debug => "36",
            Self::Info => "32",
            Self::Warn => "33",
            Self::Error => "31",
        }
    }
}

/// Print one log line in format: [LEVEL]yyyy-mm-ddThh:mm:ss.sss+08:00 - message
pub fn log(level: LogLevel, message: impl AsRef<str>) {
    let _guard = log_output_lock().lock().unwrap_or_else(|e| e.into_inner());
    let ts = Local::now().format("%Y-%m-%dT%H:%M:%S%.3f%:z");
    let level_text = format!("[{}]", level.label());
    let colored_level = format!("\x1b[{}m{}\x1b[0m", level.ansi_color(), level_text);
    let line = format!("{}{} - {}", colored_level, ts, message.as_ref());

    match level {
        LogLevel::Error => eprintln!("{}", line),
        _ => println!("{}", line),
    }
}

pub fn debug(message: impl AsRef<str>) {
    log(LogLevel::Debug, message);
}

pub fn info(message: impl AsRef<str>) {
    log(LogLevel::Info, message);
}

pub fn warn(message: impl AsRef<str>) {
    log(LogLevel::Warn, message);
}

pub fn error(message: impl AsRef<str>) {
    log(LogLevel::Error, message);
}
