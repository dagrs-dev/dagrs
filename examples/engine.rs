//! Use Engine to manage multiple Dag jobs.

extern crate dagrs;

use std::sync::Arc;

use dagrs::{
    gen_task, log, Action, Dag, DefaultTask, Engine, EnvVar, Input, LogLevel, Output, RunningError,
};
fn main() {
    // initialization log.
    log::init_logger(LogLevel::Error, None);
    // Create an Engine.
    let mut engine = Engine::default();

    // Create some task for dag1.
    let t1_a = gen_task!("Compute A1", |_input: Input, _env: Arc<EnvVar>| Ok(
        Output::new(20usize)
    ));
    let mut t1_b = gen_task!("Compute B1", |input: Input, _env: Arc<EnvVar>| {
        let mut sum = 10;
        input.get_iter().for_each(|input| {
            sum += input.get::<usize>().unwrap();
        });
        Ok(Output::new(sum))
    });
    let mut t1_c = gen_task!("Compute C1", |input: Input, _env: Arc<EnvVar>| {
        let mut sum = 20;
        input.get_iter().for_each(|input| {
            sum += input.get::<usize>().unwrap();
        });
        Ok(Output::new(sum))
    });

    let mut t1_d = gen_task!("Compute D1", |input: Input, _env: Arc<EnvVar>| {
        let mut sum = 30;
        input.get_iter().for_each(|input| {
            sum += input.get::<usize>().unwrap();
        });
        Ok(Output::new(sum))
    });
    t1_b.set_predecessors(&[&t1_a]);
    t1_c.set_predecessors(&[&t1_a]);
    t1_d.set_predecessors(&[&t1_b, &t1_c]);
    let dag1 = Dag::with_tasks(vec![t1_a, t1_b, t1_c, t1_d]);
    // Add dag1 to engine.
    engine.append_dag("graph1", dag1);

    // Create some task for dag2.
    let t2_a = gen_task!("Compute A2", |_input: Input, _env: Arc<EnvVar>| Ok(
        Output::new(2usize)
    ));
    let mut t2_b = gen_task!("Compute B2", |input: Input, _env: Arc<EnvVar>| {
        let mut sum=4;
        input.get_iter().for_each(|input|{
            sum *= input.get::<usize>().unwrap();
        });
        Ok(Output::new(sum))
    });
    let mut t2_c = gen_task!("Compute C2", |input: Input, _env: Arc<EnvVar>| {
        let mut sum=8;
        input.get_iter().for_each(|input|{
            sum *= input.get::<usize>().unwrap();
        });
        Ok(Output::new(sum))
    });
    let mut t2_d = gen_task!("Compute D2", |input: Input, _env: Arc<EnvVar>| {
        let mut sum=16;
        input.get_iter().for_each(|input|{
            sum *= input.get::<usize>().unwrap();
        });
        Ok(Output::new(sum))
    });
    t2_b.set_predecessors(&[&t2_a]);
    t2_c.set_predecessors(&[&t2_b]);
    t2_d.set_predecessors(&[&t2_c]);
    let dag2 = Dag::with_tasks(vec![t2_a, t2_b, t2_c, t2_d]);
    // Add dag2 to engine.
    engine.append_dag("graph2", dag2);
    // Read tasks from configuration files and resolve to dag3.
    let dag3 = Dag::with_yaml("tests/config/correct.yaml").unwrap();
    // Add dag3 to engine.
    engine.append_dag("graph3", dag3);
    // Execute dag in order, the order should be dag1, dag2, dag3.
    assert_eq!(engine.run_sequential(),vec![true,true,true]);
    // Get the execution results of dag1 and dag2.
    assert_eq!(engine.get_dag_result::<usize>("graph1").unwrap(),100);
    assert_eq!(engine.get_dag_result::<usize>("graph2").unwrap(),1024);
}
