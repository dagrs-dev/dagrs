//! Some tests of the dag engine.

use std::{collections::HashMap, sync::Arc};

use dagrs::{Complex, Dag, DagError, DefaultTask, EnvVar, Input, Output};
use dagrs::task::Content;

#[test]
fn yaml_task_correct_execute() {
    let mut job = Dag::with_yaml("tests/config/correct.yaml", HashMap::new()).unwrap();
    assert!(job.start().unwrap());
}

#[test]
fn yaml_task_loop_graph() {
    let res = Dag::with_yaml("tests/config/loop_error.yaml", HashMap::new())
        .unwrap()
        .start();
    assert!(matches!(res, Err(DagError::LoopGraph)))
}

#[test]
fn yaml_task_self_loop_graph() {
    let res = Dag::with_yaml("tests/config/self_loop_error.yaml", HashMap::new())
        .unwrap()
        .start();
    assert!(matches!(res, Err(DagError::LoopGraph)))
}

#[test]
fn yaml_task_failed_execute() {
    let res = Dag::with_yaml("tests/config/script_run_failed.yaml", HashMap::new())
        .unwrap()
        .start();
    assert!(!res.unwrap());
}

#[test]
fn task_loop_graph() {
    let mut a = DefaultTask::with_closure("a", |_, _| Output::empty());
    let mut b = DefaultTask::with_closure("b", |_, _| Output::empty());
    let mut c = DefaultTask::with_closure("c", |_, _| Output::empty());
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
    let tasks: Vec<DefaultTask> = Vec::new();
    let res = Dag::with_tasks(tasks).start();
    assert!(res.is_err());
}

struct FailedActionC(usize);

impl Complex for FailedActionC {
    fn run(&self, _input: Input, env: Arc<EnvVar>) -> Output {
        let base = env.get::<usize>("base").unwrap();
        Output::new(base / self.0)
    }
}

struct FailedActionD(usize);

impl Complex for FailedActionD {
    fn run(&self, _input: Input, _env: Arc<EnvVar>) -> Output {
        Output::Err(None,Some(Content::new("error".to_string())))
    }
}

macro_rules! generate_task {
    ($task:ident($val:expr),$name:literal) => {{
        pub struct $task(usize);
        impl Complex for $task {
            fn run(&self, input: Input, env: Arc<EnvVar>) -> Output {
                let base = env.get::<usize>("base").unwrap();
                let mut sum = self.0;
                input
                    .get_iter()
                    .for_each(|i| sum += i.get::<usize>().unwrap() * base);
                Output::new(sum)
            }
        }
        DefaultTask::with_action($name, $task($val))
    }};
}

#[test]
fn task_failed_execute() {
    let a = generate_task!(A(1), "Compute A");
    let mut b = generate_task!(B(2), "Compute B");
    let mut c = DefaultTask::with_action("Compute C", FailedActionC(0));
    let mut d = DefaultTask::with_action("Compute D", FailedActionD(1));
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
