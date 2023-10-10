use std::sync::Arc;

use super::{Action, Task, ID_ALLOCATOR};

/// A default implementation of the Task trait. In general, use it to define the tasks of dagrs.
pub struct DefaultTask {
    /// id is the unique identifier of each task, it will be assigned by the global [`IDAllocator`]
    /// when creating a new task, you can find this task through this identifier.
    id: usize,
    /// The task's name.
    name: String,
    /// Id of the predecessor tasks.
    precursors: Vec<usize>,
    /// Perform specific actions.
    action: Arc<dyn Action + Send + Sync>,
}

impl DefaultTask {
    /// Allocate a new [`DefaultTask`], the specific task behavior is a structure that implements [`Action`].
    ///
    /// # Example
    ///
    /// ```rust
    /// use dagrs::{DefaultTask, Output,Input, Action,EnvVar,RunningError};
    /// use std::sync::Arc;
    /// struct SimpleAction(usize);
    ///
    /// impl Action for SimpleAction {
    /// fn run(&self, input: Input, env: Arc<EnvVar>) -> Result<Output,RunningError> {
    ///     Ok(Output::new(self.0 + 10))
    /// }
    /// }
    ///
    /// let action = SimpleAction(10);
    /// let task = DefaultTask::new(action, "Increment action");
    /// ```
    ///
    /// `SimpleAction` is a struct that impl [`Action`]. Since task will be
    ///  executed in separated threads, [`Send`] and [`Sync`] is needed.
    ///
    /// **Note:** This method will take the ownership of struct that impl [`Action`].
    pub fn new(action: impl Action + 'static + Send + Sync, name: &str) -> Self {
        DefaultTask {
            id: ID_ALLOCATOR.alloc(),
            action: Arc::new(action),
            name: name.to_owned(),
            precursors: Vec::new(),
        }
    }

    /// Tasks that shall be executed before this one.
    ///
    /// # Example
    /// ```rust
    /// use dagrs::{Action,DefaultTask,Input,Output,RunningError,EnvVar};
    /// use std::sync::Arc;
    /// struct SimpleAction {};
    /// impl Action for SimpleAction {
    ///     fn run(&self, input: Input, env: Arc<EnvVar>) -> Result<Output,RunningError> {
    ///         Ok(Output::empty())
    ///     }
    /// }
    /// let mut t1 = DefaultTask::new(SimpleAction{}, "Task 1");
    /// let mut t2 = DefaultTask::new(SimpleAction{}, "Task 2");
    /// t2.set_predecessors(&[&t1]);
    /// ```
    /// In above code, `t1` will be executed before `t2`.
    pub fn set_predecessors<'a>(
        &mut self,
        predecessors: impl IntoIterator<Item = &'a &'a DefaultTask>,
    ) {
        self.precursors
            .extend(predecessors.into_iter().map(|t| t.id()))
    }

    /// The same as `exec_after`, but input are tasks' ids
    /// rather than reference to [`DefaultTask`].
    pub fn set_predecessors_by_id(&mut self, predecessors_id: impl IntoIterator<Item = usize>) {
        self.precursors.extend(predecessors_id)
    }

    pub fn set_name(&mut self,name: &str){
        self.name=name.to_owned();
    }

    pub fn set_action(&mut self,action: impl Action + 'static + Send + Sync){
        self.action=Arc::new(action);
    }
}

impl Task for DefaultTask {
    fn action(&self) -> Arc<dyn Action + Send + Sync> {
        self.action.clone()
    }

    fn precursors(&self) -> &[usize] {
        &self.precursors
    }

    fn id(&self) -> usize {
        self.id
    }

    fn name(&self) -> String {
        self.name.clone()
    }
}

impl Default for DefaultTask {
    fn default() -> Self {
        let action = crate::gen_action!(|_input, _env| { Output::empty() });
        Self {
            id: ID_ALLOCATOR.alloc(),
            name: "default".to_owned(),
            precursors: Vec::new(),
            action: Arc::new(action),
        }
    }
}
