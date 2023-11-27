//! Read the task information configured in the yaml file.

extern crate dagrs;

use dagrs::Dag;
use std::collections::HashMap;

fn main() {
    env_logger::init();
    let mut job = Dag::with_yaml("tests/config/correct.yaml", HashMap::new()).unwrap();
    assert!(job.start().unwrap());
}
