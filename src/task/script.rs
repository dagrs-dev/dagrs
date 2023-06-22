//! Specific task.
//!
//! # Two specific types of tasks offered to users.
//!
//! One is to execute sh script tasks, and the other is to execute Javascript script tasks.
//! Both of them implement the [`Action`] trait.

use std::{process::Command, sync::Arc};

use deno_core::{JsRuntime, RuntimeOptions, serde_json, serde_v8, v8};

use crate::utils::EnvVar;

use super::{Action, Input, JavaScriptExecuteError, Output, RunningError, ShExecuteError};

/// Can be used to run a sh script or sh file.
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
        let mut cmd = format!("{} ", self.script);
        input
            .get_iter()
            .for_each(|input| {
                if let Some(arg) = input.get::<String>() {
                    cmd.push_str(arg)
                }
            });
        match Command::new("sh")
            .arg("-c")
            .arg(&cmd)
            .output()
            .map(|output| String::from_utf8(output.stdout).unwrap())
        {
            Ok(res) => Ok(Output::new(res)),
            Err(err) => {
                let e = ShExecuteError::new(err.to_string(), err);
                // error!("sh task execution failed! {}", e);
                Err(e.into())
            }
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
                        //error!("JavaScript script task execution failed! {}", e);
                        Err(e.into())
                    }
                }
            }
            Err(err) => {
                let e = JavaScriptExecuteError::AnyHowError(err);
               // error!("JavaScript script task parsing failed! {}", e);
                Err(e.into())
            }
        }
    }
}
