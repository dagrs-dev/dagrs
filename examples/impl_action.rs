//! Implement the Action trait to define the task logic.

extern crate dagrs;

use std::sync::Arc;

use dagrs::{
    Action,
    Dag, DefaultTask, EnvVar, Input, log, LogLevel, Output, RunningError,
};

struct SimpleAction(usize);

/// Implement the `Action` trait for `SimpleAction`, defining the logic of the `run` function.
/// The logic here is simply to get the output value (usize) of all predecessor tasks and then accumulate.
impl Action for SimpleAction {
    fn run(&self, input: Input, env: Arc<EnvVar>) -> Result<Output, RunningError> {
        let base = env.get::<usize>("base").unwrap();
        let mut sum = self.0;
        input
            .get_iter()
            .for_each(|i| sum += i.get::<usize>().unwrap() * base);
        Ok(Output::new(sum))
    }
}

fn main() {
    // Initialize the global logger
    let _initialized = log::init_logger(LogLevel::Info, None);
    // Generate four tasks.
    let a = DefaultTask::new(SimpleAction(10), "Task a");
    let mut b = DefaultTask::new(SimpleAction(20), "Task b");
    let mut c = DefaultTask::new(SimpleAction(30), "Task c");
    let mut d = DefaultTask::new(SimpleAction(40), "Task d");
    // Set the precursor for each task.
    b.set_predecessors(&[&a]);
    c.set_predecessors(&[&a]);
    d.set_predecessors(&[&b, &c]);
    // Take these four tasks as a Dag.
    let mut dag = Dag::with_tasks(vec![a, b, c, d]);
    // Set a global environment variable for this dag.
    let mut env = EnvVar::new();
    env.set("base", 2usize);
    dag.set_env(env);
    // Begin execution.
    assert!(dag.start().unwrap());
    // Get execution result
    assert_eq!(dag.get_result::<usize>().unwrap(), 220);
}
