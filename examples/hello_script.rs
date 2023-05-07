extern crate dagrs;

use dagrs::{DagEngine, EnvVar, Input, Output, TaskTrait, TaskWrapper, init_logger, RunScript, RunType};

struct T {}

impl TaskTrait for T {
    fn run(&self, _input: Input, _env: EnvVar) -> Output {
        let script = RunScript::new("echo 'Hello Dagrs!'", RunType::SH);

        let res = script.exec(None);
        println!("{:?}", res);
        Output::empty()
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
