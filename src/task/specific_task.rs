//! Task definition of type Yaml.
//!
//! # The task type corresponding to the configuration file: [`YamlTask`]
//!
//! [`YamlTask`] implements the [`Task`] trait, which represents the tasks in the yaml
//! configuration file, and a yaml configuration file will be parsed into a series of [`YamlTask`].
//! It is different from [`DefaultTask`], in addition to the four mandatory attributes of the
//! task type, he has several additional attributes.

use std::sync::Arc;

use super::{Action, ID_ALLOCATOR, Task};

/// Task struct for yaml file.
pub struct YamlTask {
    /// `tid.0` is the unique identifier defined in yaml, and `tid.1` is the id assigned by the global id assigner.
    tid: (String, usize),
    name: String,
    /// Precursor identifier defined in yaml.
    precursors: Vec<String>,
    precursors_id: Vec<usize>,
    action: Arc<dyn Action + Sync + Send>,
}

impl YamlTask {
    pub fn new(
        yaml_id: &str,
        precursors: Vec<String>,
        name: String,
        action: impl Action + Send + Sync + 'static,
    ) -> Self {
        Self {
            tid: (yaml_id.to_owned(), ID_ALLOCATOR.alloc()),
            name,
            precursors,
            precursors_id: Vec::new(),
            action: Arc::new(action),
        }
    }
    /// After the configuration file is parsed, the id of each task has been assigned.
    /// At this time, the `precursors_id` of this task will be initialized according to
    /// the id of the predecessor task of each task.
    pub fn init_precursors(&mut self, pres_id: Vec<usize>) {
        self.precursors_id = pres_id;
    }

    /// Get the precursor identifier defined in yaml.
    pub fn str_precursors(&self) -> Vec<String> {
        self.precursors.clone()
    }
    /// Get the unique ID of the task defined in yaml.
    pub fn str_id(&self) -> String {
        self.tid.0.clone()
    }
}

impl Task for YamlTask {
    fn action(&self) -> Arc<dyn Action + Sync + Send> {
        self.action.clone()
    }
    fn predecessors(&self) -> &[usize] {
        &self.precursors_id
    }
    fn id(&self) -> usize {
        self.tid.1
    }
    fn name(&self) -> String {
        self.name.clone()
    }
}
