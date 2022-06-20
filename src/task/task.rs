use crate::engine::{DagError, EnvVar, RunningError};

use super::{Inputval, Retval};
use deno_core::{serde_json, serde_v8, v8, JsRuntime, RuntimeOptions};
use lazy_static::lazy_static;
use std::process::Command;
use std::sync::Mutex;

/// Task Trait.
///
/// Any struct implements this trait can be added into dagrs.
pub trait TaskTrait {
    fn run(&self, input: Inputval, env: EnvVar) -> Retval;
}

/// Wrapper for task that impl [`TaskTrait`].
pub struct TaskWrapper {
    id: usize,
    name: String,
    exec_after: Vec<usize>,
    input_from: Vec<usize>,
    inner: Box<dyn TaskTrait + Send + Sync>,
}

impl TaskWrapper {
    /// Allocate a new TaskWrapper.
    ///
    /// # Example
    /// ```
    /// let t = TaskWrapper::new(Task{}, "Demo Task")
    /// ```
    ///
    /// `Task` is a struct that impl [`TaskTrait`]. Since task will be
    ///  executed in seperated threads, [`send`] and [`sync`] is needed.
    ///
    /// **Note:** This method will take the ownership of struct that impl [`TaskTrait`].
    pub fn new(task: impl TaskTrait + 'static + Send + Sync, name: &str) -> Self {
        TaskWrapper {
            id: ID_ALLOCATOR.lock().unwrap().alloc(),
            name: name.to_owned(),
            exec_after: Vec::new(),
            input_from: Vec::new(),
            inner: Box::new(task),
        }
    }

    #[allow(unused)]
    /// Tasks that shall be executed before this one.
    ///
    /// # Example
    /// ```rust
    /// let mut t1 = TaskWrapper::new(T1{}, "Task 1");
    /// let mut t2 = TaskWrapper::new(T2{}, "Task 2");
    /// t2.exec_after(&[&t1]);
    /// ```
    /// In above code, `t1` will be executed before `t2`.
    pub fn exec_after(&mut self, relys: &[&TaskWrapper]) {
        self.exec_after.extend(relys.iter().map(|t| t.get_id()))
    }

    /// Input will come from the given tasks' exec result.
    ///
    /// # Example
    /// ```rust
    /// t3.exec_after(&[&t1, &t2, &t4])
    /// t3.input_from(&[&t1, &t2]);
    /// ```
    ///
    /// In aboving code, t3 will have input from `t1` and `t2`'s return value.
    pub fn input_from(&mut self, needed: &[&TaskWrapper]) {
        self.input_from.extend(needed.iter().map(|t| t.get_id()))
    }

    /// The same as `exec_after`, but input are tasks' ids
    /// rather than reference to [`TaskWrapper`].
    pub fn exec_after_id(&mut self, relys: &[usize]) {
        self.exec_after.extend(relys)
    }

    /// The same as `input_from`, but input are tasks' ids
    /// rather than reference to [`TaskWrapper`].
    pub fn input_from_id(&mut self, needed: &[usize]) {
        self.input_from.extend(needed)
    }

    pub fn get_exec_after_list(&self) -> Vec<usize> {
        self.exec_after.clone()
    }

    pub fn get_input_from_list(&self) -> Vec<usize> {
        self.input_from.clone()
    }

    pub fn get_id(&self) -> usize {
        self.id
    }

    pub fn get_name(&self) -> String {
        self.name.to_owned()
    }

    pub fn run(&self, input: Inputval, env: EnvVar) -> Retval {
        self.inner.run(input, env)
    }
}

/// IDAllocator for TaskWrapper
struct IDAllocator {
    id: usize,
}

impl IDAllocator {
    pub fn alloc(&mut self) -> usize {
        self.id += 1;

        // Return values
        self.id - 1
    }
}

lazy_static! {
    /// Instance of IDAllocator
    static ref ID_ALLOCATOR: Mutex<IDAllocator> = Mutex::new(IDAllocator { id: 1 });
}

/// Can be used to run a script cmd or file.
#[derive(Debug)]
pub struct RunScript {
    script: String,
    executor: RunType,
}

/// Run script type, now a script can be run in `sh` or embeded `deno`.
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
    /// let r = RunScript::new("echo Hello", RunType::SH);
    /// r.exec();
    ///
    /// // or a script path
    /// let r = RunScript::new("test/test.sh", RunType::SH);
    /// r.exec();
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
    /// let r = RunScript::new("echo Hello", RunType::SH);
    /// r.exec();
    /// ```
    /// If execution succeeds, it returns the result in [`String`] type, or it
    /// returns a [`DagError`].
    pub fn exec(&self, input: Option<Inputval>) -> Result<String, DagError> {
        let res = match self.executor {
            RunType::SH => self.run_sh(input),
            RunType::DENO => self.run_deno(input),
        };

        res
    }

    fn run_sh(&self, input: Option<Inputval>) -> Result<String, DagError> {
        let mut cmd = format!("{} ", self.script);
        if let Some(input) = input {
            input
                .get_iter()
                .map(|input| {
                    cmd.push_str(if let Some(dmap) = input {
                        if let Some(str) = dmap.get::<String>() {
                            str
                        } else {
                            ""
                        }
                    } else {
                        ""
                    })
                })
                .count();
        }

        let res = Command::new("sh")
            .arg("-c")
            .arg(&cmd)
            .output()
            .map(|output| format!("{}", String::from_utf8(output.stdout).unwrap()));

        res.map_err(|err| err.into())
    }

    fn run_deno(&self, _input: Option<Inputval>) -> Result<String, DagError> {
        let script = self.script.clone();
        let mut context = JsRuntime::new(RuntimeOptions {
            ..Default::default()
        });
        match context.execute_script("", &script) {
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
