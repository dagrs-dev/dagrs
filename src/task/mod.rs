//! Relevant definitions of tasks.
//!
//! ## Task execution mode of the Dag engine
//!
//! Currently, the Dag execution engine has two execution modes:
//! The first mode is to execute tasks through user-written yaml configuration file,
//! and then hand them over to the dag engine for execution. Currently, the yaml
//! configuration file supports two types of tasks, one is to execute sh scripts,
//! and the other is to execute javascript scripts.
//!
//! ### An example yaml configuration file
//! ```yaml
//! dagrs:
//!     a:
//!       name: "task1"
//!       after: [b]
//!       run:
//!           type: sh
//!           script: test.sh
//!     b:
//!       name: "task2"
//!       run:
//!           type: deno
//!           script: Deno.core.print("Hello!")
//! ```
//!
//! The second mode is that the user program defines the task, which requires the
//! user to implement the [`TaskTrait`] trait of the task module and rewrite the
//! run function.
//!
//! ### An example of a user programmatically defined task
//!
//! ```rust
//! use dagrs::TaskTrait;
//! struct MyTask{
//!     limit:u32
//! }
//!
//! impl TaskTrait for MyTask{
//!     fn run(&self, input: dagrs::Inputval, env: dagrs::EnvVar) -> dagrs::Retval {
//!         let mut sum=0;
//!         for i in 0..self.limit{
//!             sum+=i;
//!         }
//!         dagrs::Retval::new(sum)
//!     }
//! }
//!
//! ```
//!
//!

use crate::EnvVar;

pub use self::specific_task::*;
pub use self::state::{DMap, ExecState, Inputval, Retval};
pub use self::yaml_task::YamlTask;

mod specific_task;
mod state;
mod yaml_task;

use lazy_static::lazy_static;
use std::sync::Mutex;

/// Task Trait.
///
/// Any struct implements this trait can be added into dagrs.
pub trait TaskTrait {
    fn run(&self, input: Inputval, env: EnvVar) -> Retval;
}

/// Wrapper for task that impl [`TaskTrait`].
pub struct TaskWrapper {
    /// id is the unique identifier of each task, it will be assigned by the global
    /// [`IDAllocator`] when creating a new task, you can find this task through this identifier.
    id: usize,
    /// The task's name.
    name: String,
    /// Id of the successor tasks.
    exec_after: Vec<usize>,
    /// This task requires the execution results of multiple (0~n) predecessor tasks as input.
    input_from: Vec<usize>,
    /// A task to be executed.
    inner: Box<dyn TaskTrait + Send + Sync>,
}

impl TaskWrapper {
    /// Allocate a new TaskWrapper.
    ///
    /// # Example
    /// ```
    /// # struct Task {};
    /// # impl dagrs::TaskTrait for Task {
    /// #     fn run(&self, input: dagrs::Inputval, env: dagrs::EnvVar) -> dagrs::Retval {
    /// #         dagrs::Retval::empty()
    /// #     }
    /// # }
    /// let t = dagrs::TaskWrapper::new(Task{}, "Demo Task");
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
    /// # struct Task {};
    /// # impl dagrs::TaskTrait for Task {
    /// #     fn run(&self, input: dagrs::Inputval, env: dagrs::EnvVar) -> dagrs::Retval {
    /// #         dagrs::Retval::empty()
    /// #     }
    /// # }
    /// # let mut t1 = dagrs::TaskWrapper::new(Task{}, "Task 1");
    /// # let mut t2 = dagrs::TaskWrapper::new(Task{}, "Task 2");
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
    /// # struct Task {};
    /// # impl dagrs::TaskTrait for Task {
    /// #     fn run(&self, input: dagrs::Inputval, env: dagrs::EnvVar) -> dagrs::Retval {
    /// #         dagrs::Retval::empty()
    /// #     }
    /// # }
    /// # let mut t1 = dagrs::TaskWrapper::new(Task{}, "Task 1");
    /// # let mut t2 = dagrs::TaskWrapper::new(Task{}, "Task 2");
    /// # let mut t3 = dagrs::TaskWrapper::new(Task{}, "Task 3");
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
