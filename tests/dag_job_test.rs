use std::{sync::Arc, collections::HashMap};

///! Some tests of the dag engine.
use dagrs::{Action, Dag, DagError, DefaultTask, EnvVar, Input, Output, RunningError,log,LogLevel};

#[test]
fn yaml_task_correct_execute() {
    log::init_logger(LogLevel::Info, None);
    let mut job = Dag::with_yaml("tests/config/correct.yaml",HashMap::new()).unwrap();
    assert!(job.start().unwrap());
}

#[test]
fn yaml_task_loop_graph() {
    log::init_logger(LogLevel::Info, None);
    let res = Dag::with_yaml("tests/config/loop_error.yaml",HashMap::new())
        .unwrap()
        .start();
    assert!(matches!(res, Err(DagError::LoopGraph)))
}

#[test]
fn yaml_task_self_loop_graph() {
    log::init_logger(LogLevel::Info, None);
    let res = Dag::with_yaml("tests/config/self_loop_error.yaml",HashMap::new())
        .unwrap()
        .start();
    assert!(matches!(res, Err(DagError::LoopGraph)))
}

#[test]
fn yaml_task_failed_execute() {
    log::init_logger(LogLevel::Info, None);
    let res = Dag::with_yaml("tests/config/script_run_failed.yaml",HashMap::new())
        .unwrap()
        .start();
    assert!(!res.unwrap());
}

macro_rules! generate_task {
    ($task:ident($val:expr),$name:expr) => {{
        pub struct $task(usize);
        impl Action for $task {
            fn run(&self, input: Input, env: Arc<EnvVar>) -> Result<Output, RunningError> {
                let base = env.get::<usize>("base").unwrap();
                let mut sum = self.0;
                input
                    .get_iter()
                    .for_each(|i| sum += i.get::<usize>().unwrap() * base);
                Ok(Output::new(sum))
            }
        }
        DefaultTask::new($task($val), $name)
    }};
}

#[test]
fn task_loop_graph() {
    log::init_logger(LogLevel::Info, None);
    let mut a = generate_task!(A(1), "Compute A");
    let mut b = generate_task!(B(2), "Compute B");
    let mut c = generate_task!(C(4), "Compute C");
    a.set_predecessors(&[&b]);
    b.set_predecessors(&[&c]);
    c.set_predecessors(&[&a]);

    let mut env = EnvVar::new();
    env.set("base", 2usize);

    let mut job = Dag::with_tasks(vec![a, b, c]);
    job.set_env(env);
    let res = job.start();
    assert!(matches!(res, Err(DagError::LoopGraph)));
}

#[test]
fn non_job() {
    log::init_logger(LogLevel::Info, None);
    let tasks: Vec<DefaultTask> = Vec::new();
    let res = Dag::with_tasks(tasks).start();
    assert!(res.is_err());
    println!("{}", res.unwrap_err());
}

struct FailedActionC(usize);

impl Action for FailedActionC {
    fn run(&self, _input: Input, env: Arc<EnvVar>) -> Result<Output, RunningError> {
        let base = env.get::<usize>("base").unwrap();
        Ok(Output::new(base / self.0))
    }
}

struct FailedActionD(usize);

impl Action for FailedActionD {
    fn run(&self, _input: Input, _env: Arc<EnvVar>) -> Result<Output, RunningError> {
        Err(RunningError::new("error".to_string()))
    }
}

#[test]
fn task_failed_execute() {
    log::init_logger(LogLevel::Info, None);
    let a = generate_task!(A(1), "Compute A");
    let mut b = generate_task!(B(2), "Compute B");
    let mut c = DefaultTask::new(FailedActionC(0), "Compute C");
    let mut d = DefaultTask::new(FailedActionD(1), "Compute D");
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

    let mut job = Dag::with_tasks(vec![a, b, c, d, e, f, g]);
    job.set_env(env);
    assert!(!job.start().unwrap());
}
