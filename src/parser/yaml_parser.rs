//! Default yaml configuration file parser.

use std::{collections::HashMap, fs::File, io::Read};

use yaml_rust::{Yaml, YamlLoader};

use crate::task::{JavaScript, ShScript, Task, YamlTask};

use super::{
    error::{FileContentError, ParserError, YamlTaskError},
    Parser,
};

/// An implementation of [`Parser`]. It is the default yaml configuration file parser.
pub struct YamlParser;

impl YamlParser {
    /// Given file path, and load configuration file.
    fn load_file(&self, file: &str) -> Result<String, ParserError> {
        let mut content = String::new();
        let mut yaml = File::open(file)?;
        yaml.read_to_string(&mut content).unwrap();
        Ok(content)
    }
    /// Parses an item in the configuration file into a task.
    /// An item refers to:
    ///
    /// ```yaml
    ///   name: "Task 1"
    ///    after: [b, c]
    ///    run:
    ///      type: sh
    ///      script: echo a
    /// ```
    fn parse_one(&self, id: &str, item: &Yaml) -> Result<YamlTask, YamlTaskError> {
        // Get name first
        let name = item["name"]
            .as_str()
            .ok_or(YamlTaskError::NoNameAttr(id.to_owned()))?
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
        if run.is_badvalue() {
            return Err(YamlTaskError::NoRunAttr(name));
        }
        match run["type"]
            .as_str()
            .ok_or(YamlTaskError::NoTypeAttr(name.clone()))?
        {
            "sh" => {
                let sh_script = run["script"]
                    .as_str()
                    .ok_or(YamlTaskError::NoScriptAttr(name.clone()))?;
                Ok(YamlTask::new(
                    id,
                    precursors,
                    name,
                    ShScript::new(sh_script),
                ))
            }
            "deno" => {
                let js_script = run["script"]
                    .as_str()
                    .ok_or(YamlTaskError::NoScriptAttr(name.clone()))?;
                Ok(YamlTask::new(
                    id,
                    precursors,
                    name,
                    JavaScript::new(js_script),
                ))
            }
            _ => Err(YamlTaskError::UnsupportedType(name)),
        }
    }
}

impl Parser for YamlParser {
    fn parse_tasks(&self, file: &str) -> Result<Vec<Box<dyn Task>>, ParserError> {
        let content = self.load_file(file)?;
        // Parse Yaml
        let yaml_tasks = YamlLoader::load_from_str(&content)
            .map_err(FileContentError::IllegalYamlContent)?;
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

        for task in tasks.iter_mut() {
            let mut pres = Vec::new();
            for pre in task.str_precursors() {
                if map.contains_key(&pre[..]) {
                    pres.push(map[&pre[..]]);
                } else {
                    return Err(YamlTaskError::NotFoundPrecursor(task.name()).into());
                }
            }
            task.init_precursors(pres);
        }

        Ok(tasks
            .into_iter()
            .map(|task| Box::new(task) as Box<dyn Task>)
            .collect())
    }
}
