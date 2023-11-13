use dagrs_core::{
    log, DefaultTask, EnvVar, LogLevel,
};
use dagrs_derive::dependencies;

macro_rules! action {
    ($action:ident($val:expr)) => {{
        use dagrs::{Action, EnvVar, Input, Output, RunningError};
        use std::sync::Arc;
        struct $action(usize);
        impl Action for $action {
            fn run(&self, input: Input, env: Arc<EnvVar>) -> Result<Output, RunningError> {
                let base = env.get::<usize>("base").unwrap();
                let mut sum = self.0;
                input
                    .get_iter()
                    .for_each(|i| sum += i.get::<usize>().unwrap() * base);
                Ok(Output::new(sum))
            }
        }
        $action($val)
    }};
}

fn main() {
    let _initialized = log::init_logger(LogLevel::Info, None);
    let mut tasks: Vec<DefaultTask> = dependencies!(
            a -> b c d,
            b -> e g,
            c -> e f,
            d -> f,
            e -> g,
            f -> g,
            g ->
    );
    let mut x = 1;
    tasks.iter_mut().for_each(|task| {
        x *= 2;
        task.set_action(action!(Compute(x)));
    });
    let mut dag = dagrs::Dag::with_tasks(tasks);
    let mut env = EnvVar::new();
    env.set("base", 2usize);
    dag.set_env(env);
    assert!(dag.start().is_ok());
}
