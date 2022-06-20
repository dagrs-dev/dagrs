extern crate anymap;
extern crate bimap;
extern crate clap;
extern crate crossbeam;
extern crate deno_core;
extern crate lazy_static;
extern crate log;
extern crate simplelog;
extern crate yaml_rust;

mod engine;
mod task;

pub use engine::{DagEngine, DagError, EnvVar, RunningError, YamlError, YamlFormatError};
pub use task::{Inputval, Retval, RunScript, RunType, TaskTrait, TaskWrapper};

use simplelog::*;
use std::{
    env,
    fs::{create_dir, File},
};

/// Init a logger.
///
/// # Example
/// ```rust
/// // Default path (HOME/.dagrs/dagrs.log)
/// init_logger(None);
/// // or
/// init_logger(Some("./dagrs.log"));
/// ```
///
/// **Note**, this function shall only be called once.
///
/// Default logger is [Simplelog](https://crates.io/crates/simplelog), you can
/// also use other log implementations. Just remember to initialize them before
/// running dagrs.
pub fn init_logger(logpath: Option<&str>) {
    let logpath = if let Some(s) = logpath {
        s.to_owned()
    } else {
        if let Ok(home) = env::var("HOME") {
            create_dir(format!("{}/.dagrs", home)).unwrap_or(());
            format!("{}/.dagrs/dagrs.log", home)
        } else {
            "./dagrs.log".to_owned()
        }
    };

    CombinedLogger::init(vec![
        TermLogger::new(
            LevelFilter::Info,
            Config::default(),
            TerminalMode::Mixed,
            ColorChoice::Auto,
        ),
        WriteLogger::new(
            LevelFilter::Info,
            Config::default(),
            File::create(logpath).unwrap(),
        ),
    ])
    .unwrap();
}

#[test]
fn test_value_pass1() {
    use crate::task::{Inputval, Retval, TaskTrait, TaskWrapper};
    struct T1 {}
    impl TaskTrait for T1 {
        fn run(&self, _input: Inputval, _env: EnvVar) -> Retval {
            println!("T1, return 1");
            Retval::new(1i32)
        }
    }

    struct T2 {}
    impl TaskTrait for T2 {
        fn run(&self, mut input: Inputval, _env: EnvVar) -> Retval {
            let val_from_t1 = input.get::<i32>(0);
            println!("T2, receive: {:?}", val_from_t1);
            Retval::empty()
        }
    }

    let t1 = TaskWrapper::new(T1 {}, "Task 1");
    let mut t2 = TaskWrapper::new(T2 {}, "Task 2");

    t2.exec_after(&[&t1]);
    t2.input_from(&[&t1]);

    let mut dag = DagEngine::new();
    dag.add_tasks(vec![t1, t2]);

    dag.run().unwrap();
}

#[test]
fn test_value_pass2() {
    use crate::task::{Inputval, Retval, TaskTrait, TaskWrapper};
    struct T1 {}
    impl TaskTrait for T1 {
        fn run(&self, _input: Inputval, mut env: EnvVar) -> Retval {
            println!("T1, return 1, set env [Hello: World]");
            env.set("Hello", "World".to_string());
            Retval::new(1i32)
        }
    }

    struct T2 {}
    impl TaskTrait for T2 {
        fn run(&self, mut input: Inputval, _env: EnvVar) -> Retval {
            let val_from_t1 = input.get::<i32>(0);
            println!("T2, receive from T1: {:?}, return '123'", val_from_t1);
            Retval::new("123".to_string())
        }
    }

    struct T3 {}
    impl TaskTrait for T3 {
        fn run(&self, mut input: Inputval, env: EnvVar) -> Retval {
            // Order of input value is the same as the order of tasks
            // passed in `input_from`.
            let val_from_t1 = input.get::<i32>(0);
            let val_from_t2 = input.get::<String>(1);
            let eval = env.get::<String>("Hello");

            println!(
                "T3, receive from T1: {:?}, T2: {:?}, env: {:?}",
                val_from_t1, val_from_t2, eval
            );

            Retval::empty()
        }
    }

    let t1 = TaskWrapper::new(T1 {}, "Task 1");
    let mut t2 = TaskWrapper::new(T2 {}, "Task 2");
    let mut t3 = TaskWrapper::new(T3 {}, "Task 3");

    t2.exec_after(&[&t1]);
    t2.input_from(&[&t1]);

    t3.exec_after(&[&t1, &t2]);
    t3.input_from(&[&t1, &t2]);

    let mut dag = DagEngine::new();
    dag.add_tasks(vec![t1, t2, t3]);

    dag.run().unwrap();
}
