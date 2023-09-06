//! Read the task information configured in the yaml file.

extern crate dagrs;

use std::collections::HashMap;

use dagrs::{log, Dag, LogLevel};

fn main() {
    let _initialized = log::init_logger(LogLevel::Info, None);
    let mut job = Dag::with_yaml("tests/config/correct.yaml",HashMap::new()).unwrap();
    assert!(job.start().unwrap());
}
