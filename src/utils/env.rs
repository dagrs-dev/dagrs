use anymap2::{any::CloneAnySendSync, Map};
use std::collections::HashMap;

pub type Variable = Map<dyn CloneAnySendSync + Send + Sync>;

/// # Environment variable.
///
/// When multiple tasks are running, they may need to share the same data or read
/// the same configuration information. Environment variables can meet this requirement.
/// Before all tasks run, the user builds a [`EnvVar`] and sets all the environment
/// variables. One [`EnvVar`] corresponds to one dag. All tasks in a job can
/// be shared and immutable at runtime. environment variables.
#[derive(Debug, Default)]
pub struct EnvVar {
    variables: HashMap<String, Variable>,
}

impl EnvVar {
    /// Allocate a new [`EnvVar`].
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
        }
    }

    #[allow(unused)]
    /// Set a global variables.
    ///
    /// # Example
    /// ```rust
    /// # let mut env = dagrs::EnvVar::new();
    /// env.set("Hello", "World".to_string());
    /// ```
    pub fn set<H: Send + Sync + CloneAnySendSync>(&mut self, name: &str, var: H) {
        let mut v = Variable::new();
        v.insert(var);
        self.variables.insert(name.to_owned(), v);
    }

    #[allow(unused)]
    /// Get environment variables through keys of type &str.
    pub fn get<H: Send + Sync + CloneAnySendSync>(&self, name: &str) -> Option<H> {
        if let Some(content) = self.variables.get(name) {
            content.clone().remove()
        } else {
            None
        }
    }
}
