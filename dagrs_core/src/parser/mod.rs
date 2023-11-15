//! Parsing configuration files.
//!
//! # Config file parser
//!
//! When users customize configuration files, the program needs to use the configuration
//! file parser defined by this module. The parser is responsible for parsing the content
//! defined in the configuration file into a series of tasks with dependencies.
//!
//! The program provides a default Yaml configuration file parser: [`YamlParser`]. However,
//! users are allowed to customize the parser, which requires the user to implement the [`Parser`] trait.
//! Currently, the program only supports configuration files in *.yaml format, and may support
//! configuration files in *.json format in the future.
//!
//! # The basic format of the yaml configuration file is as follows:
//! ```yaml
//! dagrs:
//！   a:
//！      name: "Task 1"
//！      after: [ b, c ]
//！      cmd: echo a
//！   b:
//！     name: "Task 2"
//！     after: [ c, f, g ]
//！     cmd: echo b
//！   c:
//！     name: "Task 3"
//！     after: [ e, g ]
//！     cmd: echo c
//！   d:
//！     name: "Task 4"
//！     after: [ c, e ]
//！     cmd: echo d
//！   e:
//！     name: "Task 5"
//！     after: [ h ]
//！     cmd: echo e
//！   f:
//！     name: "Task 6"
//！     after: [ g ]
//！     cmd: python3 ./tests/config/test.py
//！   g:
//！     name: "Task 7"
//！     after: [ h ]
//！     cmd: node ./tests/config/test.js
//！   h:
//！     name: "Task 8"
//！     cmd: echo h
//! ```
//!
//! Users can execute arbitrary commands of the operating system. If users
//! want to run other types of script tasks, they need to implement the [`Action`] trait by themselves,
//! and before parsing the configuration file, they need to provide a specific type that implements
//! the [`Action`] trait in the form of key-value pairs: <id, action>.

use std::{collections::HashMap, sync::Arc};

pub use error::*;
#[cfg(feature = "yaml")]
pub use yaml_parser::YamlParser;
#[cfg(feature = "yaml")]
mod yaml_parser;

use crate::{task::Task, Action};

mod error;

/// Generic parser traits. If users want to customize the configuration file parser, they must implement this trait.
/// [`YamlParser`] is an example of [`Parser`]
pub trait Parser {
    /// Parses the contents of a configuration file into a series of tasks with dependencies.
    /// If the user customizes the script execution logic, it is necessary to provide specific
    /// types that implement the [`Action`] trait for certain tasks in the form of key-value pairs.
    fn parse_tasks(
        &self,
        file: &str,
        specific_actions: HashMap<String, Arc<dyn Action + Send + Sync>>,
    ) -> Result<Vec<Box<dyn Task>>, ParserError>;
}
