//! The Engine
//!
//! # dagrs core logic.
//!
//! [`Dag`] consists of a series of executable tasks with dependencies. A Dag can be executed
//! alone as a job. We can get the execution result and execution status of dag.
//! [`Engine`] can manage multiple [`Dag`]. An Engine can consist of multiple Dags of different
//! types of tasks. For example, you can give a Dag in the form of a yaml configuration file,
//! then give a Dag in the form of a custom configuration file, and finally give it in a programmatic way.
//! [`Engine`] stores each Dag in the form of a key-value pair (<name:String,dag:Dag>), and the user
//! can specify which task to execute by giving the name of the Dag, or follow the order in which
//! the Dags are added to the Engine , executing each Dag in turn.

pub use dag::Dag;
pub use error::DagError;

mod dag;
mod error;
mod graph;

use std::collections::HashMap;

use anymap2::any::CloneAnySendSync;
use tokio::runtime::Runtime;

use crate::log;

/// The Engine. Manage multiple Dags.
pub struct Engine {
    dags: HashMap<String, Dag>,
    /// According to the order in which Dags are added to the Engine, assign a sequence number to each Dag.
    /// Sequence numbers can be used to execute Dags sequentially.
    sequence: HashMap<usize, String>,
    /// A tokio runtime.
    /// In order to save computer resources, multiple Dags share one runtime.
    runtime: Runtime,
}

impl Engine {
    /// Add a Dag to the Engine and assign a sequence number to the Dag.
    /// It should be noted that different Dags should specify different names.
    pub fn append_dag(&mut self, name: &str, mut dag: Dag) {
        if !self.dags.contains_key(name) {
            match dag.init() {
                Ok(()) => {
                    self.dags.insert(name.to_string(), dag);
                    let len = self.sequence.len();
                    self.sequence.insert(len + 1, name.to_string());
                }
                Err(err) => {
                    log::error(format!("Some error occur: {}", err));
                }
            }
        }
    }

    /// Given a Dag name, execute this Dag.
    /// Returns true if the given Dag executes successfully, otherwise false.
    pub fn run_dag(&mut self, name: &str) -> bool {
        if !self.dags.contains_key(name) {
            log::error(format!("No job named '{}'", name));
            false
        } else {
            let job = self.dags.get(name).unwrap();
            self.runtime.block_on(job.run())
        }
    }

    /// Execute all the Dags in the Engine in sequence according to the order numbers of the Dags in
    /// the sequence from small to large. The return value is the execution status of all tasks.
    pub fn run_sequential(&mut self) ->Vec<bool>{
        let mut res=Vec::new();
        for seq in 1..self.sequence.len() + 1 {
            let name = self.sequence.get(&seq).unwrap().clone();
            res.push(self.run_dag(name.as_str()));
        }
        res
    }

    /// Given the name of the Dag, get the execution result of the specified Dag.
    pub fn get_dag_result<T: CloneAnySendSync + Send + Sync>(&self,name:&str)->Option<T>{
        if self.dags.contains_key(name) {
            self.dags.get(name).unwrap().get_result()
        } else {
            None
        }
    }
}

impl Default for Engine {
    fn default() -> Self {
        Self {
            dags: HashMap::new(),
            runtime: Runtime::new().unwrap(),
            sequence: HashMap::new(),
        }
    }
}
