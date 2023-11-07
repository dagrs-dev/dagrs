//! Relevant definitions of tasks.
//!
//! # Task execution mode of the Dag engine
//!
//! Currently, the Dag execution engine has two execution modes:
//! The first mode is to execute tasks through user-written yaml configuration file, and then
//! hand them over to the dag engine for execution. The command to be executed can be specified in yaml.
//!
//！# The basic format of the yaml configuration file is as follows:
//! ```yaml
//! dagrs:
//！   a:
//！      name: "Task 1"
//！      after: [ b, c ]
//！      cmd: echo a
//！   b:
//！     name: "Task 2"
//！     after: [ c, f, g ]
//！     cmd: echo b
//！   c:
//！     name: "Task 3"
//！     after: [ e, g ]
//！     cmd: echo c
//！   d:
//！     name: "Task 4"
//！     after: [ c, e ]
//！     cmd: echo d
//！   e:
//！     name: "Task 5"
//！     after: [ h ]
//！     cmd: echo e
//！   f:
//！     name: "Task 6"
//！     after: [ g ]
//！     cmd: python3 ./tests/config/test.py
//！   g:
//！     name: "Task 7"
//！     after: [ h ]
//！     cmd: node ./tests/config/test.js
//！   h:
//！     name: "Task 8"
//！     cmd: echo h
//! ```
//! The necessary attributes for tasks in the yaml configuration file are:
//! id: unique identifier, such as 'a'
//! name: task name
//! after: Indicates which task to execute after, this attribute is optional
//! cmd: command to execute, such as 'ls ./'
//!
//!
//! The second mode is that the user program defines the task, which requires the
//! user to implement the [`Action`] trait of the task module and rewrite the
//! run function.
//!
//! # Example
//!
//! ```rust
//! use dagrs::{Action,EnvVar,Output,RunningError,Input};
//! use std::sync::Arc;
//! struct SimpleAction{
//!     limit:u32
//! }
//!
//! impl Action for SimpleAction{
//!     fn run(&self, input: Input,env:Arc<EnvVar>) -> Result<Output,RunningError> {
//!         let mut sum=0;
//!         for i in 0..self.limit{
//!             sum+=i;
//!         }
//!         Ok(Output::new(sum))
//!     }
//! }
//!
//! ```
//!
//! # Task definition.
//!
//! Currently, two different execution modes correspond to two different task types,
//! namely [`DefaultTask`] and [`YamlTask`].
//! When users program to implement task logic, the engine uses [`DefaultTask`];
//! When the user provides the yaml configuration file, the internal engine uses [`YamlTask`];
//!
//! These two task types both implement the [`Task`] trait, that is to say, users can also
//! customize tasks and assign more functions and attributes to tasks. However, a task must
//! have four fixed properties (the four standard properties contained in DefaultTask):
//! - id: use [`ID_ALLOCATOR`] to get a global task unique identifier, the type must be `usize`
//! - name: the task name specified by the user, the type must be `String`
//! - predecessor_tasks: the predecessor task of this task, the type must be `Vec<usize>`
//! - action: the specific behavior to be performed by the task, the type must be `Arc<dyn Action + Send + Sync>`
//!
//! If users want to customize Task, they can refer to the implementation of these two specific [`Task`].

use std::fmt::Debug;
use std::sync::atomic::AtomicUsize;
use std::sync::Arc;

use async_trait::async_trait;

use crate::utils::EnvVar;

#[cfg(feature = "yaml")]
pub use self::cmd::CommandAction;
pub use self::default_task::DefaultTask;
pub use self::error::{CmdExecuteError, RunningError};
pub(crate) use self::state::ExecState;
pub use self::state::{Input, Output};
#[cfg(feature = "yaml")]
pub use self::yaml_task::YamlTask;

mod cmd;
mod default_task;
mod error;
mod state;
mod yaml_task;

/// Action Trait.
/// [`Action`] represents the specific behavior to be executed.
#[async_trait]
pub trait Action {
    /// The specific behavior to be performed by the task.
    fn run(&self, input: Input, env: Arc<EnvVar>) -> Result<Output, RunningError>;
    async fn async_run(&self, _input: Input, _env: Arc<EnvVar>) -> Result<Output, RunningError> {
        Ok(Output::empty())
    }

    fn is_async(&self) -> bool {
        false
    }
}

/// Tasks can have many attributes, among which `id`, `name`, `predecessor_tasks`, and
/// `runnable` attributes are required, and users can also customize some other attributes.
/// [`DefaultTask`] in this module is a [ `Task`], the DAG engine uses it as the basic
/// task by default.
///
/// A task must provide methods to obtain precursors and required attributes, just as
/// the methods defined below, users who want to customize tasks must implement these methods.
pub trait Task: Send + Sync {
    /// Get a reference to an executable action.
    fn action(&self) -> Arc<dyn Action + Send + Sync>;
    /// Get the id of all predecessor tasks of this task.
    fn precursors(&self) -> &[usize];
    /// Get the id of this task.
    fn id(&self) -> usize;
    /// Get the name of this task.
    fn name(&self) -> String;
}

/// IDAllocator for DefaultTask
struct IDAllocator {
    id: AtomicUsize,
}

pub struct NopAction;

impl Debug for dyn Task {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "{},\t{},\t{:?}",
            self.id(),
            self.name(),
            self.precursors()
        )
    }
}

impl IDAllocator {
    fn alloc(&self) -> usize {
        let origin = self.id.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        if origin > self.id.load(std::sync::atomic::Ordering::Relaxed) {
            panic!("Too many tasks.")
        } else {
            origin
        }
    }
}

impl Action for NopAction {
    fn run(&self, _input: Input, _env: Arc<EnvVar>) -> Result<Output, RunningError> {
        Ok(Output::empty())
    }
}

/// The global task uniquely identifies an instance of the allocator.
static ID_ALLOCATOR: IDAllocator = IDAllocator {
    id: AtomicUsize::new(1),
};

/// public function to assign task's id.
pub fn alloc_id() -> usize {
    ID_ALLOCATOR.alloc()
}
