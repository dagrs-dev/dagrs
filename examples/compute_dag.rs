//! Only use Dag, execute a job. The graph is as follows:
//!
//!    ↱----------↴
//!    B -→ E --→ G
//!  ↗    ↗     ↗
//! A --→ C    /
//!  ↘    ↘  /
//!   D -→ F
//!
//! The final execution result is 272.

extern crate dagrs;

use std::sync::Arc;

use dagrs::{log, Action, Dag, DefaultTask, EnvVar, Input, LogLevel, Output, RunningError};

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
        DefaultTask::new($action($val), $name)
    }};
}

fn main() {
    // initialization log.
    let _initialized = log::init_logger(LogLevel::Info, None);
    // generate some tasks.
    let a = generate_task!(A(1), "Compute A");
    let mut b = generate_task!(B(2), "Compute B");
    let mut c = generate_task!(C(4), "Compute C");
    let mut d = generate_task!(D(8), "Compute D");
    let mut e = generate_task!(E(16), "Compute E");
    let mut f = generate_task!(F(32), "Compute F");
    let mut g = generate_task!(G(64), "Compute G");
    // Set up task dependencies.
    b.set_predecessors(&[&a]);
    c.set_predecessors(&[&a]);
    d.set_predecessors(&[&a]);
    e.set_predecessors(&[&b, &c]);
    f.set_predecessors(&[&c, &d]);
    g.set_predecessors(&[&b, &e, &f]);
    // Create a new Dag.
    let mut dag = Dag::with_tasks(vec![a, b, c, d, e, f, g]);
    // Set a global environment variable for this dag.
    let mut env = EnvVar::new();
    env.set("base", 2usize);
    dag.set_env(env);
    // Start executing this dag
    assert!(dag.start().unwrap());
    // Get execution result.
    let res = dag.get_result::<usize>().unwrap();
    println!("The result is {}.", res);
}
