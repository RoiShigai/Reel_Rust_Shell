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
    error::Error,
    ffi::{CString, OsStr},
    fmt,
    fs::metadata,
    os::unix::{
        ffi::OsStrExt,
        fs::PermissionsExt,
    },
    path::{Path}
};

use nix::{
    errno::Errno,
    sys::wait::{waitpid, WaitStatus},
    unistd::{execve, fork, ForkResult, Pid},
};

#[derive(Debug)]
pub enum ExecError {
    FailedPipeCreation,
    InvalidPipeline,
    UnknownCommand,
    NixErrno(Errno),
    IOError(std::io::Error),
}

impl fmt::Display for ExecError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FailedPipeCreation => write!(f, "Failed To Create Pipe"),
            Self::InvalidPipeline => write!(f, "Invalid Pipeline"),
            Self::UnknownCommand => write!(f, "Unknown Command Type"),
            Self::NixErrno(error) => write!(f,"Nix Error during Execution: '{}'", error),
            Self::IOError(error) => write!(f, "StdIO Error during Execution: '{}'", error),
            Self::StdError(error) => write!(f, "StdError during Execution: '{}'", error)
        }
    }
}

impl From<Errno> for ExecError {
    fn from(_: Errno) -> Self {
        ExecError::FailedPipeCreation
    }
}

impl From<&dyn Error> for ExecError {
    fn from(value: &dyn Error) -> Self {
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

fn exec_child(
    env: &mut ShellConfig,
    command: &InputCommand,
    streams: &[CommandPipe],
    index: usize,
) -> ! {

    if let Err(error) =
        setup_stdin(env, &command.stdin, streams, index)
    {
        eprintln!("stdin setup failed: {error}");
        std::process::exit(1);
    }

    if let Err(error) =
        setup_stdout(env, &command.stdout, streams, index)
    {
        eprintln!("stdout setup failed: {error}");
        std::process::exit(1);
    }

    let c_env = match env.get_c_env() {
        Ok(value) => value,
        Err(error) => {
            eprintln!("environment error: {error}");
            std::process::exit(1);
        }
    };

    let c_exec = match CString::new(
        command.program.as_os_str().as_bytes()
    ) {
        Ok(value) => value,
        Err(error) => {
            eprintln!("invalid executable: {error}");
            std::process::exit(1);
        }
    };
    execve(&c_exec,&command.argv(),&c_env);
    unreachable!();
}

fn exec_command(
    env: &mut ShellConfig,
    command: &InputCommand,
    streams: &[CommandPipe],
    index: usize) -> Result<Pid, ExecError>{
    
    match command.kind {
        CommandType::BuiltIn => {
            todo!();
        }
        CommandType::Executable => {
            match unsafe {fork()? } {
                ForkResult::Parent {child} => {
                    Ok(child)
                },
                ForkResult::Child => exec_child(env, command, streams, index),
            }
        }
        _ => {Err(ExecError::UnknownCommand)}
    }
}

fn exec_group(env:&mut ShellConfig, group: &CommandGroup) -> Result<i32, Box<dyn Error>> {
    let streams = create_streams(group);
    let mut children = Vec::new();
    let mut last_status: i32 = 0;

    for (index, command) in group.command.iter().enumerate() {
        let child = exec_command(env, command, &streams, index)?;
        children.push(child);
    }
    drop(streams);
    for child in children {
        match waitpid(child, None)? {
            WaitStatus::Exited(_, status) => {
                last_status = status;
            }
            WaitStatus::Signaled(_, signal, _) => {
                last_status = 128 + signal as i32;
            }
            _ => {}
        }
    }
    Ok(last_status)
}

fn execute_pipeline(
    env:&mut ShellConfig,
    command_list: &mut Vec<CommandGroup>) -> Result<(), Box<dyn Error + '_>> {
    let mut last_status: i32 = 0;

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
