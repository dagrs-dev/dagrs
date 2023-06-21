//! Read the task information configured in the yaml file.

extern crate dagrs;

use dagrs::{log, Dag, LogLevel};

fn main() {
    log::init_logger(LogLevel::Info, None);
    let mut job = Dag::with_yaml("tests/config/correct.yaml").unwrap();
    assert!(job.start().unwrap());
}
