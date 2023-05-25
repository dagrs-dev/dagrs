///! Some tests of the dag engine.
extern crate dagrs;

use dagrs::DagEngine;


#[test]
fn empty_file_test(){
    let complete = DagEngine::new()
        .run_from_yaml("tests/custom_task/empty_file.yaml")
        .unwrap();
    assert_eq!(complete,true)
}

#[test]
fn only_one_task_test() {
    let complete = DagEngine::new()
        .run_from_yaml("tests/custom_task/only_one_task.yaml")
        .unwrap();
    assert_eq!(complete, true)
}

#[test]
fn multi_tasks_test() {
    let res = DagEngine::new()
        .run_from_yaml("tests/custom_task/multi_tasks.yaml")
        .unwrap();
    assert_eq!(res, true)
}

// #[test]
// fn simple_graph_loop_test() {
//     let res = DagEngine::new()
//         .run_from_yaml("tests/custom_task/test_loop1.yaml")
//         .unwrap();
//     assert_eq!(res, false)
// }

// #[test]
// fn test_complex_loop() {
//     let res = DagEngine::new()
//         .run_from_yaml("tests/custom_task/test_loop2.yaml")
//         .unwrap();
//     assert_eq!(res, false)
// }

// #[test]
// fn test_format_error1() {
//     use dagrs::{DagError, YamlError, YamlFormatError};
//     let res = DagEngine::new().run_from_yaml("tests/custom_task/test_error1.yaml");

//     assert!(matches!(
//         res,
//         Err(DagError::YamlError(YamlError::YamlFormatError(
//             YamlFormatError::NoName(_)
//         )))
//     ));
// }

// #[test]
// fn test_format_error2() {
//     use dagrs::{DagError, YamlError, YamlFormatError};
//     let res = DagEngine::new().run_from_yaml("tests/custom_task/test_error2.yaml");

//     assert!(matches!(
//         res,
//         Err(DagError::YamlError(YamlError::YamlFormatError(
//             YamlFormatError::StartWordError
//         )))
//     ));
// }

// #[test]
// fn test_rely_error() {
//     use dagrs::{DagError, RunningError};
//     let res = DagEngine::new().run_from_yaml("tests/custom_task/test_error3.yaml");

//     assert!(matches!(
//         res,
//         Err(DagError::RunningError(RunningError::RelyTaskIllegal(_)))
//     ));
// }

// #[test]
// fn test_no_runscript() {
//     use dagrs::{DagError, YamlError, YamlFormatError};
//     let res = DagEngine::new().run_from_yaml("tests/custom_task/test_error4.yaml");

//     assert!(matches!(
//         res,
//         Err(DagError::YamlError(YamlError::YamlFormatError(
//             YamlFormatError::RunScriptError(_)
//         )))
//     ));
// }



// #[test]
// fn test_value_pass1() {
//     use crate::task::{DefaultTask, Input, Output, SimpleRunnable};
//     struct T1 {}
//     impl SimpleRunnable for T1 {
//         fn run(&self, _input: Input, _env: EnvVar) -> Output {
//             println!("T1, return 1");
//             Output::new(1i32)
//         }
//     }

//     struct T2 {}
//     impl SimpleRunnable for T2 {
//         fn run(&self, mut input: Input, _env: EnvVar) -> Output {
//             let val_from_t1 = input.get::<i32>(0);
//             println!("T2, receive: {:?}", val_from_t1);
//             Output::empty()
//         }
//     }

//     let t1 = DefaultTask::new(T1 {}, "Task 1");
//     let mut t2 = DefaultTask::new(T2 {}, "Task 2");

//     t2.set_predecessors(&[&t1]);

//     let mut dag = DagEngine::new();
//     dag.add_tasks(vec![t1, t2]);

//     dag.run().unwrap();
// }

// #[test]
// fn test_value_pass2() {
//     use crate::task::{DefaultTask, Input, Output, SimpleRunnable};
//     struct T1 {}
//     impl SimpleRunnable for T1 {
//         fn run(&self, _input: Input) -> Output {
//             println!("T1, return 1, set env [Hello: World]");
//             env.set("Hello", "World".to_string());
//             Output::new(1i32)
//         }
//     }

//     struct T2 {}
//     impl SimpleRunnable for T2 {
//         fn run(&self, mut input: Input) -> Output {
//             let val_from_t1 = input.get::<i32>(0);
//             println!("T2, receive from T1: {:?}, return '123'", val_from_t1);
//             Output::new("123".to_string())
//         }
//     }

//     struct T3 {}
//     impl SimpleRunnable for T3 {
//         fn run(&self, mut input: Input) -> Output {
//             // Order of input value is the same as the order of tasks
//             // passed in `input_from`.
//             let val_from_t1 = input.get::<i32>(0);
//             let val_from_t2 = input.get::<String>(1);
//             let eval = env.get::<String>("Hello");

//             println!(
//                 "T3, receive from T1: {:?}, T2: {:?}, env: {:?}",
//                 val_from_t1, val_from_t2, eval
//             );

//             Output::empty()
//         }
//     }

//     let t1 = DefaultTask::new(T1 {}, "Task 1");
//     let mut t2 = DefaultTask::new(T2 {}, "Task 2");
//     let mut t3 = DefaultTask::new(T3 {}, "Task 3");

//     t2.set_predecessors(&[&t1]);
//     t3.set_predecessors(&[&t1, &t2]);

//     let mut dag = DagEngine::new();
//     dag.add_tasks(vec![t1, t2, t3]);

//     dag.run().unwrap();
// }
