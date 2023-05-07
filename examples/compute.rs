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
    let a = generate_task!(A(2), "Task A");
    let mut b = generate_task!(A(2), "Task B");
    let mut c = generate_task!(A(2), "Task C");
    let mut d = generate_task!(A(2), "Task D");
    let mut e = generate_task!(A(2), "Task E");
    let mut f = generate_task!(A(2), "Task F");
    let mut g = generate_task!(A(2), "Task G");
    
    b.exec_after(&[&a]);
    c.exec_after(&[&a]);
    d.exec_after(&[&a]);
    e.exec_after(&[&b,&c]);
    f.exec_after(&[&c,&d]);
    g.exec_after(&[&b,&e,&f]);
    dagrs.add_tasks(vec![a,b,c,d,e,f,g]);
    assert!(dagrs.run().unwrap());
}