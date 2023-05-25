use crate::task::{JavaScriptRunnable, Action, ShScriptRunnable, Task};
use crate::YamlTask;
use std::collections::HashMap;
use std::sync::Arc;
use std::{fs::File, io::Read};
use yaml_rust::{Yaml, YamlLoader};

use super::error::{FileContentError, ParserError};
use super::{error::YamlTaskError, Parser};

pub struct YamlParser;

impl YamlParser {
    fn load_file(&self, file: &str) -> Result<String, ParserError> {
        let mut content = String::new();
        let mut yaml = File::open(file)?;
        yaml.read_to_string(&mut content).unwrap();
        Ok(content)
    }

    fn parse_one(&self, id: &str, item: &Yaml) -> Result<YamlTask, YamlTaskError> {
        // Get name first
        let name = item["name"]
            .as_str()
            .ok_or(YamlTaskError::NoName(id.to_owned()))?
            .to_owned();
        // precursors can be empty
        let mut precursors = Vec::new();
        if let Some(after_tasks) = item["after"].as_vec() {
            after_tasks
                .iter()
                .map(|task_id| precursors.push(task_id.as_str().unwrap().to_owned()))
                .count();
        }
        // Get run script
        let run = &item["run"];
        let runnable = match run["type"]
            .as_str()
            .ok_or(YamlTaskError::NoRunnable(name.clone()))?
        {
            "sh" => {
                let sh_script = run["script"]
                    .as_str()
                    .ok_or(YamlTaskError::IllegalScript(name.clone()))?;
                Action::Simple(Arc::new(ShScriptRunnable::new(sh_script)))
            }
            "deno" => {
                let js_script = run["script"]
                    .as_str()
                    .ok_or(YamlTaskError::IllegalScript(name.clone()))?;
                Action::Simple(Arc::new(JavaScriptRunnable::new(js_script)))
            }
            _ => return Err(YamlTaskError::UnsupportedType(name.clone())),
        };
        Ok(YamlTask::new(id, precursors, name, runnable))
    }
}

impl Parser for YamlParser {
    fn parse_tasks(&self, file: &str) -> Result<Vec<Box<dyn Task>>, super::error::ParserError> {
        let content = self.load_file(file)?;
        // Parse Yaml
        let yaml_tasks = YamlLoader::load_from_str(&content)
            .map_err(|err| FileContentError::IllegalYamlContent(err))?;
        // empty file error
        if yaml_tasks.is_empty() {
            return Err(FileContentError::Empty(file.to_string()).into());
        }
        let yaml_tasks = yaml_tasks[0]["dagrs"]
            .as_hash()
            .ok_or(YamlTaskError::StartWordError)?;
        let mut tasks = Vec::new();
        let mut map = HashMap::new();
        // Read tasks
        for (v, w) in yaml_tasks {
            let id = v.as_str().unwrap();
            let task = self.parse_one(id, w)?;
            map.insert(id, task.id());
            tasks.push(task);
        }

        let tasks: Vec<Box<dyn Task>> = tasks
            .into_iter()
            .map(|mut task| {
                let pres = task
                    .str_precursors()
                    .iter()
                    .map(|pre| map[&pre[..]])
                    .collect();
                task.init_precursors(pres);
                Box::new(task) as Box<dyn Task>
            })
            .collect();

        Ok(tasks)
    }
}
