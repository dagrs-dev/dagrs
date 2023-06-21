//! Use the gen_task provided by dagrs! Macros define simple tasks.
//! Execute graph:
//!    B
//!  ↗   ↘
//! A     D
//!  ↘   ↗
//!    C

extern crate dagrs;

use std::sync::Arc;

use dagrs::{
    gen_task, log, Action, Dag, DefaultTask, EnvVar, Input, LogLevel, Output, RunningError,
};

fn main() {
    log::init_logger(LogLevel::Info, None);
    let a = gen_task!("Compute A", |_input: Input, _env: Arc<EnvVar>| Ok(
        Output::new(20usize)
    ));
    let mut b = gen_task!("Compute B", |input: Input, _env: Arc<EnvVar>| {
        let mut sum = 0;
        input
            .get_iter()
            .for_each(|i| sum += i.get::<usize>().unwrap());
        Ok(Output::new(sum))
    });

    let mut c = gen_task!("Compute C", |input: Input, env: Arc<EnvVar>| {
        let mut sum = 0;
        let base = env.get::<usize>("base").unwrap();
        input
            .get_iter()
            .for_each(|i| sum += i.get::<usize>().unwrap() * base);
        Ok(Output::new(sum))
    });
    let mut d = gen_task!("Compute D", |input: Input, env: Arc<EnvVar>| {
        let mut sum = 0;
        let base = env.get::<usize>("base").unwrap();
        input
            .get_iter()
            .for_each(|i| sum += i.get::<usize>().unwrap() - base);
        Ok(Output::new(sum))
    });

    b.set_predecessors(&[&a]);
    c.set_predecessors(&[&a]);
    d.set_predecessors(&[&b, &c]);
    let mut job = Dag::with_tasks(vec![a, b, c, d]);
    let mut env = EnvVar::new();
    env.set("base", 2usize);
    job.set_env(env);
    assert!(job.start().unwrap());
    assert_eq!(job.get_result::<usize>().unwrap(), 56);
}
