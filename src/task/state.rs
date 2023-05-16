//! Task state
//!
//! ## Input, output, and state of tasks.
//!
//! Task execution may require input: the execution of a task may require
//! output from the execution of several predecessor tasks, and use [`Input`]
//! to represent the required input.
//!
//! Execution state of the task: If the task is executed successfully, the
//! success field of the [`ExecState`] structure representing the task state
//! will be true, otherwise it will be false.
//!
//! The execution of the task may produce output: if the task is executed successfully,
//! it may produce output, and [`Output`] is used to represent the output of the task.

use anymap2::{any::CloneAnySendSync, Map};
use std::{
    slice::Iter,
    sync::atomic::{AtomicBool, AtomicPtr, Ordering},
};
use tokio::sync::Semaphore;

pub type Content = Map<dyn CloneAnySendSync + Send + Sync>;

/// Describe task's running result

#[derive(Debug)]
pub struct ExecState {
    /// The execution succeed or not.
    success: AtomicBool,
    /// Return value of the execution.
    output: AtomicPtr<Output>,
    // Task output identified by id.
    task_id: usize,
    /// The semaphore is used to control the synchronous blocking of subsequent tasks to obtain the
    /// execution results of this task.
    /// After this task is executed, it will increase by n (n is the number of subsequent tasks of
    /// this task, which can also be considered as the out-degree of the node represented by this task)
    /// permit, each subsequent task requires a permit to obtain the execution result of this task.
    semaphore: Semaphore,
}

/// Task's return value
#[derive(Debug)]
pub struct Output(Option<Content>);

/// Task's input value
pub struct Input(Vec<Content>);

#[allow(dead_code)]
impl ExecState {
    /// Get a new [`ExecState`].
    ///
    /// `success`: task finish without panic?
    ///
    /// `output`: task's return value
    pub fn new(task_id: usize) -> Self {
        Self {
            success: AtomicBool::new(false),
            output: AtomicPtr::new(std::ptr::null_mut()),
            task_id,
            semaphore: Semaphore::new(0),
        }
    }

    /// After the task is successfully executed, set the execution result.
    pub fn set_output(&self, output: Output) {
        self.success.store(true, Ordering::Relaxed);
        self.output
            .store(Box::leak(Box::new(output)), Ordering::Relaxed);
    }

    /// Consume a permit to get the output, if there is no permit currently, the thread will be blocked.
    ///
    /// This method will clone [`DMap`] that are stored in [`ExecState`]'s [`Output`].
    pub fn get_output(&self) -> Option<Content> {
        unsafe { self.output.load(Ordering::Relaxed).as_ref().unwrap() }
            .0
            .clone()
    }

    /// The task execution succeed or not.
    ///
    /// `true` means no panic occurs.
    pub fn success(&self) -> bool {
        self.success.load(Ordering::Relaxed)
    }

    // Use id to indicate the output of which task.
    pub fn get_id(&self) -> usize {
        self.task_id
    }
    pub(crate) async fn acquire_permits(&self) {
        self.semaphore.acquire().await.unwrap().forget();
    }
    /// Set the number of permits, the number of permits means that the execution
    /// result can be taken away by several subsequent tasks.
    pub(crate) fn add_permits(&self, permits: usize) {
        self.semaphore.add_permits(permits);
    }
}

impl Output {
    #[allow(unused)]
    /// Get a new [`Output`].
    ///
    /// Since the return value may be transferred between threads,
    /// [`Send`], [`Sync`], [`CloneAnySendSync`] is needed.
    ///
    /// # Example
    /// ```rust
    /// let output = dagrs::Output::new(123);
    /// ```
    pub fn new<H: Send + Sync + CloneAnySendSync>(val: H) -> Self {
        let mut map = Content::new();
        assert!(map.insert(val).is_none(), "[Error] map insert fails.");
        Self(Some(map))
    }

    /// Get empty [`Output`].
    ///
    /// # Example
    /// ```rust
    /// let output = dagrs::Output::empty();
    /// ```
    pub fn empty() -> Self {
        Self(None)
    }
}

impl Input {
    /// Get a new [`Input`], values stored in vector are ordered
    /// by that of the given Task's predecessor.
    pub fn new(input: Vec<Content>) -> Self {
        Self(input)
    }

    #[allow(unused)]
    /// This method get needed input value from [`Input`],
    /// and, it takes an index as input.
    ///
    /// When input from only one task's return value,
    /// just set index `0`. If from multi-tasks' return values, the index depends on
    /// which return value you want. The order of the return values are the same
    /// as you defined in `exec_after` function.
    ///
    /// # Example
    /// ```rust
    /// # let mut content=dagrs::Content::new();
    /// # content.insert("something".to_owned());
    /// # let mut input = dagrs::Input::new( vec![ content ] );
    /// # let input_from_t1:Option<String> = input.get(0);
    /// ```
    pub fn get<H: Send + Sync + CloneAnySendSync>(&mut self, index: usize) -> Option<H> {
        if let Some(content) = self.0.get_mut(index) {
            content.remove()
        } else {
            None
        }
    }

    /// Since [`Input`] can contain multi-input values, and it's implemented
    /// by [`Vec`] actually, of course it can be turned into a iterator.
    pub fn get_iter(&self) -> Iter<Content> {
        self.0.iter()
    }
}
