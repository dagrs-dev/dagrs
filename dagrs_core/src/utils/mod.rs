//! general tool.
//!
//! # dagrs tool module.
//!
//! This module contains common tools for the program, such as: loggers, environment
//! variables, task generation macros.

#[macro_use]
pub mod gen_macro;
mod env;

#[cfg(feature = "logger")]
mod default_logger;
pub mod log;

pub use self::env::EnvVar;
pub use self::log::{LogLevel, Logger};
