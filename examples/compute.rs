//! Execute graph
//!    ↱----------↴
//!    B -→ E --→ G
//!  ↗    ↗     ↗ 
//! A --→ C    / 
//!  ↘    ↘  /
//!   D -→ F

extern crate dagrs;

use dagrs::{init_logger, DagEngine, EnvVar, Input, Output, TaskTrait, TaskWrapper};

macro_rules! generate_task {
    ($task:ident($val:expr),$name:expr) => {{
        pub struct $task(usize);
        impl TaskTrait for $task {
            fn run(&self, input: Input, env: EnvVar) -> Output {
                let base = env.get::<usize>("base").unwrap();
                let mut sum=0;
                input.get_iter().for_each(|i|{
                    match i {
                        Some(val) => sum+=(val.get::<usize>().unwrap()*base),
                        None => {},
                    }
                });
                Output::new(sum)
            }
        }
        TaskWrapper::new($task($val), $name)
    }};
}

fn main() {
    init_logger(None);
    let mut dagrs = DagEngine::new();
    dagrs.set_env("base", 2usize);
    let a = generate_task!(A(1), "Compute A");
    let mut b = generate_task!(B(2), "Compute B");
    let mut c = generate_task!(C(4), "Compute C");
    let mut d = generate_task!(D(8), "Compute D");
    let mut e = generate_task!(E(16), "Compute E");
    let mut f = generate_task!(F(32), "Compute F");
    let mut g = generate_task!(G(64), "Compute G");
    
    b.exec_after(&[&a]);
    c.exec_after(&[&a]);
    d.exec_after(&[&a]);
    e.exec_after(&[&b,&c]);
    f.exec_after(&[&c,&d]);
    g.exec_after(&[&b,&e,&f]);
    dagrs.add_tasks(vec![a,b,c,d,e,f,g]);
    assert!(dagrs.run().unwrap());
}