use nix::{
    unistd::{execve, fork, ForkResult},
    sys::wait::waitpid,
};

use std::{
    ffi::{CStr, CString},
    path::PathBuf,
    os::unix::ffi::{OsStrExt},
};

mod shell_conf;
mod parser;
use parser::{input_commands::InputCommand};
use crate::shell_core::shell_conf::ShellConfig;

pub struct ShellCore {
    command_lst: Vec<InputCommand>,
    shell_config: ShellConfig,
}


//  fn is_executable(file: &Path) -> bool {
//      let metadata = match metadata(file) {
//          Ok(metadata) => metadata,
//          Err(_) => return false,
//      };
//      metadata.is_file() && metadata.permissions().mode() & 0o111 != 0
//  }
impl ShellCore {
    pub fn  new() -> ShellCore {
        ShellCore{
            command_lst: Vec::new(),
            shell_config: ShellConfig::new(),
        }
    }

//  pub fn process_input(&mut self, input: &str) -> Result<(), Box<dyn std::error::Error>> {
//      let cmd_token = input.split("|");
//      println!("tokenized:");
//      for cmd in cmd_token {
//          let args = cmd
//              .split_whitespace()
//              .map(String::from)
//              .collect();
//          self.command_lst.push(
//              InputCommand {args}
//          );
//          println!("command_lst: {:?}", self.command_lst);
//      }
//      self.execute()?;
//      self.command_lst.clear();
//      Ok(())
//  }

    fn find_executable(&self, program: &String) -> Option<PathBuf> {
        let path = self.shell_config.return_exec_path()?;

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
