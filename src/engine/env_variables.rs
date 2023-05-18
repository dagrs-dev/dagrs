//! Global environment variables
//!
//! ## Implementation for global environment variables.
//!
//! Users can specify global environment variables for the DAG engine when
//! the task is running, which may be used during task execution.

use crate::task::Content;
use anymap2::any::CloneAnySendSync;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

/// Global environment variables.
///
/// Since it will be shared between tasks,
/// [`Arc`] and [`Mutex`] are needed.

pub struct EnvVar(Arc<Mutex<HashMap<String, Content>>>);

impl EnvVar {
    /// Allocate a new [`EnvVar`].
    pub fn new() -> Self {
        Self(Arc::new(Mutex::new(HashMap::new())))
    }

    #[allow(unused)]
    /// Set a global variables.
    ///
    /// # Example
    /// ```rust
    /// # let mut env = dagrs::EnvVar::new();
    /// env.set("Hello", "World".to_string());
    /// ```
    ///
    /// Lock operations are wrapped inside, so no need to worry.
    pub fn set<H: Send + Sync + CloneAnySendSync>(&mut self, name: &str, var: H) {
        let mut v = Content::new();
        v.insert(var);
        self.0.clone().lock().unwrap().insert(name.to_owned(), v);
    }

    #[allow(unused)]
    /// Get environment variables through keys of type &str.
    ///
    /// # Example
    /// ```rust
    /// # let mut env = dagrs::EnvVar::new();
    /// # env.set("Hello", "World".to_string());
    /// let res:Option<String> = env.get("Hello");
    /// # let res = if let Some(tmp) = res { tmp } else { String::new() };
    /// # assert_eq!(res, "World".to_string());
    /// ```
    pub fn get<H: Send + Sync + CloneAnySendSync>(&self, name: &str) -> Option<H> {
        if let Some(content) = self.0.clone().lock().unwrap().get(name) {
            content.clone().remove()
        } else {
            None
        }
    }
}

impl Clone for EnvVar {
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}

impl Default for EnvVar {
    fn default() -> Self {
        EnvVar(Arc::new(Mutex::new(HashMap::new())))
    }
}
