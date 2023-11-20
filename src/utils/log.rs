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
    fmt::{Debug, Display},
    fs::File,
    sync::{Arc, OnceLock},
};

#[cfg(feature = "logger")]
use super::default_logger::{
    default_debug, default_error, default_info, default_warn, init_default_logger,
};

/// Log level.
#[derive(Clone, Copy, Debug)]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
    Off,
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

#[derive(Debug)]
pub enum LoggerError {
    AlreadyInitialized,
}

impl Display for LoggerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Logger has been already initialized!")
    }
}

/// Logger instance.
pub(crate) static LOG: OnceLock<Arc<dyn Logger + Sync + Send>> = OnceLock::new();

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
#[allow(unused_variables)]
pub fn init_logger(fix_log_level: LogLevel, log_file: Option<File>) -> Result<(), LoggerError> {
    #[cfg(feature = "logger")]
    {
        init_default_logger(fix_log_level, log_file)
    }
    #[cfg(not(feature = "logger"))]
    {
        Ok(())
    }
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
    init_custom_logger_dyn(Arc::new(logger))
}

pub fn init_custom_logger_dyn(logger: Arc<dyn Logger + Send + Sync>) -> Result<(), LoggerError> {
    if LOG.set(logger).is_err() {
        return Err(LoggerError::AlreadyInitialized);
    }
    Ok(())
}

/// The following `debug`, `info`, `warn`, and `error` functions are the recording functions
/// provided by the logger for users.

#[allow(unused_variables)]
pub fn debug(msg: String) {
    #[cfg(feature = "logger")]
    {
        default_debug(msg);
    }
}

#[allow(unused_variables)]
pub fn info(msg: String) {
    #[cfg(feature = "logger")]
    {
        default_info(msg);
    }
}

#[allow(unused_variables)]
pub fn warn(msg: String) {
    #[cfg(feature = "logger")]
    {
        default_warn(msg);
    }
}

#[allow(unused_variables)]
pub fn error(msg: String) {
    #[cfg(feature = "logger")]
    {
        default_error(msg);
    }
}
