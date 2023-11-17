//! The default logger implementation when the `logger` feature is enabled.

use std::{
    fmt::Display,
    fs::File,
    io::Write,
    sync::{Arc, Mutex},
};

use crate::{
    log::{LoggerError, LOG},
    LogLevel, Logger,
};

/// Default logger.
pub(crate) struct DefaultLogger {
    level: LogLevel,
    log_pos: Option<Mutex<File>>,
}

impl DefaultLogger {
    fn log(&self, msg: String) {
        match self.log_pos {
            Some(ref file) => {
                let mut file = file.lock().unwrap();
                let _ = writeln!(file, "{}", msg);
            }
            None => {
                println!("{}", msg);
            }
        }
    }
}

impl Logger for DefaultLogger {
    fn level(&self) -> LogLevel {
        self.level
    }

    fn debug(&self, msg: String) {
        self.log(msg)
    }

    fn info(&self, msg: String) {
        self.log(msg)
    }

    fn warn(&self, msg: String) {
        self.log(msg)
    }

    fn error(&self, msg: String) {
        self.log(msg)
    }
}

impl Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LogLevel::Debug => write!(f, "Debug"),
            LogLevel::Info => write!(f, "Info"),
            LogLevel::Warn => write!(f, "warn"),
            LogLevel::Error => write!(f, "error"),
            LogLevel::Off => write!(f, "off"),
        }
    }
}

/// Initialize the default logger, the user needs to specify the logging level of the logger,
/// and can also specify the location of the log output, if the log_file parameter is passed in
/// None, the log information will be printed to the terminal, otherwise, the log information
/// will be output to the file.
///
/// # Example
///
/// ```rust
/// use dagrs::{log, LogLevel};
/// let _initialized = log::init_logger(LogLevel::Info,None);
/// ```
pub(crate) fn init_default_logger(
    fix_log_level: LogLevel,
    log_file: Option<File>,
) -> Result<(), LoggerError> {
    let logger = match log_file {
        Some(file) => DefaultLogger {
            level: fix_log_level,
            log_pos: Some(Mutex::new(file)),
        },
        None => DefaultLogger {
            level: fix_log_level,
            log_pos: None,
        },
    };
    if LOG.set(Arc::new(logger)).is_err() {
        return Err(LoggerError::AlreadyInitialized);
    }
    Ok(())
}

pub(crate) fn get_logger() -> Arc<dyn Logger + Send + Sync + 'static> {
    LOG.get().expect("Logger is not initialized!").clone()
}

/// The following `debug`, `info`, `warn`, and `error` functions are the recording functions
/// provided by the logger for users.

pub(crate) fn default_debug(msg: String) {
    let logger = get_logger();
    if logger.level().check_level(LogLevel::Debug) {
        logger.debug(msg);
    }
}

pub(crate) fn default_info(msg: String) {
    let logger = get_logger();
    if logger.level().check_level(LogLevel::Info) {
        logger.info(msg);
    }
}

pub(crate) fn default_warn(msg: String) {
    let logger = get_logger();
    if logger.level().check_level(LogLevel::Warn) {
        logger.warn(msg);
    }
}

pub(crate) fn default_error(msg: String) {
    let logger = get_logger();
    if logger.level().check_level(LogLevel::Error) {
        logger.error(msg);
    }
}
