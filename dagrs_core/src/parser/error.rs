//! Errors that may occur during configuration file parsing.

use thiserror::Error;

/// Errors that may occur while parsing task configuration files.
#[derive(Debug, Error)]
pub enum ParserError {
    /// Configuration file not found.
    #[error("File not found. [{0}]")]
    FileNotFound(#[from] std::io::Error),
    #[error("{0}")]
    YamlTaskError(YamlTaskError),
    #[error("{0}")]
    FileContentError(FileContentError),
}

/// Error about file information.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum FileContentError {
    /// The format of the yaml configuration file is not standardized.
    #[error("{0}")]
    IllegalYamlContent(#[from] yaml_rust::ScanError),
    /// Config file has no content.
    #[error("File is empty! [{0}]")]
    Empty(String),
}

/// Errors about task configuration items.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum YamlTaskError {
    /// The configuration file should start with `dagrs:`.
    #[error("File content is not start with 'dagrs'.")]
    StartWordError,
    /// No task name configured.
    #[error("Task has no name field. [{0}]")]
    NoNameAttr(String),
    /// The specified task predecessor was not found.
    #[error("Task cannot find the specified predecessor. [{0}]")]
    NotFoundPrecursor(String),
    /// `script` is not defined.
    #[error("The 'script' attribute is not defined. [{0}]")]
    NoScriptAttr(String),
}

impl From<FileContentError> for ParserError {
    fn from(value: FileContentError) -> Self {
        ParserError::FileContentError(value)
    }
}

impl From<YamlTaskError> for ParserError {
    fn from(value: YamlTaskError) -> Self {
        ParserError::YamlTaskError(value)
    }
}

