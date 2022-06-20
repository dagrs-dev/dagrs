//! Implementation for global environment variables.

use crate::task::DMap;
use anymap::CloneAny;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

/// Global environment variables.
/// 
/// Since it will be shared between tasks, [`Arc`] and [`Mutex`] 
/// are needed.
pub struct EnvVar(Arc<Mutex<HashMap<String, DMap>>>);

impl EnvVar {
    /// Allocate a new [`EnvVar`].
    pub fn new() -> Self {
        Self(Arc::new(Mutex::new(HashMap::new())))
    }

    #[allow(unused)]
    /// Set a gloval variables.
    /// 
    /// # Example
    /// ```rust
    /// env.set("Hello", "World".to_string());
    /// ```
    /// 
    /// Lock operations are wrapped inside, so no need to worry.
    pub fn set<H: Send + Sync + CloneAny>(&mut self, name: &str, var: H) {
        let mut v = DMap::new();
        v.insert(var);
        self.0.lock().unwrap().insert(name.to_owned(), v);
    }

    #[allow(unused)]
    /// This method get needed input value from [`Inputval`].
    /// 
    /// # Example
    /// ```rust
    /// env.set("Hello", "World".to_string());
    /// let res = env.get("Hello").unwrap();
    /// assert_eq!(res, "World".to_string());
    /// ```
    pub fn get<H: Send + Sync + CloneAny>(&self, name: &str) -> Option<H> {
        if let Some(dmap) = self.0.lock().unwrap().get(name) {
            dmap.clone().remove()
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