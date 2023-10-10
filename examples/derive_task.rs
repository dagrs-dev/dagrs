use std::sync::Arc;

use dagrs::{Action, Task};
use dagrs_core::Output;

#[derive(Task)]
struct MyTask {
    #[attr = "id"]
    id: usize,
    #[attr = "name"]
    name: String,
    #[attr = "precursors"]
    pre: Vec<usize>,
    #[attr = "action"]
    action: Arc<dyn Action + Send + Sync>,
}

struct SimpleAction(i32);
impl Action for SimpleAction {
    fn run(
        &self,
        _input: dagrs_core::Input,
        _env: Arc<dagrs_core::EnvVar>,
    ) -> Result<dagrs_core::Output, dagrs_core::RunningError> {
        Ok(Output::empty())
    }
}

fn main() {
    let action = Arc::new(SimpleAction(10));
    let task = MyTask {
        id: 10,
        name: "mytask".to_owned(),
        pre: vec![1, 2],
        action,
    };
    println!("{}\t{}\t{:?}", task.id(), task.name(), task.precursors());
}
