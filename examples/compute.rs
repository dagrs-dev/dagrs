//! Execute graph
//!    ↱----------↴
//!    B -→ E --→ G
//!  ↗    ↗     ↗ 
//! A --→ C    / 
//!  ↘    ↘  /
//!   D -→ F
//! 
//! The final execution result is 272.

extern crate dagrs;

use dagrs::{init_logger, DagEngine, EnvVar, Input, Output, SimpleAction, DefaultTask};

macro_rules! generate_task {
    ($task:ident($val:expr),$name:expr) => {{
        pub struct $task(usize);
        impl TaskTrait for $task {
            fn run(&self, input: Input, env: EnvVar) -> Output {
                let base = env.get::<usize>("base").unwrap();
                let mut sum=self.0;
                input.get_iter().for_each(|i|{
                    sum+=i.get::<usize>().unwrap()*base
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
    
    b.set_predecessors(&[&a]);
    c.set_predecessors(&[&a]);
    d.set_predecessors(&[&a]);
    e.set_predecessors(&[&b,&c]);
    f.set_predecessors(&[&c,&d]);
    g.set_predecessors(&[&b,&e,&f]);
    dagrs.add_tasks(vec![a,b,c,d,e,f,g]);
    assert!(dagrs.run().unwrap());
    let res = dagrs.get_result::<usize>().unwrap();
    println!("The result is {}.",res);
    
}