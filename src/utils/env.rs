//! Global environment variables
//!
//! ## Global variables of the dagrs program.
//!
//! Allows the user to set global environment variables required during execution
//! before all tasks are executed.
//!
//! If multiple tasks need to rely on some of the same data, you can consider
//! storing these data as global environment variables. For example:
//!
//! ```rust
//! use std::thread;
//! use dagrs::utils;
//! utils::set_env("v1", 1i32);
//! utils::set_env("v2", 2usize);
//! utils::set_env("v3", "3");
//! utils::set_env("v4", "4".to_owned());
//! let mut handles=Vec::new();
//! handles.push(thread::spawn(|| {
//!     assert_eq!(utils::get_env::<i32>("v1").unwrap(), 1);
//! }));
//! handles.push(thread::spawn(|| {
//!     assert_eq!(utils::get_env::<usize>("v2").unwrap(), 2);
//! }));
//! handles.push(thread::spawn(|| {
//!     assert_eq!(utils::get_env::<&str>("v3").unwrap(), "3");
//! }));
//! handles.push(thread::spawn(|| {
//!     assert_eq!(utils::get_env::<String>("v4").unwrap(), "4".to_owned());
//! }));

//! handles.push(thread::spawn(|| {
//!     assert_eq!(utils::get_env::<String>("v5"), None);
//! }));

//! for handle in handles {
//!     handle.join().unwrap();
//! }
//! ```
//!
//! Before all tasks start, users can increase environment variables. When tasks
//! start to execute, the engine will call the `env_unchangeable` function. At
//! this time, users will no longer be able to increase environment variables.

use anymap2::{any::CloneAnySendSync, Map};
use once_cell::unsync::OnceCell;
use std::{collections::HashMap, sync::Once};
use log::info;

type Variable = Map<dyn CloneAnySendSync + Send + Sync>;

/// The instance of the global environment variable.
static mut GLOBAL_ENV: OnceCell<HashMap<String, Variable>> = OnceCell::new();
/// Whether the environment variable can be modified.
static DISABLE: Once = Once::new();

/// Set environment variables before all tasks are executed.
pub fn set_env<T: Send + Sync + CloneAnySendSync>(key: &str, val: T) {
    if !DISABLE.is_completed() {
        let mut v = Variable::new();
        v.insert(val);
        unsafe {
            if GLOBAL_ENV.get().is_none(){
                GLOBAL_ENV.set(HashMap::new()).unwrap();
            }
            GLOBAL_ENV.get_mut().unwrap().insert(key.to_string(), v);
        };
    }
}

/// The specified environment variable can be obtained in any thread.
pub fn get_env<T: Send + Sync + CloneAnySendSync>(key:&str)->Option<T>{
    if let Some(env)=unsafe{GLOBAL_ENV.get()}{
        match env.get(key) {
            None => {None}
            Some(variable) => {
                variable.clone().remove()
            }
        }
    }else{
        None
    }
}

/// The engine calls this function internally before all tasks are executed, and
/// disables the user's behavior of modifying environment variables (even if the
/// user calls the set_env method after the function is called, no new environment
/// variables will be added).
pub(crate) fn env_unchangeable(){
    if !DISABLE.is_completed(){
        DISABLE.call_once(||{
            info!("The global environment variable is initialized and cannot be modified.")
        });
    }
}