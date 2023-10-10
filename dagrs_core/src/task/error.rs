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

/// command produces incorrect behavior when run.
#[derive(Error, Debug)]
pub struct CmdExecuteError {
    msg: String,
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

impl CmdExecuteError {
    pub fn new(msg: String) -> Self {
        Self { msg }
    }
}

impl Display for CmdExecuteError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "cmd execution error: {}", self.msg)
    }
}

impl From<CmdExecuteError> for RunningError {
    fn from(value: CmdExecuteError) -> Self {
        RunningError {
            msg: value.to_string(),
        }
    }
}
