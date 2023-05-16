//! The Engine
//!
//! ## [`DagEngine`] is dagrs's main body.
//!
//! [`DagEngine`] is the execution engine of the task graph, and the constructed tasks are
//! stored in the form of [`Graph`]. The execution process of the engine is as follows:
//!
//! First, check that the built graph cannot have loops, otherwise the execution will fail;
//! Then obtain the sequence of tasks according to topological sorting, and execute the tasks in order.
//! It should be noted that the execution mode of the tasks is asynchronous;
//! Finally, the task The execution output will be stored in the `execute_states` field.
//! The next task gets the required input through the `execute_states` field.

use super::{
    env_variables::EnvVar,
    error_handler::{DagError, RunningError},
    graph::Graph,
};
use crate::task::{ExecState, Input, TaskWrapper, YamlTask};
use anymap2::any::CloneAnySendSync;
use log::*;
use std::{collections::HashMap, sync::Arc};
use tokio::task::JoinHandle;
/// dagrs's function is wrapped in DagEngine struct.
pub struct DagEngine {
    /// Store all tasks' infos.
    ///
    /// Arc but no mutex, because only one thread will change [`TaskWrapper`]at a time.
    /// And no modification to [`TaskWrapper`] happens during the execution of it.
    tasks: HashMap<usize, Arc<TaskWrapper>>,
    /// Store dependency relations.
    rely_graph: Graph,
    /// Store a task's running result.Execution results will be read and written asynchronously by several threads.
    execute_states: HashMap<usize, Arc<ExecState>>,
    /// Environment Variables.
    env: EnvVar,
    /// The id of the last task.
    last_task_id: usize,
}

impl DagEngine {
    /// Allocate a new DagEngine.
    ///
    /// # Example
    /// ```
    /// let dagrs = dagrs::DagEngine::new();
    /// ```
    pub fn new() -> DagEngine {
        DagEngine {
            tasks: HashMap::new(),
            rely_graph: Graph::new(),
            execute_states: HashMap::new(),
            env: EnvVar::new(),
            last_task_id: 0,
        }
    }

    pub fn set_env<T: Send + Sync + CloneAnySendSync>(&mut self, k: &str, v: T) {
        self.env.set(k, v);
    }

    /// Add new tasks into dagrs.
    ///
    /// # Example
    /// ```
    /// # let mut dagrs = dagrs::DagEngine::new();
    /// # struct T {};
    /// # impl dagrs::TaskTrait for T {
    /// #     fn run( &self, input: dagrs::Input, env: dagrs::EnvVar ) -> dagrs::Output {
    /// #         dagrs::Output::empty()
    /// #     }
    /// # }
    /// # let task1 = dagrs::TaskWrapper::new( T{}, "name1" );
    /// # let task2 = dagrs::TaskWrapper::new( T{}, "name2" );
    /// dagrs.add_tasks(vec![task1, task2]);
    /// ```
    ///
    /// You should defined the struct(here is T) and the function run in TaskTrait by yourself.
    /// You can find more information about TaskWrapper in src/task/task.rs
    pub fn add_tasks(&mut self, tasks: Vec<TaskWrapper>) {
        for task in tasks {
            self.tasks.insert(task.get_id(), Arc::new(task));
        }
    }

    /// Do dagrs's job.
    ///
    /// # Example
    /// ```
    /// # let mut dagrs = dagrs::DagEngine::new();
    /// # //Add some tasks to dagrs.
    /// dagrs.run();
    /// ```
    ///
    /// **Note:** This method must be called after all tasks have been added into dagrs.
    pub fn run(&mut self) -> Result<bool, DagError> {
        self.create_graph()?;
        let rt = tokio::runtime::Runtime::new().unwrap();
        Ok(rt.block_on(async { self.check_dag().await }))
    }

    /// Do dagrs's job from yaml file.
    ///
    /// # Example
    /// ```
    /// # let dagrs = dagrs::DagEngine::new();
    /// dagrs.run_from_yaml("test/test_dag1.yaml");
    /// ```
    ///
    /// This method is similar to `run`, but read tasks from yaml file,
    /// thus no need to add tasks manually.
    pub fn run_from_yaml(mut self, filename: &str) -> Result<bool, DagError> {
        self.read_tasks(filename)?;
        self.run()
    }

    /// Read tasks into engine through yaml.
    ///
    /// This operation will read all info in yaml file into `dagrs.tasks` if no error occurs.
    fn read_tasks(&mut self, filename: &str) -> Result<(), DagError> {
        let tasks = YamlTask::from_yaml(filename)?;
        tasks.into_iter().map(|t| self.add_tasks(vec![t])).count();
        Ok(())
    }

    /// create rely map between tasks.
    ///
    /// This operation will initialize `dagrs.rely_graph` if no error occurs.
    fn create_graph(&mut self) -> Result<(), DagError> {
        let size = self.tasks.len();
        self.rely_graph.set_graph_size(size);

        // Add Node (create id - index mapping)
        self.tasks
            .iter()
            .map(|(&n, _)| self.rely_graph.add_node(n))
            .count();

        // Form Graph
        for (&id, task) in self.tasks.iter() {
            let index = self.rely_graph.find_index_by_id(&id).unwrap();

            for rely_task_id in task.get_predecessors_id() {
                // Rely task existence check
                let rely_index = self.rely_graph.find_index_by_id(&rely_task_id).ok_or(
                    DagError::running_error(RunningError::RelyTaskIllegal(task.get_name())),
                )?;

                self.rely_graph.add_edge(rely_index, index);
            }
        }

        Ok(())
    }

    fn init_execute_states(&mut self, tasks_id: &[usize]) {
        tasks_id.iter().for_each(|id| {
            let task_id = id.clone();
            self.execute_states
                .insert(task_id, Arc::new(ExecState::new(task_id)));
        });
    }

    /// Check whether it's DAG or not.
    ///
    /// If it is a DAG, dagrs will start executing tasks in a feasible order and
    /// return true when execution done, or it return a false.
    ///
    async fn check_dag(&mut self) -> bool {
        if let Some(seq) = self.rely_graph.topo_sort() {
            let seq: Vec<usize> = seq
                .into_iter()
                .map(|index| self.rely_graph.find_id_by_index(index).unwrap())
                .collect();
            // If there is no task, return true directly.
            if seq.is_empty() {
                return true;
            }
            self.print_seq(&seq);
            // Set the execution results of all tasks to empty and set them to the status of unsuccessful execution.
            self.init_execute_states(&seq);
            // Set the id of the last task, which can be used to get the final execution result.
            self.last_task_id = seq.last().unwrap().clone();
            // storage execute JoinHandle<bool>.
            let mut handles = Vec::new();
            seq.iter().for_each(|id|{
                let task = self.tasks[id].clone();
                let env = self.env.clone();
                let execute_state = self.execute_states[id].clone();
                let task_out_degree = self.rely_graph.get_node_out_degree(id);
                let wait_for_input: Vec<Arc<ExecState>> = task
                    .get_predecessors_id()
                    .iter()
                    .map(|id| self.execute_states[id].clone())
                    .collect();
                // async execute
                handles.push(self.execute_task(task, wait_for_input, env, execute_state, task_out_degree));
            });
            // Wait for the status of each task to execute. If there is an error in the execution of a task,
            // the engine will fail to execute and give up executing tasks that have not yet been executed.
            for handle in handles {
                match handle.await {
                    Ok(complete) => {
                        if !complete {
                            std::process::abort()
                        }
                    }
                    Err(_) => std::process::abort(),
                }
            }
            true
        } else {
            error!("Loop Detect");
            false
        }
    }

    /// Print possible execution sequences.
    fn print_seq(&self, seq: &[usize]) {
        let mut res = String::from("[Start]");
        seq.iter()
            .map(|id| res.push_str(&format!(" -> {}", self.tasks[id].get_name())))
            .count();
        info!("{} -> [End]", res);
    }
    /// Execute a given task asynchronously.
    fn execute_task(
        &self,
        task: Arc<TaskWrapper>,
        wait_for_input: Vec<Arc<ExecState>>,
        env: EnvVar,
        execute_state: Arc<ExecState>,
        task_out_degree: usize,
    ) -> JoinHandle<bool> {
        tokio::spawn(async move {
            info!("Executing Task[name: {}]", task.get_name());
            // Wait for the execution result of the predecessor task
            let mut inputs = Vec::new();
            for wait_for in wait_for_input {
                wait_for.acquire_permits().await;
                if let Some(content) = wait_for.get_output() {
                    if !content.is_empty() {
                        inputs.push(content);
                    }
                }
            }
            // Start run task
            match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                task.run(Input::new(inputs), env)
            })) {
                Ok(output) => {
                    info!("Finish Task[name: {}]", task.get_name());
                    // Store execution results
                    execute_state.set_output(output);
                    execute_state.add_permits(task_out_degree);
                    true
                }
                Err(err) => {
                    error!("Task Failed[name: {}, err: {:?}]", task.get_name(), err);
                    false
                }
            }
        })
    }

    /// Get the final execution result.
    pub fn get_result<T: CloneAnySendSync + Send + Sync>(&mut self) -> Option<T> {
        match self.execute_states[&self.last_task_id].get_output() {
            Some(ref content) => content.clone().remove(),
            None => None,
        }
    }
}

impl Default for DagEngine {
    fn default() -> Self {
        DagEngine {
            tasks: HashMap::new(),
            rely_graph: Graph::new(),
            execute_states: HashMap::new(),
            env: EnvVar::new(),
            last_task_id: 0,
        }
    }
}
