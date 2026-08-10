use nix::{
    unistd::{execve, fork, ForkResult},
    sys::wait::waitpid,
};

use std::{
    collections::HashMap,
    ffi::{OsString, CStr, CString},
    path::PathBuf,
    os::unix::ffi::{OsStrExt},
};

mod shell_conf;
use crate::shell_core::shell_conf::ShellConfig;

#[derive(Debug)]
pub struct InputCommand {
    args: Vec<String>,
}

impl InputCommand {
    pub fn argv(&self) -> Vec<CString>{
        self.args
            .iter()
            .map(
            |arg| CString::new(
                arg.as_str()
            )).collect::<Result<_, _>>()
            .expect("Failed to convert args into C string")
    }

    pub fn get_exec(&self) -> &String {
        &self.args[0]
    }
}

pub struct ShellCore {
    command_lst: Vec<InputCommand>,
    conf: HashMap<OsString, OsString>,
    shell_config: ShellConfig,
}

impl ShellCore {
    pub fn  new() -> ShellCore {
        let config_keys = std::env::vars_os().collect();
        ShellCore{
            command_lst: Vec::new(),
            conf: config_keys,
            shell_config: ShellConfig::new(),
        }
    }

    pub fn process_input(&mut self, input: &str) -> Result<(), Box<dyn std::error::Error>> {
        let cmd_token = input.split("|");
        println!("tokenized:");
        for cmd in cmd_token {
            let args = cmd
                .split_whitespace()
                .map(String::from)
                .collect();
            self.command_lst.push(
                InputCommand {args}
            );
            println!("command_lst: {:?}", self.command_lst);
        }
        self.execute()?;
        self.command_lst.clear();
        Ok(())
    }

    fn find_executable(&self, program: &String) -> Option<PathBuf> {
        let path = self.conf.get(&OsString::from("PATH"))?;

        for directory in std::env::split_paths(path) {
            let candidate = directory.join(program);
            if candidate.is_file() {
                return Some(candidate);
            }
        }
        None
    }

    fn execute(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        for command in &self.command_lst {
           let executable = self.find_executable(
                command.get_exec()
            ).ok_or_else(|| {
                    format!("{}: Command not found", command.get_exec())
                })?;
            let exec_cstr = CString::new(
                executable
                    .as_os_str()
                    .as_bytes()
            )?;
            match unsafe {fork()?} {
                ForkResult::Parent {child} => {
                    waitpid(child, None)?;
                }
                ForkResult::Child => {
                    let env_string = self.shell_config.get_c_env()?;
                    let env: Vec<&CStr> = env_string.iter().map(
                        |var| var.as_c_str()
                    ).collect();
                    execve(&exec_cstr, &command.argv(), &env)?;
                }
            }
        }
        Ok(())
    }
}
