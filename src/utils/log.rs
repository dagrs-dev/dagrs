//! logger for dagrs.
//!
//! # The Logger
//!
//! [`Logger`] is a log programming interface provided for users. The framework implements
//! a default logger. If users want to customize the logger, they can
//! implement the [`Logger`] trait.
//! There are five log levels: Debug is the highest level, and Off is the lowest level.
//! Before dagrs is executed, the user should first specify the logging level. The level
//! of the logging function called will be compared with the currently specified log level.
//! If it is greater than the currently specified If the log level is set, the log information
//! will not be recorded, otherwise the record will be printed to the specified location.
//! Logs are generally recorded in two locations, which are printed on the terminal or output
//! to a file, which needs to be specified by the user.

use std::{
    fmt::Display,
    fs::File,
    io::Write,
    sync::{Arc, Mutex, OnceLock},
};

use thiserror::Error;

/// Log level.
#[derive(Clone, Copy, Debug)]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
    Off,
}

/// Log interface.
pub trait Logger {
    /// Returns the current log level.
    fn level(&self) -> LogLevel;
    /// Record debug information.
    fn debug(&self, msg: String);
    /// Record info information.
    fn info(&self, msg: String);
    /// Record warn information.
    fn warn(&self, msg: String);
    /// Record error information.
    fn error(&self, msg: String);
}

/// Default logger.
struct DefaultLogger {
    level: LogLevel,
    log_pos: Option<Mutex<File>>,
}

impl LogLevel {
    /// Used to filter log information.
    /// This function may be used when the user defines a logger. This function will compare the
    /// log level of the information to be recorded with the level of the current logger.
    /// If the function returns false, it means that the log should not be recorded.
    pub fn check_level(&self, other: Self) -> bool {
        let self_level = Self::map_to_number_level(*self);
        let other_level = Self::map_to_number_level(other);
        self_level >= other_level
    }

    /// In order to facilitate the comparison of log levels, the log levels are mapped to numbers 1~5.
    fn map_to_number_level(level: LogLevel) -> usize {
        match level {
            LogLevel::Debug => 5,
            LogLevel::Info => 4,
            LogLevel::Warn => 3,
            LogLevel::Error => 2,
            LogLevel::Off => 1,
        }
    }
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

/// Logger instance.
static LOG: OnceLock<Arc<dyn Logger + Sync + Send + 'static>> = OnceLock::new();

#[derive(Debug, Error)]
pub enum LoggerError {
    #[error("Logger has been already initialized!")]
    AlreadyInitialized,
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
pub fn init_logger(fix_log_level: LogLevel, log_file: Option<File>) -> Result<(), LoggerError> {
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

/// Initialize a custom logger. Users implement the [`Logger`] trait to implement logging logic.
/// For custom loggers, please refer to `custom_log` in the example.
/// /// # Example
///
/// ```rust
/// use dagrs::{log,LogLevel};
/// let _initialized = log::init_logger(LogLevel::Info,None);
/// log::info("some message.".to_string())
/// ```

pub fn init_custom_logger(logger: impl Logger + Send + Sync + 'static) -> Result<(), LoggerError> {
    if LOG.set(Arc::new(logger)).is_err() {
        return Err(LoggerError::AlreadyInitialized);
    }
    Ok(())
}

/// The following `debug`, `info`, `warn`, and `error` functions are the recording functions
/// provided by the logger for users.

pub fn debug(msg: String) {
    let logger = get_logger();
    if logger.level().check_level(LogLevel::Debug) {
        logger.debug(msg);
    }
}

pub fn info(msg: String) {
    let logger = get_logger();
    if logger.level().check_level(LogLevel::Info) {
        logger.info(msg);
    }
}

pub fn warn(msg: String) {
    let logger = get_logger();
    if logger.level().check_level(LogLevel::Warn) {
        logger.warn(msg);
    }
}

pub fn error(msg: String) {
    let logger = get_logger();
    if logger.level().check_level(LogLevel::Error) {
        logger.error(msg);
    }
}

fn get_logger() -> Arc<dyn Logger + Send + Sync + 'static> {
    LOG.get().expect("Logger is not initialized!").clone()
}
