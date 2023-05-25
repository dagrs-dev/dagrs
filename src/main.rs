// use dagrs::{ComplexAction, DefaultTask, Output};
// use std::{fs::File, io::Write};
// struct Action {
//     content: String,
// }

fn main(){
    
}

// impl ComplexRunnable for Action {
//     fn before_run(&mut self) {
//         // Suppose you open the file and read the content into `content`.
//         self.content = "hello world".to_owned()
//     }

//     fn run(&self, input: dagrs::Input, env: dagrs::EnvVar) -> dagrs::Output {
//         Output::new(self.content.split(" "))
//     }
    
//     fn after_run(&mut self) {
//         // Suppose you delete a temporary file generated when a task runs.
//         self.content = "".to_owned();
//     }
// }
// fn main() {
//     let mut runnable = Action {
//         content: "".to_owned(),
//     };
//     let task = DefaultTask::complex_task(runnable, "Increment action");
// }
