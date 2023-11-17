//! general tool.
//!
//! This module contains common tools for the program, such as: loggers, environment
//! variables, task generation macros.

#[cfg(feature = "logger")]
mod default_logger;
mod env;
pub mod log;
mod parser;

pub use self::env::EnvVar;
pub use self::log::{LogLevel, Logger};
pub use self::parser::{ParseError, Parser};
