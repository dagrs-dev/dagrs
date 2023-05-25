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
//! user to implement the [`SimpleAction`] trait of the task module and rewrite the
//! run function.
//!
//! ### An example of a user programmatically defined task
//!
//! ```rust
//! use dagrs::SimpleAction;
//! struct MyTask{
//!     limit:u32
//! }
//!
//! impl SimpleAction for MyTask{
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

pub use self::specific_task::*;
pub use self::state::*;
pub use self::script::*;
pub use self::error::*;

mod error;
mod specific_task;
mod state;
mod script;

use std::sync::Arc;
use std::sync::atomic::AtomicUsize;

/// SimpleAction Trait.
///
/// [`SimpleAction`] represents the specific behavior to be executed.
///
/// # Example
///
/// ```rust
/// use dagrs::{Input,EnvVar,Output,SimpleAction};
/// struct Action(usize);
/// impl SimpleAction for Action{
///     fn run(&self, input: Input) -> Output{
///         Output::new(self.0+1)
///     }
/// }
/// ```
pub trait SimpleAction {
    /// The specific behavior to be performed by the task.
    ///
    /// Parameter Description
    /// - `input` represents the execution result from the predecessor task.
    /// - `env` stands for global environment variables, users can continue
    /// to add or access environment variables.
    fn run(&self, input: Input) -> Result<Output,RunningError>;
}

/// ComplexAction Trait.
///
/// [`ComplexAction`] is an enhanced version of [`SimpleAction`], which supports pre-processing
/// and post-processing of specific execution behaviors.Users can do some preprocessing in
/// the preprocessing method `before_run`, such as reading files. You can do post-processing
/// work in the post-processing method `after_run`, such as closing files, closing network
/// streams, etc.
///
/// # Example
///
/// ```rust
/// use dagrs::{ComplexAction, DefaultTask, Output};
/// use std::{fs::File, io::Write};
/// struct FileOperation {
///     content: String,
/// }
///
/// impl ComplexAction for FileOperation {
///     fn before_run(&mut self) {
///         // Suppose you open the file and read the content into `content`.
///         self.content = "hello world".to_owned()
///     }
///
///     fn run(&self, input: dagrs::Input) -> dagrs::Output {
///         Output::new(self.content.split(" "))
///     }
///
///     fn after_run(&mut self) {
///         // Suppose you delete a temporary file generated when a task runs.
///         self.content = "".to_owned();
///     }
/// }
/// ```
pub trait ComplexAction {
    /// Executed before the `run` function to do preprocessing.
    fn before_run(&mut self);
    /// The specific behavior to be performed by the task.
    ///
    /// Parameter Description
    /// - `input` represents the execution result from the predecessor task.
    /// - `env` stands for global environment variables, users can continue
    /// to add or access environment variables.
    fn run(&self, input: Input) -> Result<Output,RunningError>;
    /// Executed after the `run` function, for aftermath work.
    fn after_run(&mut self);
}

pub enum Action {
    Simple(Arc<dyn SimpleAction + Send + Sync>),
    Complex(Arc<dyn ComplexAction + Send + Sync>),
}



/// Tasks can have many attributes, among which `id`, `name`, `predecessor_tasks`, and
/// `runnable` attributes are required, and users can also customize some other attributes.
/// [`DefaultTask`] in this module is a [ `Task`], the DAG engine uses it as the basic
/// task by default.
///
/// A task must provide methods to obtain precursors and required attributes, just as
/// the methods defined below, users who want to customize tasks must implement these methods.
pub trait Task{
    /// Get a reference to an executable action.
    fn runnable(&self) -> Action;
    /// Get the id of all predecessor tasks of this task.
    fn predecessors(&self) -> &[usize];
    /// Get the id of this task.
    fn id(&self) -> usize;
    /// Get the name of this task.
    fn name(&self) -> String;
}

/// Default implementation of abstract tasks.
pub struct DefaultTask {
    /// id is the unique identifier of each task, it will be assigned by the global [`IDAllocator`]
    /// when creating a new task, you can find this task through this identifier.
    id: usize,
    /// The task's name.
    name: String,
    /// Id of the predecessor tasks.
    predecessor_tasks: Vec<usize>,
    /// Perform specific actions.
    action: Action,
}

impl Clone for Action{
    fn clone(&self) -> Self {
        match self {
            Action::Simple(simple)=>Action::Simple(simple.clone()),
            Action::Complex(complex)=>Action::Complex(complex.clone())
        }
    }
}

impl DefaultTask {
    /// Allocate a new [`DefaultTask`], the specific task behavior is a structure that implements [`SimpleRunner`].
    ///
    /// # Example
    ///
    /// ```rust
    /// use dagrs::{DefaultTask, Output, SimpleAction};
    ///
    /// struct Action(usize);
    ///
    /// impl SimpleAction for Action {
    /// fn run(&self, input: dagrs::Input, env: dagrs::EnvVar) -> Output {
    ///     Output::new(self.0 + 10)
    /// }
    /// }
    ///
    /// let runnable = Action(10);
    /// let task = DefaultTask::simple_task(runnable, "Increment action");
    /// ```
    ///
    /// `Action` is a struct that impl [`SimpleAction`]. Since task will be
    ///  executed in separated threads, [`Send`] and [`Sync`] is needed.
    ///
    /// **Note:** This method will take the ownership of struct that impl [`SimpleAction`].
    pub fn simple_task(runnable: impl SimpleAction + 'static + Send + Sync, name: &str) -> Self {
        DefaultTask {
            id: ID_ALLOCATOR.alloc(),
            action: Action::Simple(Arc::new(runnable)),
            name: name.to_owned(),
            predecessor_tasks: Vec::new(),
        }
    }
    /// Allocate a new [`DefaultTask`], the specific task behavior is a structure that implements [`ComplexAction`].
    ///
    /// # Example
    ///
    /// ```rust
    /// use dagrs::{ComplexAction, DefaultTask, Output};
    /// use std::{fs::File, io::Write};
    /// struct FileOperation {
    ///     content: String,
    /// }
    ///
    /// impl ComplexAction for FileOperation {
    ///     fn before_run(&mut self) {
    ///         // Suppose you open the file and read the content into `content`.
    ///         self.content = "hello world".to_owned()
    ///     }
    ///
    ///     fn run(&self, input: dagrs::Input, env: dagrs::EnvVar) -> dagrs::Output {
    ///         Output::new(self.content.split(" "))
    ///     }
    ///
    ///     fn after_run(&mut self) {
    ///         // Suppose you delete a temporary file generated when a task runs.
    ///         self.content = "".to_owned();
    ///     }
    /// }
    /// let mut runnable = Action {content: "".to_owned()};
    /// let task = DefaultTask::complex_task(runnable, "Increment action");
    /// ```
    ///
    /// `FileOperation` is a struct that impl [`ComplexAction`]. Since task will be
    ///  executed in separated threads, [`Send`] and [`Sync`] is needed.
    ///
    /// **Note:** This method will take the ownership of struct that impl [`ComplexAction`].
    pub fn complex_task(
        runnable: impl ComplexAction + 'static + Send + Sync,
        name: &str,
    ) -> Self {
        DefaultTask {
            id: ID_ALLOCATOR.alloc(),
            action: Action::Complex(Arc::new(runnable)),
            name: name.to_owned(),
            predecessor_tasks: Vec::new(),
        }
    }

    #[allow(unused)]
    /// Tasks that shall be executed before this one.
    ///
    /// # Example
    /// ```rust
    /// # struct Task {};
    /// # impl dagrs::SimpleAction for Task {
    /// #     fn run(&self, input: dagrs::Input, env: dagrs::EnvVar) -> dagrs::Output {
    /// #         dagrs::Output::empty()
    /// #     }
    /// # }
    /// # let mut t1 = dagrs::DefaultTask::new(Task{}, "Task 1");
    /// # let mut t2 = dagrs::DefaultTask::new(Task{}, "Task 2");
    /// t2.set_predecessors(&[&t1]);
    /// ```
    /// In above code, `t1` will be executed before `t2`.
    pub fn set_predecessors(&mut self, predecessors: &[&DefaultTask]) {
        self.predecessor_tasks
            .extend(predecessors.iter().map(|t| t.id()))
    }

    /// The same as `exec_after`, but input are tasks' ids
    /// rather than reference to [`DefaultTask`].
    pub fn set_predecessors_by_id(&mut self, predecessors_id: &[usize]) {
        self.predecessor_tasks.extend(predecessors_id)
    }
}

impl Task for DefaultTask {
    fn id(&self) -> usize {
        self.id
    }

    fn name(&self) -> String {
        self.name.clone()
    }

    fn predecessors(&self) -> &[usize] {
        &self.predecessor_tasks
    }
    fn runnable(&self) -> Action {
        self.action.clone()
    }
}


/// IDAllocator for DefaultTask
struct IDAllocator {
    id: AtomicUsize,
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

static ID_ALLOCATOR: IDAllocator = IDAllocator { id: AtomicUsize::new(1) };

#[allow(unused)]
pub fn alloc_task_id()->usize{
    ID_ALLOCATOR.alloc()
}

