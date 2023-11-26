use criterion::{criterion_group, criterion_main, Criterion};

use dagrs::{log, Dag, DefaultTask, EnvVar, Input, LogLevel, Output, Task};
use rand::distributions::{Distribution, Uniform};
use std::{collections::HashSet, sync::Arc};

fn calc(input: Input, env: Arc<EnvVar>) -> Output {
    let base = env.get::<usize>("base").unwrap();
    let mut sum = 2;
    input
        .get_iter()
        .for_each(|i| sum += i.get::<usize>().unwrap() * base);
    Output::new(sum)
}

fn compute_dag(tasks: Vec<DefaultTask>) {
    let mut dag = Dag::with_tasks(tasks);
    let mut env = EnvVar::new();
    env.set("base", 2usize);
    dag.set_env(env);

    assert!(dag.start().unwrap());

    // Get execution result.
    let _res = dag.get_result::<usize>().unwrap();
}

fn compute_dag_bench(bencher: &mut Criterion) {
    let _initialized = log::init_logger(LogLevel::Off, None);

    let mut tasks = (0..200usize)
        .into_iter()
        .map(|i_task| DefaultTask::with_closure(&i_task.to_string(), calc))
        .collect::<Vec<_>>();

    let mut rng = rand::thread_rng();

    // consider up to 16 random predecessors from the previous 20 tasks.
    for i_task in 20..tasks.len() {
        let predecessors_id = (0..16usize)
            .into_iter()
            .filter_map(|_| {
                let between = Uniform::from(i_task - 20..i_task);
                let i_random = between.sample(&mut rng);

                if i_random != i_task {
                    Some(tasks[i_random].id())
                } else {
                    None
                }
            })
            .collect::<HashSet<_>>();

        tasks[i_task].set_predecessors_by_id(predecessors_id);
    }

    bencher.bench_function("compute dag", |b| b.iter(|| compute_dag(tasks.clone())));
}

criterion_group!(
  name = benches;
  config = {
    let criterion = Criterion::default().sample_size(500);
    criterion
  };
  targets = compute_dag_bench
);

criterion_main!(benches);
