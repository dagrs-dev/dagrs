//! Implement the Task trait to customize task properties.
//! MyTask is basically the same as DefaultTask provided by dagrs.

use std::sync::Arc;

use dagrs::{alloc_id, log, Action, Dag, EnvVar, Input, LogLevel, Output, RunningError, Task};

struct MyTask {
    id: usize,
    name: String,
    predecessor_tasks: Vec<usize>,
    action: Arc<dyn Action + Send + Sync>,
}

impl MyTask {
    pub fn new(action: impl Action + 'static + Send + Sync, name: &str) -> Self {
        MyTask {
            id: alloc_id(),
            action: Arc::new(action),
            name: name.to_owned(),
            predecessor_tasks: Vec::new(),
        }
    }

    pub fn set_predecessors(&mut self, predecessors: &[&MyTask]) {
        self.predecessor_tasks
            .extend(predecessors.iter().map(|t| t.id()))
    }
}

impl Task for MyTask {
    fn action(&self) -> Arc<dyn Action + Send + Sync> {
        self.action.clone()
    }

    fn precursors(&self) -> &[usize] {
        &self.predecessor_tasks
    }

    fn id(&self) -> usize {
        self.id
    }

    fn name(&self) -> String {
        self.name.clone()
    }
}

macro_rules! generate_task {
    ($action:ident($val:expr),$name:expr) => {{
        pub struct $action(usize);
        impl Action for $action {
            fn run(&self, input: Input, env: Arc<EnvVar>) -> Result<Output, RunningError> {
                let base = env.get::<usize>("base").unwrap();
                let mut sum = self.0;
                input
                    .get_iter()
                    .for_each(|i| sum += i.get::<usize>().unwrap() * base);
                Ok(Output::new(sum))
            }
        }
        MyTask::new($action($val), $name)
    }};
}

fn main() {
    let _initialized = log::init_logger(LogLevel::Info, None);
    let a = generate_task!(A(1), "Compute A");
    let mut b = generate_task!(B(2), "Compute B");
    let mut c = generate_task!(C(4), "Compute C");
    let mut d = generate_task!(D(8), "Compute D");
    let mut e = generate_task!(E(16), "Compute E");
    let mut f = generate_task!(F(32), "Compute F");
    let mut g = generate_task!(G(64), "Compute G");

    b.set_predecessors(&[&a]);
    c.set_predecessors(&[&a]);
    d.set_predecessors(&[&a]);
    e.set_predecessors(&[&b, &c]);
    f.set_predecessors(&[&c, &d]);
    g.set_predecessors(&[&b, &e, &f]);

    let mut env = EnvVar::new();
    env.set("base", 2usize);

    let mut dag = Dag::with_tasks(vec![a, b, c, d, e, f, g]);
    dag.set_env(env);
    assert!(dag.start().unwrap());

    let res = dag.get_result::<usize>().unwrap();
    println!("The result is {}.", res);
}
