//! OS command `Action`.
//!
//! # `cmd` attribute command wrapper.
//!
//! Specify the command to be executed in the `cmd` attribute of the `yaml` configuration
//! file, and the `Yaml` parser will package the command as a `CommandAction`, which implements
//! the `Action` trait and defines the specific logic of executing the command.

use std::{process::Command, sync::Arc};

use crate::{log, utils::EnvVar};

use super::{Action, CmdExecuteError, Input, Output, RunningError};

/// Can be used to run a command.
pub struct CommandAction {
    command: String,
}

impl CommandAction {
    pub fn new(cmd: &str) -> Self {
        Self {
            command: cmd.to_owned(),
        }
    }
}

impl Action for CommandAction {
    fn run(&self, input: Input, _env: Arc<EnvVar>) -> Result<Output, RunningError> {
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
            Err(e) => {
                return Err(CmdExecuteError::new(e.to_string()).into())
            }
        };
        let code = out.status.code().unwrap_or(-1);
        if code == 0 {
            Ok(Output::new(String::from_utf8(out.stdout).unwrap()))
        } else {
            let err_msg = String::from_utf8(out.stderr).unwrap();
            log::error(err_msg.clone());
            Err(CmdExecuteError::new(err_msg).into())
        }
    }
}
