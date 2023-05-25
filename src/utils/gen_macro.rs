
// /// Macros for generating simple tasks.
// ///
// /// # Example
// ///
// /// ```rust
// /// # use dagrs::{generate_task,DefaultTask,Input,Output,EnvVar,TaskTrait};
// /// # let task = generate_task!("task A", |input, env| {
// /// #     Output::empty()
// /// # });
// /// # println!("{},{}", task.get_id(), task.get_name());
// /// ```
// #[macro_export]
// macro_rules! generate_task {
//     ($task_name:expr,$action:expr) => {{
//         pub struct Task {}
//         impl TaskTrait for Task {
//             fn run(&self, input: Input, env: EnvVar) -> Output {
//                 $action(input, env)
//             }
//         }
//         DefaultTask::new(Task {}, $task_name)
//     }};
// }