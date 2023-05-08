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
//!     fn run(&self, input: dagrs::Input, env: dagrs::EnvVar) -> dagrs::Output {
//!         let mut sum=0;
//!         for i in 0..self.limit{
//!             sum+=i;
//!         }
//!         dagrs::Output::new(sum)
//!     }
//! }
//!
//! ```
//!
//!

use crate::EnvVar;

pub use self::specific_task::*;
pub use self::state::{DMap, ExecState, Input, Output};
pub use self::yaml_task::YamlTask;

mod specific_task;
mod state;
mod yaml_task;

use lazy_static::lazy_static;
use std::sync::Mutex;
use tokio::sync::Semaphore;

/// Task Trait.
///
/// Any struct implements this trait can be added into dagrs.
pub trait TaskTrait {
    fn run(&self, input: Input, env: EnvVar) -> Output;
}

/// Wrapper for task that impl [`TaskTrait`].
pub struct TaskWrapper {
    /// id is the unique identifier of each task, it will be assigned by the global
    /// [`IDAllocator`] when creating a new task, you can find this task through this identifier.
    id: usize,
    /// The task's name.
    name: String,
    /// Id of the predecessor tasks.
    predecessor_tasks: Vec<usize>,
    /// A task to be executed.
    inner: Box<dyn TaskTrait + Send + Sync>,
    /// The semaphore is used to control the synchronous blocking of subsequent tasks to obtain the
    /// execution results of this task.
    /// After this task is executed, it will increase by n (n is the number of subsequent tasks of
    /// this task, which can also be considered as the out-degree of the node represented by this task)
    /// permit, each subsequent task requires a permit to obtain the execution result of this task.
    semaphore: Semaphore,
}

impl TaskWrapper {
    /// Allocate a new TaskWrapper.
    ///
    /// # Example
    /// ```
    /// # struct Task {};
    /// # impl dagrs::TaskTrait for Task {
    /// #     fn run(&self, input: dagrs::Input, env: dagrs::EnvVar) -> dagrs::Output {
    /// #         dagrs::Output::empty()
    /// #     }
    /// # }
    /// let t = dagrs::TaskWrapper::new(Task{}, "Demo Task");
    /// ```
    ///
    /// `Task` is a struct that impl [`TaskTrait`]. Since task will be
    ///  executed in separated threads, [`Send`] and [`Sync`] is needed.
    ///
    /// **Note:** This method will take the ownership of struct that impl [`TaskTrait`].
    pub fn new(task: impl TaskTrait + 'static + Send + Sync, name: &str) -> Self {
        TaskWrapper {
            id: ID_ALLOCATOR.lock().unwrap().alloc(),
            name: name.to_owned(),
            predecessor_tasks: Vec::new(),
            inner: Box::new(task),
            semaphore: Semaphore::new(0),
        }
    }

    #[allow(unused)]
    /// Tasks that shall be executed before this one.
    ///
    /// # Example
    /// ```rust
    /// # struct Task {};
    /// # impl dagrs::TaskTrait for Task {
    /// #     fn run(&self, input: dagrs::Input, env: dagrs::EnvVar) -> dagrs::Output {
    /// #         dagrs::Output::empty()
    /// #     }
    /// # }
    /// # let mut t1 = dagrs::TaskWrapper::new(Task{}, "Task 1");
    /// # let mut t2 = dagrs::TaskWrapper::new(Task{}, "Task 2");
    /// t2.set_predecessors(&[&t1]);
    /// ```
    /// In above code, `t1` will be executed before `t2`.
    pub fn set_predecessors(&mut self, predecessors: &[&TaskWrapper]) {
        self.predecessor_tasks
            .extend(predecessors.iter().map(|t| t.get_id()))
    }

    /// The same as `exec_after`, but input are tasks' ids
    /// rather than reference to [`TaskWrapper`].
    pub fn set_predecessors_by_id(&mut self, predecessors_id: &[usize]) {
        self.predecessor_tasks.extend(predecessors_id)
    }

    pub fn get_predecessors_id(&self) -> Vec<usize> {
        self.predecessor_tasks.clone()
    }

    pub fn get_id(&self) -> usize {
        self.id
    }

    pub fn get_name(&self) -> String {
        self.name.to_owned()
    }

    pub fn run(&self, input: Input, env: EnvVar) -> Output {
        self.inner.run(input, env)
    }

    pub(crate) async fn acquire_permits(&self) {
        self.semaphore.acquire().await.unwrap().forget()
    }

    pub(crate) fn init_permits(&self, permits: usize) {
        self.semaphore.add_permits(permits);
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

/// Macros for generating simple tasks.
///
/// # Example
///
/// ```rust
/// # use dagrs::{generate_task,TaskWrapper,Input,Output,EnvVar,TaskTrait};
/// # let task = generate_task!("task A", |input, env| {
/// #     Output::empty()
/// # });
/// # println!("{},{}", task.get_id(), task.get_name());
/// ```
#[macro_export]
macro_rules! generate_task {
    ($task_name:expr,$action:expr) => {{
        pub struct Task {}
        impl TaskTrait for Task {
            fn run(&self, input: Input, env: EnvVar) -> Output {
                $action(input, env)
            }
        }
        TaskWrapper::new(Task {}, $task_name)
    }};
}