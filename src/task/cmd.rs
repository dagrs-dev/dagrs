use crate::{Complex, EnvVar, Input, Output};
use std::process::Command;
use std::sync::Arc;

/// [`CommandAction`] is a specific implementation of [`Complex`], used to execute operating system commands.
pub struct CommandAction {
    command: String,
}

impl CommandAction {
    #[allow(unused)]
    pub fn new(cmd: &str) -> Self {
        Self {
            command: cmd.to_owned(),
        }
    }
}

impl Complex for CommandAction {
    fn run(&self, input: Input, _env: Arc<EnvVar>) -> Output {
        let mut args = Vec::new();
        let mut cmd = if cfg!(target_os = "windows") {
            args.push("-Command");
            Command::new("powershell")
        } else {
            args.push("-c");
            Command::new("sh")
        };
        args.push(&self.command);

        input.get_iter().for_each(|input| {
            if let Some(inp) = input.get::<String>() {
                args.push(inp)
            }
        });
        let out = match cmd.args(args).output() {
            Ok(o) => o,
            Err(e) => return Output::Err(e.to_string()),
        };

        if out.status.success() {
            let mut out = String::from_utf8(out.stdout).unwrap();
            if cfg!(target_os = "windows") {
                out = out.replace("\r\n", " ").replace('\n', " ");
            }
            Output::new(out)
        } else {
            Output::Err(String::from_utf8(out.stderr).unwrap())
        }
    }
}
