//! User-defined task flow configuration file parser.
//!
//! ## Configuration file parsing process
//!
//! There are several task configuration information in the configuration file,
//! and each task may have 0~n predecessors and successors (the first executed
//! task has no predecessor, and the last task has no successor). Each task has a
//! unique identification, and the parser will determine all the predecessors and
//! successors of a task according to the id, and then parse and encapsulate a task
//! configuration information into a [`TaskWrapper`], and all tasks are finally put
//! into a [`Vec`] container.
//! The engine supports two types of tasks given by the configuration file,
//! namely running sh scripts and JavaScript scripts, and the parser will generate
//! [`TaskWrapper`] according to the type of tasks.

use super::{Action, Task, ID_ALLOCATOR};

/// Task struct for YAML file.
pub struct YamlTask {
    /// Task's id in yaml file.
    yaml_id: (String, usize),
    /// Task's name.
    name: String,
    /// Record tasks' `yaml_id` that shall be executed before this task.
    precursors: Vec<String>,
    precursors_id: Vec<usize>,
    /// A field shall be wrapper into [`TaskWrapper`] later.
    ///
    /// Why [`Cell`] and [`Option`]? Useful in function `from_yaml`.
    runnable: Action,
}

// /// !TODO
// pub struct JsonTask {}

impl YamlTask {
    pub fn new(yaml_id: &str, precursors: Vec<String>, name: String, runnable: Action) -> Self {
        Self {
            yaml_id: (yaml_id.to_owned(), ID_ALLOCATOR.alloc()),
            name,
            precursors,
            precursors_id:Vec::new(),
            runnable
        }
    }
    pub fn init_precursors(&mut self, pres_id: Vec<usize>) {
        self.precursors_id=pres_id;
    }
    pub fn str_precursors(&self)->Vec<String>{
        self.precursors.clone()
    }
    pub fn str_id(&self)->String{
        self.yaml_id.0.clone()
    }
}

impl Task for YamlTask {
    fn id(&self) -> usize {
        self.yaml_id.1
    }
    fn name(&self) -> String {
        self.name.clone()
    }
    fn predecessors(&self) -> &[usize] {
        &self.precursors_id
    }
    fn runnable(&self) -> Action {
        self.runnable.clone()
    }
}

// impl YamlTask {
//     /// Parse a task from yaml.
//     ///
//     /// # Example
//     /// ```
//     /// use std::io::Read;
//     /// let mut yaml_cont = String::new();
//     /// let mut yaml_file = std::fs::File::open("test/test_dag1.yaml");
//     /// yaml_file.expect("REASON").read_to_string(&mut yaml_cont);
//     /// let yaml_tasks = yaml_rust::YamlLoader::load_from_str(&yaml_cont).expect("REASON");
//     /// let yaml_tasks = yaml_tasks[0]["dagrs"].as_hash().expect("REASON");
//     /// for (id, yaml) in yaml_tasks {
//     ///     let task = dagrs::YamlTask::parse_one(id.as_str().unwrap(), yaml);
//     /// }
//     /// ```

//     /// Parse all tasks from yaml file.
//     ///
//     /// # Example
//     /// ```
//     /// let tasks = dagrs::YamlTask::parse_tasks("test/test_dag.yaml");
//     /// ```
//     pub fn parse_tasks(filename: &str) -> Result<Vec<Self>, DagError> {}

//     /// Parse all tasks from yaml file into format recognized by dagrs.
//     ///
//     /// # Example
//     /// ```
//     /// let tasks = dagrs::YamlTask::from_yaml("test/test_dag1.yaml");
//     /// ```
//     ///
//     /// Used in [`crate::DagEngine`].
//     pub fn from_yaml(filename: &str) -> Result<Vec<DefaultTask>, DagError> {
//         let yaml_tasks = YamlTask::parse_tasks(filename)?;
//         let mut tasks = Vec::new();
//         let mut yid2id = HashMap::new();

//         // Form tasks
//         for ytask in &yaml_tasks {
//             let task = DefaultTask::new(
//                 ytask
//                     .runnable
//                     .replace(None)
//                     .expect("[Fatal] Abnormal error occurs."),
//                 &ytask.name,
//             );
//             yid2id.insert(ytask.yaml_id.clone(), task.get_id());
//             tasks.push(task);
//         }

//         for (index, ytask) in yaml_tasks.iter().enumerate() {
//             let afters: Vec<usize> = ytask
//                 .preducessors
//                 .iter()
//                 .map(|after| yid2id.get(after).unwrap_or(&0).to_owned())
//                 .collect();
//             // Task 0 won't exist in normal state, thus this will trigger an RelyTaskIllegal Error later.
//             tasks[index].set_predecessors(&afters);
//         }

//         Ok(tasks)
//     }
// }

