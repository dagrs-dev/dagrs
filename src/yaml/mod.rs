//! yaml configuration file type parser
//!
//! # Config file parser
//!
//! Use yaml configuration files to define a series of tasks, which eliminates the need for users to write code.
//! [`YamlParser`] is responsible for parsing the yaml configuration file into a series of [`YamlTask`].
//! The program specifies the properties of the yaml task configuration file. The basic format of the yaml
//! configuration file is as follows:
//!
//! ```yaml
//! dagrs:
//!   a:
//!     name: "Task 1"
//!     after: [ b, c ]
//!     cmd: echo a
//!   b:
//!     name: "Task 2"
//!     after: [ c, f, g ]
//!     cmd: echo b
//!   c:
//!     name: "Task 3"
//!     after: [ e, g ]
//!     cmd: echo c
//!   d:
//!     name: "Task 4"
//!     after: [ c, e ]
//!     cmd: echo d
//!   e:
//!     name: "Task 5"
//!     after: [ h ]
//!     cmd: echo e
//!   f:
//!     name: "Task 6"
//!     after: [ g ]
//!     cmd: python3 ./tests/config/test.py
//!   g:
//!     name: "Task 7"
//!     after: [ h ]
//!     cmd: node ./tests/config/test.js
//!   h:
//!     name: "Task 8"
//!     cmd: echo h
//! ```
//!
//! Users can read the yaml configuration file programmatically or by using the compiled `dagrs`
//! command line tool. Either way, you need to enable the `yaml` feature.
//!
//! # Example
//!
//! ```rust
//! use dagrs::Dag;
//! let dag = Dag::with_yaml("some_path",std::collections::HashMap::new());
//! ```

mod yaml_parser;
mod yaml_task;

pub use self::yaml_parser::YamlParser;
pub use self::yaml_task::YamlTask;

use crate::ParseError;

/// Errors about task configuration items.
#[derive(Debug)]
pub enum YamlTaskError {
    /// The configuration file should start with `dagrs:`.
    StartWordError,
    /// No task name configured.
    NoNameAttr(String),
    /// The specified task predecessor was not found.
    NotFoundPrecursor(String),
    /// `script` is not defined.
    NoScriptAttr(String),
}

/// Error about file information.
#[derive(Debug)]
pub enum FileContentError {
    /// The format of the yaml configuration file is not standardized.
    IllegalYamlContent(yaml_rust::ScanError),
    Empty(String),
}

/// Configuration file not found.
pub struct FileNotFound(pub std::io::Error);

impl From<YamlTaskError> for ParseError {
    fn from(value: YamlTaskError) -> Self {
        match value {
            YamlTaskError::StartWordError => {
                "File content is not start with 'dagrs'.".to_string().into()
            }
            YamlTaskError::NoNameAttr(ref msg) => {
                format!("Task has no name field. [{}]", msg).into()
            }
            YamlTaskError::NotFoundPrecursor(ref msg) => {
                format!("Task cannot find the specified predecessor. [{}]", msg).into()
            }
            YamlTaskError::NoScriptAttr(ref msg) => {
                format!("The 'script' attribute is not defined. [{}]", msg).into()
            }
        }
    }
}

impl From<FileContentError> for ParseError {
    fn from(value: FileContentError) -> Self {
        match value {
            FileContentError::IllegalYamlContent(ref err) => err.to_string().into(),
            FileContentError::Empty(ref file) => format!("File is empty! [{}]", file).into(),
        }
    }
}

impl From<FileNotFound> for ParseError {
    fn from(value: FileNotFound) -> Self {
        format!("File not found. [{}]", value.0).into()
    }
}
