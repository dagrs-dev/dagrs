use thiserror::Error;

#[derive(Debug, Error)]
pub enum ParserError {
    #[error("File not found. [{0}]")]
    FileNotFound(#[from] std::io::Error),
    #[error("{0}")]
    YamlTaskError(YamlTaskError),
    // #[error("{0}")]
    // JsonTaskError(JsonTaskError),
    #[error("{0}")]
    FileContentError(FileContentError),
}

#[derive(Debug, Error)]
pub enum FileContentError {
    #[error("{0}")]
    IllegalYamlContent(#[from]yaml_rust::ScanError),
    #[error("File is empty! [{0}]")]
    Empty(String)
}

#[derive(Debug, Error)]
pub enum YamlTaskError {
    #[error("File content is not start with 'dagrs'.")]
    StartWordError,
    #[error("Task [{0}] has no name field.")]
    NoName(String),
    #[error("Task [{0}] cannot find the specified predecessor.")]
    NoPrecursor(String),
    #[error("Unsupported task type. [{0}]")]
    UnsupportedType(String),
    #[error("Undefined executable behavior. [{0}]")]
    NoRunnable(String),
    #[error("Invalid script definition. [{0}]")]
    IllegalScript(String)
}


// #[derive(Debug, Error)]
// pub enum JsonTaskError{}

impl From<FileContentError> for ParserError{
    fn from(value: FileContentError) -> Self {
        ParserError::FileContentError(value)
    }
}

impl From<YamlTaskError> for ParserError{
    fn from(value: YamlTaskError) -> Self {
        ParserError::YamlTaskError(value)
    }
}
