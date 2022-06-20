use super::{Inputval, Retval, RunScript, RunType, TaskTrait, TaskWrapper};
use crate::engine::{DagError, YamlFormatError, EnvVar};
use std::{cell::Cell, collections::HashMap, fs::File, io::Read};
use yaml_rust::{Yaml, YamlLoader};

#[derive(Debug)]
/// Task Struct for YAML file.
struct YamlTaskInner {
    /// Running Script
    run: RunScript,
}

/// Task struct for YAML file.
pub struct YamlTask {
    /// Task's id in yaml file.
    ///
    /// Be careful that `yaml_id` is different from [`TaskWrapper`]'s id.
    yaml_id: String,
    /// Task's name.
    name: String,
    /// Record tasks' `yaml_id` that shall be executed before this task.
    afters: Vec<String>,
    /// Record tasks' `yaml_id` that shall give their execution results to this task.
    froms: Vec<String>,
    /// A field shall be wrapper into [`TaskWrapper`] later.
    ///
    /// Why [`Cell`] and [`Option`]? Useful in funtion `from_yaml`.
    inner: Cell<Option<YamlTaskInner>>,
}

impl TaskTrait for YamlTaskInner {
    fn run(&self, input: Inputval, _env: EnvVar) -> Retval {
        if let Ok(res) = self.run.exec(Some(input)) {
            Retval::new(res)
        } else {
            Retval::empty()
        }
    }
}

impl YamlTask {
    /// Parse a task from yaml.
    ///
    /// # Example
    /// ```
    /// let task = Task::parse_one(id, yaml);
    /// ```
    /// Here `id` and `yaml` comes from:
    /// ```
    /// let yaml_tasks = YamlLoader::load_from_str(&yaml_cont)?;
    /// let yaml_tasks = yaml_tasks[0]["dagrs"]
    /// .as_hash()
    /// .ok_or(DagError::format_error("", FormatErrorMark::StartWordError))?;
    ///
    /// for(id, yaml) in yaml_tasks{
    ///     ...
    /// }
    /// ```
    fn parse_one(id: &str, info: &Yaml) -> Result<YamlTask, DagError> {
        // Get name first
        let name = info["name"]
            .as_str()
            .ok_or(DagError::format_error(YamlFormatError::NoName(
                id.to_owned(),
            )))?
            .to_owned();

        // Get run script
        let run = &info["run"];

        let executor = match run["type"].as_str().ok_or(DagError::format_error(
            YamlFormatError::RunScriptError(id.into()),
        ))? {
            "sh" => RunType::SH,
            "deno" => RunType::DENO,
            _ => {
                return Err(DagError::format_error(YamlFormatError::RunScriptError(
                    id.into(),
                )))
            }
        };

        let run_script = run["script"].as_str().ok_or(DagError::format_error(
            YamlFormatError::RunScriptError(id.into()),
        ))?;

        // afters can be empty
        let mut afters = Vec::new();
        if let Some(after_tasks) = info["after"].as_vec() {
            after_tasks
                .iter()
                .map(|task_id| afters.push(task_id.as_str().unwrap().to_owned()))
                .count();
        }

        // froms can be empty, too
        let mut froms = Vec::new();
        if let Some(from_tasks) = info["from"].as_vec() {
            from_tasks
                .iter()
                .map(|task_id| froms.push(task_id.as_str().unwrap().to_owned()))
                .count();
        }

        let inner = Cell::new(Some(YamlTaskInner {
            run: RunScript::new(run_script, executor),
        }));

        Ok(YamlTask {
            yaml_id: id.to_string(),
            name,
            afters,
            froms,
            inner,
        })
    }

    /// Parse all tasks from yaml file.
    ///
    /// # Example
    /// ```
    /// let tasks = YamlTask::parse_tasks("test/test_dag.yaml")?;
    /// ```
    fn parse_tasks(filename: &str) -> Result<Vec<Self>, DagError> {
        let mut yaml_cont = String::new();

        let mut yaml_file = File::open(filename)?;
        yaml_file.read_to_string(&mut yaml_cont)?;

        // Parse Yaml
        let yaml_tasks = YamlLoader::load_from_str(&yaml_cont)?;
        let yaml_tasks = yaml_tasks[0]["dagrs"]
            .as_hash()
            .ok_or(DagError::format_error(YamlFormatError::StartWordError))?;

        let mut tasks = Vec::new();
        // Read tasks
        for (v, w) in yaml_tasks {
            let id = v.as_str().unwrap();
            let task = YamlTask::parse_one(id, w)?;

            tasks.push(task);
        }

        Ok(tasks)
    }

    /// Parse all tasks from yaml file into format recognized by dagrs.
    ///
    /// # Example
    /// ```
    /// let tasks = YamlTask::from_yaml(filename)?;
    /// ```
    ///
    /// Used in [`crate::DagEngine`].
    pub fn from_yaml(filename: &str) -> Result<Vec<TaskWrapper>, DagError> {
        let yaml_tasks = YamlTask::parse_tasks(filename)?;
        let mut tasks = Vec::new();
        let mut yid2id = HashMap::new();

        // Form tasks
        for ytask in &yaml_tasks {
            let task = TaskWrapper::new(
                ytask
                    .inner
                    .replace(None)
                    .expect("[Fatal] Abnormal error occurs."),
                &ytask.name,
            );
            yid2id.insert(ytask.yaml_id.clone(), task.get_id());
            tasks.push(task);
        }

        for (index, ytask) in yaml_tasks.iter().enumerate() {
            let afters: Vec<usize> = ytask
                .afters
                .iter()
                .map(|after| yid2id.get(after).unwrap_or(&0).to_owned())
                .collect();
            // Task 0 won't exist in normal state, thus this will trigger an RelyTaskIllegal Error later.

            let froms: Vec<usize> = ytask
                .froms
                .iter()
                .map(|from| yid2id.get(from).unwrap_or(&0).to_owned())
                .collect();

            tasks[index].exec_after_id(&afters);
            tasks[index].input_from_id(&froms);
        }

        Ok(tasks)
    }
}
