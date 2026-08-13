use crate::{
    parser::commands::{
    InputCommand,
    CommandOperator,
    CommandGroup,
    CommandType,
    },
    shell_config::shell_conf::{ConfigError, ShellConfig},
    executor::streams::{create_streams, CommandPipe, setup_stdin, setup_stdout},
};

use std::{
    error::Error, ffi::{CStr, CString, OsStr}, fmt, fs::metadata, os::unix::{
        ffi::OsStrExt,
        fs::PermissionsExt,
    }, path::{Path, PathBuf}
};

use nix::{
    errno::Errno, sys::wait::waitpid, unistd::{execve, fork, ForkResult, Pid}
};

#[derive(Debug)]
pub enum ExecError {
    FailedPipeCreation,
    InvalidPipeline,
}

impl fmt::Display for ExecError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FailedPipeCreation => write!(f, "Failed To Create Pipe"),
            Self::InvalidPipeline => write!(f, "Invalid Pipeline"),
        }
    }
}

impl From<Errno> for ExecError {
    fn from(_: Errno) -> Self {
        ExecError::FailedPipeCreation
    }
}

impl From<Error> for ExecError {
    fn from(value: dyn Error) -> Self {
        ExecError::FailedPipeCreation
    }
}

fn is_executable(file: &Path) -> bool {
    let metadata = match metadata(file) {
        Ok(metadata) => metadata,
        Err(_) => return false,
    };
    metadata.is_file() && metadata.permissions().mode() & 0o111 != 0
}


//  fn execute(&mut self, &mut command_lst: Vec<CommandGroup>) -> Result<(), Box<dyn std::error::Error>> {
//      for command in command_lst {
//         let executable = self.find_executable(
//              command.get_exec()
//          ).ok_or_else(|| {
//                  format!("{}: Command not found", command.get_exec())
//              })?;
//          let exec_cstr = CString::new(
//              executable
//                  .as_os_str()
//                  .as_bytes()
//          )?;
//          match unsafe {fork()?} {
//              ForkResult::Parent {child} => {
//                  waitpid(child, None)?;
//              }
//              ForkResult::Child => {
//                  let env_string = self.shell_config.get_c_env()?;
//                  let env: Vec<&CStr> = env_string.iter().map(
//                      |var| var.as_c_str()
//                  ).collect();
//                  execve(&exec_cstr, &command.argv(), &env)?;
//              }
//          }
//      }
//      Ok(())
//  }
fn check_builtin(command: &OsStr) -> CommandType {
    match command.to_str() {
        Some("cd") => CommandType::BuiltIn,
        Some("pwd") => CommandType::BuiltIn,
        Some("export") => CommandType::BuiltIn,
        Some("unset") => CommandType::BuiltIn,
        Some("exit") => CommandType::BuiltIn,
        _ => CommandType::Unknown,
    }
}

fn resolve_path(
    shell_config: &ShellConfig,
    command_list: &mut Vec<CommandGroup>) -> Result<(), ConfigError<'_>> {
    for commandgroup in command_list {
        for input_command in &mut commandgroup.command {
            input_command.kind = check_builtin(&input_command.program);
            if input_command.kind == CommandType::Unknown {
                        input_command.program = match shell_config.build_path(
                        &input_command.program) {
                        Ok(path) => path,
                        Err(e) => return Err(e),
                    };
                }
            }
        }
    Ok(())
}

fn exec_command(
    env: &mut ShellConfig,
    command: InputCommand,
    streams: &[CommandPipe],
    index: usize) -> Result<Pid, ExecError>{
    
    match command.kind {
        CommandType::BuiltIn => {
            todo()!;
        }
        CommandType::Executable => {
            match unsafe {fork()?; } {
                ForkResult::Parent { child } => {
                    Ok(child)
                },
                ForkResult::Child => {
                    if index > 0 {
                        setup_stdin(env, &command.stdin, streams, index)?;
                    }
                    if index < streams.len() {
                        setup_stdout(env, &command.stdout, streams, index)?;
                    }
                    let c_env = env.get_c_env()?;
                    execve(
                        &Cstring::new(command.program.as_os_str().as_bytes()),
                        &command.argv(),
                        &c_env
                    );
                    Ok(())
                },
            }
        }
    }
}

fn exec_group(env:&mut ShellConfig, group: &CommandGroup) -> Result<u8, Box<dyn Error>> {
    let streams = create_streams(group);
    let children = Vec::new();

    for (index, command) in group.command.iter().enumerate() {
        let child = exec_command(env, command, streams, index)?;
        children.push(child);
    }
    for child in children {
        waitpid(child, None);
    }
    Ok(0)
}

fn execute_pipeline(
    env:&mut ShellConfig,
    command_list: &mut Vec<CommandGroup>) -> Result<(), Box<dyn Error + '_>> {
    let mut last_status: u8 = 0;

    resolve_path(env ,command_list)?;
    for group in command_list {
        let should_exec = match group.next {
            None => true,
            Some(CommandOperator::Sequence) => true,
            Some(CommandOperator::Or) => {
                last_status != 0 
            },
            Some(CommandOperator::And) => {
                last_status == 0 
            }
        };
        if should_exec {
            last_status = exec_group(env, group)?;
        }
    }
    Ok(())
}
