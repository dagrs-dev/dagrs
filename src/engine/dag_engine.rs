//! Dag Engine is dagrs's main body

use super::{
    env_variables::EnvVar,
    error_handler::{DagError, RunningError},
    graph::Graph,
};
use crate::task::{ExecState, Inputval, Retval, TaskWrapper, YamlTask};
use log::*;
use std::{collections::HashMap, sync::Arc};

/// dagrs's function is wrapped in DagEngine struct
pub struct DagEngine {
    /// Store all tasks' infos.
    ///
    /// Arc but no mutex, because only one thread will change [`TaskWrapper`]
    /// at a time. And no modification to [`TaskWrapper`] happens during the execution of it.
    tasks: HashMap<usize, Arc<TaskWrapper>>,
    /// Store dependency relations
    rely_graph: Graph,
    /// Store a task's running result
    execstate_store: HashMap<usize, ExecState>,
    // Environment Variables
    env: EnvVar,
}

impl DagEngine {
    /// Allocate a new DagEngine.
    ///
    /// # Example
    /// ```
    /// let dagrs = DagEngine::new();
    /// ```
    pub fn new() -> DagEngine {
        DagEngine {
            tasks: HashMap::new(),
            rely_graph: Graph::new(),
            execstate_store: HashMap::new(),
            env: EnvVar::new(),
        }
    }

    /// Do dagrs's job.
    ///
    /// # Example
    /// ```
    /// let dagrs = DagEngine::new();
    /// dagrs.add_tasks(vec![task1, task2]);
    /// ```
    ///
    /// Here `task1` and `task2` are user defined task wrapped in [`TaskWrapper`].
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
    /// let dagrs = DagEngine::new();
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

    /// Add new tasks into dagrs
    ///
    /// # Example
    /// ```
    /// let dagrs = DagEngine::new();
    /// dagrs.add_tasks(vec![task1, task2]);
    /// dagrs.run();
    /// ```
    ///
    /// Here `task1` and `task2` are user defined task wrapped in [`TaskWrapper`].
    pub fn add_tasks(&mut self, tasks: Vec<TaskWrapper>) {
        for task in tasks {
            self.tasks.insert(task.get_id(), Arc::new(task));
        }
    }

    /// Push a task's [`ExecState`] into hash store
    fn push_execstate(&mut self, id: usize, state: ExecState) {
        assert!(
            !self.execstate_store.contains_key(&id),
            "[Error] Repetitive push execstate, id: {}",
            id
        );
        self.execstate_store.insert(id, state);
    }

    /// Fetch a task's [`ExecState`], this won't delete it from the hash map.
    fn pull_execstate(&self, id: &usize) -> &ExecState {
        self.execstate_store
            .get(id)
            .expect("[Error] Pull execstate fails")
    }

    /// Prepare a task's [`Inputval`].
    fn form_input(&self, id: &usize) -> Inputval {
        let froms = self.tasks[id].get_input_from_list();
        Inputval::new(
            froms
                .iter()
                .map(|from| self.pull_execstate(from).get_dmap())
                .collect(),
        )
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

    /// Check whether it's DAG or not.
    ///
    /// If it is a DAG, dagrs will start executing tasks in a feasible order and 
    /// return true when execution done, or it return a false.
    async fn check_dag(&mut self) -> bool {
        if let Some(seq) = self.rely_graph.topo_sort() {
            let seq = seq
                .into_iter()
                .map(|index| self.rely_graph.find_id_by_index(index).unwrap())
                .collect();
            self.print_seq(&seq);

            // Start Executing
            for id in seq {
                info!("Executing Task[name: {}]", self.tasks[&id].get_name());

                let input = self.form_input(&id);
                let env = self.env.clone();

                let task = self.tasks[&id].clone();
                let handle = tokio::spawn(async move { task.run(input, env) });

                // Recore executing state.
                let state = if let Ok(val) = handle.await {
                    ExecState::new(true, val)
                } else {
                    ExecState::new(false, Retval::empty())
                };

                info!(
                    "Finish Task[name: {}], success: {}",
                    self.tasks[&id].get_name(),
                    state.success()
                );
                // Push executing state in to store.
                self.push_execstate(id, state);
            }

            true
        } else {
            error!("Loop Detect");
            false
        }
    }

    /// Print possible execution sequnces.
    fn print_seq(&self, seq: &Vec<usize>) {
        let mut res = String::from("[Start]");
        seq.iter()
            .map(|id| res.push_str(&format!(" -> {}", self.tasks[id].get_name())))
            .count();
        info!("{} -> [End]", res);
    }
}
