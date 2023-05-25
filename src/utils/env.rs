//! Global environment variables
//!
//! ## Implementation for global environment variables.
//!
//! Users can specify global environment variables for the DAG engine when
//! the task is running, which may be used during task execution.

use anymap2::{any::CloneAnySendSync, Map};
use once_cell::unsync::Lazy;
use std::collections::HashMap;

// /// Global environment variables.

type Variable = Map<dyn CloneAnySendSync + Send + Sync>;

static mut GLOBAL_ENV: (Lazy<HashMap<String, Variable>>, bool) =
    (Lazy::new(|| HashMap::new()), true);

#[allow(unused)]
pub fn set_env<H: Send + Sync + CloneAnySendSync>(key: &str, val: H) {
    if unsafe { GLOBAL_ENV.1 } {
        let mut v = Variable::new();
        v.insert(val);
        unsafe { Lazy::get_mut(&mut GLOBAL_ENV.0).unwrap() }.insert(key.to_owned(), v);
    }
}

#[allow(unused)]
pub fn get_env<H: Send + Sync + CloneAnySendSync>(key: &str) -> Option<H> {
    if let Some(content) = unsafe { GLOBAL_ENV.0.get(key) } {
        content.clone().remove()
    } else {
        None
    }
}

pub(crate) fn env_unchangeable() {
    unsafe { GLOBAL_ENV.1 = false };
}
