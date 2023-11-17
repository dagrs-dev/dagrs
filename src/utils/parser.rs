//! Task configuration file parser interface
use crate::{task::Task, Action};
use std::{collections::HashMap, error::Error, fmt::Display};

/// Generic parser traits. If users want to customize the configuration file parser, they must implement this trait.
/// The yaml module's `YamlParser` is an example.
pub trait Parser {
    /// Parses the contents of a configuration file into a series of tasks with dependencies.
    /// Parameter Description:
    /// - file: path information of the configuration file
    /// - specific_actions: When parsing the configuration file, the specific execution logic
    /// of some tasks does not need to be specified in the configuration file, but is given
    /// through this map. In the map's key-value pair, the key represents the unique identifier
    /// of the task in the task's configuration file, and the value represents the execution
    /// logic given by the user.
    ///
    /// Return value description:
    /// If an error is encountered during the parsing process, the return result is ParserError.
    /// Instead, return a series of concrete types that implement the [`Task`] trait.
    /// This may involve user-defined [`Task`], you can refer to `YamlTask` under the yaml module.
    fn parse_tasks(
        &self,
        file: &str,
        specific_actions: HashMap<String, Action>,
    ) -> Result<Vec<Box<dyn Task>>, ParseError>;
}

/// Errors that may occur during configuration file parsing.
/// This structure stores error information. Users need to customize error types and implement conversion
/// from custom error types to [`ParseError`].
/// By default, a conversion from `String` type to [`ParseError`] is provided.
#[derive(Debug)]
pub struct ParseError(pub Box<dyn Error>);

impl Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0.to_string())
    }
}

impl From<String> for ParseError {
    fn from(value: String) -> Self {
        ParseError(value.into())
    }
}
