//! A simple error handler implementation.

use thiserror::Error;

#[derive(Debug, Error)]
/// A synthesis of all possible errors.
pub enum DagError {
    /// IO Error, like file not exist, etc.
    #[error("{0}")]
    IOError(#[from] std::io::Error),
    /// YAML Error, like format error, etc.
    #[error("{0}")]
    YamlError(YamlError),
    /// Error that occurs when running dagrs.
    #[error("{0}")]
    RunningError(RunningError),
}

#[derive(Debug, Error)]
/// Format Error, point out which part has what kinds of error.
pub enum YamlFormatError {
    #[error("Not start with 'dagrs'")]
    StartWordError,
    #[error("Task[{0}] has no name field")]
    NoName(String),
    #[error("Task[{0}] run script format error")]
    RunScriptError(String)
}

#[derive(Debug, Error)]
/// Error that occurs when parsing YAML file.
pub enum YamlError {
    #[error("{0}")]
    YamlParserError(#[from] yaml_rust::ScanError),
    #[error("{0}")]
    YamlFormatError(YamlFormatError),
}

#[derive(Debug, Error)]
/// Error that occurs when running dagrs
pub enum RunningError {
    #[error("Task[{0}] dependency task not exist")]
    RelyTaskIllegal(String),
    #[error("Task[{0}] run script fails, details: {1}")]
    RunScriptFailure(String, String)
}

impl DagError {
    /// Throw a format error
    ///
    /// # Example
    /// ```
    /// DagError::format_error(YamlFormatError::NoName("a".to_string()));
    /// ```
    /// This will throw a error that says, yaml task 'a' has no name field.
    pub fn format_error(error: YamlFormatError) -> Self {
        Self::YamlError(YamlError::YamlFormatError(error))
    }

    /// Throw a running error
    ///
    /// # Example
    /// ```
    /// DagError::running_error(RunningError::RelyTaskIllegal("task 1".to_string()))
    /// ```
    /// This will throw a error that says, task with name "task 1" has non-exist rely tasks.
    pub fn running_error(error: RunningError) -> Self {
        Self::RunningError(error)
    }
}

impl From<yaml_rust::ScanError> for DagError {
    fn from(e: yaml_rust::ScanError) -> Self {
        Self::YamlError(YamlError::YamlParserError(e))
    }
}