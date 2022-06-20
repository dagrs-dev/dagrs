extern crate dagrs;

use dagrs::{DagEngine, EnvVar, Inputval, Retval, TaskTrait, TaskWrapper, init_logger, RunScript, RunType};

struct T {}

impl TaskTrait for T {
    fn run(&self, _input: Inputval, _env: EnvVar) -> Retval {
        let script = RunScript::new("echo 'Hello Dagrs!'", RunType::SH);

        let res = script.exec(None);
        println!("{:?}", res);
        Retval::empty()
    }
}

fn main() {
    // Use dagrs provided logger
    init_logger(None);

    let t = TaskWrapper::new(T{}, "Task");
    let mut dagrs = DagEngine::new();

    dagrs.add_tasks(vec![t]);
    assert!(dagrs.run().unwrap())
}
