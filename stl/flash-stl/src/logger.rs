//! Structured logging for Flash apps.

/// Log levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

/// Log a message at the given level.
pub fn log(level: LogLevel, message: &str) {
    let prefix = match level {
        LogLevel::Debug => "DEBUG",
        LogLevel::Info => "INFO",
        LogLevel::Warn => "WARN",
        LogLevel::Error => "ERROR",
    };
    eprintln!("[flash:{}] {}", prefix, message);
}

/// Log a debug message.
pub fn debug(message: &str) {
    log(LogLevel::Debug, message);
}

/// Log an info message.
pub fn info(message: &str) {
    log(LogLevel::Info, message);
}

/// Log a warning.
pub fn warn(message: &str) {
    log(LogLevel::Warn, message);
}

/// Log an error.
pub fn error(message: &str) {
    log(LogLevel::Error, message);
}

/// Log a labeled value for debugging.
pub fn log_value(level: LogLevel, label: &str, value: &str) {
    log(level, &format!("{} = {}", label, value));
}
