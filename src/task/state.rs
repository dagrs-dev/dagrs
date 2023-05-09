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

use std::{fmt::Display, slice::Iter};

use anymap2::{any::CloneAnySendSync, Map};

pub type Content = Map<dyn CloneAnySendSync + Send + Sync>;

/// Describe task's running result

#[derive(Debug)]
pub struct ExecState {
    /// The execution succeed or not
    success: bool,
    /// Return value of the execution.
    output: Output,
    /// TODO
    executed: bool,
    
    task_id: usize,
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
            success: false,
            output: Output::empty(),
            executed: false,
            task_id,
        }
    }

    /// TODO
    pub fn set_output(&mut self, output: Output) {
        self.success = true;
        self.executed = true;
        self.output = output;
    }

    /// TODO
    pub fn set_executed(&mut self) {
        self.executed = true;
    }

    /// Get [`ExecState`]'s return value.
    ///
    /// This method will clone [`DMap`] that are stored in [`ExecState`]'s [`Output`].
    pub fn get_output(&self) -> Option<Content> {
        self.output.0.clone()
    }

    /// The task execution succeed or not.
    ///
    /// `true` means no panic occurs.
    pub fn success(&self) -> bool {
        self.success
    }
    
    pub fn executed(&self) -> bool {
        self.executed
    }

    pub fn get_id(&self) -> usize {
        self.task_id
    }
}

impl Display for ExecState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "state --- success: {}, executed: {}, output: {}>",
            self.success,
            self.executed,
            match self.output.0 {
                Some(_) => {
                    "Some"
                }
                None => {
                    "None"
                }
            }
        )
    }
}

impl Output {
    #[allow(unused)]
    /// Get a new [`Output`].
    ///
    /// Since the return value may be transfered between threads,
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
    /// # let mut input = dagrs::Input::new( vec![ None ] );
    /// let input_from_t1:Option<String> = input.get(0);
    /// ```
    pub fn get<H: Send + Sync + CloneAnySendSync>(&mut self, index: usize) -> Option<H> {
        if let Some(content) = self.0.get_mut(index) {
            content.remove()
        } else {
            None
        }
    }

    /// Since [`Input`] can contain mult-input values, and it's implemented
    /// by [`Vec`] actually, of course it can be turned into a iterator.
    pub fn get_iter(&self) -> Iter<Content> {
        self.0.iter()
    }
}
