//! Specific task
//!
//! ## Two specific types of tasks offered to users.
//!
//! One is to execute sh script tasks, and the other is to execute Javascript script tasks.

use super::{Input, Output, SimpleAction, ShExecuteError, JavaScriptExecuteError};

use deno_core::{serde_json, serde_v8, v8, JsRuntime, RuntimeOptions};
use std::process::Command;

/// Can be used to run a script cmd or file.

pub struct ShScriptRunnable {
    script: String,
}
pub struct JavaScriptRunnable {
    script: String,
}

impl ShScriptRunnable {
    pub fn new(script: &str) -> Self {
        Self {
            script: script.to_owned(),
        }
    }
}

impl SimpleAction for ShScriptRunnable {
    fn run(&self, input: Input) -> Result<Output,super::RunningError> {
        let mut cmd = format!("{} ", self.script);
        input
            .get_iter()
            .map(|input| {
                if let Some(arg) = input.get::<String>() {
                    cmd.push_str(arg)
                }
            })
            .count();
        match Command::new("sh")
                            .arg("-c")
                            .arg(&cmd)
                            .output()
                            .map(|output| String::from_utf8(output.stdout).unwrap()) {
            Ok(res) => Ok(Output::new(res)),
            Err(err) => {
                let e = ShExecuteError::new(err.to_string(), err);
                // !todo: log error message
                Err(e.into())
            },
        }
        
    }
}

impl JavaScriptRunnable {
    pub fn new(script: &str) -> Self {
        Self {
            script: script.to_owned(),
        }
    }
}

impl SimpleAction for JavaScriptRunnable {
    fn run(&self, _input: Input) -> Result<Output,super::RunningError> {
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
                        let e=JavaScriptExecuteError::SerializeError(err);
                        // !todo: log error message
                        Err(e.into())
                    },
                }
            },
            Err(err) => {
                let e=JavaScriptExecuteError::AnyHowError(err);
                // !todo: need log error message
                Err(e.into())
            },
        }
    }
}
