//! Use the simplelog and log libraries to implement the Logger trait to customize the log manager.

extern crate dagrs;
extern crate log;
extern crate simplelog;

use dagrs::{Dag, LogLevel, Logger};
use simplelog::*;

struct MyLogger {
    level: LogLevel,
}

impl MyLogger {
    /// Create MyLogger and set the log level of simplelog.
    fn new(level: LogLevel) -> Self {
        let filter = match level {
            LogLevel::Debug => LevelFilter::Debug,
            LogLevel::Info => LevelFilter::Info,
            LogLevel::Warn => LevelFilter::Warn,
            LogLevel::Error => LevelFilter::Error,
            LogLevel::Off => LevelFilter::Off,
        };
        CombinedLogger::init(vec![TermLogger::new(
            filter,
            Config::default(),
            TerminalMode::Mixed,
            ColorChoice::Auto,
        )])
        .unwrap();
        MyLogger { level }
    }
}

/// In turn, use the corresponding logging macro of log to override the Logger method.
impl Logger for MyLogger {
    fn level(&self) -> LogLevel {
        self.level
    }

    fn debug(&self, msg: String) {
        log::debug!("{}", msg);
    }

    fn info(&self, msg: String) {
        log::info!("{}", msg);
    }

    fn warn(&self, msg: String) {
        log::warn!("{}", msg);
    }

    fn error(&self, msg: String) {
        log::error!("{}", msg);
    }
}

fn main() {
    // Initialize the global logger with a custom logger.
    dagrs::log::init_custom_logger(MyLogger::new(LogLevel::Info));
    let mut dag = Dag::with_yaml("tests/config/correct.yaml").unwrap();
    assert!(dag.start().unwrap());
}
