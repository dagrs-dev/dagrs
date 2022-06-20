extern crate dagrs;

use dagrs::{DagEngine, EnvVar, Inputval, Retval, TaskTrait, TaskWrapper, init_logger};

struct T1 {}

impl TaskTrait for T1 {
    fn run(&self, _input: Inputval, _env: EnvVar) -> Retval {
        let hello_dagrs = String::from("Hello Dagrs!");
        Retval::new(hello_dagrs)
    }
}

struct T2 {}

impl TaskTrait for T2 {
    fn run(&self, mut input: Inputval, _env: EnvVar) -> Retval {
        let val = input.get::<String>(0).unwrap();
        println!("{}", val);
        Retval::empty()
    }
}

fn main() {
    // Use dagrs provided logger
    init_logger(None);

    let t1 = TaskWrapper::new(T1{}, "Task 1");
    let mut t2 = TaskWrapper::new(T2{}, "Task 2");
    let mut dagrs = DagEngine::new();

    // Set up dependencies
    t2.exec_after(&[&t1]);
    t2.input_from(&[&t1]);

    dagrs.add_tasks(vec![t1, t2]);
    assert!(dagrs.run().unwrap())
}
