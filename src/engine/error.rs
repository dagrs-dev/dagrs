//! Errors that may be raised by building and running dag jobs.

use thiserror::Error;

use crate::parser::ParserError;
use crate::task::RunningError;

#[derive(Debug, Error)]
/// A synthesis of all possible errors.
pub enum DagError {
    /// Error that occurs when running action.
    #[error("{0}")]
    RunningError(RunningError),
    /// Yaml file parsing error.
    #[error("{0}")]
    YamlParserError(ParserError),
    /// Task dependency error.
    #[error("Task[{0}] dependency task not exist.")]
    RelyTaskIllegal(String),
    /// There are loops in task dependencies.
    #[error("Illegal directed a cyclic graph, loop Detect!")]
    LoopGraph,
    /// There are no tasks in the job.
    #[error("There are no tasks in the job.")]
    EmptyJob,
}

impl From<ParserError> for DagError {
    fn from(value: ParserError) -> Self {
        Self::YamlParserError(value)
    }
}

impl From<RunningError> for DagError {
    fn from(value: RunningError) -> Self {
        Self::RunningError(value)
    }
}
