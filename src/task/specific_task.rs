//! Specific task
//!
//! ## Two specific types of tasks offered to users.
//!
//! One is to execute sh script tasks, and the other is to execute Javascript script tasks.

use crate::{
    engine::{DagError, RunningError},
    Input,
};
use deno_core::{serde_json, serde_v8, v8, JsRuntime, RuntimeOptions};
use std::process::Command;

/// Can be used to run a script cmd or file.
#[derive(Debug)]
pub struct RunScript {
    script: String,
    executor: RunType,
}

/// Run script type, now a script can be run in `sh` or embedded `deno`.
///
/// **Note** this features is not quite perfect, or rather, need lots of improvements.
#[derive(Debug)]
pub enum RunType {
    SH,
    DENO,
}

impl RunScript {
    /// Generate a new run script.
    ///
    /// # Example
    /// ```
    /// // `script` can be a commnad
    /// let r = dagrs::RunScript::new("echo Hello", dagrs::RunType::SH);
    ///
    /// // or a script path
    /// let r = dagrs::RunScript::new("test/test.sh", dagrs::RunType::SH);
    /// ```
    pub fn new(script: &str, executor: RunType) -> Self {
        Self {
            script: script.to_owned(),
            executor,
        }
    }

    /// Execute the script.
    ///
    /// # Example
    /// ```
    /// let r = dagrs::RunScript::new("echo Hello", dagrs::RunType::SH);
    /// r.exec(None);
    /// ```
    /// If execution succeeds, it returns the result in [`String`] type, or it
    /// returns a [`DagError`].
    pub fn exec(&self, input: Option<Input>) -> Result<String, DagError> {
        match self.executor {
            RunType::SH => self.run_sh(input),
            RunType::DENO => self.run_deno(input),
        }
    }

    fn run_sh(&self, input: Option<Input>) -> Result<String, DagError> {
        let mut cmd = format!("{} ", self.script);
        if let Some(input) = input {
            input
                .get_iter()
                .map(|input| {
                    if input.is_some() {
                        if let Some(arg) = input.as_ref().unwrap().get::<String>() {
                            cmd.push_str(arg);
                        }
                    }
                })
                .count();
        }

        let res = Command::new("sh")
            .arg("-c")
            .arg(&cmd)
            .output()
            .map(|output| String::from_utf8(output.stdout).unwrap());

        res.map_err(|err| err.into())
    }

    fn run_deno(&self, _input: Option<Input>) -> Result<String, DagError> {
        let script = self.script.clone().into_boxed_str();
        let mut context = JsRuntime::new(RuntimeOptions {
            ..Default::default()
        });
        match context.execute_script("", deno_core::FastString::Owned(script)) {
            Ok(global) => {
                let scope = &mut context.handle_scope();
                let local = v8::Local::new(scope, global);

                let deserialized_value = serde_v8::from_v8::<serde_json::Value>(scope, local);

                match deserialized_value {
                    Ok(value) => Ok(value.to_string()),
                    Err(err) => Err(DagError::running_error(RunningError::RunScriptFailure(
                        "?".into(),
                        format!("Cannot deserialize value: {:?}", err),
                    ))),
                }
            }
            Err(err) => Err(DagError::running_error(RunningError::RunScriptFailure(
                "?".into(),
                format!("{:?}", err),
            ))),
        }
    }
}
