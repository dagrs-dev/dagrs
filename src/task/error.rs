//! There is an error about task runtime.

use std::fmt::Display;

use thiserror::Error;

/// Errors that may be generated when the specific behavior of the task is run.
/// This is just a simple error handling. When running the tasks in the configuration file,
/// some errors can be found by the user, which is convenient for debugging.
/// It also allows users to return expected errors in custom task behavior. However, even
/// if this error is expected, it will cause the execution of the entire task to fail.
#[derive(Debug)]
pub struct RunningError {
    msg: String,
}

/// Sh script produces incorrect behavior when run.
#[derive(Error, Debug)]
pub struct ShExecuteError {
    msg: String,
    #[source]
    err: std::io::Error,
}

/// Javascript script produces incorrect behavior when run.
#[derive(Error, Debug)]
pub enum JavaScriptExecuteError {
    #[error("{0}")]
    AnyHowError(deno_core::anyhow::Error),
    #[error("{0}")]
    SerializeError(deno_core::serde_v8::Error),
}

impl RunningError {
    pub fn new(msg: String) -> Self {
        Self { msg }
    }
    pub fn from_err<T: Display>(err: T) -> Self {
        Self {
            msg: err.to_string(),
        }
    }
}

impl Display for RunningError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.msg)
    }
}

impl ShExecuteError {
    pub fn new(msg: String, err: std::io::Error) -> Self {
        Self { msg, err }
    }
}

impl Display for ShExecuteError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "sh script execution error: {}\n,{}", self.msg, self.err)
    }
}

impl From<ShExecuteError> for RunningError {
    fn from(value: ShExecuteError) -> Self {
        RunningError { msg: value.to_string() }
    }
}

impl From<JavaScriptExecuteError> for RunningError {
    fn from(value: JavaScriptExecuteError) -> Self {
        RunningError { msg: value.to_string() }
    }
}

