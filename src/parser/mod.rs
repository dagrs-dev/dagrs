
use crate::task::Task;

use self::error::ParserError;

pub use self::yaml_parser::YamlParser;

mod error;
mod json_parser;
mod yaml_parser;

pub trait Parser {
    fn parse_tasks(&self,file:&str) -> Result<Vec<Box<dyn Task>>,ParserError>;
}