//! Specific task.
//!
//! # Two specific types of tasks offered to users.
//!
//! One is to execute sh script tasks, and the other is to execute Javascript script tasks.
//! Both of them implement the [`Action`] trait.

use std::{process::Command, sync::Arc};

use deno_core::{serde_json, serde_v8, v8, JsRuntime, RuntimeOptions};

use crate::{log, utils::EnvVar};

use super::{Action, Input, JavaScriptExecuteError, Output, RunningError, ShExecuteError};

/// Can be used to run a sh script.
pub struct ShScript {
    script: String,
}

/// Can be used to execute javascript scripts.
pub struct JavaScript {
    script: String,
}

impl ShScript {
    pub fn new(script: &str) -> Self {
        Self {
            script: script.to_owned(),
        }
    }
}

impl Action for ShScript {
    fn run(&self, input: Input, _env: Arc<EnvVar>) -> Result<Output, RunningError> {
        let args: Vec<String> = input
            .get_iter()
            .map(|input| input.get::<String>())
            .filter(|input| input.is_some())
            .map(|input| input.unwrap().clone())
            .collect();
        let out = Command::new("sh")
            .arg("-c")
            .arg(&self.script)
            .args(args)
            .output()
            .unwrap();
        if !out.stderr.is_empty() {
            let err_msg = String::from_utf8(out.stderr).unwrap();
            log::error(err_msg.clone());
            Err(ShExecuteError::new(err_msg).into())
        } else {
            Ok(Output::new(String::from_utf8(out.stdout).unwrap()))
        }
    }
}

impl JavaScript {
    pub fn new(script: &str) -> Self {
        Self {
            script: script.to_owned(),
        }
    }
}

impl Action for JavaScript {
    fn run(&self, _input: Input, _env: Arc<EnvVar>) -> Result<Output, RunningError> {
        let script = self.script.clone().into_boxed_str();
        let mut context = JsRuntime::new(RuntimeOptions {
            ..Default::default()
        });
        match context.execute_script("", deno_core::FastString::Owned(script)) {
            Ok(global) => {
                let scope = &mut context.handle_scope();
                let local = v8::Local::new(scope, global);
                match serde_v8::from_v8::<serde_json::Value>(scope, local) {
                    Ok(value) => Ok(Output::new(value.to_string())),
                    Err(err) => {
                        let e = JavaScriptExecuteError::SerializeError(err);
                        log::error(format!("JavaScript script task execution failed! {}", e));
                        Err(e.into())
                    }
                }
            }
            Err(err) => {
                let e = JavaScriptExecuteError::AnyHowError(err);
                log::error(format!("JavaScript script task parsing failed! {}", e));
                Err(e.into())
            }
        }
    }
}
