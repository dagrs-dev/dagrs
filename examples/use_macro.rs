//! Execute graph
//!    B
//!  ↗   ↘
//! A     D 
//!  ↘   ↗
//!    C 

extern crate dagrs;

use dagrs::{generate_task, init_logger, DagEngine, EnvVar, Input, Output, SimpleAction, DefaultTask};

fn main() {
    init_logger(None);
    let mut dagrs = DagEngine::new();
    dagrs.set_env("base", 2usize);
    let a = generate_task!("Compute A", |_input: Input, _env: EnvVar| {
        Output::new(20usize)
    });
    let mut b = generate_task!("Compute B", |input: Input, _env: EnvVar| {
        let mut sum = 0;
        input.get_iter().for_each(|i| {
            sum += i.get::<usize>().unwrap()
        });
        Output::new(sum)
    });

    let mut c = generate_task!("Compute C", |input: Input, env: EnvVar| {
        let mut sum = 0;
        let base = env.get::<usize>("base").unwrap();
        input.get_iter().for_each(|i| {
            sum += i.get::<usize>().unwrap() * base
        });
        Output::new(sum)
    });
    let mut d = generate_task!("Compute D", |input: Input, env: EnvVar| {
        let mut sum = 0;
        let base = env.get::<usize>("base").unwrap();
        input.get_iter().for_each(|i| {
            sum += i.get::<usize>().unwrap() - base
        });
        Output::new(sum)
    });

    b.set_predecessors(&[&a]);
    c.set_predecessors(&[&a]);
    d.set_predecessors(&[&b, &c]);
    dagrs.add_tasks(vec![a, b, c, d]);
    assert!(dagrs.run().unwrap());
}
