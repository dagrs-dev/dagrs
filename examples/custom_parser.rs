//! Implement the Parser interface to customize the task configuration file parser.
//! The content of the configuration file is as follows:
//! 
//! ```
//! a,Task a,b c,sh,echo a
//! b,Task b,c f g,sh,echo b
//! c,Task c,e g,sh,echo c
//! d,Task d,c e,sh,echo d
//! e,Task e,h,sh,echo e
//! f,Task f,g,deno,Deno.core.print("f\n")
//! g,Task g,h,deno,Deno.core.print("g\n")
//! h,Task h,,sh,echo h
//! ```

extern crate dagrs;

use std::{fs, sync::Arc};
use std::collections::HashMap;
use std::fmt::{Display, Formatter};

use dagrs::{Action, Dag, log,LogLevel, JavaScript, Parser, ParserError, ShScript, Task};

struct MyTask {
    tid: (String, usize),
    name: String,
    precursors: Vec<String>,
    precursors_id: Vec<usize>,
    action: Arc<dyn Action + Sync + Send>,
}

impl MyTask {
    pub fn new(
        txt_id: &str,
        precursors: Vec<String>,
        name: String,
        action: impl Action + Send + Sync + 'static,
    ) -> Self {
        Self {
            tid: (txt_id.to_owned(), dagrs::alloc_id()),
            name,
            precursors,
            precursors_id: Vec::new(),
            action: Arc::new(action),
        }
    }

    pub fn init_precursors(&mut self, pres_id: Vec<usize>) {
        self.precursors_id = pres_id;
    }

    pub fn str_precursors(&self) -> Vec<String> {
        self.precursors.clone()
    }

    pub fn str_id(&self) -> String {
        self.tid.0.clone()
    }
}

impl Task for MyTask {
    fn action(&self) -> Arc<dyn Action + Sync + Send> {
        self.action.clone()
    }
    fn predecessors(&self) -> &[usize] {
        &self.precursors_id
    }
    fn id(&self) -> usize {
        self.tid.1
    }
    fn name(&self) -> String {
        self.name.clone()
    }
}

impl Display for MyTask {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{},{},{},{:?}", self.name, self.tid.0, self.tid.1, self.precursors)
    }
}

struct ConfigParser;

impl ConfigParser {
    fn load_file(&self, file: &str) -> Result<Vec<String>, ParserError> {
        let contents = fs::read_to_string(file)?;
        let lines: Vec<String> = contents.lines().map(|line| line.to_string()).collect();
        Ok(lines)
    }

    fn parse_one(&self, item: String) -> MyTask {
        let attr: Vec<&str> = item.split(",").collect();

        let pres_item = attr.get(2).unwrap().clone();
        let pres = if pres_item.eq("") {
            Vec::new()
        } else {
            pres_item.split(" ").map(|pre| pre.to_string()).collect()
        };

        let id = attr.get(0).unwrap().clone();
        let name = attr.get(1).unwrap().to_string();
        let script = attr.get(4).unwrap().clone();
        let t_type = attr.get(3).unwrap().clone();
        if t_type.eq("sh") {
            MyTask::new(
                id,
                pres,
                name,
                ShScript::new(script),
            )
        } else {
            MyTask::new(
                id,
                pres,
                name,
                JavaScript::new(script),
            )
        }
    }
}

impl Parser for ConfigParser {
    fn parse_tasks(&self, file: &str) -> Result<Vec<Box<dyn Task>>, ParserError> {
        let content = self.load_file(file)?;
        let mut map = HashMap::new();
        let mut tasks = Vec::new();
        content.into_iter().for_each(|line| {
            let task = self.parse_one(line);
            map.insert(task.str_id(), task.id());
            tasks.push(task);
        });

        for task in tasks.iter_mut() {
            let mut pres = Vec::new();
            let str_pre = task.str_precursors();
            if !str_pre.is_empty() {
                for pre in str_pre {
                    pres.push(map[&pre[..]]);
                }
                task.init_precursors(pres);
            }
        }
        Ok(tasks
            .into_iter()
            .map(|task| Box::new(task) as Box<dyn Task>)
            .collect())
    }
}

fn main() {
    log::init_logger(LogLevel::Info, None);
    let file = "tests/config/custom_file_task.txt";
    let mut dag = Dag::with_config_file_and_parser(file, Box::new(ConfigParser)).unwrap();
    assert!(dag.start().unwrap());
}