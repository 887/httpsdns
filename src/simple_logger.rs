extern crate log;

use log::*;

pub struct SimpleLogger;

impl log::Log for SimpleLogger {
    fn enabled(&self, metadata: &LogMetadata) -> bool {
        //metadata.level() <= LogLevel::Trace
        metadata.level() <= LogLevel::Info
    }

    fn log(&self, record: &LogRecord) {
        use chrono::Local;
        // println!("{}: {}",
        // Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        if self.enabled(record.metadata()) {
            println!("{}: {} - {}",
                     Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
                     record.level(),
                     record.args());
        }
    }
}
