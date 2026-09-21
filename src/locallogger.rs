use std::io::{self, Write};

use log::{Record, Level, Metadata};

pub struct LocalLogger {
    pub log_level: Level,
}

impl LocalLogger {
    pub fn setup() -> Result<(), Box<dyn std::error::Error>> {
        let log_level = match std::env::var("LOG_LEVEL") {
            Ok(value) => match value.to_lowercase().as_str() {
                "trace" => log::Level::Trace,
                "debug" => log::Level::Debug,
                "info" => log::Level::Info,
                "warn" => log::Level::Warn,
                "warning" => log::Level::Warn,
                "error" => log::Level::Error,
                _ => log::Level::Info,
            }
            Err(_) => log::Level::Info,
        };
        let logger = Self { log_level };
        let log_level_filter = logger.log_level.to_level_filter();

        log::set_boxed_logger(Box::new(logger))
            .map(move |()| log::set_max_level(log_level_filter))
            .map_err(|e| format!("failed to install local logger: {e}"))?;

        Ok(())
    }
}

impl log::Log for LocalLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= self.log_level
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            println!("[{}] {}", record.level(), record.args());
        }
    }

    fn flush(&self) {
        let _ = io::stdout().lock().flush();
        let _ = io::stderr().lock().flush();
    }
}
