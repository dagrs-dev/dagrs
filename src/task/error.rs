//! !TODO()
//!

use std::fmt::Display;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum RunningError {
    #[error("A recoverable error was encountered. [{0}]")]
    RecoverableError(String),
    #[error("Encountered an unrecoverable error! [{0}]")]
    UnRecoverableError(String),
}

#[derive(Error, Debug)]
pub struct ShExecuteError {
    msg: String,
    #[source]
    err: std::io::Error,
}

#[derive(Error, Debug)]
pub enum JavaScriptExecuteError{
    #[error("{0}")]
    AnyHowError(deno_core::anyhow::Error),
    #[error("{0}")]
    SerializeError(deno_core::serde_v8::Error)
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

impl Into<RunningError> for ShExecuteError {
    fn into(self) -> RunningError {
        RunningError::UnRecoverableError(self.msg)
    }
}

impl Into<RunningError> for JavaScriptExecuteError{
    fn into(self) -> RunningError {
        RunningError::UnRecoverableError(self.to_string())
    }
}