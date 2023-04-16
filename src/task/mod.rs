//! Relevant definitions of tasks.
//! 
//! ## Task execution mode of the Dag engine
//! 
//! Currently, the Dag execution engine has two execution modes:
//! The first mode is to execute tasks through user-written yaml configuration file, 
//! and then hand them over to the dag engine for execution. Currently, the yaml
//! configuration file supports two types of tasks, one is to execute sh scripts,
//! and the other is to execute javascript scripts.
//! 
//! ### An example yaml configuration file
//! ```yaml
//! dagrs:
//!     a:
//!       name: "task1"
//!       after: [b]
//!       run:
//!           type: sh
//!           script: test.sh
//!     b:
//!       name: "task2"
//!       run:
//!           type: deno
//!           script: Deno.core.print("Hello!")
//! ```
//! 
//! The second mode is that the user program defines the task, which requires the
//! user to implement the [`TaskTrait`] trait of the task module and rewrite the
//! run function.
//! 
//! ### An example of a user programmatically defined task
//! 
//! ```rust
//! use dagrs::TaskTrait;
//! struct MyTask{
//!     ...
//! }
//! 
//! impl TaskTrait for MyTask{
//!     fn run(&self, input: dagrs::Inputval, env: dagrs::EnvVar) -> dagrs::Retval {
//!         let mut sum=0;
//!         for i in 0..100{
//!             sum+=i;
//!         }
//!         dagrs::Retval::new(sum)
//!     }
//! }
//! 
//! ```

pub use self::task::*;
pub use self::yaml_task::YamlTask;
pub use self::state::Retval;
pub use self::state::{Inputval, ExecState, DMap};

mod task;
mod yaml_task;
mod state;