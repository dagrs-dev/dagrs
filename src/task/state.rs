//! Task state
//!
//! # Input, output, and state of tasks.
//!
//! [`Output`] represents the output generated when the task completes successfully.
//! The user can use the `new` function to construct an [`Output`] representing the
//! generated output, and use the `empty` function to construct an empty [`Output`]
//! if the task does not generate output.
//!
//! [`Input`] represents the input required by the task, and the input comes from the
//! output produced by multiple predecessor tasks of this task. Users can read and
//! write the content of [`Input`] ([`Input`] is actually constructed by cloning multiple
//! [`Output`]), so as to realize the logic of the program.
//!
//! [`ExeState`] internally stores [`]Output`], which represents whether the execution of
//! the task is successful, and its internal semaphore is used to synchronously obtain
//! the output of the predecessor task as the input of this task.

use std::{
    slice::Iter,
    sync::atomic::{AtomicBool, AtomicPtr, Ordering},
};

use anymap2::{any::CloneAnySendSync, Map};
use tokio::sync::Semaphore;

pub type Content = Map<dyn CloneAnySendSync + Send + Sync>;

/// Describe task's running result.
#[derive(Debug)]
pub(crate) struct ExecState {
    /// The execution succeed or not.
    success: AtomicBool,
    /// Output produced by a task.
    output: AtomicPtr<Output>,
    /// Task output identified by id.
    tid: usize,
    /// The semaphore is used to control the synchronous blocking of subsequent tasks to obtain the
    /// execution results of this task.
    /// When a task is successfully executed, the permits inside the semaphore will be increased to
    /// n (n represents the number of successor tasks of this task or can also be called the output
    /// of the node), which means that the output of the task is available, and then each successor
    /// The task will obtain a permits synchronously (the permit will not be returned), which means
    /// that the subsequent task has obtained the execution result of this task.
    semaphore: Semaphore,
}

/// Output produced by a task.
#[derive(Debug)]
pub struct Output(Option<Content>);

/// Task's input value.
pub struct Input(Vec<Content>);

#[allow(dead_code)]
impl ExecState {
    /// Construct a new [`ExeState`].
    pub(crate) fn new(task_id: usize) -> Self {
        Self {
            success: AtomicBool::new(false),
            output: AtomicPtr::new(std::ptr::null_mut()),
            tid: task_id,
            semaphore: Semaphore::new(0),
        }
    }

    /// After the task is successfully executed, set the execution result.
    pub(crate) fn set_output(&self, output: Output) {
        self.success.store(true, Ordering::Relaxed);
        self.output
            .store(Box::leak(Box::new(output)), Ordering::Relaxed);
    }

    /// [`Output`] for fetching internal storage.
    /// This function is generally not called directly, but first uses the semaphore for synchronization control.
    pub(crate) fn get_output(&self) -> Option<Content> {
        unsafe { self.output.load(Ordering::Relaxed).as_ref().unwrap() }
            .0
            .clone()
    }

    /// The task execution succeed or not.
    /// `true` means no panic occurs.
    pub(crate) fn success(&self) -> bool {
        self.success.load(Ordering::Relaxed)
    }

    /// Use id to indicate the output of which task.
    pub(crate) fn tid(&self) -> usize {
        self.tid
    }
    /// The semaphore is used to control the synchronous acquisition of task output results.
    /// Under normal circumstances, first use the semaphore to obtain a permit, and then call
    /// the `get_output` function to obtain the output. If the current task is not completed
    /// (no output is generated), the subsequent task will be blocked until the current task
    /// is completed and output is generated.
    pub(crate) fn semaphore(&self) -> &Semaphore {
        &self.semaphore
    }
}

impl Output {
    /// Construct a new [`Output`].
    ///
    /// Since the return value may be transferred between threads,
    /// [`Send`], [`Sync`], [`CloneAnySendSync`] is needed.
    pub fn new<H: Send + Sync + CloneAnySendSync>(val: H) -> Self {
        let mut map = Content::new();
        assert!(map.insert(val).is_none(), "[Error] map insert fails.");
        Self(Some(map))
    }

    /// Construct an empty [`Output`].
    pub fn empty() -> Self {
        Self(None)
    }
}

impl Input {
    /// Constructs input using output produced by a non-empty predecessor task.
    pub fn new(input: Vec<Content>) -> Self {
        Self(input)
    }

    /// Since [`Input`] can contain multi-input values, and it's implemented
    /// by [`Vec`] actually, of course it can be turned into a iterator.
    pub fn get_iter(&self) -> Iter<Content> {
        self.0.iter()
    }
}
