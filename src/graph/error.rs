#[derive(Clone, Debug)]
pub enum GraphError {
    GraphLoopDetected,
    GraphNotActive,
    ExecutionFailed(String),
    PanicOccurred(String),
    MultipleErrors(Vec<GraphError>),
}

impl std::fmt::Display for GraphError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl From<String> for GraphError {
    fn from(error: String) -> Self {
        GraphError::ExecutionFailed(error)
    }
}

impl From<&str> for GraphError {
    fn from(error: &str) -> Self {
        GraphError::ExecutionFailed(error.to_string())
    }
}
