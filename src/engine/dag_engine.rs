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
//! Finally, the task The execution output will be stored in the `execstate_store` field.
//! The next task gets the required input through the `execstate_store` field.

use super::{
    env_variables::EnvVar,
    error_handler::{DagError, RunningError},
    graph::Graph,
};
use crate::{task::{ExecState, Input, TaskWrapper, YamlTask}, Output};
use anymap2::any::CloneAnySendSync;
use log::*;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::{Notify,RwLock};

/// dagrs's function is wrapped in DagEngine struct.
pub struct DagEngine {
    /// Store all tasks' infos.
    ///
    /// Arc but no mutex, because only one thread will change [`TaskWrapper`]at a time.
    /// And no modification to [`TaskWrapper`] happens during the execution of it.
    tasks: HashMap<usize, Arc<TaskWrapper>>,
    /// Store dependency relations.
    rely_graph: Graph,
    /// Store a task's running result.
    execstate_store: Arc<RwLock<HashMap<usize, ExecState>>>,
    // Environment Variables.
    env: EnvVar,
    notifies: HashMap<usize, Arc<Notify>>,
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
            execstate_store: Arc::new(RwLock::new(HashMap::new())),
            env: EnvVar::new(),
            notifies:HashMap::new()
        }
    }

    /// Add new tasks into dagrs.
    ///
    /// # Example
    /// ```
    /// # let mut dagrs = dagrs::DagEngine::new();
    /// # struct T {};
    /// # impl dagrs::TaskTrait for T {
    /// #     fn run( &self, input: dagrs::Inputval, env: dagrs::EnvVar ) -> dagrs::Retval {
    /// #         dagrs::Retval::empty()
    /// #     }
    /// # }
    /// # let task1 = dagrs::TaskWrapper::new( T{}, "name1" );
    /// # let task2 = dagrs::TaskWrapper::new( T{}, "name2" );
    /// dagrs.add_tasks(vec![task1, task2]);
    /// ```
    ///
    /// You should defined the struct(here is T) and the function run in TaskTrait by yourself.
    /// You can find more infomation about TaskWrapper in src/task/task.rs
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
    /// thus no need to add tasks mannually.
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

            for rely_task_id in task.get_exec_after_list() {
                // Rely task existence check
                let rely_index = self.rely_graph.find_index_by_id(&rely_task_id).ok_or(
                    DagError::running_error(RunningError::RelyTaskIllegal(task.get_name())),
                )?;

                self.rely_graph.add_edge(rely_index, index);
            }
        }

        Ok(())
    }

    fn init_notifies(&mut self, tasks_id: &[usize]) {
        tasks_id.into_iter().for_each(|id| {
            self.notifies.insert(id.clone(), Arc::new(Notify::new()));
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
            self.print_seq(&seq);
            // init notifiers
            self.init_notifies(&seq);
            // storage execute status
            let mut handles=Vec::new();
            // Start Executing
            for id in seq {
                let task_name=self.tasks[&id].get_name();
                let task=self.tasks[&id].clone();
                let env=self.env.clone();
                let execstate_storage=self.execstate_store.clone();
                
                let signal=self.notifies[&id].clone();
                let waite_input:Vec<(usize,Arc<Notify>)>=self.tasks.get(&id).unwrap().get_exec_after_list().iter().map(|i|{
                    (i.clone(),self.notifies.get(i).unwrap().clone())
                }).collect();
                
                // async execute
                let handle = tokio::spawn(async move {
                    info!("Executing Task[name: {}]", task_name);
                    let mut inputs=Vec::new();
                    for (input_id,notifier) in waite_input{
                        notifier.notified().await;
                        let reader = execstate_storage.read().await;
                        let execstate=reader.get(&input_id).unwrap();
                        if !execstate.success(){
                            return false;
                        }
                        inputs.push(execstate.get_dmap());
                    }

                    let state=match std::panic::catch_unwind(std::panic::AssertUnwindSafe(||{
                                            task.run(Input::new(inputs), env)
                                        })) {
                        Ok(res) => {
                            info!("Finish Task[name: {}]",task_name);
                            ExecState::new(true, res)
                        },
                        Err(err) => {
                            error!("Task Failed[name: {}],error with {:?}",task_name,err);
                            ExecState::new(false, Output::empty())
                        }
                    };
                    execstate_storage.write().await.insert(id, state);                  
                    signal.notify_waiters();
                    true
                });
                handles.push(handle);
            }
            for handle in handles{
                handle.await.unwrap();
            }
            true
        } else {
            error!("Loop Detect");
            false
        }
    }

    /// Print possible execution sequnces.
    fn print_seq(&self, seq: &[usize]) {
        let mut res = String::from("[Start]");
        seq.iter()
            .map(|id| res.push_str(&format!(" -> {}", self.tasks[id].get_name())))
            .count();
        info!("{} -> [End]", res);
    }

    pub fn set_env<T:Send + Sync + CloneAnySendSync>(&mut self,k:&str,v:T){
        self.env.set(k, v);
    }
}

impl Default for DagEngine {
    fn default() -> Self {
        DagEngine {
            tasks: HashMap::new(),
            rely_graph: Graph::new(),
            execstate_store: Arc::new(RwLock::new(HashMap::new())),
            env: EnvVar::new(),
            notifies:HashMap::new()
        }
    }
}
